use std::sync::atomic::{AtomicU64, Ordering};

use dioxus::prelude::*;

use crate::foundation::state::sync_signal;

use crate::compat::field::{
    FieldControlAttrs, FieldDefinition, FieldErrorRegistration, FieldOrientation,
    FieldRelationships, push_unique_id_tokens, resolve_aria_describedby,
};

static NEXT_REGISTRATION_TOKEN: AtomicU64 = AtomicU64::new(1);

/// Monotonic per-mount instance guard preventing keyed-swap `use_drop` race erasure.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub struct RegistrationToken(u64);

impl RegistrationToken {
    /// Allocates a globally unique, monotonically increasing mount token.
    pub fn next() -> Self {
        Self(NEXT_REGISTRATION_TOKEN.fetch_add(1, Ordering::Relaxed))
    }

    pub const fn raw(&self) -> u64 {
        self.0
    }
}

/// Shared reactive context provided by [`super::FieldRoot`] and [`super::FieldSet`].
///
/// `Copy + PartialEq` via Dioxus `Signal`/`CopyValue` handles. All registration/unregistration
/// helpers guard writes with non-subscribing `try_peek()` equality checks.
#[derive(Clone, Copy, PartialEq)]
pub struct FieldContext {
    pub(crate) relationships: CopyValue<FieldRelationships>,
    pub(crate) name: Signal<Option<String>>,
    pub(crate) invalid: Signal<bool>,
    pub(crate) disabled: Signal<bool>,
    pub(crate) required: Signal<bool>,
    pub(crate) read_only: Signal<bool>,
    pub(crate) orientation: Signal<FieldOrientation>,
    pub(crate) is_fieldset: Signal<bool>,
    pub(crate) primary_control_id: Signal<Option<String>>,
    pub(crate) mounted_labels: Signal<Vec<(RegistrationToken, String)>>,
    pub(crate) mounted_legend: Signal<Vec<(RegistrationToken, String)>>,
    pub(crate) mounted_descriptions: Signal<Vec<(RegistrationToken, String)>>,
    pub(crate) mounted_errors: Signal<Vec<(RegistrationToken, FieldErrorRegistration)>>,
    pub(crate) mounted_error_messages: Signal<Vec<(RegistrationToken, Vec<String>)>>,
    pub(crate) next_control_index: CopyValue<usize>,
}

impl FieldContext {
    pub fn relationships(&self) -> FieldRelationships {
        self.relationships
            .try_read()
            .map(|r| r.clone())
            .unwrap_or_else(|_| FieldRelationships::new("fallback"))
    }

    pub fn name(&self) -> Option<String> {
        self.name.read().clone()
    }

    pub fn is_invalid(&self) -> bool {
        *self.invalid.read()
    }

    pub fn is_disabled(&self) -> bool {
        *self.disabled.read()
    }

    pub fn is_required(&self) -> bool {
        *self.required.read()
    }

    pub fn is_read_only(&self) -> bool {
        *self.read_only.read()
    }

    pub fn orientation(&self) -> FieldOrientation {
        *self.orientation.read()
    }

    pub fn is_fieldset(&self) -> bool {
        *self.is_fieldset.read()
    }

    pub fn primary_control_id(&self) -> Option<String> {
        self.primary_control_id.read().clone()
    }

    /// Resolves the primary control ID (`primary_control_id` override if set, else
    /// deterministic `relationships.control_id()`).
    pub fn resolved_control_id(&self) -> String {
        self.primary_control_id
            .read()
            .clone()
            .filter(|id| !id.trim().is_empty())
            .unwrap_or_else(|| self.relationships().control_id().to_owned())
    }

    /// Allocates the next deterministic 0-based control index for 1:N `FieldControl` slots.
    pub fn allocate_control_index(&self) -> usize {
        let mut slot = self.next_control_index;
        if let Ok(mut idx) = slot.try_write() {
            let current = *idx;
            *idx = current.saturating_add(1);
            current
        } else {
            0
        }
    }

    /// Sets or clears the primary control ID using a non-subscribing `try_peek()` guard.
    pub fn set_primary_control_id(&self, next: Option<String>) {
        let normalized = next.filter(|id| !id.trim().is_empty());
        let mut sig = self.primary_control_id;
        sync_signal(&mut sig, normalized);
    }

    /// Registers a mounted `FieldLabel` ID under `token` (no-op if unchanged).
    pub fn register_label(&self, token: RegistrationToken, id: String) {
        register_id_entry(self.mounted_labels, token, id);
    }

    /// Unregisters the `FieldLabel` owned by `token` (no-op if absent or already replaced).
    pub fn unregister_label(&self, token: RegistrationToken) {
        unregister_id_entry(self.mounted_labels, token);
    }

    /// Registers a mounted `FieldLegend` ID under `token` (no-op if unchanged).
    pub fn register_legend(&self, token: RegistrationToken, id: String) {
        register_id_entry(self.mounted_legend, token, id);
    }

    /// Unregisters the `FieldLegend` owned by `token`.
    pub fn unregister_legend(&self, token: RegistrationToken) {
        unregister_id_entry(self.mounted_legend, token);
    }

    /// Registers a mounted `FieldDescription` ID under `token` (no-op if unchanged).
    pub fn register_description(&self, token: RegistrationToken, id: String) {
        register_id_entry(self.mounted_descriptions, token, id);
    }

    /// Unregisters the `FieldDescription` owned by `token`.
    pub fn unregister_description(&self, token: RegistrationToken) {
        unregister_id_entry(self.mounted_descriptions, token);
    }

    /// Registers or updates a mounted `FieldError` entry under `token` (no-op if unchanged).
    pub fn register_error(&self, token: RegistrationToken, id: String, active: bool) {
        let next = FieldErrorRegistration::new(id, active);
        let unchanged = self.mounted_errors.try_peek().is_ok_and(|entries| {
            entries
                .iter()
                .any(|(tok, reg)| *tok == token && *reg == next)
        });
        if unchanged {
            return;
        }
        let mut sig = self.mounted_errors;
        if let Ok(mut entries) = sig.try_write() {
            if let Some((_, existing)) = entries.iter_mut().find(|(tok, _)| *tok == token) {
                *existing = next;
            } else {
                entries.push((token, next));
            }
        }
    }

    /// Unregisters the `FieldError` owned by `token`.
    pub fn unregister_error(&self, token: RegistrationToken) {
        let present = self
            .mounted_errors
            .try_peek()
            .is_ok_and(|entries| entries.iter().any(|(tok, _)| *tok == token));
        if !present {
            return;
        }
        let mut sig = self.mounted_errors;
        if let Ok(mut entries) = sig.try_write() {
            entries.retain(|(tok, _)| *tok != token);
        }
    }

    /// Registers or updates active error messages for a mounted `FieldError` under `token`.
    pub fn register_error_messages(&self, token: RegistrationToken, messages: Vec<String>) {
        if messages.is_empty() {
            self.unregister_error_messages(token);
            return;
        }
        let unchanged = self.mounted_error_messages.try_peek().is_ok_and(|entries| {
            entries
                .iter()
                .any(|(tok, cur)| *tok == token && *cur == messages)
        });
        if unchanged {
            return;
        }
        let mut sig = self.mounted_error_messages;
        if let Ok(mut entries) = sig.try_write() {
            if let Some((_, existing)) = entries.iter_mut().find(|(tok, _)| *tok == token) {
                *existing = messages;
            } else {
                entries.push((token, messages));
            }
        }
    }

    /// Unregisters error messages owned by `token`.
    pub fn unregister_error_messages(&self, token: RegistrationToken) {
        let present = self
            .mounted_error_messages
            .try_peek()
            .is_ok_and(|entries| entries.iter().any(|(tok, _)| *tok == token));
        if !present {
            return;
        }
        let mut sig = self.mounted_error_messages;
        if let Ok(mut entries) = sig.try_write() {
            entries.retain(|(tok, _)| *tok != token);
        }
    }

    /// Returns deduplicated active error messages from mounted `FieldError` parts (subscribing).
    pub fn active_error_messages(&self) -> Vec<String> {
        let mut out = Vec::new();
        for (_, msgs) in self.mounted_error_messages.read().iter() {
            for msg in msgs {
                if !out.contains(msg) {
                    out.push(msg.clone());
                }
            }
        }
        out
    }

    /// Returns deduplicated active error messages without subscribing (drop-safe for agent readers).
    pub(crate) fn peek_active_error_messages(&self) -> Vec<String> {
        let mut out = Vec::new();
        if let Ok(entries) = self.mounted_error_messages.try_peek() {
            for (_, msgs) in entries.iter() {
                for msg in msgs {
                    if !out.contains(msg) {
                        out.push(msg.clone());
                    }
                }
            }
        }
        out
    }

    /// Returns mounted `aria-labelledby` tokens (labels, or legend if present), or `None` if none mounted.
    pub fn aria_labelledby(&self) -> Option<String> {
        let mut tokens: Vec<String> = Vec::new();
        for (_, id) in self.mounted_labels.read().iter() {
            push_unique_id_tokens(&mut tokens, id);
        }
        if tokens.is_empty() {
            for (_, id) in self.mounted_legend.read().iter() {
                push_unique_id_tokens(&mut tokens, id);
            }
        }
        (!tokens.is_empty()).then(|| tokens.join(" "))
    }

    /// Computes mounted-only deduplicated `aria-describedby` tokens from mounted descriptions,
    /// active/invalid mounted errors, and optional `extra` tokens.
    pub fn aria_describedby(&self, extra: Option<&str>) -> Option<String> {
        let desc_ids: Vec<String> = self
            .mounted_descriptions
            .read()
            .iter()
            .map(|(_, id)| id.clone())
            .collect();
        let errs: Vec<FieldErrorRegistration> = self
            .mounted_errors
            .read()
            .iter()
            .map(|(_, reg)| reg.clone())
            .collect();
        resolve_aria_describedby(&desc_ids, &errs, self.is_invalid(), extra)
    }

    /// Snapshots the current reactive field state as a pure [`FieldDefinition`].
    pub fn definition(&self) -> FieldDefinition {
        FieldDefinition::new(self.relationships())
            .with_optional_name(self.name())
            .with_invalid(self.is_invalid())
            .with_disabled(self.is_disabled())
            .with_required(self.is_required())
            .with_read_only(self.is_read_only())
            .with_orientation(self.orientation())
            .with_fieldset(self.is_fieldset())
    }
}

/// Scoped context provided by [`super::FieldControl`] for 1:N control/label pairing.
#[derive(Clone, Copy, PartialEq)]
pub struct FieldControlContext {
    pub(crate) control_id: Signal<String>,
    pub(crate) default_label_id: CopyValue<String>,
    pub(crate) mounted_labels: Signal<Vec<(RegistrationToken, String)>>,
    pub(crate) explicit_id_locked: CopyValue<bool>,
}

impl FieldControlContext {
    pub fn control_id(&self) -> String {
        self.control_id.read().clone()
    }

    pub fn default_label_id(&self) -> String {
        self.default_label_id
            .try_read()
            .map(|s| s.clone())
            .unwrap_or_default()
    }

    pub fn mounted_label_id(&self) -> Option<String> {
        let mut tokens: Vec<String> = Vec::new();
        for (_, id) in self.mounted_labels.read().iter() {
            push_unique_id_tokens(&mut tokens, id);
        }
        (!tokens.is_empty()).then(|| tokens.join(" "))
    }

    pub fn sync_control_id(&self, next: String) {
        if next.trim().is_empty() {
            return;
        }
        let mut sig = self.control_id;
        sync_signal(&mut sig, next);
    }

    pub fn is_explicit_id_locked(&self) -> bool {
        self.explicit_id_locked
            .try_read()
            .map(|locked| *locked)
            .unwrap_or(false)
    }

    pub fn register_label(&self, token: RegistrationToken, id: String) {
        register_id_entry(self.mounted_labels, token, id);
    }

    pub fn unregister_label(&self, token: RegistrationToken) {
        unregister_id_entry(self.mounted_labels, token);
    }
}

fn register_id_entry(
    mut sig: Signal<Vec<(RegistrationToken, String)>>,
    token: RegistrationToken,
    id: String,
) {
    if id.trim().is_empty() {
        return;
    }
    let unchanged = sig
        .try_peek()
        .is_ok_and(|entries| entries.iter().any(|(tok, cur)| *tok == token && *cur == id));
    if unchanged {
        return;
    }
    if let Ok(mut entries) = sig.try_write() {
        if let Some((_, existing)) = entries.iter_mut().find(|(tok, _)| *tok == token) {
            *existing = id;
        } else {
            entries.push((token, id));
        }
    }
}

fn unregister_id_entry(
    mut sig: Signal<Vec<(RegistrationToken, String)>>,
    token: RegistrationToken,
) {
    let present = sig
        .try_peek()
        .is_ok_and(|entries| entries.iter().any(|(tok, _)| *tok == token));
    if !present {
        return;
    }
    if let Ok(mut entries) = sig.try_write() {
        entries.retain(|(tok, _)| *tok != token);
    }
}

/// Resolved handle returned by [`use_field_control`] and [`use_field_control_with_id`].
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct FieldControlHandle {
    attrs: FieldControlAttrs,
}

impl FieldControlHandle {
    pub fn new(attrs: FieldControlAttrs) -> Self {
        Self { attrs }
    }

    /// Returns the pure computed [`FieldControlAttrs`].
    pub fn attrs(&self) -> &FieldControlAttrs {
        &self.attrs
    }

    /// Consumes the handle and returns [`FieldControlAttrs`].
    pub fn into_attrs(self) -> FieldControlAttrs {
        self.attrs
    }

    pub fn id(&self) -> &str {
        &self.attrs.id
    }

    pub fn name(&self) -> Option<&str> {
        self.attrs.name.as_deref()
    }

    pub const fn is_invalid(&self) -> bool {
        self.attrs.aria_invalid.is_some()
    }

    pub const fn is_disabled(&self) -> bool {
        self.attrs.disabled
    }

    pub const fn is_required(&self) -> bool {
        self.attrs.required
    }

    pub const fn is_read_only(&self) -> bool {
        self.attrs.read_only
    }

    pub fn aria_labelledby(&self) -> Option<&str> {
        self.attrs.aria_labelledby.as_deref()
    }

    pub fn aria_describedby(&self) -> Option<&str> {
        self.attrs.aria_describedby.as_deref()
    }

    /// Converts all active control ARIA, state, and `data-*` attributes into Dioxus
    /// `GlobalAttributes` suitable for direct `..control.to_global_attributes()` spreading.
    pub fn to_global_attributes(&self) -> Vec<Attribute> {
        let mut out = Vec::with_capacity(14);
        out.push(Attribute::new("id", self.attrs.id.clone(), None, false));
        if let Some(ref name) = self.attrs.name {
            out.push(Attribute::new("name", name.clone(), None, false));
        }
        if let Some(ref labelledby) = self.attrs.aria_labelledby {
            out.push(Attribute::new(
                "aria-labelledby",
                labelledby.clone(),
                None,
                false,
            ));
        }
        if let Some(ref describedby) = self.attrs.aria_describedby {
            out.push(Attribute::new(
                "aria-describedby",
                describedby.clone(),
                None,
                false,
            ));
        }
        if let Some(invalid) = self.attrs.aria_invalid {
            out.push(Attribute::new("aria-invalid", invalid, None, false));
        }
        if let Some(required) = self.attrs.aria_required {
            out.push(Attribute::new("aria-required", required, None, false));
        }
        if let Some(disabled) = self.attrs.aria_disabled {
            out.push(Attribute::new("aria-disabled", disabled, None, false));
        }
        if let Some(readonly) = self.attrs.aria_readonly {
            out.push(Attribute::new("aria-readonly", readonly, None, false));
        }
        if self.attrs.disabled {
            out.push(Attribute::new("disabled", true, None, false));
        }
        if self.attrs.required {
            out.push(Attribute::new("required", true, None, false));
        }
        if self.attrs.read_only {
            out.push(Attribute::new("readonly", true, None, false));
        }
        if let Some(data_invalid) = self.attrs.data_invalid {
            out.push(Attribute::new("data-invalid", data_invalid, None, false));
        }
        if let Some(data_disabled) = self.attrs.data_disabled {
            out.push(Attribute::new("data-disabled", data_disabled, None, false));
        }
        if let Some(data_required) = self.attrs.data_required {
            out.push(Attribute::new("data-required", data_required, None, false));
        }
        if let Some(data_readonly) = self.attrs.data_readonly {
            out.push(Attribute::new("data-readonly", data_readonly, None, false));
        }
        out.push(Attribute::new(
            "data-monoxus-field-control",
            self.attrs.data_monoxus_field_control,
            None,
            false,
        ));
        out
    }
}

/// Consumes the enclosing [`FieldContext`], panicking if called outside `FieldRoot`/`FieldSet`.
pub fn use_field_context() -> FieldContext {
    try_use_field_context().expect("Field parts must be used inside a FieldRoot or FieldSet")
}

/// Fallibly consumes the enclosing [`FieldContext`].
pub fn try_use_field_context() -> Option<FieldContext> {
    try_consume_context::<FieldContext>()
}

/// Fallibly consumes the enclosing [`FieldControlContext`].
pub fn try_use_field_control_context() -> Option<FieldControlContext> {
    try_consume_context::<FieldControlContext>()
}

/// Resolves a reactive [`FieldControlHandle`] from the nearest `FieldControl` / `FieldRoot` / `FieldSet`.
/// Returns `None` when called outside any field context.
pub fn use_field_control() -> Option<FieldControlHandle> {
    use_field_control_with_id(None)
}

/// Like [`use_field_control`], allowing a control with an explicit `id` prop to coordinate its ID
/// with sibling `FieldLabel` parts.
pub fn use_field_control_with_id(explicit_id: Option<String>) -> Option<FieldControlHandle> {
    let field_ctx = try_use_field_context()?;
    let control_ctx = try_use_field_control_context();

    let explicit_clean = explicit_id.filter(|id| !id.trim().is_empty());

    let resolved_id = if let Some(ref explicit) = explicit_clean {
        if let Some(ctrl_ctx) = control_ctx {
            if !ctrl_ctx.is_explicit_id_locked() {
                ctrl_ctx.sync_control_id(explicit.clone());
            }
        } else if !field_ctx.is_fieldset() {
            field_ctx.set_primary_control_id(Some(explicit.clone()));
        }
        explicit.clone()
    } else if let Some(ctrl_ctx) = control_ctx {
        ctrl_ctx.control_id()
    } else {
        field_ctx.resolved_control_id()
    };

    let aria_labelledby = if let Some(ctrl_ctx) = control_ctx {
        ctrl_ctx
            .mounted_label_id()
            .or_else(|| (!field_ctx.is_fieldset()).then(|| field_ctx.aria_labelledby()).flatten())
    } else {
        field_ctx.aria_labelledby()
    };

    let aria_describedby = field_ctx.aria_describedby(None);
    let attrs = FieldControlAttrs::new(
        resolved_id,
        field_ctx.name(),
        aria_labelledby,
        aria_describedby,
        field_ctx.is_invalid(),
        field_ctx.is_disabled(),
        field_ctx.is_required(),
        field_ctx.is_read_only(),
    );

    Some(FieldControlHandle::new(attrs))
}

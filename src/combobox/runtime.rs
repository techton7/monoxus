use std::rc::Rc;

use dioxus::prelude::keyboard_types::{Key, Modifiers, NamedKey};
use dioxus::prelude::*;
use ui_sdk::kernel::{ChangeContext, ChangeEvent, ChangeReason, ChangeSource};

pub use crate::foundation::overlay::serialize_css_custom_properties as serialize_combobox_css_custom_properties;

use crate::foundation::{
    browser::{
        FocusOptions, focus_element_sync_or_defer, get_element_scroll_metrics,
        query_descendant_order, set_input_selection_range, set_input_value_by_id,
    },
    compose::MountedHandle,
    keyboard::is_ime_composing_key,
    overlay::{
        DismissBehavior, DismissLayerHandle, FloatingHandle, FloatingPlacement, FloatingReadiness,
        GeometryVars, PlacementAlign, PlacementSide, PortalHost, PresenceHandle, PresenceState,
        StickyStrategy, ensure_active_overlay_layer, is_overlay_escape_consumed,
        unregister_active_overlay_layer,
    },
    shared::EscapeKeyDownEvent,
    state::{ChangeOutcome, ControllableSignal, sync_signal},
};

use super::{
    attrs::{
        ComboboxContentAttrs, ComboboxInputAttrs, ComboboxItemAttrs, ComboboxTriggerAttrs,
        ComboboxViewportAttrs,
    },
    monitors::ComboboxInteractOutsideEvent,
    registry::ComboboxOptionRegistry,
    relationships::ComboboxRelationships,
    state::{
        ComboboxState, apply_multiple_toggle, apply_single_toggle, resolve_input_keydown,
    },
    types::{
        ComboboxContext, ComboboxItemData, ComboboxMode, ComboboxNavKey, ComboboxScrollAlignment,
        PressedComboboxItem,
    },
};

pub type ComboboxOpenChangeHandler = Rc<dyn Fn(bool)>;
pub type ComboboxValueChangeHandler = Rc<dyn Fn(Option<String>)>;
pub type ComboboxValuesChangeHandler = Rc<dyn Fn(Vec<String>)>;
pub type ComboboxInputValueChangeHandler = Rc<dyn Fn(String)>;

pub fn use_combobox_context() -> ComboboxContext {
    use_context::<ComboboxContext>()
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum ComboboxPendingHighlight {
    First,
    Last,
}

#[derive(Clone, Copy)]
pub struct ComboboxRuntimeState {
    // Reactive ComboboxRoot config signals (4B D-4: zero stale mount snapshots)
    pub mode: Signal<ComboboxMode>,
    pub disabled: Signal<bool>,
    pub required: Signal<bool>,
    pub invalid: Signal<bool>,
    pub root_invalid_override: Signal<Option<bool>>,
    pub root_aria_describedby: Signal<Option<String>>,
    pub loop_focus: Signal<bool>,
    pub scroll_alignment: Signal<ComboboxScrollAlignment>,
    pub allow_deselect: Signal<bool>,
    pub clear_on_deselect: Signal<bool>,
    pub name: Signal<Option<String>>,
    pub form: Signal<Option<String>>,

    // Non-reactive ComboboxRoot callbacks (`Dioxus #5894`)
    pub on_open_change_complete: CopyValue<Option<EventHandler<bool>>>,
    pub on_composite_focus_in: CopyValue<Option<EventHandler<FocusEvent>>>,
    pub on_composite_focus_out: CopyValue<Option<EventHandler<FocusEvent>>>,

    // Value / open / highlight state
    pub open: Signal<bool>,
    pub open_ctl: ControllableSignal<bool>,
    pub value: Signal<Option<String>>,
    pub value_ctl: ControllableSignal<Option<String>>,
    pub values: Signal<Vec<String>>,
    pub values_ctl: ControllableSignal<Vec<String>>,
    pub input_value: Signal<String>,
    pub input_value_ctl: ControllableSignal<String>,
    pub highlighted_value: Signal<Option<String>>,
    pub pending_open_highlight: Signal<Option<ComboboxPendingHighlight>>,
    pub pending_input_reconcile: Signal<u64>,
    pub registry: Signal<ComboboxOptionRegistry>,
    pub registry_version: Signal<u64>,
    pub pressed_item: Signal<Option<PressedComboboxItem>>,
    pub suppress_next_click: Signal<bool>,
    pub is_composing: Signal<bool>,
    pub user_has_scrolled: Signal<bool>,
    pub content_mounted: Signal<bool>,
    pub is_static_content: Signal<bool>,
    pub default_value_initial: Signal<Option<String>>,
    pub default_values_initial: Signal<Vec<String>>,
    pub default_input_value_initial: Signal<String>,
    pub reset_epoch: Signal<u64>,

    pub input_id: Signal<String>,
    pub trigger_id: Signal<String>,
    pub content_id: Signal<String>,
    pub viewport_id: Signal<String>,
    pub arrow_id: Signal<Option<String>>,

    pub preferred_side: Signal<PlacementSide>,
    pub preferred_align: Signal<PlacementAlign>,
    pub side_offset: Signal<f32>,
    pub align_offset: Signal<f32>,
    pub avoid_collisions: Signal<bool>,
    pub collision_boundary: Signal<Option<String>>,
    pub collision_padding: Signal<f32>,
    pub arrow_padding: Signal<f32>,
    pub hide_when_detached: Signal<bool>,
    pub sticky: Signal<Option<StickyStrategy>>,
    pub custom_anchor: Signal<Option<String>>,
    pub force_mount: Signal<bool>,
    pub keep_mounted: Signal<bool>,
    pub escape_key_down_behavior: Signal<Option<DismissBehavior>>,
    pub interact_outside_behavior: Signal<Option<DismissBehavior>>,

    pub can_scroll_up: Signal<bool>,
    pub can_scroll_down: Signal<bool>,

    pub on_interact_outside: CopyValue<Option<EventHandler<ComboboxInteractOutsideEvent>>>,
    pub on_escape_key_down: CopyValue<Option<EventHandler<EscapeKeyDownEvent>>>,

    pub presence: PresenceHandle,
    pub floating: FloatingHandle,
    pub dismiss: DismissLayerHandle,
}

#[derive(Clone)]
pub struct ComboboxRuntime {
    pub relationships: ComboboxRelationships,
    pub state: ComboboxRuntimeState,
}

impl PartialEq for ComboboxRuntime {
    fn eq(&self, other: &Self) -> bool {
        self.relationships == other.relationships
    }
}

impl ComboboxRuntime {
    pub fn relationships(&self) -> &ComboboxRelationships {
        &self.relationships
    }

    pub fn config(&self) -> ComboboxState {
        ComboboxState::new()
            .with_mode(self.mode_untracked())
            .with_disabled(self.is_disabled_untracked())
            .with_required(self.is_required_untracked())
            .with_loop_focus(self.loop_focus_untracked())
            .with_scroll_alignment(self.scroll_alignment_untracked())
            .with_allow_deselect(self.allow_deselect_untracked())
            .with_clear_on_deselect(self.clear_on_deselect())
            .with_name(self.name_untracked())
            .with_form(self.form_untracked())
            .with_portal_host(PortalHost::default())
    }

    pub fn mode(&self) -> ComboboxMode {
        *self.state.mode.read()
    }

    pub fn mode_untracked(&self) -> ComboboxMode {
        self.state
            .mode
            .try_peek()
            .map(|g| *g)
            .unwrap_or(ComboboxMode::Single)
    }

    pub fn is_disabled(&self) -> bool {
        *self.state.disabled.read()
    }

    pub fn is_disabled_untracked(&self) -> bool {
        self.state.disabled.try_peek().map(|g| *g).unwrap_or(false)
    }

    pub fn is_required(&self) -> bool {
        *self.state.required.read()
    }

    pub fn is_required_untracked(&self) -> bool {
        self.state.required.try_peek().map(|g| *g).unwrap_or(false)
    }

    pub fn is_invalid(&self) -> bool {
        *self.state.invalid.read()
    }

    pub fn root_invalid_override(&self) -> Option<bool> {
        *self.state.root_invalid_override.read()
    }

    pub fn root_aria_describedby(&self) -> Option<String> {
        self.state.root_aria_describedby.read().clone()
    }

    pub fn notify_composite_focus_in(&self, evt: FocusEvent) {
        if let Some(cb) = self
            .state
            .on_composite_focus_in
            .try_read()
            .ok()
            .and_then(|s| *s)
        {
            cb.call(evt);
        }
    }

    pub fn notify_composite_focus_out(&self, evt: FocusEvent) {
        if let Some(cb) = self
            .state
            .on_composite_focus_out
            .try_read()
            .ok()
            .and_then(|s| *s)
        {
            cb.call(evt);
        }
    }

    pub fn loop_focus(&self) -> bool {
        *self.state.loop_focus.read()
    }

    pub fn loop_focus_untracked(&self) -> bool {
        self.state
            .loop_focus
            .try_peek()
            .map(|g| *g)
            .unwrap_or(false)
    }

    pub fn scroll_alignment(&self) -> ComboboxScrollAlignment {
        *self.state.scroll_alignment.read()
    }

    pub fn scroll_alignment_untracked(&self) -> ComboboxScrollAlignment {
        self.state
            .scroll_alignment
            .try_peek()
            .map(|g| *g)
            .unwrap_or(ComboboxScrollAlignment::Nearest)
    }

    pub fn allow_deselect(&self) -> bool {
        *self.state.allow_deselect.read()
    }

    pub fn allow_deselect_untracked(&self) -> bool {
        self.state
            .allow_deselect
            .try_peek()
            .map(|g| *g)
            .unwrap_or(true)
    }

    pub fn name(&self) -> Option<String> {
        self.state.name.read().clone()
    }

    pub fn name_untracked(&self) -> Option<String> {
        self.state.name.try_peek().ok().and_then(|g| g.clone())
    }

    pub fn form(&self) -> Option<String> {
        self.state.form.read().clone()
    }

    pub fn form_untracked(&self) -> Option<String> {
        self.state.form.try_peek().ok().and_then(|g| g.clone())
    }

    pub fn clear_on_deselect(&self) -> bool {
        self.state
            .clear_on_deselect
            .try_peek()
            .map(|g| *g)
            .unwrap_or(false)
    }

    pub fn set_clear_on_deselect(&self, clear: bool) {
        let mut sig = self.state.clear_on_deselect;
        sync_signal(&mut sig, clear);
    }

    pub fn set_composing(&self, composing: bool) {
        let mut sig = self.state.is_composing;
        sync_signal(&mut sig, composing);
    }

    pub fn is_composing_untracked(&self) -> bool {
        self.state
            .is_composing
            .try_peek()
            .map(|g| *g)
            .unwrap_or(false)
    }

    pub fn set_user_has_scrolled(&self, scrolled: bool) {
        let mut sig = self.state.user_has_scrolled;
        sync_signal(&mut sig, scrolled);
    }

    pub fn user_has_scrolled_untracked(&self) -> bool {
        self.state
            .user_has_scrolled
            .try_peek()
            .map(|g| *g)
            .unwrap_or(false)
    }

    pub fn set_content_mounted(&self, mounted: bool) {
        let mut sig = self.state.content_mounted;
        sync_signal(&mut sig, mounted);
    }

    pub fn is_content_mounted(&self) -> bool {
        *self.state.content_mounted.read()
    }

    pub fn set_is_static_content(&self, is_static: bool) {
        let mut sig = self.state.is_static_content;
        sync_signal(&mut sig, is_static);
    }

    pub fn should_render_content(&self) -> bool {
        self.state.presence.should_render()
    }

    pub fn data_starting_style(&self) -> Option<&'static str> {
        self.state.presence.data_starting_style()
    }

    pub fn data_ending_style(&self) -> Option<&'static str> {
        self.state.presence.data_ending_style()
    }

    pub fn is_hidden(&self) -> bool {
        self.state.presence.is_hidden()
    }

    pub fn set_keep_mounted(&self, keep_mounted: bool) {
        let mut sig = self.state.keep_mounted;
        sync_signal(&mut sig, keep_mounted);
        self.state.presence.sync_keep_mounted(keep_mounted);
    }

    pub fn presence_state(&self) -> PresenceState {
        self.state.presence.state()
    }

    pub fn finish_active_exit(&self) -> bool {
        self.state.presence.finish_active_exit()
    }

    pub fn set_suppress_next_click(&self, suppress: bool) {
        let mut sig = self.state.suppress_next_click;
        sync_signal(&mut sig, suppress);
    }

    pub fn take_suppress_next_click(&self) -> bool {
        let mut sig = self.state.suppress_next_click;
        if sig.try_peek().is_ok_and(|g| *g) {
            let _ = sig.try_write().map(|mut w| *w = false);
            true
        } else {
            false
        }
    }

    fn bump_registry_version(&self) {
        let mut ver = self.state.registry_version;
        let _ = ver.try_write().map(|mut w| *w = w.wrapping_add(1));
    }

    fn bump_pending_input_reconcile(&self) {
        let mut sig = self.state.pending_input_reconcile;
        let _ = sig.try_write().map(|mut w| *w = w.wrapping_add(1));
    }

    pub fn current_input_id(&self) -> String {
        self.state
            .input_id
            .try_read()
            .map(|g| g.clone())
            .unwrap_or_else(|_| self.relationships.input_id().to_owned())
    }

    pub fn current_input_id_untracked(&self) -> String {
        self.state
            .input_id
            .try_peek()
            .map(|g| g.clone())
            .unwrap_or_else(|_| self.relationships.input_id().to_owned())
    }

    pub fn set_input_id(&self, id: String) {
        let mut sig = self.state.input_id;
        sync_signal(&mut sig, id);
    }

    pub fn current_trigger_id(&self) -> String {
        self.state
            .trigger_id
            .try_read()
            .map(|g| g.clone())
            .unwrap_or_else(|_| self.relationships.trigger_id().to_owned())
    }

    pub fn current_trigger_id_untracked(&self) -> String {
        self.state
            .trigger_id
            .try_peek()
            .map(|g| g.clone())
            .unwrap_or_else(|_| self.relationships.trigger_id().to_owned())
    }

    pub fn set_trigger_id(&self, id: String) {
        let mut sig = self.state.trigger_id;
        sync_signal(&mut sig, id);
    }

    pub fn current_content_id(&self) -> String {
        self.state
            .content_id
            .try_read()
            .map(|g| g.clone())
            .unwrap_or_else(|_| self.relationships.content_id().to_owned())
    }

    pub fn current_content_id_untracked(&self) -> String {
        self.state
            .content_id
            .try_peek()
            .map(|g| g.clone())
            .unwrap_or_else(|_| self.relationships.content_id().to_owned())
    }

    pub fn set_content_id(&self, id: String) {
        let mut sig = self.state.content_id;
        sync_signal(&mut sig, id.clone());
        self.state.dismiss.rename_layer_id(&id);
    }

    pub fn current_viewport_id(&self) -> String {
        self.state
            .viewport_id
            .try_read()
            .map(|g| g.clone())
            .unwrap_or_else(|_| self.relationships.viewport_id().to_owned())
    }

    pub fn current_viewport_id_untracked(&self) -> String {
        self.state
            .viewport_id
            .try_peek()
            .map(|g| g.clone())
            .unwrap_or_else(|_| self.relationships.viewport_id().to_owned())
    }

    pub fn set_viewport_id(&self, id: String) {
        let mut sig = self.state.viewport_id;
        sync_signal(&mut sig, id);
    }

    pub fn arrow_id(&self) -> Option<String> {
        self.state
            .floating
            .arrow_id()
            .or_else(|| self.state.arrow_id.read().clone())
    }

    pub fn register_arrow_id(&self, id: &str) {
        let mut sig = self.state.arrow_id;
        sync_signal(&mut sig, Some(id.to_owned()));
        self.state.floating.register_arrow_id(id);
    }

    pub fn unregister_arrow_id(&self, id: &str) {
        let mut sig = self.state.arrow_id;
        if sig.try_peek().is_ok_and(|cur| cur.as_deref() == Some(id))
            && let Ok(mut w) = sig.try_write()
        {
            *w = None;
        }
        self.state.floating.unregister_arrow_id(id);
    }

    pub fn capture_input_mount(&self, handle: MountedHandle) {
        self.state.floating.capture_anchor(handle);
    }

    pub fn capture_content_mount(&self, handle: MountedHandle) {
        self.state.floating.capture_content(handle);
    }

    pub fn is_open(&self) -> bool {
        *self.state.open.read()
    }

    pub fn is_open_untracked(&self) -> bool {
        self.state.open.try_peek().map(|g| *g).unwrap_or(false)
    }

    pub fn set_open(&self, next: bool) {
        let _ = self.set_open_with_context(
            next,
            ChangeContext::new(ChangeReason::Selection, ChangeSource::Mouse),
        );
    }

    pub(crate) fn set_open_with_context(&self, next: bool, ctx: ChangeContext) -> ChangeOutcome {
        let event = ChangeEvent::new(next, ctx);
        self.set_open_with_event(next, &event)
    }

    pub(crate) fn set_open_with_event(
        &self,
        next: bool,
        event: &ChangeEvent<bool>,
    ) -> ChangeOutcome {
        if self.is_disabled_untracked() && next {
            return ChangeOutcome::Unchanged;
        }
        let current = self.is_open_untracked();
        if current == next {
            return ChangeOutcome::Unchanged;
        }
        let outcome = self.state.open_ctl.set_with_event(next, event);
        if !outcome.is_committed() {
            return outcome;
        }

        let cid = self.current_content_id_untracked();
        if !next {
            let mut pending = self.state.pending_open_highlight;
            let _ = pending.try_write().map(|mut w| *w = None);
            self.set_user_has_scrolled(false);
            self.set_highlighted(None, false);
            self.stop_dismiss_monitor();
            unregister_active_overlay_layer(&cid);
            self.state.presence.sync(false, &cid);
        } else {
            self.set_user_has_scrolled(false);
            ensure_active_overlay_layer(&cid);
            self.start_dismiss_monitor();
            self.state.presence.sync(true, &cid);
        }
        outcome
    }

    pub fn open_menu(&self) {
        self.set_open(true);
    }

    pub fn close_menu(&self) {
        self.set_open(false);
    }

    pub fn toggle_menu(&self) {
        self.set_open(!self.is_open_untracked());
    }

    pub fn value(&self) -> Option<String> {
        self.state.value.read().clone()
    }

    pub fn value_untracked(&self) -> Option<String> {
        self.state.value.try_peek().ok().and_then(|g| g.clone())
    }

    pub fn set_value(&self, next: Option<String>) {
        let _ = self.set_value_with_context(
            next,
            ChangeContext::new(ChangeReason::Selection, ChangeSource::Mouse),
        );
    }

    pub(crate) fn set_value_with_context(
        &self,
        next: Option<String>,
        ctx: ChangeContext,
    ) -> ChangeOutcome {
        self.state.value_ctl.set_with_context(next, ctx)
    }

    pub fn values(&self) -> Vec<String> {
        self.state.values.read().clone()
    }

    pub fn values_untracked(&self) -> Vec<String> {
        self.state
            .values
            .try_peek()
            .map(|g| g.clone())
            .unwrap_or_default()
    }

    pub fn set_values(&self, next: Vec<String>) {
        let _ = self.set_values_with_context(
            next,
            ChangeContext::new(ChangeReason::Selection, ChangeSource::Mouse),
        );
    }

    pub(crate) fn set_values_with_context(
        &self,
        next: Vec<String>,
        ctx: ChangeContext,
    ) -> ChangeOutcome {
        self.state.values_ctl.set_with_context(next, ctx)
    }

    pub fn is_selected(&self, val: &str) -> bool {
        match self.mode() {
            ComboboxMode::Single => self.value().as_deref() == Some(val),
            ComboboxMode::Multiple => self.values().iter().any(|v| v == val),
        }
    }

    pub fn input_value(&self) -> String {
        self.state.input_value.read().clone()
    }

    pub fn input_value_untracked(&self) -> String {
        self.state
            .input_value
            .try_peek()
            .map(|g| g.clone())
            .unwrap_or_default()
    }

    pub fn set_input_value(&self, next: String, sync_dom: bool) {
        let _ = self.set_input_value_with_context(
            next,
            sync_dom,
            ChangeContext::new(ChangeReason::Input, ChangeSource::Keyboard),
        );
    }

    pub(crate) fn set_input_value_with_context(
        &self,
        next: String,
        sync_dom: bool,
        ctx: ChangeContext,
    ) -> ChangeOutcome {
        let is_controlled = self.state.input_value_ctl.is_controlled();
        let outcome = self.state.input_value_ctl.set_with_context(next.clone(), ctx);
        let effective = self.input_value_untracked();
        if is_controlled {
            if effective != next || sync_dom {
                let iid = self.current_input_id_untracked();
                set_input_value_by_id(&iid, &effective);
                let utf16_len = effective.encode_utf16().count();
                set_input_selection_range(&iid, utf16_len, utf16_len);
            }
        } else if outcome.is_committed() && sync_dom {
            let iid = self.current_input_id_untracked();
            set_input_value_by_id(&iid, &next);
            let utf16_len = next.encode_utf16().count();
            set_input_selection_range(&iid, utf16_len, utf16_len);
        }
        outcome
    }

    pub fn highlighted_value(&self) -> Option<String> {
        self.state.highlighted_value.read().clone()
    }

    pub fn highlighted_value_untracked(&self) -> Option<String> {
        self.state
            .highlighted_value
            .try_peek()
            .ok()
            .and_then(|g| g.clone())
    }

    pub fn highlighted_item_id(&self) -> Option<String> {
        let hl = self.highlighted_value()?;
        let reg = self.state.registry.read();
        if reg.is_disabled(&hl) {
            return None;
        }
        Some(
            reg.dom_id_for(&hl)
                .map(str::to_owned)
                .unwrap_or_else(|| self.relationships.item_id(&hl)),
        )
    }

    pub fn set_highlighted(&self, next: Option<String>, should_scroll: bool) {
        let effective = next.filter(|v| {
            self.state
                .registry
                .try_peek()
                .map(|r| !r.is_disabled(v))
                .unwrap_or(true)
        });
        let mut sig = self.state.highlighted_value;
        if sig.try_peek().ok().as_deref() != Some(&effective) {
            let _ = sig.try_write().map(|mut w| *w = effective.clone());
        }
        if should_scroll && let Some(val) = effective {
            self.scroll_value_into_view(&val);
        }
    }

    pub fn scroll_value_into_view(&self, value: &str) {
        let item_id = self
            .state
            .registry
            .try_peek()
            .ok()
            .and_then(|r| r.dom_id_for(value).map(str::to_owned))
            .unwrap_or_else(|| self.relationships.item_id(value));
        let vp_id = self.current_viewport_id_untracked();
        let content_id = self.current_content_id_untracked();
        super::monitors::scroll_item_in_viewport(
            &vp_id,
            &content_id,
            &item_id,
            self.scroll_alignment_untracked(),
        );
    }

    pub fn focus_input(&self) {
        let iid = self.current_input_id_untracked();
        focus_element_sync_or_defer(&iid, FocusOptions::new(true));
        let utf16_len = self.input_value_untracked().encode_utf16().count();
        set_input_selection_range(&iid, utf16_len, utf16_len);
    }

    pub fn seed_items(&self, items: &[ComboboxItemData]) {
        if items.is_empty() {
            return;
        }
        let mut reg = self.state.registry;
        if let Ok(mut w) = reg.try_write() {
            w.seed_items(items);
        }
    }

    pub fn register_item(&self, value: String, label: String, dom_id: String, disabled: bool) {
        let mut reg = self.state.registry;
        if let Ok(mut w) = reg.try_write() {
            w.register(value, label, dom_id, disabled);
        }
        self.bump_registry_version();
        if self.is_open_untracked() {
            let cid = self.current_content_id_untracked();
            self.sync_dom_order(&cid);
        }
        if !disabled
            && self.is_open_untracked()
            && let Ok(Some(pending)) = self.state.pending_open_highlight.try_peek().map(|g| *g)
        {
            let candidate = self.state.registry.try_peek().ok().and_then(|r| match pending {
                ComboboxPendingHighlight::First => r.first_enabled().map(|i| i.value.clone()),
                ComboboxPendingHighlight::Last => r.last_enabled().map(|i| i.value.clone()),
            });
            if let Some(val) = candidate {
                self.set_highlighted(Some(val), true);
            }
        }
    }

    pub fn unregister_item(&self, dom_id: &str) {
        let (removed_val, old_enabled_idx) = self
            .state
            .registry
            .try_peek()
            .ok()
            .map(|r| {
                let val = r
                    .items()
                    .iter()
                    .find(|i| i.dom_id == dom_id)
                    .map(|i| i.value.clone());
                let idx = val
                    .as_deref()
                    .and_then(|v| r.enabled_items().iter().position(|i| i.value == v));
                (val, idx)
            })
            .unwrap_or((None, None));

        let mut removed = false;
        let mut reg = self.state.registry;
        if let Ok(mut w) = reg.try_write() {
            removed = w.unregister_by_dom_id(dom_id);
        }
        if removed {
            self.bump_registry_version();
            if self.is_open_untracked()
                && removed_val.is_some()
                && self.highlighted_value_untracked() == removed_val
            {
                let replacement = self.state.registry.try_peek().ok().and_then(|r| {
                    let enabled = r.enabled_items();
                    if enabled.is_empty() {
                        None
                    } else {
                        let idx = old_enabled_idx
                            .map(|i| i.min(enabled.len() - 1))
                            .unwrap_or(0);
                        enabled.get(idx).map(|i| i.value.clone())
                    }
                });
                self.set_highlighted(replacement, false);
            }
        }
    }

    pub fn sync_dom_order(&self, content_id: &str) {
        let dom_ids: Vec<String> = self
            .state
            .registry
            .try_peek()
            .map(|r| r.ordered_dom_ids().into_iter().map(str::to_owned).collect())
            .unwrap_or_default();
        if dom_ids.len() <= 1 {
            return;
        }
        let refs: Vec<&str> = dom_ids.iter().map(String::as_str).collect();
        if let Ok(ordered) = query_descendant_order(content_id, &refs)
            && ordered != dom_ids
        {
            let mut reg = self.state.registry;
            if let Ok(mut w) = reg.try_write() {
                w.reorder_by_dom_ids(&ordered);
            }
        }
    }

    pub fn flush_pending_open_and_input_reconcile(
        &self,
        seen_reconcile_ver: &mut u64,
        current_reconcile_ver: u64,
    ) {
        if !self.is_open_untracked() {
            return;
        }

        if let Ok(Some(pending)) = self.state.pending_open_highlight.try_peek().map(|g| *g) {
            let has_items = self
                .state
                .registry
                .try_peek()
                .map(|r| !r.is_empty())
                .unwrap_or(false);
            if has_items {
                let selected = match self.mode_untracked() {
                    ComboboxMode::Single => self.value_untracked(),
                    ComboboxMode::Multiple => self.values_untracked().first().cloned(),
                };
                let candidate = self.state.registry.try_peek().ok().and_then(|r| {
                    selected
                        .as_deref()
                        .filter(|val| r.contains_enabled(val))
                        .map(str::to_owned)
                        .or_else(|| match pending {
                            ComboboxPendingHighlight::First => {
                                r.first_enabled().map(|i| i.value.clone())
                            }
                            ComboboxPendingHighlight::Last => {
                                r.last_enabled().map(|i| i.value.clone())
                            }
                        })
                });
                self.set_highlighted(candidate, true);
                let settled_ver = self
                    .state
                    .registry_version
                    .try_peek()
                    .map(|g| *g)
                    .unwrap_or(0);
                let rt = self.clone();
                crate::foundation::browser::spawn_root_task(async move {
                    struct YieldOnce(bool);
                    impl std::future::Future for YieldOnce {
                        type Output = ();
                        fn poll(
                            mut self: std::pin::Pin<&mut Self>,
                            cx: &mut std::task::Context<'_>,
                        ) -> std::task::Poll<()> {
                            if self.0 {
                                std::task::Poll::Ready(())
                            } else {
                                self.0 = true;
                                cx.waker().wake_by_ref();
                                std::task::Poll::Pending
                            }
                        }
                    }
                    YieldOnce(false).await;
                    let cur_ver = rt
                        .state
                        .registry_version
                        .try_peek()
                        .map(|g| *g)
                        .unwrap_or(0);
                    if cur_ver == settled_ver {
                        let mut p = rt.state.pending_open_highlight;
                        let _ = p.try_write().map(|mut w| *w = None);
                    }
                });
            }
        }

        if current_reconcile_ver != *seen_reconcile_ver {
            *seen_reconcile_ver = current_reconcile_ver;
            self.reconcile_highlight_for_input();
        }
    }

    pub fn reconcile_highlight_for_input(&self) {
        if !self.is_open_untracked() {
            return;
        }
        let first = self
            .state
            .registry
            .try_peek()
            .ok()
            .and_then(|r| r.first_enabled().map(|i| i.value.clone()));
        self.set_highlighted(first, true);
    }

    pub fn handle_input_change(&self, next_value: String) {
        if self.is_disabled_untracked() {
            return;
        }
        self.set_input_value(next_value, false);
        if !self.is_open_untracked() {
            self.open_menu();
        }
        self.bump_pending_input_reconcile();
        self.refresh_scroll_buttons();
    }

    pub fn select_item(&self, target_value: &str) {
        let _ = self.select_item_with_context(
            target_value,
            ChangeContext::new(ChangeReason::Selection, ChangeSource::Mouse),
        );
    }

    pub(crate) fn select_item_with_context(
        &self,
        target_value: &str,
        ctx: ChangeContext,
    ) -> ChangeOutcome {
        if self.is_disabled_untracked() {
            return ChangeOutcome::Unchanged;
        }
        let (is_disabled, label) = self
            .state
            .registry
            .try_peek()
            .map(|r| {
                let dis = r.is_disabled(target_value);
                let lbl = r
                    .label_for(target_value)
                    .map(str::to_owned)
                    .unwrap_or_else(|| target_value.to_owned());
                (dis, lbl)
            })
            .unwrap_or_else(|_| (false, target_value.to_owned()));

        if is_disabled {
            return ChangeOutcome::Unchanged;
        }

        match self.mode_untracked() {
            ComboboxMode::Single => {
                let current = self.value_untracked();
                let is_val_controlled = self.state.value_ctl.is_controlled();
                let outcome = apply_single_toggle(
                    current.as_deref(),
                    target_value,
                    &label,
                    self.allow_deselect_untracked(),
                    self.clear_on_deselect(),
                );
                let change_outcome = if outcome.changed {
                    self.set_value_with_context(outcome.next_value.clone(), ctx)
                } else {
                    ChangeOutcome::Unchanged
                };
                if change_outcome.is_canceled() {
                    return change_outcome;
                }
                let accepted =
                    !outcome.changed || !is_val_controlled || self.value_untracked() == outcome.next_value;
                if accepted && let Some(next_input) = outcome.next_input_value {
                    let _ = self.set_input_value_with_context(next_input, true, ctx);
                }
                if outcome.should_close {
                    let _ = self.set_open_with_context(false, ctx);
                }
                change_outcome
            }
            ComboboxMode::Multiple => {
                let current = self.values_untracked();
                let is_vals_controlled = self.state.values_ctl.is_controlled();
                let outcome = apply_multiple_toggle(
                    &current,
                    target_value,
                    &label,
                    self.clear_on_deselect(),
                );
                let change_outcome = if outcome.changed {
                    self.set_values_with_context(outcome.next_values.clone(), ctx)
                } else {
                    ChangeOutcome::Unchanged
                };
                if change_outcome.is_canceled() {
                    return change_outcome;
                }
                let accepted = !outcome.changed
                    || !is_vals_controlled
                    || self.values_untracked() == outcome.next_values;
                if accepted && let Some(next_input) = outcome.next_input_value {
                    let _ = self.set_input_value_with_context(next_input, true, ctx);
                }
                change_outcome
            }
        }
    }

    /// Handles a single Escape keydown on `ComboboxInput` or `ComboboxContent` with IME gating
    /// (`radix#4153`) and single-invocation deduplication (`mark_overlay_escape_consumed`).
    pub fn handle_escape_keydown(&self, evt: &KeyboardEvent) -> bool {
        if is_ime_composing_key(evt) || self.is_composing_untracked() {
            return false;
        }
        if !self.is_open_untracked() {
            return false;
        }
        let escape_behavior = self
            .state
            .escape_key_down_behavior
            .try_peek()
            .ok()
            .and_then(|g| *g)
            .unwrap_or(DismissBehavior::Close);
        if !escape_behavior.dismisses() {
            return false;
        }
        self.state.dismiss.handle_keydown(evt)
    }

    pub fn handle_input_keydown(&self, evt: &KeyboardEvent) {
        if self.is_disabled_untracked() {
            return;
        }

        // Gate Escape/Tab and navigation during IME composition (`radix#4153`, `FM-OVL-1`).
        let is_composing = is_ime_composing_key(evt) || self.is_composing_untracked();
        if is_composing {
            return;
        }

        let Some(nav_key) = translate_combobox_key(&evt.key(), evt.modifiers()) else {
            return;
        };

        let is_open = self.is_open_untracked();

        if nav_key == ComboboxNavKey::Escape {
            if is_overlay_escape_consumed() {
                // Capture-phase `use_dismissable_layer` already handled this Escape keypress;
                // prevent default on the real `KeyboardEvent` so Blitz Native does not blur `<input>`.
                evt.prevent_default();
                evt.stop_propagation();
                return;
            }
            if is_open {
                let _ = self.handle_escape_keydown(evt);
                return;
            }
        }

        if is_open {
            let cid = self.current_content_id_untracked();
            self.sync_dom_order(&cid);
        }

        if matches!(
            nav_key,
            ComboboxNavKey::ArrowDown
                | ComboboxNavKey::ArrowUp
                | ComboboxNavKey::PageDown
                | ComboboxNavKey::PageUp
        ) {
            self.set_user_has_scrolled(false);
        }

        let highlighted = self.highlighted_value_untracked();
        let selected = match self.mode_untracked() {
            ComboboxMode::Single => self.value_untracked(),
            ComboboxMode::Multiple => self.values_untracked().first().cloned(),
        };
        let input_non_empty = !self.input_value_untracked().is_empty();

        let outcome = self
            .state
            .registry
            .try_peek()
            .map(|reg| {
                resolve_input_keydown(
                    &reg,
                    is_open,
                    highlighted.as_deref(),
                    selected.as_deref(),
                    nav_key,
                    false,
                    self.loop_focus_untracked(),
                    self.is_disabled_untracked(),
                    input_non_empty,
                )
            })
            .unwrap_or_default();

        if outcome.prevent_default {
            evt.prevent_default();
        }

        if let Some(next_open) = outcome.next_open {
            if next_open && !is_open {
                let pending_mode = match nav_key {
                    ComboboxNavKey::ArrowUp => Some(ComboboxPendingHighlight::Last),
                    ComboboxNavKey::ArrowDown
                    | ComboboxNavKey::Printable
                    | ComboboxNavKey::Backspace => Some(ComboboxPendingHighlight::First),
                    _ => None,
                };
                if let Some(mode) = pending_mode
                    && outcome
                        .next_highlighted
                        .as_ref()
                        .and_then(|o| o.as_ref())
                        .is_none()
                {
                    let mut p = self.state.pending_open_highlight;
                    let _ = p.try_write().map(|mut w| *w = Some(mode));
                }
            }
            self.set_open(next_open);
        }

        if let Some(next_hl) = outcome.next_highlighted {
            if next_hl.is_some() {
                let mut p = self.state.pending_open_highlight;
                let _ = p.try_write().map(|mut w| *w = None);
            }
            self.set_highlighted(next_hl, true);
        }

        if outcome.trigger_select && let Some(hl) = highlighted {
            self.select_item(&hl);
        }
    }

    pub fn record_pressed_item(&self, value: &str, client_x: f64, client_y: f64) {
        let mut sig = self.state.pressed_item;
        let _ = sig.try_write().map(|mut w| {
            *w = Some(PressedComboboxItem {
                value: value.to_owned(),
                client_x,
                client_y,
            })
        });
    }

    pub fn take_pressed_item(&self) -> Option<PressedComboboxItem> {
        let mut sig = self.state.pressed_item;
        sig.try_write().ok().and_then(|mut w| w.take())
    }

    pub fn handle_form_reset(&self) {
        let accepted = match self.mode_untracked() {
            ComboboxMode::Single => {
                let is_val_controlled = self.state.value_ctl.is_controlled();
                let default_val = self
                    .state
                    .default_value_initial
                    .try_peek()
                    .ok()
                    .and_then(|g| g.clone());
                self.set_value(default_val.clone());
                !is_val_controlled || self.value_untracked() == default_val
            }
            ComboboxMode::Multiple => {
                let is_vals_controlled = self.state.values_ctl.is_controlled();
                let default_vals = self
                    .state
                    .default_values_initial
                    .try_peek()
                    .map(|g| g.clone())
                    .unwrap_or_default();
                self.set_values(default_vals.clone());
                !is_vals_controlled || self.values_untracked() == default_vals
            }
        };
        if accepted {
            let default_input = self
                .state
                .default_input_value_initial
                .try_peek()
                .map(|g| g.clone())
                .unwrap_or_default();
            self.set_input_value(default_input, true);
        }
    }

    pub fn refresh_scroll_buttons(&self) {
        let vp_id = self.current_viewport_id_untracked();
        let (up, down) = if let Some(m) = get_element_scroll_metrics(&vp_id) {
            let can_up = m.scroll_top > 0.5;
            let can_down = (m.scroll_top + m.client_height) < (m.scroll_height - 0.5);
            (can_up, can_down)
        } else {
            (false, false)
        };
        let mut up_sig = self.state.can_scroll_up;
        sync_signal(&mut up_sig, up);
        let mut down_sig = self.state.can_scroll_down;
        sync_signal(&mut down_sig, down);
    }

    pub fn can_scroll_up(&self) -> bool {
        *self.state.can_scroll_up.read()
    }

    pub fn can_scroll_down(&self) -> bool {
        *self.state.can_scroll_down.read()
    }

    pub fn input_attributes(&self, id: String, disabled: bool) -> ComboboxInputAttrs {
        let is_open = self.is_open();
        let force_mount = *self.state.force_mount.read();
        let mounted = is_open || force_mount || self.is_content_mounted();
        let content_id = if mounted {
            Some(self.current_content_id())
        } else {
            None
        };
        let highlighted_id = if is_open {
            self.highlighted_item_id()
        } else {
            None
        };
        ComboboxInputAttrs::new(
            id,
            content_id,
            is_open,
            highlighted_id,
            disabled || self.is_disabled(),
        )
        .with_required(self.is_required())
        .with_invalid(self.is_invalid())
        .with_aria_describedby(self.root_aria_describedby())
    }

    pub fn trigger_attributes(&self, id: String, disabled: bool) -> ComboboxTriggerAttrs {
        let is_open = self.is_open();
        let force_mount = *self.state.force_mount.read();
        let mounted = is_open || force_mount || self.is_content_mounted();
        let content_id = if mounted {
            Some(self.current_content_id())
        } else {
            None
        };
        ComboboxTriggerAttrs::new(id, content_id, is_open, disabled || self.is_disabled())
            .with_required(self.is_required())
            .with_invalid(self.is_invalid())
            .with_aria_describedby(self.root_aria_describedby())
    }

    pub fn content_attributes(
        &self,
        id: String,
        side: PlacementSide,
        align: PlacementAlign,
    ) -> ComboboxContentAttrs {
        ComboboxContentAttrs::new(id, self.mode(), self.is_open(), side, align)
    }

    pub fn viewport_attributes(&self, id: String) -> ComboboxViewportAttrs {
        ComboboxViewportAttrs::new(id)
    }

    pub fn item_attributes(
        &self,
        id: String,
        value: &str,
        label: &str,
        disabled: bool,
    ) -> ComboboxItemAttrs {
        let selected = self.is_selected(value);
        let highlighted = self.highlighted_value().as_deref() == Some(value);
        ComboboxItemAttrs::new(id, value, label, selected, highlighted, disabled)
    }

    pub fn placement(&self) -> Option<FloatingPlacement> {
        self.state.floating.placement()
    }

    pub fn content_readiness(&self) -> FloatingReadiness {
        self.state.floating.readiness()
    }

    pub fn content_z_index(&self) -> Option<String> {
        self.state.floating.content_z_index()
    }

    pub fn sync_content_z_index(&self) {
        self.state.floating.sync_z_index();
    }

    pub fn clear_content_z_index(&self) {
        self.state.floating.clear_z_index();
    }

    pub fn side(&self) -> PlacementSide {
        self.placement()
            .map(|p| p.side())
            .unwrap_or_else(|| self.preferred_side())
    }

    pub fn align(&self) -> PlacementAlign {
        self.placement()
            .map(|p| p.align())
            .unwrap_or_else(|| self.preferred_align())
    }

    pub fn preferred_side(&self) -> PlacementSide {
        self.state
            .preferred_side
            .try_peek()
            .map(|g| *g)
            .unwrap_or(PlacementSide::Bottom)
    }

    pub fn preferred_align(&self) -> PlacementAlign {
        self.state
            .preferred_align
            .try_peek()
            .map(|g| *g)
            .unwrap_or(PlacementAlign::Start)
    }

    pub fn side_offset(&self) -> f32 {
        self.state.side_offset.try_peek().map(|g| *g).unwrap_or(4.0)
    }

    pub fn align_offset(&self) -> f32 {
        self.state
            .align_offset
            .try_peek()
            .map(|g| *g)
            .unwrap_or(0.0)
    }

    pub fn avoid_collisions(&self) -> bool {
        self.state
            .avoid_collisions
            .try_peek()
            .map(|g| *g)
            .unwrap_or(true)
    }

    pub fn collision_boundary(&self) -> Option<String> {
        self.state
            .collision_boundary
            .try_peek()
            .ok()
            .and_then(|g| g.clone())
    }

    pub fn collision_padding(&self) -> f32 {
        self.state
            .collision_padding
            .try_peek()
            .map(|g| *g)
            .unwrap_or(0.0)
    }

    pub fn arrow_padding(&self) -> f32 {
        self.state
            .arrow_padding
            .try_peek()
            .map(|g| *g)
            .unwrap_or(0.0)
    }

    pub fn hide_when_detached(&self) -> bool {
        self.state
            .hide_when_detached
            .try_peek()
            .map(|g| *g)
            .unwrap_or(false)
    }

    pub fn sticky(&self) -> Option<StickyStrategy> {
        self.state.sticky.try_peek().ok().and_then(|g| *g)
    }

    pub fn custom_anchor(&self) -> Option<String> {
        self.state
            .custom_anchor
            .try_peek()
            .ok()
            .and_then(|g| g.clone())
    }

    pub fn force_mount(&self) -> bool {
        self.state
            .force_mount
            .try_peek()
            .map(|g| *g)
            .unwrap_or(false)
    }

    pub fn content_css_custom_properties(&self) -> String {
        let vars = self
            .placement()
            .map(|placement| combobox_content_css_variables(placement.geometry()))
            .unwrap_or_default();
        serialize_combobox_css_custom_properties(&vars)
    }
}

pub fn combobox_content_css_variables(geometry: &GeometryVars) -> Vec<(String, String)> {
    let mut vars: Vec<(String, String)> = geometry.css_iter().collect();
    vars.push((
        "--monoxus-combobox-anchor-width".to_string(),
        format!("{}px", geometry.anchor_width()),
    ));
    vars.push((
        "--monoxus-combobox-anchor-height".to_string(),
        format!("{}px", geometry.anchor_height()),
    ));
    vars.push((
        "--monoxus-combobox-content-available-width".to_string(),
        format!("{}px", geometry.available_width()),
    ));
    vars.push((
        "--monoxus-combobox-content-available-height".to_string(),
        format!("{}px", geometry.available_height()),
    ));
    vars.push((
        "--bits-combobox-anchor-width".to_string(),
        format!("{}px", geometry.anchor_width()),
    ));
    vars.push((
        "--bits-combobox-anchor-height".to_string(),
        format!("{}px", geometry.anchor_height()),
    ));
    vars.push((
        "--bits-combobox-content-available-width".to_string(),
        format!("{}px", geometry.available_width()),
    ));
    vars.push((
        "--bits-combobox-content-available-height".to_string(),
        format!("{}px", geometry.available_height()),
    ));
    vars
}

pub fn translate_combobox_key(key: &Key, modifiers: Modifiers) -> Option<ComboboxNavKey> {
    let has_alt = modifiers.contains(Modifiers::ALT);
    let has_ctrl_or_meta =
        modifiers.contains(Modifiers::CONTROL) || modifiers.contains(Modifiers::META);

    match key {
        Key::Named(NamedKey::ArrowDown) => Some(ComboboxNavKey::ArrowDown),
        Key::Named(NamedKey::ArrowUp) if has_alt => Some(ComboboxNavKey::AltArrowUp),
        Key::Named(NamedKey::ArrowUp) => Some(ComboboxNavKey::ArrowUp),
        Key::Named(NamedKey::PageDown) => Some(ComboboxNavKey::PageDown),
        Key::Named(NamedKey::PageUp) => Some(ComboboxNavKey::PageUp),
        Key::Named(NamedKey::Enter) => Some(ComboboxNavKey::Enter),
        Key::Named(NamedKey::Tab) => Some(ComboboxNavKey::Tab),
        Key::Named(NamedKey::Escape) => Some(ComboboxNavKey::Escape),
        Key::Named(NamedKey::Home) => Some(ComboboxNavKey::Home),
        Key::Named(NamedKey::End) => Some(ComboboxNavKey::End),
        Key::Named(NamedKey::Backspace) => Some(ComboboxNavKey::Backspace),
        Key::Character(s) if !has_ctrl_or_meta && !has_alt && !s.is_empty() => {
            Some(ComboboxNavKey::Printable)
        }
        _ => None,
    }
}

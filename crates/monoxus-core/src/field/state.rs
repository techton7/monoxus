use super::{relationships::FieldRelationships, types::FieldOrientation};

/// Immutable configuration for a field container (`FieldRoot` or `FieldSet`).
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct FieldDefinition {
    pub relationships: FieldRelationships,
    pub name: Option<String>,
    pub invalid: bool,
    pub disabled: bool,
    pub required: bool,
    pub read_only: bool,
    pub orientation: FieldOrientation,
    pub is_fieldset: bool,
}

impl FieldDefinition {
    pub fn new(relationships: FieldRelationships) -> Self {
        Self {
            relationships,
            name: None,
            invalid: false,
            disabled: false,
            required: false,
            read_only: false,
            orientation: FieldOrientation::default(),
            is_fieldset: false,
        }
    }

    pub fn with_name(mut self, name: impl Into<String>) -> Self {
        let name = name.into();
        self.name = (!name.is_empty()).then_some(name);
        self
    }

    pub fn with_optional_name(mut self, name: Option<String>) -> Self {
        self.name = name.filter(|n| !n.is_empty());
        self
    }

    pub const fn with_invalid(mut self, invalid: bool) -> Self {
        self.invalid = invalid;
        self
    }

    pub const fn with_disabled(mut self, disabled: bool) -> Self {
        self.disabled = disabled;
        self
    }

    pub const fn with_required(mut self, required: bool) -> Self {
        self.required = required;
        self
    }

    pub const fn with_read_only(mut self, read_only: bool) -> Self {
        self.read_only = read_only;
        self
    }

    pub const fn with_orientation(mut self, orientation: FieldOrientation) -> Self {
        self.orientation = orientation;
        self
    }

    pub const fn with_fieldset(mut self, is_fieldset: bool) -> Self {
        self.is_fieldset = is_fieldset;
        self
    }

    pub const fn with_is_fieldset(self, is_fieldset: bool) -> Self {
        self.with_fieldset(is_fieldset)
    }

    pub fn relationships(&self) -> &FieldRelationships {
        &self.relationships
    }

    pub fn name(&self) -> Option<&str> {
        self.name.as_deref()
    }

    pub const fn is_invalid(&self) -> bool {
        self.invalid
    }

    pub const fn is_disabled(&self) -> bool {
        self.disabled
    }

    pub const fn is_required(&self) -> bool {
        self.required
    }

    pub const fn is_read_only(&self) -> bool {
        self.read_only
    }

    pub const fn orientation(&self) -> FieldOrientation {
        self.orientation
    }

    pub const fn is_fieldset(&self) -> bool {
        self.is_fieldset
    }
}

/// Mounted `FieldError` registration entry.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct FieldErrorRegistration {
    pub id: String,
    pub active: bool,
}

impl FieldErrorRegistration {
    pub fn new(id: impl Into<String>, active: bool) -> Self {
        Self {
            id: id.into(),
            active,
        }
    }
}

/// Appends unique whitespace-separated ID tokens into `out` preserving first-seen order.
pub fn push_unique_id_tokens(out: &mut Vec<String>, raw: &str) {
    for token in raw.split_ascii_whitespace() {
        if !token.is_empty() && !out.iter().any(|existing| existing == token) {
            out.push(token.to_owned());
        }
    }
}

/// Resolves mounted-only `aria-describedby` tokens:
/// 1. Deduped mounted `description_ids`
/// 2. Deduped mounted `error.id` strictly when `error.active || field_invalid`
/// 3. Deduped `extra` tokens
///
/// Returns `None` when empty so no dangling or blank `aria-describedby` is ever emitted.
pub fn resolve_aria_describedby(
    description_ids: &[String],
    errors: &[FieldErrorRegistration],
    field_invalid: bool,
    extra: Option<&str>,
) -> Option<String> {
    let mut tokens: Vec<String> = Vec::new();

    for desc_id in description_ids {
        push_unique_id_tokens(&mut tokens, desc_id);
    }

    for err in errors {
        if err.active || field_invalid {
            push_unique_id_tokens(&mut tokens, &err.id);
        }
    }

    if let Some(extra_raw) = extra {
        push_unique_id_tokens(&mut tokens, extra_raw);
    }

    (!tokens.is_empty()).then(|| tokens.join(" "))
}

/// Deduplicates non-empty error messages while preserving first-seen order.
pub fn dedupe_error_messages(messages: &[String]) -> Vec<String> {
    let mut unique: Vec<String> = Vec::new();
    for msg in messages {
        let trimmed = msg.trim();
        if trimmed.is_empty() {
            continue;
        }
        if !unique.iter().any(|existing| existing == trimmed) {
            unique.push(trimmed.to_owned());
        }
    }
    unique
}

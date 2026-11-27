use std::borrow::Cow;

use crate::foundation::shared::ScopeHandle;

/// Deterministic, collision-safe ARIA/DOM ID relationships for a `FieldRoot` or `FieldSet`.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct FieldRelationships {
    scope: ScopeHandle,
    root_id: String,
    fieldset_id: String,
    legend_id: String,
    control_id: String,
    label_id: String,
    description_id: String,
    error_id: String,
}

impl FieldRelationships {
    /// Creates relationships backed by `ScopeHandle::root("field").child(stable_id)`.
    pub fn new(stable_id: impl Into<Cow<'static, str>>) -> Self {
        Self::from_scope(ScopeHandle::root("field").child(stable_id))
    }

    /// Creates relationships directly from an existing [`ScopeHandle`].
    pub fn from_scope(scope: ScopeHandle) -> Self {
        Self {
            root_id: scope.token(),
            fieldset_id: scope.qualify("fieldset"),
            legend_id: scope.qualify("legend"),
            control_id: scope.qualify("control"),
            label_id: scope.qualify("label"),
            description_id: scope.qualify("description"),
            error_id: scope.qualify("error"),
            scope,
        }
    }

    pub fn scope(&self) -> &ScopeHandle {
        &self.scope
    }

    pub fn root_id(&self) -> &str {
        &self.root_id
    }

    pub fn fieldset_id(&self) -> &str {
        &self.fieldset_id
    }

    pub fn legend_id(&self) -> &str {
        &self.legend_id
    }

    pub fn control_id(&self) -> &str {
        &self.control_id
    }

    /// Deterministic control ID for 1:N `FieldControl` slots (`index == 0` yields [`Self::control_id`]).
    pub fn control_id_indexed(&self, index: usize) -> String {
        if index == 0 {
            self.control_id.clone()
        } else {
            self.scope.qualify(format!("control-{index}"))
        }
    }

    pub fn label_id(&self) -> &str {
        &self.label_id
    }

    /// Deterministic label ID for 1:N `FieldControl` slots (`index == 0` yields [`Self::label_id`]).
    pub fn label_id_indexed(&self, index: usize) -> String {
        if index == 0 {
            self.label_id.clone()
        } else {
            self.scope.qualify(format!("label-{index}"))
        }
    }

    pub fn description_id(&self) -> &str {
        &self.description_id
    }

    pub fn error_id(&self) -> &str {
        &self.error_id
    }

    pub fn with_root_id(mut self, id: impl Into<String>) -> Self {
        self.root_id = normalize_id_override(id, || self.scope.token());
        self
    }

    pub fn with_fieldset_id(mut self, id: impl Into<String>) -> Self {
        self.fieldset_id = normalize_id_override(id, || self.scope.qualify("fieldset"));
        self
    }

    pub fn with_legend_id(mut self, id: impl Into<String>) -> Self {
        self.legend_id = normalize_id_override(id, || self.scope.qualify("legend"));
        self
    }

    pub fn with_control_id(mut self, id: impl Into<String>) -> Self {
        self.control_id = normalize_id_override(id, || self.scope.qualify("control"));
        self
    }

    pub fn with_label_id(mut self, id: impl Into<String>) -> Self {
        self.label_id = normalize_id_override(id, || self.scope.qualify("label"));
        self
    }

    pub fn with_description_id(mut self, id: impl Into<String>) -> Self {
        self.description_id = normalize_id_override(id, || self.scope.qualify("description"));
        self
    }

    pub fn with_error_id(mut self, id: impl Into<String>) -> Self {
        self.error_id = normalize_id_override(id, || self.scope.qualify("error"));
        self
    }
}

fn normalize_id_override(candidate: impl Into<String>, fallback: impl FnOnce() -> String) -> String {
    let candidate = candidate.into();
    if candidate.trim().is_empty() {
        fallback()
    } else {
        candidate
    }
}

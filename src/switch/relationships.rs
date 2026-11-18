use crate::foundation::shared::ScopeHandle;

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct SwitchRelationships {
    scope: ScopeHandle,
    root_id: String,
    thumb_id: String,
    hidden_input_id: String,
}

impl SwitchRelationships {
    pub fn new(scope: ScopeHandle) -> Self {
        Self {
            root_id: scope.token(),
            thumb_id: scope.qualify("thumb"),
            hidden_input_id: scope.qualify("input"),
            scope,
        }
    }

    pub fn with_explicit_root_id(scope: ScopeHandle, root_id: impl Into<String>) -> Self {
        let root_id = root_id.into();
        Self {
            thumb_id: format!("{}-thumb", root_id),
            hidden_input_id: format!("{}-input", root_id),
            root_id,
            scope,
        }
    }

    pub fn scope(&self) -> &ScopeHandle {
        &self.scope
    }

    pub fn root_id(&self) -> &str {
        &self.root_id
    }

    pub fn thumb_id(&self) -> &str {
        &self.thumb_id
    }

    pub fn hidden_input_id(&self) -> &str {
        &self.hidden_input_id
    }
}

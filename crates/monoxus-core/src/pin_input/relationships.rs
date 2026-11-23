use crate::foundation::shared::ScopeHandle;

/// Deterministic DOM ID builder for `PinInputRoot`, internal `<input>`, and `PinInputCell` parts.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct PinInputRelationships {
    scope: ScopeHandle,
    root_id: String,
    input_id: String,
    hidden_input_id: String,
}

impl PinInputRelationships {
    pub fn new(scope: ScopeHandle) -> Self {
        Self {
            root_id: scope.token(),
            input_id: scope.qualify("input"),
            hidden_input_id: scope.qualify("hidden-input"),
            scope,
        }
    }

    pub fn scope(&self) -> &ScopeHandle {
        &self.scope
    }

    pub fn root_id(&self) -> &str {
        &self.root_id
    }

    pub fn input_id(&self) -> &str {
        &self.input_id
    }

    pub fn hidden_input_id(&self) -> &str {
        &self.hidden_input_id
    }

    pub fn cell_id(&self, index: usize) -> String {
        self.scope.qualify(format!("cell-{index}"))
    }
}

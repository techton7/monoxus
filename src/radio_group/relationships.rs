use crate::foundation::shared::ScopeHandle;

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct RadioGroupRelationships {
    scope: ScopeHandle,
    root_id: String,
}

impl RadioGroupRelationships {
    pub fn new(scope: ScopeHandle) -> Self {
        Self {
            root_id: scope.token(),
            scope,
        }
    }

    pub fn scope(&self) -> &ScopeHandle {
        &self.scope
    }

    pub fn root_id(&self) -> &str {
        &self.root_id
    }

    pub fn item_id(&self, value: &str) -> String {
        self.scope.qualify(&format!("item-{value}"))
    }

    pub fn indicator_id(&self, value: &str) -> String {
        self.scope.qualify(&format!("indicator-{value}"))
    }

    pub fn hidden_input_id(&self, value: &str) -> String {
        self.scope.qualify(&format!("hidden-{value}"))
    }
}

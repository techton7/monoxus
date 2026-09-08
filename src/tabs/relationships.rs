use crate::foundation::shared::ScopeHandle;

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct TabsRelationships {
    scope: ScopeHandle,
    root_id: String,
    list_id: String,
}

impl TabsRelationships {
    pub fn new(scope: ScopeHandle) -> Self {
        Self {
            root_id: scope.token(),
            list_id: scope.qualify("list"),
            scope,
        }
    }

    pub fn scope(&self) -> &ScopeHandle {
        &self.scope
    }

    pub fn root_id(&self) -> &str {
        &self.root_id
    }

    pub fn list_id(&self) -> &str {
        &self.list_id
    }

    pub fn trigger_id(&self, value: &str) -> String {
        self.scope.qualify(&format!("trigger-{value}"))
    }

    pub fn content_id(&self, value: &str) -> String {
        self.scope.qualify(&format!("content-{value}"))
    }
}

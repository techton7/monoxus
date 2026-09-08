use crate::foundation::shared::ScopeHandle;

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct AccordionRelationships {
    scope: ScopeHandle,
    root_id: String,
}

impl AccordionRelationships {
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

    pub fn header_id(&self, value: &str) -> String {
        self.scope.qualify(&format!("header-{value}"))
    }

    pub fn trigger_id(&self, value: &str) -> String {
        self.scope.qualify(&format!("trigger-{value}"))
    }

    pub fn content_id(&self, value: &str) -> String {
        self.scope.qualify(&format!("content-{value}"))
    }
}

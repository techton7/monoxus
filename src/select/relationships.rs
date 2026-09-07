use crate::foundation::shared::ScopeHandle;

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct SelectRelationships {
    scope: ScopeHandle,
    root_id: String,
    trigger_id: String,
    content_id: String,
}

impl SelectRelationships {
    pub fn new(scope: ScopeHandle) -> Self {
        Self {
            root_id: scope.token(),
            trigger_id: scope.qualify("trigger"),
            content_id: scope.qualify("content"),
            scope,
        }
    }

    pub fn scope(&self) -> &ScopeHandle {
        &self.scope
    }

    pub fn root_id(&self) -> &str {
        &self.root_id
    }

    pub fn trigger_id(&self) -> &str {
        &self.trigger_id
    }

    pub fn content_id(&self) -> &str {
        &self.content_id
    }

    pub fn item_id(&self, value: &str) -> String {
        self.scope.qualify(&format!("item-{value}"))
    }

    pub fn group_id(&self, group_key: &str) -> String {
        self.scope.qualify(&format!("group-{group_key}"))
    }

    pub fn label_id(&self, group_key: &str) -> String {
        self.scope.qualify(&format!("label-{group_key}"))
    }
}

use crate::foundation::shared::ScopeHandle;

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct ToolbarRelationships {
    scope: ScopeHandle,
    root_id: String,
}

impl ToolbarRelationships {
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

    pub fn button_id(&self, key: &str) -> String {
        self.scope.qualify(&format!("button-{key}"))
    }

    pub fn link_id(&self, key: &str) -> String {
        self.scope.qualify(&format!("link-{key}"))
    }

    pub fn separator_id(&self, key: &str) -> String {
        self.scope.qualify(&format!("separator-{key}"))
    }

    pub fn toggle_group_id(&self, key: &str) -> String {
        self.scope.qualify(&format!("toggle-group-{key}"))
    }

    pub fn toggle_item_id(&self, group_key: &str, value: &str) -> String {
        self.scope.qualify(&format!("toggle-{group_key}-{value}"))
    }
}

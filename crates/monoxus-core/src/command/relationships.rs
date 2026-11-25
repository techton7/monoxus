use crate::foundation::shared::ScopeHandle;

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct CommandRelationships {
    scope: ScopeHandle,
}

impl CommandRelationships {
    pub fn new(scope: ScopeHandle) -> Self {
        Self { scope }
    }

    pub fn scope(&self) -> &ScopeHandle {
        &self.scope
    }

    pub fn root_id(&self) -> String {
        self.scope.token()
    }

    pub fn label_id(&self) -> String {
        self.scope.qualify("label")
    }

    pub fn input_id(&self) -> String {
        self.scope.qualify("input")
    }

    pub fn list_id(&self) -> String {
        self.scope.qualify("list")
    }

    pub fn viewport_id(&self) -> String {
        self.scope.qualify("viewport")
    }

    pub fn empty_id(&self) -> String {
        self.scope.qualify("empty")
    }

    pub fn loading_id(&self) -> String {
        self.scope.qualify("loading")
    }

    pub fn group_id(&self, group_key: &str) -> String {
        self.scope.qualify(format!("group-{group_key}"))
    }

    pub fn group_heading_id(&self, group_key: &str) -> String {
        self.scope.qualify(format!("group-heading-{group_key}"))
    }

    pub fn group_items_id(&self, group_key: &str) -> String {
        self.scope.qualify(format!("group-items-{group_key}"))
    }

    pub fn item_id(&self, item_key: &str) -> String {
        self.scope.qualify(format!("item-{item_key}"))
    }

    pub fn separator_id(&self, key: &str) -> String {
        self.scope.qualify(format!("separator-{key}"))
    }
}

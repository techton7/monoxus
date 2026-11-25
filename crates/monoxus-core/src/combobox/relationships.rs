use crate::foundation::shared::ScopeHandle;

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct ComboboxRelationships {
    scope: ScopeHandle,
    root_id: String,
    input_id: String,
    trigger_id: String,
    content_id: String,
    viewport_id: String,
}

impl ComboboxRelationships {
    pub fn new(scope: ScopeHandle) -> Self {
        Self {
            root_id: scope.token(),
            input_id: scope.qualify("input"),
            trigger_id: scope.qualify("trigger"),
            content_id: scope.qualify("content"),
            viewport_id: scope.qualify("viewport"),
            scope,
        }
    }

    pub fn with_input_id(mut self, input_id: impl Into<String>) -> Self {
        self.input_id = input_id.into();
        self
    }

    pub fn with_trigger_id(mut self, trigger_id: impl Into<String>) -> Self {
        self.trigger_id = trigger_id.into();
        self
    }

    pub fn with_content_id(mut self, content_id: impl Into<String>) -> Self {
        self.content_id = content_id.into();
        self
    }

    pub fn with_viewport_id(mut self, viewport_id: impl Into<String>) -> Self {
        self.viewport_id = viewport_id.into();
        self
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

    pub fn trigger_id(&self) -> &str {
        &self.trigger_id
    }

    pub fn content_id(&self) -> &str {
        &self.content_id
    }

    pub fn viewport_id(&self) -> &str {
        &self.viewport_id
    }

    pub fn group_id(&self, group_key: &str) -> String {
        self.scope.qualify(format!("group-{group_key}"))
    }

    pub fn group_heading_id(&self, group_key: &str) -> String {
        self.scope.qualify(format!("group-heading-{group_key}"))
    }

    pub fn item_id(&self, item_key: &str) -> String {
        self.scope.qualify(format!("item-{item_key}"))
    }
}

use crate::foundation::shared::ScopeHandle;

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct HoverCardRelationships {
    scope: ScopeHandle,
    root_id: String,
    trigger_id: String,
    portal_id: String,
    content_id: String,
    wrapper_id: String,
    arrow_id: String,
}

impl HoverCardRelationships {
    pub fn new(scope: ScopeHandle) -> Self {
        Self {
            root_id: scope.token(),
            trigger_id: scope.qualify("trigger"),
            portal_id: scope.qualify("portal"),
            content_id: scope.qualify("content"),
            wrapper_id: scope.qualify("wrapper"),
            arrow_id: scope.qualify("arrow"),
            scope,
        }
    }

    pub fn with_trigger_id(mut self, trigger_id: impl Into<String>) -> Self {
        self.trigger_id = trigger_id.into();
        self
    }

    pub fn with_content_id(mut self, content_id: impl Into<String>) -> Self {
        self.content_id = content_id.into();
        self
    }

    pub fn with_arrow_id(mut self, arrow_id: impl Into<String>) -> Self {
        self.arrow_id = arrow_id.into();
        self
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

    pub fn portal_id(&self) -> &str {
        &self.portal_id
    }

    pub fn content_id(&self) -> &str {
        &self.content_id
    }

    pub fn wrapper_id(&self) -> &str {
        &self.wrapper_id
    }

    pub fn arrow_id(&self) -> &str {
        &self.arrow_id
    }
}

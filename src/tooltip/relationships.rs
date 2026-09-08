use crate::foundation::shared::ScopeHandle;

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct TooltipRelationships {
    scope: ScopeHandle,
    root_id: String,
    trigger_id: String,
    content_id: String,
    arrow_id: String,
}

impl TooltipRelationships {
    pub fn new(scope: ScopeHandle) -> Self {
        Self {
            root_id: scope.token(),
            trigger_id: scope.qualify("trigger"),
            content_id: scope.qualify("content"),
            arrow_id: scope.qualify("arrow"),
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

    pub fn arrow_id(&self) -> &str {
        &self.arrow_id
    }
}

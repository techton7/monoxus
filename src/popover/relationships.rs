use crate::foundation::shared::ScopeHandle;

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct PopoverRelationships {
    scope: ScopeHandle,
    root_id: String,
    trigger_id: String,
    anchor_id: String,
    content_id: String,
    arrow_id: String,
    close_id: String,
}

impl PopoverRelationships {
    pub fn new(scope: ScopeHandle) -> Self {
        Self {
            root_id: scope.token(),
            trigger_id: scope.qualify("trigger"),
            anchor_id: scope.qualify("anchor"),
            content_id: scope.qualify("content"),
            arrow_id: scope.qualify("arrow"),
            close_id: scope.qualify("close"),
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

    pub fn anchor_id(&self) -> &str {
        &self.anchor_id
    }

    pub fn content_id(&self) -> &str {
        &self.content_id
    }

    pub fn arrow_id(&self) -> &str {
        &self.arrow_id
    }

    pub fn close_id(&self) -> &str {
        &self.close_id
    }
}

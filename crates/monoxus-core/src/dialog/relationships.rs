use crate::foundation::shared::ScopeHandle;

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct DialogRelationships {
    scope: ScopeHandle,
    root_id: String,
    trigger_id: String,
    overlay_id: String,
    content_id: String,
    title_id: String,
    description_id: String,
    close_id: String,
    focus_guard_before_id: String,
    focus_guard_after_id: String,
}

impl DialogRelationships {
    pub fn new(scope: ScopeHandle) -> Self {
        Self {
            root_id: scope.token(),
            trigger_id: scope.qualify("trigger"),
            overlay_id: scope.qualify("overlay"),
            content_id: scope.qualify("content"),
            title_id: scope.qualify("title"),
            description_id: scope.qualify("description"),
            close_id: scope.qualify("close"),
            focus_guard_before_id: scope.qualify("focus-guard-before"),
            focus_guard_after_id: scope.qualify("focus-guard-after"),
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

    pub fn overlay_id(&self) -> &str {
        &self.overlay_id
    }

    pub fn content_id(&self) -> &str {
        &self.content_id
    }

    pub fn title_id(&self) -> &str {
        &self.title_id
    }

    pub fn description_id(&self) -> &str {
        &self.description_id
    }

    pub fn close_id(&self) -> &str {
        &self.close_id
    }

    pub fn focus_guard_before_id(&self) -> &str {
        &self.focus_guard_before_id
    }

    pub fn focus_guard_after_id(&self) -> &str {
        &self.focus_guard_after_id
    }
}

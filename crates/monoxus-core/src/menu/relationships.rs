use crate::foundation::shared::ScopeHandle;

/// Deterministic DOM/ARIA ID relationships for a root menu instance and its owned parts.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct MenuRelationships {
    scope: ScopeHandle,
    root_id: String,
    trigger_id: String,
    anchor_id: String,
    portal_id: String,
    content_id: String,
    wrapper_id: String,
    arrow_id: String,
}

impl MenuRelationships {
    pub fn new(scope: ScopeHandle) -> Self {
        Self {
            root_id: scope.token(),
            trigger_id: scope.qualify("trigger"),
            anchor_id: scope.qualify("anchor"),
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

    pub fn with_anchor_id(mut self, anchor_id: impl Into<String>) -> Self {
        self.anchor_id = anchor_id.into();
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

    pub fn anchor_id(&self) -> &str {
        &self.anchor_id
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

    pub fn sub(&self, child_scope: ScopeHandle) -> MenuSubRelationships {
        MenuSubRelationships::new(child_scope)
    }

    pub fn group(&self, group_scope: ScopeHandle) -> MenuGroupRelationships {
        MenuGroupRelationships::new(group_scope)
    }
}

/// Deterministic DOM/ARIA ID relationships for a nested `Sub` menu (`SubTrigger` <-> `SubContent`).
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct MenuSubRelationships {
    scope: ScopeHandle,
    root_id: String,
    trigger_id: String,
    portal_id: String,
    content_id: String,
    wrapper_id: String,
    arrow_id: String,
}

impl MenuSubRelationships {
    pub fn new(scope: ScopeHandle) -> Self {
        Self {
            root_id: scope.token(),
            trigger_id: scope.qualify("sub-trigger"),
            portal_id: scope.qualify("sub-portal"),
            content_id: scope.qualify("sub-content"),
            wrapper_id: scope.qualify("sub-wrapper"),
            arrow_id: scope.qualify("sub-arrow"),
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

/// Deterministic DOM/ARIA ID relationships pairing `Group` (`aria-labelledby`) with `GroupHeading`.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct MenuGroupRelationships {
    scope: ScopeHandle,
    group_id: String,
    heading_id: String,
}

impl MenuGroupRelationships {
    pub fn new(scope: ScopeHandle) -> Self {
        Self {
            group_id: scope.qualify("group"),
            heading_id: scope.qualify("group-heading"),
            scope,
        }
    }

    pub fn with_group_id(mut self, group_id: impl Into<String>) -> Self {
        self.group_id = group_id.into();
        self
    }

    pub fn with_heading_id(mut self, heading_id: impl Into<String>) -> Self {
        self.heading_id = heading_id.into();
        self
    }

    pub fn scope(&self) -> &ScopeHandle {
        &self.scope
    }

    pub fn group_id(&self) -> &str {
        &self.group_id
    }

    pub fn heading_id(&self) -> &str {
        &self.heading_id
    }
}

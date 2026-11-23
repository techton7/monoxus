use crate::foundation::shared::ScopeHandle;

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct NavigationMenuRelationships {
    scope: ScopeHandle,
    root_id: String,
    list_id: String,
    track_id: String,
    indicator_id: String,
    viewport_id: String,
}

impl NavigationMenuRelationships {
    pub fn new(scope: ScopeHandle) -> Self {
        Self {
            root_id: scope.token(),
            list_id: scope.qualify("list"),
            track_id: scope.qualify("track"),
            indicator_id: scope.qualify("indicator"),
            viewport_id: scope.qualify("viewport"),
            scope,
        }
    }

    pub fn scope(&self) -> &ScopeHandle {
        &self.scope
    }

    pub fn root_id(&self) -> &str {
        &self.root_id
    }

    pub fn list_id(&self) -> &str {
        &self.list_id
    }

    pub fn track_id(&self) -> &str {
        &self.track_id
    }

    pub fn indicator_id(&self) -> &str {
        &self.indicator_id
    }

    pub fn viewport_id(&self) -> &str {
        &self.viewport_id
    }

    pub fn item_id(&self, value: &str) -> String {
        self.scope.child("item").qualify(value)
    }

    pub fn trigger_id(&self, value: &str) -> String {
        self.scope.child("trigger").qualify(value)
    }

    pub fn content_id(&self, value: &str) -> String {
        self.scope.child("content").qualify(value)
    }

    pub fn focus_proxy_id(&self, value: &str) -> String {
        self.scope.child("focus-proxy").qualify(value)
    }

    pub fn aria_owns_id(&self, value: &str) -> String {
        self.scope.child("aria-owns").qualify(value)
    }

    pub fn link_id(&self, key: &str) -> String {
        self.scope.child("link").qualify(key)
    }

    pub fn sub_scope(&self, sub_key: &str) -> ScopeHandle {
        self.scope.child("sub").child(sub_key.to_owned())
    }
}

use crate::foundation::shared::ScopeHandle;

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct ProgressRelationships {
    scope: ScopeHandle,
    root_id: String,
    indicator_id: String,
}

impl ProgressRelationships {
    pub fn new(scope: ScopeHandle) -> Self {
        Self {
            root_id: scope.token(),
            indicator_id: scope.qualify("indicator"),
            scope,
        }
    }

    pub fn with_indicator_id(mut self, indicator_id: impl Into<String>) -> Self {
        self.indicator_id = indicator_id.into();
        self
    }

    pub fn with_root_id(mut self, root_id: impl Into<String>) -> Self {
        self.root_id = root_id.into();
        self
    }

    pub fn scope(&self) -> &ScopeHandle {
        &self.scope
    }

    pub fn root_id(&self) -> &str {
        &self.root_id
    }

    pub fn indicator_id(&self) -> &str {
        &self.indicator_id
    }
}

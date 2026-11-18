use crate::foundation::shared::ScopeHandle;

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct CheckboxRelationships {
    scope: ScopeHandle,
    root_id: String,
    indicator_id: String,
    hidden_input_id: String,
}

impl CheckboxRelationships {
    pub fn new(scope: ScopeHandle) -> Self {
        Self {
            root_id: scope.token(),
            indicator_id: scope.qualify("indicator"),
            hidden_input_id: scope.qualify("input"),
            scope,
        }
    }

    pub fn with_explicit_root_id(scope: ScopeHandle, root_id: impl Into<String>) -> Self {
        let root_id = root_id.into();
        Self {
            indicator_id: format!("{}-indicator", root_id),
            hidden_input_id: format!("{}-input", root_id),
            root_id,
            scope,
        }
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

    pub fn hidden_input_id(&self) -> &str {
        &self.hidden_input_id
    }
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct CheckboxGroupRelationships {
    scope: ScopeHandle,
    group_id: String,
    group_label_id: String,
}

impl CheckboxGroupRelationships {
    pub fn new(scope: ScopeHandle) -> Self {
        Self {
            group_id: scope.token(),
            group_label_id: scope.qualify("label"),
            scope,
        }
    }

    pub fn with_explicit_group_id(scope: ScopeHandle, group_id: impl Into<String>) -> Self {
        let group_id = group_id.into();
        Self {
            group_label_id: format!("{}-label", group_id),
            group_id,
            scope,
        }
    }

    pub fn scope(&self) -> &ScopeHandle {
        &self.scope
    }

    pub fn group_id(&self) -> &str {
        &self.group_id
    }

    pub fn group_label_id(&self) -> &str {
        &self.group_label_id
    }
}

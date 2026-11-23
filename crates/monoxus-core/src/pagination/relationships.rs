use crate::foundation::shared::ScopeHandle;

pub const PREV_ROVING_KEY: &str = "prev";
pub const NEXT_ROVING_KEY: &str = "next";

pub fn page_roving_key(page: usize) -> String {
    format!("page-{page}")
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct PaginationRelationships {
    scope: ScopeHandle,
    root_id: String,
    prev_id: String,
    next_id: String,
}

impl PaginationRelationships {
    pub fn new(scope: ScopeHandle) -> Self {
        Self {
            root_id: scope.token(),
            prev_id: scope.qualify("prev"),
            next_id: scope.qualify("next"),
            scope,
        }
    }

    pub fn scope(&self) -> &ScopeHandle {
        &self.scope
    }

    pub fn root_id(&self) -> &str {
        &self.root_id
    }

    pub fn prev_id(&self) -> &str {
        &self.prev_id
    }

    pub fn next_id(&self) -> &str {
        &self.next_id
    }

    pub fn page_id(&self, page: usize) -> String {
        self.scope.qualify(format!("page-{page}"))
    }
}

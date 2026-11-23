use crate::foundation::shared::ScopeHandle;

/// Deterministic ARIA and DOM ID coordinator for all parts of a `RatingGroup` instance.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct RatingGroupRelationships {
    scope: ScopeHandle,
    root_id: String,
    input_id: String,
}

impl RatingGroupRelationships {
    /// Creates a new relationship coordinator scoped to the given [`ScopeHandle`].
    pub fn new(scope: ScopeHandle) -> Self {
        Self {
            root_id: scope.token(),
            input_id: scope.qualify("input"),
            scope,
        }
    }

    /// Returns the underlying scope handle.
    pub fn scope(&self) -> &ScopeHandle {
        &self.scope
    }

    /// Returns the deterministic root element ID.
    pub fn root_id(&self) -> &str {
        &self.root_id
    }

    /// Returns the deterministic hidden form input ID.
    pub fn input_id(&self) -> &str {
        &self.input_id
    }

    /// Derives the deterministic ID for an item at `index`.
    pub fn item_id(&self, index: usize) -> String {
        self.scope.qualify(&format!("item-{index}"))
    }
}

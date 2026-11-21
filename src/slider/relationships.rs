use crate::foundation::shared::ScopeHandle;

/// Deterministic ARIA and DOM ID coordinator for all compound parts of a slider instance.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct SliderRelationships {
    scope: ScopeHandle,
    root_id: String,
    track_id: String,
    range_id: String,
}

impl SliderRelationships {
    /// Creates a new relationship coordinator scoped to the given [`ScopeHandle`].
    pub fn new(scope: ScopeHandle) -> Self {
        Self {
            root_id: scope.token(),
            track_id: scope.qualify("track"),
            range_id: scope.qualify("range"),
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

    /// Returns the deterministic track element ID.
    pub fn track_id(&self) -> &str {
        &self.track_id
    }

    /// Returns the deterministic range fill element ID.
    pub fn range_id(&self) -> &str {
        &self.range_id
    }

    /// Derives the deterministic ID for a thumb at the specified index.
    pub fn thumb_id(&self, index: usize) -> String {
        self.scope.qualify(&format!("thumb-{index}"))
    }

    /// Derives the deterministic ID for a hidden form input at the specified index.
    pub fn input_id(&self, index: usize) -> String {
        self.scope.qualify(&format!("input-{index}"))
    }

    /// Derives the deterministic ID for a tick mark at the specified index.
    pub fn tick_id(&self, index: usize) -> String {
        self.scope.qualify(&format!("tick-{index}"))
    }

    /// Derives the deterministic ID for a tick label at the specified index.
    pub fn tick_label_id(&self, index: usize) -> String {
        self.scope.qualify(&format!("tick-label-{index}"))
    }

    /// Derives the deterministic ID for a thumb label at the specified index.
    pub fn thumb_label_id(&self, index: usize) -> String {
        self.scope.qualify(&format!("thumb-label-{index}"))
    }
}

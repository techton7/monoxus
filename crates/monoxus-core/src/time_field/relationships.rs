use crate::foundation::{date_time::SegmentKind, shared::ScopeHandle};

/// Deterministic ARIA ID relationships for `TimeField`.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct TimeFieldRelationships {
    scope: ScopeHandle,
    input_id: String,
    label_id: String,
    description_id: String,
    live_region_id: String,
    hidden_input_id: String,
}

impl TimeFieldRelationships {
    pub fn new(scope: ScopeHandle) -> Self {
        let input_id = scope.qualify("input");
        let label_id = scope.qualify("label");
        let description_id = scope.qualify("description");
        let live_region_id = scope.qualify("live-region");
        let hidden_input_id = scope.qualify("hidden-input");
        Self {
            scope,
            input_id,
            label_id,
            description_id,
            live_region_id,
            hidden_input_id,
        }
    }

    pub fn scope(&self) -> &ScopeHandle {
        &self.scope
    }

    pub fn input_id(&self) -> &str {
        &self.input_id
    }

    pub fn label_id(&self) -> &str {
        &self.label_id
    }

    pub fn description_id(&self) -> &str {
        &self.description_id
    }

    pub fn live_region_id(&self) -> &str {
        &self.live_region_id
    }

    pub fn hidden_input_id(&self) -> &str {
        &self.hidden_input_id
    }

    pub fn segment_id(&self, index: usize, kind: SegmentKind) -> String {
        self.scope
            .qualify(format!("segment-{index}-{}", kind.as_data_segment()))
    }
}

use crate::{foundation::shared::ScopeHandle, time_field::TimeFieldRelationships};

/// Deterministic ARIA ID relationships for `TimeRangeField`.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct TimeRangeFieldRelationships {
    scope: ScopeHandle,
    root_id: String,
    label_id: String,
    start_relationships: TimeFieldRelationships,
    end_relationships: TimeFieldRelationships,
}

impl TimeRangeFieldRelationships {
    pub fn new(scope: ScopeHandle) -> Self {
        let root_id = scope.token();
        let label_id = scope.qualify("label");
        let start_relationships = TimeFieldRelationships::new(scope.child("start"));
        let end_relationships = TimeFieldRelationships::new(scope.child("end"));
        Self {
            scope,
            root_id,
            label_id,
            start_relationships,
            end_relationships,
        }
    }

    pub fn scope(&self) -> &ScopeHandle {
        &self.scope
    }

    pub fn root_id(&self) -> &str {
        &self.root_id
    }

    pub fn label_id(&self) -> &str {
        &self.label_id
    }

    pub fn start_relationships(&self) -> &TimeFieldRelationships {
        &self.start_relationships
    }

    pub fn end_relationships(&self) -> &TimeFieldRelationships {
        &self.end_relationships
    }
}

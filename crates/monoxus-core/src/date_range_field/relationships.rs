use crate::{date_field::DateFieldRelationships, foundation::shared::ScopeHandle};

/// Deterministic ARIA ID relationships for `DateRangeField`.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct DateRangeFieldRelationships {
    scope: ScopeHandle,
    root_id: String,
    label_id: String,
    start_relationships: DateFieldRelationships,
    end_relationships: DateFieldRelationships,
}

impl DateRangeFieldRelationships {
    pub fn new(scope: ScopeHandle) -> Self {
        let root_id = scope.token();
        let label_id = scope.qualify("label");
        let start_relationships = DateFieldRelationships::new(scope.child("start"));
        let end_relationships = DateFieldRelationships::new(scope.child("end"));
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

    pub fn start_relationships(&self) -> &DateFieldRelationships {
        &self.start_relationships
    }

    pub fn end_relationships(&self) -> &DateFieldRelationships {
        &self.end_relationships
    }
}

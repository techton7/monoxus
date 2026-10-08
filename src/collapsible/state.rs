use crate::foundation::{shared::ScopeHandle, state::DataState};

use super::{
    attrs::{CollapsibleContentAttributes, CollapsibleRootAttributes, CollapsibleTriggerAttributes},
    relationships::CollapsibleRelationships,
    types::{COLLAPSIBLE_PARTS, CollapsiblePart, CollapsibleStateRequest},
};

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Collapsible {
    relationships: CollapsibleRelationships,
    open: bool,
    disabled: bool,
}

impl Collapsible {
    pub fn new(scope: ScopeHandle, initial_open: bool) -> Self {
        Self {
            relationships: CollapsibleRelationships::new(scope),
            open: initial_open,
            disabled: false,
        }
    }

    pub const fn parts() -> &'static [CollapsiblePart] {
        &COLLAPSIBLE_PARTS
    }

    pub fn relationships(&self) -> &CollapsibleRelationships {
        &self.relationships
    }

    pub fn is_open(&self) -> bool {
        self.open
    }

    pub fn is_disabled(&self) -> bool {
        self.disabled
    }

    pub fn with_disabled(mut self, disabled: bool) -> Self {
        self.disabled = disabled;
        self
    }

    pub fn with_open(mut self, open: bool) -> Self {
        self.open = open;
        self
    }

    pub fn set_open(&mut self, open: bool) -> bool {
        let changed = self.open != open;
        self.open = open;
        changed
    }

    pub fn toggle(&mut self) -> bool {
        self.open = !self.open;
        true
    }

    pub fn request_state(&mut self, request: CollapsibleStateRequest) -> bool {
        let next = request.next_open(self.open);
        self.set_open(next)
    }

    pub fn root(&self) -> CollapsibleRootAttributes {
        let data_state = if self.open {
            DataState::Open
        } else {
            DataState::Closed
        };

        CollapsibleRootAttributes {
            id: self.relationships.root_id().to_owned(),
            data_state,
            disabled: self.disabled,
        }
    }

    pub fn trigger(&self) -> CollapsibleTriggerAttributes {
        let data_state = if self.open {
            DataState::Open
        } else {
            DataState::Closed
        };

        CollapsibleTriggerAttributes {
            id: self.relationships.trigger_id().to_owned(),
            aria_controls: self.relationships.content_id().to_owned(),
            aria_expanded: if self.open { "true" } else { "false" },
            data_state,
            disabled: self.disabled,
            state_request: CollapsibleStateRequest::Toggle,
        }
    }

    pub fn content(&self) -> CollapsibleContentAttributes {
        let data_state = if self.open {
            DataState::Open
        } else {
            DataState::Closed
        };

        CollapsibleContentAttributes {
            id: self.relationships.content_id().to_owned(),
            hidden: !self.open,
            data_state,
            disabled: self.disabled,
        }
    }
}

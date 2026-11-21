use crate::foundation::{shared::ScopeHandle, state::DataState};

use super::{
    attrs::{
        CollapsibleContentAttributes, CollapsibleRootAttributes, CollapsibleTriggerAttributes,
    },
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

        let content_id = self.relationships.content_id().to_owned();
        let aria_controls = if self.open {
            Some(content_id.clone())
        } else {
            None
        };

        CollapsibleTriggerAttributes {
            id: self.relationships.trigger_id().to_owned(),
            content_id,
            aria_controls,
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

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_collapsible_parts_inventory() {
        assert_eq!(COLLAPSIBLE_PARTS.len(), 3);
        assert_eq!(CollapsiblePart::Root.as_str(), "root");
        assert_eq!(CollapsiblePart::Trigger.as_str(), "trigger");
        assert_eq!(CollapsiblePart::Content.as_str(), "content");
        assert_eq!(Collapsible::parts().len(), 3);
    }

    #[test]
    fn test_collapsible_state_request_transitions() {
        let req_open = CollapsibleStateRequest::Open;
        assert!(req_open.next_open(false));
        assert!(req_open.next_open(true));
        assert_eq!(req_open.data_state(false), DataState::Open);

        let req_close = CollapsibleStateRequest::Close;
        assert!(!req_close.next_open(true));
        assert!(!req_close.next_open(false));
        assert_eq!(req_close.data_state(true), DataState::Closed);

        let req_toggle = CollapsibleStateRequest::Toggle;
        assert!(req_toggle.next_open(false));
        assert!(!req_toggle.next_open(true));
    }

    #[test]
    fn test_collapsible_relationships_deterministic_ids() {
        let scope = ScopeHandle::root("collapsible-test").child("main");
        let relationships = CollapsibleRelationships::new(scope.clone());

        assert_eq!(relationships.scope(), &scope);
        assert_eq!(relationships.root_id(), scope.token());
        assert_eq!(relationships.trigger_id(), scope.qualify("trigger"));
        assert_eq!(relationships.content_id(), scope.qualify("content"));
        assert_eq!(relationships.root_id(), "16:collapsible-test|4:main");
        assert_eq!(
            relationships.trigger_id(),
            "16:collapsible-test|4:main|7:trigger"
        );
        assert_eq!(
            relationships.content_id(),
            "16:collapsible-test|4:main|7:content"
        );
    }

    #[test]
    fn test_collapsible_pure_state_contract() {
        let scope = ScopeHandle::root("collapsible").child("demo");
        let mut col = Collapsible::new(scope, false);
        let expected_content_id = col.relationships().content_id().to_string();

        assert!(!col.is_open());
        assert!(!col.is_disabled());

        let root_attrs = col.root();
        assert_eq!(root_attrs.data_state_str(), "closed");
        assert!(!root_attrs.is_disabled());
        assert_eq!(root_attrs.data_disabled(), None);

        let trigger_attrs = col.trigger();
        assert_eq!(trigger_attrs.data_state_str(), "closed");
        assert_eq!(trigger_attrs.aria_expanded(), "false");
        assert_eq!(trigger_attrs.target_content_id(), expected_content_id);
        assert_eq!(trigger_attrs.aria_controls(), None);
        assert!(!trigger_attrs.is_disabled());
        assert_eq!(trigger_attrs.data_disabled(), None);

        let content_attrs = col.content();
        assert_eq!(content_attrs.data_state_str(), "closed");
        assert!(content_attrs.is_hidden());
        assert!(!content_attrs.is_disabled());
        assert_eq!(content_attrs.data_disabled(), None);

        assert!(col.toggle());
        assert!(col.is_open());
        assert_eq!(col.root().data_state_str(), "open");
        assert_eq!(col.trigger().aria_expanded(), "true");
        assert_eq!(col.trigger().data_state_str(), "open");
        assert_eq!(col.trigger().target_content_id(), expected_content_id);
        assert_eq!(
            col.trigger().aria_controls(),
            Some(expected_content_id.as_str())
        );
        assert_eq!(col.content().data_state_str(), "open");
        assert!(!col.content().is_hidden());

        let disabled_col = col.with_disabled(true);
        assert!(disabled_col.is_disabled());
        assert!(disabled_col.root().is_disabled());
        assert_eq!(disabled_col.root().data_disabled(), Some(""));
        assert!(disabled_col.trigger().is_disabled());
        assert_eq!(disabled_col.trigger().data_disabled(), Some(""));
        assert!(disabled_col.content().is_disabled());
        assert_eq!(disabled_col.content().data_disabled(), Some(""));
    }
}

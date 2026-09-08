use monoxus::{
    collapsible::{
        COLLAPSIBLE_PARTS, Collapsible, CollapsiblePart, CollapsibleRelationships,
    },
    foundation::{shared::ScopeHandle, state::DataState},
};

#[test]
fn collapsible_part_inventory_matches_exhaustive_surface() {
    let parts: Vec<_> = Collapsible::parts()
        .iter()
        .map(CollapsiblePart::as_str)
        .collect();
    assert_eq!(parts, vec!["root", "trigger", "content"]);
    assert_eq!(COLLAPSIBLE_PARTS.len(), 3);
}

#[test]
fn collapsible_relationships_produce_deterministic_ids() {
    let scope = ScopeHandle::root("collapsible-test").child("main");
    let relationships = CollapsibleRelationships::new(scope.clone());

    assert_eq!(relationships.scope(), &scope);
    assert_eq!(relationships.root_id(), scope.token());
    assert_eq!(relationships.trigger_id(), scope.qualify("trigger"));
    assert_eq!(relationships.content_id(), scope.qualify("content"));
}

#[test]
fn collapsible_attributes_publish_wai_aria_and_data_attributes() {
    let scope = ScopeHandle::root("collapsible-test").child("attrs");
    let mut collapsible = Collapsible::new(scope.clone(), false);

    // Initial closed state
    let root_attrs = collapsible.root();
    assert_eq!(root_attrs.id(), scope.token());
    assert_eq!(root_attrs.data_state(), DataState::Closed);
    assert_eq!(root_attrs.data_state_str(), "closed");
    assert!(!root_attrs.is_disabled());

    let trigger_attrs = collapsible.trigger();
    assert_eq!(trigger_attrs.id(), scope.qualify("trigger"));
    assert_eq!(trigger_attrs.aria_controls(), scope.qualify("content"));
    assert_eq!(trigger_attrs.aria_expanded(), "false");
    assert_eq!(trigger_attrs.data_state(), DataState::Closed);
    assert_eq!(trigger_attrs.data_state_str(), "closed");
    assert!(!trigger_attrs.is_disabled());

    let content_attrs = collapsible.content();
    assert_eq!(content_attrs.id(), scope.qualify("content"));
    assert_eq!(content_attrs.data_state(), DataState::Closed);
    assert_eq!(content_attrs.data_state_str(), "closed");
    assert!(content_attrs.is_hidden());
    assert!(!content_attrs.is_disabled());

    // Toggle open
    let changed = collapsible.set_open(true);
    assert!(changed);
    assert!(collapsible.is_open());

    let trigger_open = collapsible.trigger();
    assert_eq!(trigger_open.aria_expanded(), "true");
    assert_eq!(trigger_open.data_state(), DataState::Open);
    assert_eq!(trigger_open.data_state_str(), "open");

    let content_open = collapsible.content();
    assert_eq!(content_open.data_state(), DataState::Open);
    assert_eq!(content_open.data_state_str(), "open");
    assert!(!content_open.is_hidden());
}

#[test]
fn collapsible_disabled_cascades_to_trigger_and_content() {
    let scope = ScopeHandle::root("collapsible-test").child("disabled");
    let collapsible = Collapsible::new(scope.clone(), true).with_disabled(true);

    assert!(collapsible.is_disabled());
    assert!(collapsible.root().is_disabled());
    assert!(collapsible.trigger().is_disabled());
    assert!(collapsible.content().is_disabled());
}

#[test]
fn collapsible_toggle_transition_idempotency() {
    let scope = ScopeHandle::root("collapsible-test").child("toggle");
    let mut collapsible = Collapsible::new(scope, false);

    assert!(!collapsible.is_open());
    assert!(collapsible.toggle());
    assert!(collapsible.is_open());
    assert!(collapsible.toggle());
    assert!(!collapsible.is_open());

    // Setting same state is idempotent
    assert!(!collapsible.set_open(false));
    assert!(!collapsible.is_open());
}

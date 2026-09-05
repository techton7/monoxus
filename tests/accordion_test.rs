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

use monoxus::accordion::{
    ACCORDION_PARTS, Accordion, AccordionDirection, AccordionItemRegistration, AccordionMode,
    AccordionOrientation, AccordionPart, AccordionRelationships,
};

#[test]
fn accordion_part_inventory_matches_exhaustive_surface() {
    let parts: Vec<_> = Accordion::parts().iter().map(AccordionPart::as_str).collect();
    assert_eq!(parts, vec!["root", "item", "header", "trigger", "content"]);
    assert_eq!(ACCORDION_PARTS.len(), 5);
}

#[test]
fn accordion_relationships_produce_deterministic_ids() {
    let scope = ScopeHandle::root("accordion-test").child("scope");
    let relationships = AccordionRelationships::new(scope.clone());

    assert_eq!(relationships.scope(), &scope);
    assert_eq!(relationships.root_id(), scope.token());
    assert_eq!(relationships.item_id("item-1"), scope.qualify("item-item-1"));
    assert_eq!(relationships.header_id("item-1"), scope.qualify("header-item-1"));
    assert_eq!(relationships.trigger_id("item-1"), scope.qualify("trigger-item-1"));
    assert_eq!(relationships.content_id("item-1"), scope.qualify("content-item-1"));
}

#[test]
fn accordion_single_non_collapsible_mode() {
    let scope = ScopeHandle::root("accordion-test").child("single-non-collapsible");
    let mut accordion = Accordion::new(scope, AccordionMode::Single { collapsible: false })
        .with_value("item-1");

    assert!(accordion.is_open("item-1"));
    assert!(!accordion.is_open("item-2"));

    // Item 1 trigger attributes (open in single non-collapsible)
    let trigger_1 = accordion.trigger("item-1", false);
    assert_eq!(trigger_1.aria_expanded(), "true");
    assert_eq!(trigger_1.aria_disabled(), Some("true")); // DEC-P3.4-004 & BI-P3.4-005
    assert_eq!(trigger_1.tabindex(), 0);
    assert_eq!(trigger_1.data_state_str(), "open");

    // Item 2 trigger attributes (closed)
    let trigger_2 = accordion.trigger("item-2", false);
    assert_eq!(trigger_2.aria_expanded(), "false");
    assert_eq!(trigger_2.aria_disabled(), None);
    assert_eq!(trigger_2.tabindex(), -1);
    assert_eq!(trigger_2.data_state_str(), "closed");

    // Toggling already-open item in non-collapsible mode is a NO-OP
    assert!(!accordion.toggle_item("item-1"));
    assert!(accordion.is_open("item-1"));

    // Toggling closed item opens it and closes item-1
    assert!(accordion.toggle_item("item-2"));
    assert!(!accordion.is_open("item-1"));
    assert!(accordion.is_open("item-2"));
    assert_eq!(accordion.current_tab_stop(), "item-2");

    let trigger_2_now_open = accordion.trigger("item-2", false);
    assert_eq!(trigger_2_now_open.aria_expanded(), "true");
    assert_eq!(trigger_2_now_open.aria_disabled(), Some("true"));
}

#[test]
fn accordion_single_collapsible_mode() {
    let scope = ScopeHandle::root("accordion-test").child("single-collapsible");
    let mut accordion = Accordion::new(scope, AccordionMode::Single { collapsible: true })
        .with_value("item-1");

    assert!(accordion.is_open("item-1"));
    let trigger_1 = accordion.trigger("item-1", false);
    assert_eq!(trigger_1.aria_disabled(), None); // Collapsible: trigger is not aria-disabled

    // Toggling open item collapses it
    assert!(accordion.toggle_item("item-1"));
    assert!(!accordion.is_open("item-1"));
    assert!(accordion.active_values().is_empty());

    let trigger_1_closed = accordion.trigger("item-1", false);
    assert_eq!(trigger_1_closed.aria_expanded(), "false");
    assert_eq!(trigger_1_closed.aria_disabled(), None);
}

#[test]
fn accordion_multiple_mode() {
    let scope = ScopeHandle::root("accordion-test").child("multiple");
    let mut accordion = Accordion::new(scope, AccordionMode::Multiple)
        .with_values(vec!["item-1".to_string()]);

    assert!(accordion.is_open("item-1"));
    assert!(!accordion.is_open("item-2"));

    // Open item-2 simultaneously
    assert!(accordion.toggle_item("item-2"));
    assert!(accordion.is_open("item-1"));
    assert!(accordion.is_open("item-2"));
    assert_eq!(accordion.active_values(), &["item-1", "item-2"]);

    // Close item-1 while item-2 stays open
    assert!(accordion.toggle_item("item-1"));
    assert!(!accordion.is_open("item-1"));
    assert!(accordion.is_open("item-2"));
    assert_eq!(accordion.active_values(), &["item-2"]);
}

#[test]
fn accordion_header_and_content_semantics() {
    let scope = ScopeHandle::root("accordion-test").child("semantics");
    let accordion = Accordion::new(scope.clone(), AccordionMode::Single { collapsible: true })
        .with_value("item-1");

    let header = accordion.header("item-1", false);
    assert_eq!(header.id(), scope.qualify("header-item-1"));
    assert_eq!(header.role(), "heading");
    assert_eq!(header.aria_level(), 3);
    assert_eq!(header.data_heading_level(), 3);
    assert_eq!(header.data_state_str(), "open");

    let content = accordion.content("item-1");
    assert_eq!(content.id(), scope.qualify("content-item-1"));
    assert_eq!(content.role(), "region");
    assert_eq!(content.aria_labelledby(), scope.qualify("trigger-item-1"));
    assert!(!content.is_hidden());
    assert_eq!(content.data_state_str(), "open");

    let content_closed = accordion.content("item-2");
    assert!(content_closed.is_hidden());
    assert_eq!(content_closed.data_state_str(), "closed");
}

#[test]
fn accordion_roving_focus_and_keyboard_navigation() {
    let scope = ScopeHandle::root("accordion-test").child("roving");
    let accordion = Accordion::new(scope, AccordionMode::Single { collapsible: true })
        .with_value("item-1");

    let items = vec![
        AccordionItemRegistration {
            value: "item-1".to_string(),
            disabled: false,
        },
        AccordionItemRegistration {
            value: "item-2".to_string(),
            disabled: true, // disabled item should be skipped!
        },
        AccordionItemRegistration {
            value: "item-3".to_string(),
            disabled: false,
        },
    ];

    // Down arrow from item-1 skips disabled item-2 and reaches item-3
    let next = accordion.resolve_key_navigation(&items, "item-1", "ArrowDown");
    assert_eq!(next, Some("item-3".to_string()));

    // Down arrow from item-3 with loop wraps to item-1
    let wrap_next = accordion.resolve_key_navigation(&items, "item-3", "ArrowDown");
    assert_eq!(wrap_next, Some("item-1".to_string()));

    // Up arrow from item-1 with loop wraps to item-3
    let wrap_prev = accordion.resolve_key_navigation(&items, "item-1", "ArrowUp");
    assert_eq!(wrap_prev, Some("item-3".to_string()));

    // Boundary keys Home / End
    assert_eq!(accordion.resolve_key_navigation(&items, "item-3", "Home"), Some("item-1".to_string()));
    assert_eq!(accordion.resolve_key_navigation(&items, "item-1", "End"), Some("item-3".to_string()));

    // Horizontal with RTL: ArrowRight goes prev, ArrowLeft goes next
    let rtl_accordion = Accordion::new(
        ScopeHandle::root("accordion-test").child("rtl"),
        AccordionMode::Single { collapsible: true },
    )
    .with_orientation(AccordionOrientation::Horizontal)
    .with_direction(AccordionDirection::Rtl);

    let rtl_next = rtl_accordion.resolve_key_navigation(&items, "item-1", "ArrowLeft");
    assert_eq!(rtl_next, Some("item-3".to_string()));

    let rtl_prev = rtl_accordion.resolve_key_navigation(&items, "item-3", "ArrowRight");
    assert_eq!(rtl_prev, Some("item-1".to_string()));
}


use monoxus::{
    accordion::{ACCORDION_PARTS, Accordion, AccordionMode, AccordionPart, AccordionRelationships},
    foundation::shared::ScopeHandle,
};

#[test]
fn accordion_part_inventory_matches_exhaustive_surface() {
    let parts: Vec<_> = Accordion::parts()
        .iter()
        .map(AccordionPart::as_str)
        .collect();
    assert_eq!(parts, vec!["root", "item", "header", "trigger", "content"]);
    assert_eq!(ACCORDION_PARTS.len(), 5);
}

#[test]
fn accordion_relationships_produce_deterministic_ids() {
    let scope = ScopeHandle::root("accordion-test").child("scope");
    let relationships = AccordionRelationships::new(scope.clone());

    assert_eq!(relationships.scope(), &scope);
    assert_eq!(relationships.root_id(), scope.token());
    assert_eq!(
        relationships.item_id("item-1"),
        scope.qualify("item-item-1")
    );
    assert_eq!(
        relationships.header_id("item-1"),
        scope.qualify("header-item-1")
    );
    assert_eq!(
        relationships.trigger_id("item-1"),
        scope.qualify("trigger-item-1")
    );
    assert_eq!(
        relationships.content_id("item-1"),
        scope.qualify("content-item-1")
    );
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
    assert_eq!(header.data_disabled(), None);

    let header_h1 = accordion.header_with_level("item-1", false, 1);
    assert_eq!(header_h1.aria_level(), 1);
    assert_eq!(header_h1.data_heading_level(), 1);

    let header_clamped = accordion.header_with_level("item-1", false, 10);
    assert_eq!(header_clamped.aria_level(), 6);
    assert_eq!(header_clamped.data_heading_level(), 6);

    let trigger_open = accordion.trigger("item-1", false);
    assert_eq!(trigger_open.aria_controls(), Some(scope.qualify("content-item-1").as_str()));
    assert_eq!(trigger_open.target_content_id(), scope.qualify("content-item-1"));

    let trigger_closed = accordion.trigger("item-2", false);
    assert_eq!(trigger_closed.aria_controls(), None);
    assert_eq!(trigger_closed.target_content_id(), scope.qualify("content-item-2"));

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
fn accordion_orientation_aware_navigation_keys() {
    use monoxus::accordion::AccordionOrientation;

    assert!(Accordion::is_navigation_key_for_orientation(AccordionOrientation::Horizontal, "ArrowLeft"));
    assert!(Accordion::is_navigation_key_for_orientation(AccordionOrientation::Horizontal, "ArrowRight"));
    assert!(!Accordion::is_navigation_key_for_orientation(AccordionOrientation::Horizontal, "ArrowUp"));
    assert!(!Accordion::is_navigation_key_for_orientation(AccordionOrientation::Horizontal, "ArrowDown"));

    assert!(!Accordion::is_navigation_key_for_orientation(AccordionOrientation::Vertical, "ArrowLeft"));
    assert!(!Accordion::is_navigation_key_for_orientation(AccordionOrientation::Vertical, "ArrowRight"));
    assert!(Accordion::is_navigation_key_for_orientation(AccordionOrientation::Vertical, "ArrowUp"));
    assert!(Accordion::is_navigation_key_for_orientation(AccordionOrientation::Vertical, "ArrowDown"));

    assert!(Accordion::is_navigation_key_for_orientation(AccordionOrientation::Horizontal, "Home"));
    assert!(Accordion::is_navigation_key_for_orientation(AccordionOrientation::Vertical, "End"));
}

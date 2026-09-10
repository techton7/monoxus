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

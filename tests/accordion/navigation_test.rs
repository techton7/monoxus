use monoxus::{
    accordion::{
        Accordion, AccordionDirection, AccordionItemRegistration, AccordionMode,
        AccordionOrientation,
    },
    foundation::shared::ScopeHandle,
};

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

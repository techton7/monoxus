use monoxus::{
    accordion::{Accordion, AccordionMode},
    foundation::shared::ScopeHandle,
};

#[test]
fn accordion_single_non_collapsible_mode() {
    let scope = ScopeHandle::root("accordion-test").child("single-non-collapsible");
    let mut accordion =
        Accordion::new(scope, AccordionMode::Single { collapsible: false }).with_value("item-1");

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
    let mut accordion =
        Accordion::new(scope, AccordionMode::Single { collapsible: true }).with_value("item-1");

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
    let mut accordion =
        Accordion::new(scope, AccordionMode::Multiple).with_values(vec!["item-1".to_string()]);

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

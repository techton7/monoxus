//! Headless Native Accordion Transition Lifecycle Test
//!
//! Verifies that AccordionContent retains its DOM element in [data-state="closed"]
//! during CSS height transition, and only applies `hidden: true` when
//! Oxidase / Blitz Native dispatches the TransitionEnd event without wall-clock sleeps.

mod common;
use common::native_fixture::*;
use dioxus::prelude::*;
use monoxus::accordion::*;

static TEST_LOCK: std::sync::Mutex<()> = std::sync::Mutex::new(());

#[test]
fn test_native_accordion_height_transition_lifecycle() {
    let _lock = TEST_LOCK.lock().unwrap_or_else(|e| e.into_inner());

    fn accordion_scenario() -> Element {
        rsx! {
            style {
                r#"
                .accordion-content[data-state='open'] {{
                    height: 100px;
                    transition: height 200ms ease;
                }}
                .accordion-content[data-state='closed'] {{
                    height: 0px;
                    transition: height 200ms ease;
                }}
                "#
            }
            AccordionRoot {
                mode: AccordionMode::Single { collapsible: true },
                default_value: Some("item-1".to_string()),
                AccordionItem {
                    value: "item-1".to_string(),
                    AccordionHeader {
                        AccordionTrigger {
                            id: "trigger-1".to_string(),
                            "Trigger 1"
                        }
                    }
                    AccordionContent {
                        class: "accordion-content".to_string(),
                        div {
                            id: "inner-content",
                            "Accordion Content Body"
                        }
                    }
                }
            }
        }
    }

    let mut harness = mount_native_scenario(accordion_scenario);

    // Initial state: Item 1 is open, hidden attribute is false / None
    let content_node_id = harness
        .query(".accordion-content")
        .expect("Content node must be mounted");
    {
        let base = harness.base();
        let content_node = base.get_node(content_node_id).expect("Node exists");
        assert_eq!(content_node.attr("data-state".into()), Some("open"));
        assert_eq!(
            content_node.attr(blitz_dom::local_name!("hidden")),
            None,
            "Must not be hidden when open"
        );
    }

    // Click trigger to initiate close
    click_and_pump(&mut harness, "#trigger-1");

    // Immediately after click: data-state becomes 'closed', but DOM node is STILL retained and NOT hidden!
    let content_node_id = harness
        .query(".accordion-content")
        .expect("Content node must be retained during transition");
    {
        let base = harness.base();
        let content_node = base.get_node(content_node_id).expect("Node exists");
        assert_eq!(content_node.attr("data-state".into()), Some("closed"));
        assert_eq!(
            content_node.attr(blitz_dom::local_name!("hidden")),
            None,
            "Content must NOT receive hidden attribute while height transition is running"
        );
    }

    // Advance virtual monotonic animation clock by 0.25s (past the 200ms transition duration)
    advance_frame_and_pump(&mut harness, 0.25);

    // After TransitionEnd: hidden attribute is now applied!
    let content_node_id = harness
        .query(".accordion-content")
        .expect("Content node in DOM");
    {
        let base = harness.base();
        let content_node = base.get_node(content_node_id).expect("Node exists");
        assert_eq!(
            content_node.attr(blitz_dom::local_name!("hidden")),
            Some("true"),
            "Content must receive hidden attribute after TransitionEnd event completes"
        );
    }
}

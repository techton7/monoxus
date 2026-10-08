//! Contract test verifying the headless native test harness fixture.
//!
//! Demonstrates mounting a real Dioxus UI component scenario headlessly,
//! dispatching synthetic pointer and keyboard interactions, and asserting
//! relational geometry and focus contracts.

mod common;
use common::native_fixture::*;
use dioxus::prelude::*;
use keyboard_types::{Code, Key};
use monoxus::collapsible::{CollapsibleContent, CollapsibleRoot, CollapsibleTrigger};

// Include the playground scenario directly to demonstrate mounting playground components
#[path = "../examples/playground/collapsible.rs"]
mod playground_collapsible;

/// Standalone scenario with explicit dimensions for strict relational alignment testing.
#[component]
fn NativeRelationalTestScenario() -> Element {
    let mut is_open = use_signal(|| false);

    rsx! {
        div {
            style: "width: 400px; padding: 20px; display: flex; flex-direction: column; gap: 8px;",
            CollapsibleRoot {
                id: "aligned-root".to_string(),
                default_open: false,
                on_open_change: move |open| is_open.set(open),
                CollapsibleTrigger {
                    id: "aligned-trigger".to_string(),
                    style: "width: 200px; height: 36px; display: block;",
                    "Toggle Details"
                }
                CollapsibleContent {
                    id: "aligned-content".to_string(),
                    style: "width: 200px; height: 100px; display: block;",
                    div {
                        "Expanded Content Details"
                    }
                }
            }
        }
    }
}

#[test]
fn test_native_harness_playground_collapsible_scenario() {
    let mut harness = mount_native_scenario(playground_collapsible::BasicCollapsibleScenario);

    // Initial state: trigger must exist with non-empty rect
    assert_rect_non_empty(&harness, "#collapsible-trigger");
    assert!(
        harness.query("#collapsible-content").is_none(),
        "Content should not be mounted when collapsible is closed"
    );

    // Interaction: Click trigger to open
    click_and_pump(&mut harness, "#collapsible-trigger");

    // After click: Content should mount and have non-empty rect
    assert_rect_non_empty(&harness, "#collapsible-content");

    // Relational geometry: Content must be vertically positioned below trigger
    assert_relative_below(&harness, "#collapsible-trigger", "#collapsible-content", 10.0);

    // Focus verification: Explicitly focus the trigger and assert active focus
    focus_node(&mut harness, "#collapsible-trigger");
    assert_native_focus_id(&harness, "collapsible-trigger");

    // State attribute verification
    let trigger_data_state = harness.attr("#collapsible-trigger", "data-state");
    assert_eq!(trigger_data_state.as_deref(), Some("open"));

    // Interaction: Click trigger again to close
    click_and_pump(&mut harness, "#collapsible-trigger");
    assert!(
        harness.query("#collapsible-content").is_none(),
        "Content should be unmounted after closing collapsible"
    );
}

#[test]
fn test_native_harness_relational_geometry_and_keyboard() {
    let mut harness = mount_native_scenario(NativeRelationalTestScenario);

    // Initial layout assertions
    assert_rect_non_empty(&harness, "#aligned-trigger");

    // Keyboard interaction: Send key to harness
    focus_node(&mut harness, "#aligned-trigger");
    assert_native_focus_id(&harness, "aligned-trigger");

    // Click trigger to expand
    click_and_pump(&mut harness, "#aligned-trigger");
    assert_rect_non_empty(&harness, "#aligned-content");

    // Relational geometric assertions
    assert_relative_below(&harness, "#aligned-trigger", "#aligned-content", 5.0);
    assert_relative_aligned(&harness, "#aligned-trigger", "#aligned-content", 2.0);

    // Keyboard dispatch helper test
    send_key_and_pump(&mut harness, Key::Enter, Code::Enter);
}

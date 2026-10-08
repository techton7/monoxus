//! Native Non-Floating Primitives Verification Test Suite.
//!
//! Validates non-floating primitives (Collapsible, Accordion, Tabs) using
//! Blitz test harness over shared scenarios directly from `examples/playground/`.

mod common;
use common::native_fixture::*;
use keyboard_types::{Code, Key};

#[path = "../examples/playground/collapsible.rs"]
mod playground_collapsible;

#[path = "../examples/playground/accordion.rs"]
mod playground_accordion;

#[path = "../examples/playground/tabs.rs"]
mod playground_tabs;

/// Test 1: Collapsible Live Layout and Toggle
///
/// Verifies initial closed state, expansion via pointer click, live non-empty layout rects,
/// relational alignment (content positioned below trigger), and closing toggle.
#[test]
fn test_native_collapsible_live_layout_and_toggle() {
    let mut harness = mount_native_scenario(playground_collapsible::BasicCollapsibleScenario);

    // Initial state: trigger exists with non-empty rect, closed state, content unmounted
    assert_rect_non_empty(&harness, "#collapsible-trigger");
    assert_eq!(
        harness.attr("#collapsible-trigger", "data-state").as_deref(),
        Some("closed"),
        "Trigger should initially have data-state='closed'"
    );
    assert!(
        harness.query("#collapsible-content").is_none(),
        "Content should not be mounted when collapsible is closed"
    );

    // Interaction 1: Click trigger to expand
    click_and_pump(&mut harness, "#collapsible-trigger");

    // Open state assertions
    assert_eq!(
        harness.attr("#collapsible-trigger", "data-state").as_deref(),
        Some("open"),
        "Trigger should have data-state='open' after click"
    );
    assert_rect_non_empty(&harness, "#collapsible-content");
    assert_relative_below(&harness, "#collapsible-trigger", "#collapsible-content", 10.0);

    // Interaction 2: Click trigger again to collapse
    click_and_pump(&mut harness, "#collapsible-trigger");

    // Closed state assertions
    assert_eq!(
        harness.attr("#collapsible-trigger", "data-state").as_deref(),
        Some("closed"),
        "Trigger should return to data-state='closed' after second click"
    );
    assert!(
        harness.query("#collapsible-content").is_none(),
        "Content should be unmounted after closing collapsible"
    );
}

/// Test 2: Accordion Single and Multiple Expansion
///
/// Verifies:
/// 1. Single collapsible mode enforces mutual exclusion when expanding items.
/// 2. Multiple selection mode allows simultaneous item expansion.
#[test]
fn test_native_accordion_single_and_multiple_expansion() {
    // -------------------------------------------------------------------------
    // 1. Single Collapsible Mode: Mutual Exclusion
    // -------------------------------------------------------------------------
    let mut single_harness = mount_native_scenario(playground_accordion::SingleCollapsibleSection);

    // Initial state: closed
    assert_eq!(
        single_harness.attr("#accordion-trigger-1", "data-state").as_deref(),
        Some("closed")
    );
    assert_eq!(
        single_harness.attr("#accordion-content-1", "data-state").as_deref(),
        Some("closed")
    );

    // Click item 1 -> opens item 1
    click_and_pump(&mut single_harness, "#accordion-trigger-1");
    assert_eq!(
        single_harness.attr("#accordion-trigger-1", "data-state").as_deref(),
        Some("open"),
        "Item 1 trigger should be open after click"
    );
    assert_eq!(
        single_harness.attr("#accordion-content-1", "data-state").as_deref(),
        Some("open"),
        "Item 1 content should be open after click"
    );
    assert_rect_non_empty(&single_harness, "#accordion-content-1");

    // Click item 2 -> opens item 2 and closes item 1 (mutual exclusion)
    click_and_pump(&mut single_harness, "#accordion-trigger-2");
    assert_eq!(
        single_harness.attr("#accordion-trigger-2", "data-state").as_deref(),
        Some("open"),
        "Item 2 trigger should be open after click"
    );
    assert_eq!(
        single_harness.attr("#accordion-content-2", "data-state").as_deref(),
        Some("open"),
        "Item 2 content should be open after click"
    );
    assert_rect_non_empty(&single_harness, "#accordion-content-2");

    assert_eq!(
        single_harness.attr("#accordion-trigger-1", "data-state").as_deref(),
        Some("closed"),
        "Item 1 trigger should close when item 2 opens in single mode"
    );
    assert_eq!(
        single_harness.attr("#accordion-content-1", "data-state").as_deref(),
        Some("closed"),
        "Item 1 content should close when item 2 opens in single mode"
    );

    // -------------------------------------------------------------------------
    // 2. Multiple Selection Mode: Simultaneous Expansion
    // -------------------------------------------------------------------------
    let mut multi_harness = mount_native_scenario(playground_accordion::MultipleAccordionSection);

    // Initial state: closed
    assert_eq!(
        multi_harness.attr("#accordion-trigger-1", "data-state").as_deref(),
        Some("closed")
    );
    assert_eq!(
        multi_harness.attr("#accordion-trigger-2", "data-state").as_deref(),
        Some("closed")
    );

    // Click item 1 -> opens item 1
    click_and_pump(&mut multi_harness, "#accordion-trigger-1");
    assert_eq!(
        multi_harness.attr("#accordion-trigger-1", "data-state").as_deref(),
        Some("open"),
        "Item 1 trigger should be open"
    );
    assert_eq!(
        multi_harness.attr("#accordion-content-1", "data-state").as_deref(),
        Some("open"),
        "Item 1 content should be open"
    );

    // Click item 2 -> opens item 2 while keeping item 1 open
    click_and_pump(&mut multi_harness, "#accordion-trigger-2");
    assert_eq!(
        multi_harness.attr("#accordion-trigger-1", "data-state").as_deref(),
        Some("open"),
        "Item 1 trigger must stay open in multiple mode"
    );
    assert_eq!(
        multi_harness.attr("#accordion-content-1", "data-state").as_deref(),
        Some("open"),
        "Item 1 content must stay open in multiple mode"
    );
    assert_eq!(
        multi_harness.attr("#accordion-trigger-2", "data-state").as_deref(),
        Some("open"),
        "Item 2 trigger must be open in multiple mode"
    );
    assert_eq!(
        multi_harness.attr("#accordion-content-2", "data-state").as_deref(),
        Some("open"),
        "Item 2 content must be open in multiple mode"
    );
    assert_rect_non_empty(&multi_harness, "#accordion-content-1");
    assert_rect_non_empty(&multi_harness, "#accordion-content-2");
}

/// Test 3: Tabs Arrow Navigation and Roving Focus
///
/// Verifies default active tab, keyboard navigation via ArrowRight / ArrowLeft,
/// roving tabindex and focus ownership, automatic tab selection, and boundary
/// navigation (Home / End).
#[test]
fn test_native_tabs_arrow_navigation_and_roving_focus() {
    let mut harness = mount_native_scenario(playground_tabs::TabsPlayground);

    // Default state: #tabs-trigger-1 active, content 1 rendered, content 2 unmounted
    assert_eq!(
        harness.attr("#tabs-trigger-1", "data-state").as_deref(),
        Some("active"),
        "Trigger 1 should be active by default"
    );
    assert_eq!(
        harness.attr("#tabs-trigger-1", "tabindex").as_deref(),
        Some("0"),
        "Trigger 1 should have tabindex=0 as initial tab stop"
    );
    assert_eq!(
        harness.attr("#tabs-trigger-2", "data-state").as_deref(),
        Some("inactive"),
        "Trigger 2 should be inactive by default"
    );
    assert_eq!(
        harness.attr("#tabs-trigger-2", "tabindex").as_deref(),
        Some("-1"),
        "Trigger 2 should have tabindex=-1"
    );
    assert!(
        harness.query("#tabs-content-1").is_some(),
        "Content 1 should be mounted"
    );
    assert!(
        harness.query("#tabs-content-2").is_none(),
        "Content 2 should be unmounted"
    );

    // Initial focus on #tabs-trigger-1
    focus_node(&mut harness, "#tabs-trigger-1");
    assert_native_focus_id(&harness, "tabs-trigger-1");

    // Keyboard navigation: ArrowRight -> moves roving focus and active tab to #tabs-trigger-2
    send_key_and_pump(&mut harness, Key::ArrowRight, Code::ArrowRight);

    assert_eq!(
        harness.attr("#tabs-trigger-2", "data-state").as_deref(),
        Some("active"),
        "Trigger 2 should transition to active after ArrowRight"
    );
    assert_eq!(
        harness.attr("#tabs-trigger-2", "tabindex").as_deref(),
        Some("0"),
        "Trigger 2 should become current tab stop (tabindex=0)"
    );
    assert_eq!(
        harness.attr("#tabs-trigger-1", "data-state").as_deref(),
        Some("inactive"),
        "Trigger 1 should become inactive after ArrowRight"
    );
    assert_eq!(
        harness.attr("#tabs-trigger-1", "tabindex").as_deref(),
        Some("-1"),
        "Trigger 1 should have tabindex=-1"
    );
    assert!(
        harness.query("#tabs-content-2").is_some(),
        "Content 2 should mount upon active tab transition"
    );
    assert!(
        harness.query("#tabs-content-1").is_none(),
        "Content 1 should unmount upon active tab transition"
    );
    assert_native_focus_id(&harness, "tabs-trigger-2");

    // Boundary Navigation: End -> jumps to #tabs-trigger-3
    send_key_and_pump(&mut harness, Key::End, Code::End);
    assert_eq!(
        harness.attr("#tabs-trigger-3", "data-state").as_deref(),
        Some("active"),
        "Trigger 3 should be active after End key"
    );
    assert_eq!(
        harness.attr("#tabs-trigger-3", "tabindex").as_deref(),
        Some("0")
    );
    assert!(harness.query("#tabs-content-3").is_some());
    assert!(harness.query("#tabs-content-2").is_none());
    assert_native_focus_id(&harness, "tabs-trigger-3");

    // Boundary Navigation: Home -> jumps to #tabs-trigger-1
    send_key_and_pump(&mut harness, Key::Home, Code::Home);
    assert_eq!(
        harness.attr("#tabs-trigger-1", "data-state").as_deref(),
        Some("active"),
        "Trigger 1 should be active after Home key"
    );
    assert_eq!(
        harness.attr("#tabs-trigger-1", "tabindex").as_deref(),
        Some("0")
    );
    assert!(harness.query("#tabs-content-1").is_some());
    assert!(harness.query("#tabs-content-3").is_none());
    assert_native_focus_id(&harness, "tabs-trigger-1");

    // Navigation: ArrowLeft -> wraps to #tabs-trigger-3 (looping focus)
    send_key_and_pump(&mut harness, Key::ArrowLeft, Code::ArrowLeft);
    assert_eq!(
        harness.attr("#tabs-trigger-3", "data-state").as_deref(),
        Some("active"),
        "Trigger 3 should wrap around on ArrowLeft from first tab"
    );
    assert_native_focus_id(&harness, "tabs-trigger-3");
}

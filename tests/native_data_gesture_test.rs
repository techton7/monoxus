//! Native Data and Gesture Primitives Verification Test Suite.
//!
//! Validates data and gesture primitives (Select, Toast) using
//! Blitz test harness over shared scenarios directly from `examples/playground/`.

#![allow(unused_imports, dead_code)]

use dioxus::prelude::*;
mod common;
use common::native_fixture::*;
use keyboard_types::{Code, Key, Modifiers, NamedKey};
use monoxus::toast::{TOAST_STORE, ToastOptions, toast};
use std::time::Duration;

#[path = "../examples/playground/scenarios/select/mod.rs"]
mod playground_select;

#[path = "../examples/playground/pages/toast_page.rs"]
mod playground_toast;

static TEST_LOCK: std::sync::Mutex<()> = std::sync::Mutex::new(());

/// Test 1: Select document order, roving focus, and synchronous/closed typeahead
///
/// Verifies initial closed state, trigger mounting, opening via click,
/// content mounting and relative placement below trigger, roving focus across
/// items in true document order via arrow keys, selection commit closing the dropdown,
/// synchronous multi-character typeahead ('b' -> banana, 'bl' -> blueberry), and
/// closed-state typeahead commit ('p' -> pineapple).
#[test]
fn test_native_select_document_order_and_roving_focus() {
    let _lock = TEST_LOCK.lock().unwrap_or_else(|e| e.into_inner());
    let mut harness = mount_native_scenario(playground_select::BasicFruitSelectSection);

    // Initial state: trigger exists with valid non-empty rect, content is unmounted
    assert_rect_non_empty(&harness, "#select-trigger");
    assert_eq!(
        harness.attr("#select-trigger", "aria-expanded").as_deref(),
        Some("false"),
        "Select trigger should initially be collapsed"
    );
    assert!(
        harness.query("#select-content").is_none(),
        "Select content should not be mounted initially"
    );

    // Interaction 1: Click trigger to open select dropdown
    click_and_pump(&mut harness, "#select-trigger");
    advance_frame_and_pump(&mut harness, 0.02);
    advance_frame_and_pump(&mut harness, 0.02);

    // Open state assertions: content mounts and has non-empty layout rect
    assert!(
        harness.query("#select-content").is_some(),
        "Select content should mount when open"
    );
    assert_rect_non_empty(&harness, "#select-content");
    assert_relative_below(&harness, "#select-trigger", "#select-content", 10.0);

    // Focus content to drive keyboard roving focus
    focus_node(&mut harness, "#select-content");

    // Initial item highlighted corresponds to pre-selected value "apple"
    let initial_descendant = harness.attr("#select-content", "aria-activedescendant");
    assert!(
        initial_descendant
            .as_deref()
            .unwrap_or("")
            .contains("apple"),
        "Initial active descendant should be apple, got {initial_descendant:?}"
    );

    // Send ArrowDown: roving focus advances to next item in document tree order ("banana")
    send_key_and_pump(&mut harness, Key::Named(NamedKey::ArrowDown), Code::ArrowDown);
    let after_down1 = harness.attr("#select-content", "aria-activedescendant");
    assert!(
        after_down1.as_deref().unwrap_or("").contains("banana"),
        "Roving focus should advance to banana after ArrowDown, got {after_down1:?}"
    );

    // Send ArrowDown again: roving focus advances to "blueberry"
    send_key_and_pump(&mut harness, Key::Named(NamedKey::ArrowDown), Code::ArrowDown);
    let after_down2 = harness.attr("#select-content", "aria-activedescendant");
    assert!(
        after_down2.as_deref().unwrap_or("").contains("blueberry"),
        "Roving focus should advance to blueberry after second ArrowDown, got {after_down2:?}"
    );

    // Send ArrowUp: roving focus steps backward to "banana"
    send_key_and_pump(&mut harness, Key::Named(NamedKey::ArrowUp), Code::ArrowUp);
    let after_up = harness.attr("#select-content", "aria-activedescendant");
    assert!(
        after_up.as_deref().unwrap_or("").contains("banana"),
        "Roving focus should move back to banana after ArrowUp, got {after_up:?}"
    );

    // Interaction 2: Send Enter to commit selection of "banana" and close dropdown
    send_key_and_pump(&mut harness, Key::Named(NamedKey::Enter), Code::Enter);

    // Closed state assertions: trigger is collapsed
    assert_eq!(
        harness.attr("#select-trigger", "aria-expanded").as_deref(),
        Some("false"),
        "Select trigger should be collapsed after selection commit"
    );

    // Wait for exit presence animation/transition to unmount content
    advance_frame_and_pump(&mut harness, 0.25);

    assert!(
        harness.query("#select-content").is_none(),
        "Select content should unmount after committing selection"
    );

    // =========================================================================
    // Explicit order-sensitive typeahead verification:
    // a. Open dropdown on `#select-trigger`, focus `#select-content`.
    // =========================================================================
    click_and_pump(&mut harness, "#select-trigger");
    advance_frame_and_pump(&mut harness, 0.02);
    advance_frame_and_pump(&mut harness, 0.02);

    assert!(
        harness.query("#select-content").is_some(),
        "Select content should mount for typeahead testing"
    );
    focus_node(&mut harness, "#select-content");

    // b. Dispatch character 'b' key event.
    // Assert aria-activedescendant matches "banana" and option text matches.
    send_key_and_pump_with(
        &mut harness,
        Key::Character("b".into()),
        Code::KeyB,
        Modifiers::default(),
    );
    let descendant_b = harness.attr("#select-content", "aria-activedescendant");
    assert!(
        descendant_b.as_deref().unwrap_or("").contains("banana"),
        "Typeahead 'b' should highlight banana, got {descendant_b:?}"
    );
    let content_text = harness.text_content("#select-content");
    println!("DEBUG: content_text={:?}", content_text);
    assert!(
        content_text.contains("Banana"),
        "Option text for highlighted item should match banana"
    );

    // c. Dispatch character 'l' key event immediately.
    // Assert aria-activedescendant advances to "blueberry".
    send_key_and_pump_with(
        &mut harness,
        Key::Character("l".into()),
        Code::KeyL,
        Modifiers::default(),
    );
    let descendant_bl = harness.attr("#select-content", "aria-activedescendant");
    assert!(
        descendant_bl.as_deref().unwrap_or("").contains("blueberry"),
        "Typeahead 'bl' should advance to blueberry, got {descendant_bl:?}"
    );

    // d. Press Enter to commit "blueberry" and close dropdown.
    send_key_and_pump(&mut harness, Key::Named(NamedKey::Enter), Code::Enter);
    assert_eq!(
        harness.attr("#select-trigger", "aria-expanded").as_deref(),
        Some("false"),
        "Dropdown should close after Enter commit"
    );
    advance_frame_and_pump(&mut harness, 0.25);
    assert!(
        harness.query("#select-content").is_none(),
        "Content should unmount after committing blueberry"
    );

    // e. Now with dropdown closed, focus `#select-trigger`.
    focus_node(&mut harness, "#select-trigger");

    // f. Dispatch character 'p' key event.
    send_key_and_pump_with(
        &mut harness,
        Key::Character("p".into()),
        Code::KeyP,
        Modifiers::default(),
    );

    // g. Assert closed-state typeahead updates selected value and commits to trigger text ("pineapple").
    let trigger_text = harness.text_content("#select-trigger");
    assert!(
        trigger_text.to_lowercase().contains("pineapple"),
        "Closed-state typeahead with 'p' should commit pineapple to trigger text, got {trigger_text}"
    );
}

/// Test 2: Toast swipe dismissal gesture lifecycle
///
/// Verifies dispatching a toast notification, synthesizing pointer drag exceeding
/// the 45px swipe threshold, asserting transient and swipe-out attributes,
/// and verifying unmount from viewport.
#[test]
fn test_native_toast_swipe_dismiss_gesture() {
    let _lock = TEST_LOCK.lock().unwrap_or_else(|e| e.into_inner());

    let mut harness = mount_native_scenario(playground_toast::ToastPage);
    harness.set_viewport_size(800, 1400);
    harness.pump();

    // a. Clean any residual toast store state within Dioxus runtime
    {
        let _guard = dioxus::core::RuntimeGuard::new(harness.doc.vdom.runtime());
        toast::dismiss(None);
        TOAST_STORE.write().toasts.clear();
    }
    harness.pump();

    // Viewport landmark assertion
    assert!(
        harness.query("#monoxus-toast-viewport").is_some(),
        "Toast viewport should be present"
    );
    let vp_rect = harness.layout_rect("#monoxus-toast-viewport");
    assert!(
        vp_rect.width > 0.0,
        "Toast viewport should have non-empty width (356px)"
    );
    assert_eq!(
        harness.attr("#monoxus-toast-viewport", "role").as_deref(),
        Some("region"),
        "Toast viewport should have role='region'"
    );

    // Initial state: no toasts active
    assert!(
        harness.query("li[data-sonner-toast]").is_none(),
        "No toasts should be mounted initially"
    );

    // b. Click #dispatch-info to dispatch a toast with standard duration (so it does not auto-close early)
    click_and_pump(&mut harness, "#dispatch-info");

    // c. Assert toast mounts with data-state="open" and non-empty rect
    assert!(
        harness.query("#monoxus-toast-viewport li").is_some(),
        "Toast item should mount after #dispatch-info click"
    );
    assert_rect_non_empty(&harness, "#monoxus-toast-viewport li");
    assert_eq!(
        harness
            .attr("#monoxus-toast-viewport li", "data-state")
            .as_deref(),
        Some("open"),
        "Active toast should have data-state='open'"
    );

    // d. Query toast layout rect
    let rect = harness.layout_rect("#monoxus-toast-viewport li");
    assert!(
        rect.width > 0.0 && rect.height > 0.0,
        "Toast should have non-empty rect"
    );

    // e. Synthesize real pointer drag exceeding 45px swipe threshold:
    harness.mouse_down_at(rect.x + 20.0, rect.y + 20.0);
    harness.move_mouse_to(rect.x + 100.0, rect.y + 20.0);
    harness.pump();

    // Assert transient attributes:
    assert_eq!(
        harness
            .attr("#monoxus-toast-viewport li", "data-swiping")
            .as_deref(),
        Some("true"),
        "Toast should reflect data-swiping='true' while actively dragging"
    );

    harness.mouse_up_at(rect.x + 100.0, rect.y + 20.0);
    harness.pump();

    // Assert swipe out attributes:
    assert_eq!(
        harness
            .attr("#monoxus-toast-viewport li", "data-swipe-out")
            .as_deref(),
        Some("true"),
        "Toast should reflect data-swipe-out='true' after release exceeding threshold"
    );
    assert_eq!(
        harness
            .attr("#monoxus-toast-viewport li", "data-swipe-direction")
            .as_deref(),
        Some("right"),
        "Toast should reflect data-swipe-direction='right'"
    );

    // Advance animation clock past swipe-out exit animation (200ms)
    advance_frame_and_pump(&mut harness, 0.25);

    // Assert toast is unmounted from #monoxus-toast-viewport
    assert!(
        harness.query("#monoxus-toast-viewport li").is_none(),
        "Toast should be unmounted from #monoxus-toast-viewport after swipe dismissal"
    );
}

/// Test 3: Toast timer auto-close lifecycle
///
/// Verifies that short-duration toasts auto-close after their duration expires
/// and are evicted from the DOM without any pointer interaction.
#[test]
fn test_native_toast_timer_auto_close_lifecycle() {
    let _lock = TEST_LOCK.lock().unwrap_or_else(|e| e.into_inner());

    let mut harness = mount_native_scenario(playground_toast::ToastPage);

    // a. Clean any residual toast store state within Dioxus runtime
    {
        let _guard = dioxus::core::RuntimeGuard::new(harness.doc.vdom.runtime());
        toast::dismiss(None);
        TOAST_STORE.write().toasts.clear();
    }
    harness.pump();

    assert!(
        harness.query("#monoxus-toast-viewport").is_some(),
        "Toast viewport should be present"
    );

    // b. Dispatch short-duration toast (60ms) via toast::message
    {
        let _guard = dioxus::core::RuntimeGuard::new(harness.doc.vdom.runtime());
        toast::message(
            "Auto-dismiss timer toast",
            Some(ToastOptions {
                duration: Some(Duration::from_millis(60)),
                ..Default::default()
            }),
        );
    }
    harness.pump();

    // c. Assert toast mounts with data-state="open"
    assert!(
        harness.query("#monoxus-toast-viewport li").is_some(),
        "Timer toast should mount"
    );
    assert_rect_non_empty(&harness, "#monoxus-toast-viewport li");
    assert_eq!(
        harness
            .attr("#monoxus-toast-viewport li", "data-state")
            .as_deref(),
        Some("open"),
        "Timer toast should initially have data-state='open'"
    );

    // d. Advance virtual frame past timer duration (60ms) and exit transition (350ms)
    advance_frame_and_pump(&mut harness, 0.1);
    advance_frame_and_pump(&mut harness, 0.35);

    // e. Assert toast is automatically evicted from DOM without any pointer interaction
    assert!(
        harness.query("#monoxus-toast-viewport li").is_none(),
        "Timer toast should be automatically evicted after duration expires"
    );
}

/// Select outside click dismissal lifecycle
///
/// Verifies that clicking outside an open Select dropdown triggers outside dismissal:
/// trigger aria-expanded collapses to "false", exit presence transition runs,
/// and content unmounts completely from the DOM without panicking.
#[test]
fn test_native_select_outside_click_dismissal() {
    let _lock = TEST_LOCK.lock().unwrap_or_else(|e| e.into_inner());
    let mut harness = mount_native_scenario(playground_select::BasicFruitSelectSection);

    // Initial state: trigger collapsed
    assert_eq!(
        harness.attr("#select-trigger", "aria-expanded").as_deref(),
        Some("false"),
        "Select trigger should initially be collapsed"
    );

    // Click trigger to open dropdown
    click_and_pump(&mut harness, "#select-trigger");
    advance_frame_and_pump(&mut harness, 0.02);
    advance_frame_and_pump(&mut harness, 0.02);

    // Assert open state
    assert_eq!(
        harness.attr("#select-trigger", "aria-expanded").as_deref(),
        Some("true"),
        "Select trigger should be expanded when open"
    );
    assert!(
        harness.query("#select-content").is_some(),
        "Select content should mount when open"
    );

    // Interaction: Click outside dropdown at blank canvas coordinates (500.0, 500.0)
    click_at_and_pump(&mut harness, 500.0, 500.0);

    // Assert immediate trigger collapse
    assert_eq!(
        harness.attr("#select-trigger", "aria-expanded").as_deref(),
        Some("false"),
        "Select trigger should collapse immediately on outside click"
    );

    // Advance frame past exit animation (0.25s)
    advance_frame_and_pump(&mut harness, 0.25);

    // Assert content is fully unmounted
    assert!(
        harness.query("#select-content").is_none(),
        "Select content should be unmounted after outside click dismissal"
    );
}

/// Select Escape keydown dismissal lifecycle
///
/// Verifies that pressing Escape while Select is open and focused dismisses the dropdown:
/// trigger aria-expanded collapses to "false", exit presence transition runs,
/// and content unmounts completely from the DOM without panicking.
#[test]
fn test_native_select_escape_dismissal() {
    let _lock = TEST_LOCK.lock().unwrap_or_else(|e| e.into_inner());
    let mut harness = mount_native_scenario(playground_select::BasicFruitSelectSection);

    // Initial state: trigger collapsed
    assert_eq!(
        harness.attr("#select-trigger", "aria-expanded").as_deref(),
        Some("false"),
        "Select trigger should initially be collapsed"
    );

    // Click trigger to open dropdown
    click_and_pump(&mut harness, "#select-trigger");
    advance_frame_and_pump(&mut harness, 0.02);
    advance_frame_and_pump(&mut harness, 0.02);

    // Assert open state
    assert_eq!(
        harness.attr("#select-trigger", "aria-expanded").as_deref(),
        Some("true"),
        "Select trigger should be expanded when open"
    );
    assert!(
        harness.query("#select-content").is_some(),
        "Select content should mount when open"
    );

    // Focus content to receive key events
    focus_node(&mut harness, "#select-content");

    // Send Escape key
    send_key_and_pump(&mut harness, Key::Named(NamedKey::Escape), Code::Escape);
    advance_frame_and_pump(&mut harness, 0.02);

    // Assert immediate trigger collapse
    assert_eq!(
        harness.attr("#select-trigger", "aria-expanded").as_deref(),
        Some("false"),
        "Select trigger should collapse immediately on Escape key"
    );

    // Advance frame past exit animation (0.25s)
    advance_frame_and_pump(&mut harness, 0.25);

    // Assert content is fully unmounted
    assert!(
        harness.query("#select-content").is_none(),
        "Select content should be unmounted after Escape dismissal"
    );
}

/// Test 6: Select wheel scroll alignment in scrollable container
///
/// Verifies Invariant A & Invariant D:
/// When Select is open inside a scrollable container and mouse wheel scroll is dispatched,
/// the floating position monitor recalculates placement via the synchronous fast-path,
/// keeping SelectContent anchored to SelectTrigger with side_offset (4.0px) preserved.
#[test]
fn test_native_select_wheel_scroll_alignment() {
    let _lock = TEST_LOCK.lock().unwrap_or_else(|e| e.into_inner());

    #[component]
    fn ScrollTestContainer() -> Element {
        rsx! {
            div {
                id: "scroll-container",
                style: "width: 500px; height: 350px; overflow-y: auto; padding: 20px;",
                div {
                    style: "height: 80px;",
                    "Top spacer"
                }
                playground_select::BasicFruitSelectSection {}
                div {
                    style: "height: 800px;",
                    "Bottom spacer"
                }
            }
        }
    }

    let mut harness = mount_native_scenario(ScrollTestContainer);

    // Initial state: trigger collapsed
    assert_eq!(
        harness.attr("#select-trigger", "aria-expanded").as_deref(),
        Some("false"),
        "Select trigger should initially be collapsed"
    );

    // Click trigger to open dropdown
    click_and_pump(&mut harness, "#select-trigger");
    advance_frame_and_pump(&mut harness, 0.02);
    advance_frame_and_pump(&mut harness, 0.02);

    assert_eq!(
        harness.attr("#select-trigger", "aria-expanded").as_deref(),
        Some("true"),
        "Select trigger should be expanded"
    );
    assert!(
        harness.query("#select-content").is_some(),
        "Select content should be mounted"
    );

    let initial_trigger_rect = harness.layout_rect("#select-trigger");
    let initial_content_rect = harness.layout_rect("#select-content");
    let initial_gap = initial_content_rect.y - (initial_trigger_rect.y + initial_trigger_rect.height);
    assert!(
        (initial_gap - 4.0).abs() <= 1.0,
        "Initial gap between trigger bottom and content top should be ~4.0px, got {initial_gap}"
    );

    // Dispatch mouse wheel scroll down by 50px on the scroll container
    wheel_at_and_pump(&mut harness, 200.0, 150.0, 0.0, -50.0);

    // Re-verify layout rects after scroll
    let scrolled_trigger_rect = harness.layout_rect("#select-trigger");
    let scrolled_content_rect = harness.layout_rect("#select-content");

    // Non-vacuous check: Trigger must have moved upward under wheel scroll down
    assert!(
        scrolled_trigger_rect.y < initial_trigger_rect.y - 40.0,
        "Trigger must have moved upward under wheel scroll down (initial y={}, scrolled y={})",
        initial_trigger_rect.y,
        scrolled_trigger_rect.y
    );

    // Verify container scroll offset updated directly on the overflow ancestor node
    let container_node = harness.node("#scroll-container");
    let container_scroll_y = harness.base().get_node(container_node).unwrap().scroll_offset().y;
    assert_eq!(
        container_scroll_y, 50.0,
        "Container scroll offset must be 50.0px after wheel scroll"
    );

    // Content must have repositioned upward to follow the trigger
    assert!(
        scrolled_content_rect.y < initial_content_rect.y - 40.0,
        "Content must have repositioned upward to follow trigger under container scroll (initial y={}, scrolled y={})",
        initial_content_rect.y,
        scrolled_content_rect.y
    );

    let scrolled_gap = scrolled_content_rect.y - (scrolled_trigger_rect.y + scrolled_trigger_rect.height);
    assert!(
        (scrolled_gap - 4.0).abs() <= 1.0,
        "Scrolled gap between trigger bottom and content top should remain ~4.0px, got {scrolled_gap}"
    );
}

/// Test 8: Select window/viewport scroll alignment (AC-2 Guard)
///
/// Verifies that when the window / viewport scrolls with the Select open,
/// the fixed content stays anchored to the in-flow trigger with constant side-offset gap (4.0px).
#[test]
fn test_native_select_window_viewport_scroll_alignment() {
    let _lock = TEST_LOCK.lock().unwrap_or_else(|e| e.into_inner());

    #[component]
    fn WindowScrollScenario() -> Element {
        rsx! {
            div {
                id: "window-scroll-root",
                style: "width: 800px; height: 1800px; padding: 30px;",
                div {
                    style: "height: 100px;",
                    "Window top spacer"
                }
                playground_select::BasicFruitSelectSection {}
                div {
                    style: "height: 1200px;",
                    "Window bottom spacer"
                }
            }
        }
    }

    let mut harness = mount_native_scenario(WindowScrollScenario);
    harness.set_viewport_size(800, 600);
    harness.pump();

    // Click trigger to open dropdown
    click_and_pump(&mut harness, "#select-trigger");
    advance_frame_and_pump(&mut harness, 0.02);
    advance_frame_and_pump(&mut harness, 0.02);

    assert_eq!(
        harness.attr("#select-trigger", "aria-expanded").as_deref(),
        Some("true"),
        "Select trigger should be expanded"
    );
    assert!(
        harness.query("#select-content").is_some(),
        "Select content should be mounted"
    );

    let initial_trigger_rect = harness.layout_rect("#select-trigger");
    let initial_content_rect = harness.layout_rect("#select-content");
    let initial_gap = initial_content_rect.y - (initial_trigger_rect.y + initial_trigger_rect.height);
    assert!(
        (initial_gap - 4.0).abs() <= 1.0,
        "Initial gap between trigger bottom and content top should be ~4.0px, got {initial_gap}"
    );

    // Dispatch window / viewport scroll down by 50px
    wheel_at_and_pump(&mut harness, 400.0, 300.0, 0.0, -50.0);

    let vp_scroll = harness.base().viewport_scroll();
    assert_eq!(
        vp_scroll.y, 50.0,
        "Viewport must have scrolled vertically by 50.0px"
    );

    let scrolled_trigger_rect = harness.layout_rect("#select-trigger");
    let scrolled_content_rect = harness.layout_rect("#select-content");

    // In viewport coordinates (what the user actually sees on screen):
    // 1. In-flow trigger moved upward relative to the viewport:
    let trigger_viewport_y = scrolled_trigger_rect.y - vp_scroll.y as f32;
    assert!(
        trigger_viewport_y < initial_trigger_rect.y - 40.0,
        "Trigger viewport y must have moved upward under window scroll (initial={}, viewport_y={})",
        initial_trigger_rect.y,
        trigger_viewport_y
    );

    // 2. Fixed content floating coordinate updated to stay anchored:
    let style = harness.attr("#select-content", "style").unwrap_or_default();
    assert!(
        style.contains("--monoxus-select-floating-y: 178px"),
        "Content floating y variable must update to 178px (228px - 50px), got style: {style}"
    );

    // 3. Maintained side-offset gap (~4.0px) in viewport coordinates:
    let content_viewport_y = 178.0f32;
    let viewport_gap = content_viewport_y - (trigger_viewport_y + scrolled_trigger_rect.height);
    assert!(
        (viewport_gap - 4.0).abs() <= 1.0,
        "Viewport gap between trigger bottom and content top must remain ~4.0px, got {viewport_gap}"
    );

    // 4. Maintained side-offset gap (~4.0px) in page coordinates:
    let page_gap = scrolled_content_rect.y - (scrolled_trigger_rect.y + scrolled_trigger_rect.height);
    assert!(
        (page_gap - 4.0).abs() <= 1.0,
        "Page gap between trigger bottom and content top under viewport scroll should remain ~4.0px, got {page_gap}"
    );
}


/// Test 7: Bottom viewport collision flips to Top on initial open without wheel scroll
///
/// Verifies Task 5.3 & Invariant 3:
/// When Select is mounted near the bottom of the viewport where downward space is insufficient,
/// clicking the trigger to open causes layout resolution via the native post-mount loop.
/// The dropdown automatically flips to PlacementSide::Top (content_rect.y + content_rect.height <= trigger_rect.y)
/// and reaches FloatingReadiness::Ready without any mouse wheel or scroll events.
#[test]
fn test_native_select_bottom_collision_flips_to_top_without_wheel_scroll() {
    let _lock = TEST_LOCK.lock().unwrap_or_else(|e| e.into_inner());

    #[component]
    fn BottomCollisionScenario() -> Element {
        rsx! {
            div {
                id: "viewport-root",
                style: "width: 800px; height: 600px; position: relative;",
                div {
                    style: "height: 480px;",
                    "Top spacer pushing trigger to bottom of 600px viewport"
                }
                playground_select::BasicFruitSelectSection {}
            }
        }
    }

    let mut harness = mount_native_scenario(BottomCollisionScenario);

    // Initial state: trigger collapsed
    assert_eq!(
        harness.attr("#select-trigger", "aria-expanded").as_deref(),
        Some("false"),
        "Select trigger should initially be collapsed"
    );

    // Click trigger to open dropdown
    click_and_pump(&mut harness, "#select-trigger");

    // Advance frames to simulate host render frame ticks and native layout resolution
    advance_frame_and_pump(&mut harness, 0.016);
    advance_frame_and_pump(&mut harness, 0.016);

    assert_eq!(
        harness.attr("#select-trigger", "aria-expanded").as_deref(),
        Some("true"),
        "Select trigger should be expanded"
    );
    assert!(
        harness.query("#select-content").is_some(),
        "Select content should be mounted"
    );

    let trigger_rect = harness.layout_rect("#select-trigger");
    let content_rect = harness.layout_rect("#select-content");

    // Assert that the content flipped above the trigger (PlacementSide::Top)
    assert!(
        content_rect.y + content_rect.height <= trigger_rect.y + 1.0,
        "Dropdown must flip to PlacementSide::Top: content bottom ({}) should be <= trigger top ({})",
        content_rect.y + content_rect.height,
        trigger_rect.y
    );

    // Assert that side_offset (~4px) is preserved between content bottom and trigger top
    let gap = trigger_rect.y - (content_rect.y + content_rect.height);
    assert!(
        (gap - 4.0).abs() <= 1.5,
        "Gap between flipped content bottom and trigger top should be ~4.0px, got {gap}"
    );

    // Assert that content readiness is ready (data-positioning-state="positioned")
    let positioning_state = harness.attr("#select-content", "data-positioning-state");
    assert_eq!(
        positioning_state.as_deref(),
        Some("positioned"),
        "Content must have data-positioning-state='positioned' (FloatingReadiness::Ready)"
    );

    // Assert that data-side is flipped to "top"
    let side_attr = harness.attr("#select-content", "data-side");
    assert_eq!(
        side_attr.as_deref(),
        Some("top"),
        "Content must have data-side='top'"
    );
}

/// Test 5: Toast W3C Pointer Capture out-of-bounds release, interactive button guard, and lostpointercapture recovery
#[test]
fn test_native_toast_pointer_capture_out_of_bounds_release_button_guard_and_lost_capture() {
    use blitz_traits::events::{
        BlitzPointerEvent, BlitzPointerId, MouseEventButton, MouseEventButtons, PointerCoords,
        PointerDetails, UiEvent,
    };
    use monoxus::toast::ToastActionData;

    let _lock = TEST_LOCK.lock().unwrap_or_else(|e| e.into_inner());
    let mouse_pid = BlitzPointerId::Mouse.to_dom_pointer_id();

    let drag_move_to = |harness: &mut blitz_test_harness::Harness<dioxus_native_dom::DioxusDocument>,
                        x: f32,
                        y: f32| {
        harness.dispatch(UiEvent::PointerMove(BlitzPointerEvent {
            id: BlitzPointerId::Mouse,
            is_primary: true,
            coords: PointerCoords {
                page_x: x,
                page_y: y,
                screen_x: x,
                screen_y: y,
                client_x: x,
                client_y: y,
            },
            button: MouseEventButton::Main,
            buttons: MouseEventButtons::Primary,
            mods: Default::default(),
            details: PointerDetails::default(),
            element: Default::default(),
            active_pointers: Default::default(),
        }));
        harness.pump();
    };

    let mut harness = mount_native_scenario(playground_toast::ToastPage);
    harness.set_viewport_size(800, 1400);
    harness.pump();

    {
        let _guard = dioxus::core::RuntimeGuard::new(harness.doc.vdom.runtime());
        toast::dismiss(None);
        TOAST_STORE.write().toasts.clear();
        toast::message(
            "Pointer Capture Toast",
            Some(ToastOptions {
                action: Some(ToastActionData::new("Undo", "Undo action")),
                duration: Some(Duration::from_secs(60)),
                ..Default::default()
            }),
        );
    }
    harness.pump();
    advance_frame_and_pump(&mut harness, 0.02);

    let toast_sel = "#monoxus-toast-viewport li";
    assert!(harness.query(toast_sel).is_some(), "Toast must mount");
    let toast_nid = harness.node(toast_sel);

    // -------------------------------------------------------------------------
    // Part A: Interactive <button> child guard (ToastAction button)
    // -------------------------------------------------------------------------
    let action_sel = "#monoxus-toast-viewport li button[aria-label=\"Undo action\"]";
    assert!(
        harness.query(action_sel).is_some(),
        "ToastAction button must be mounted"
    );
    let action_rect = harness.layout_rect(action_sel);
    harness.mouse_down_at(
        action_rect.x + action_rect.width * 0.5,
        action_rect.y + action_rect.height * 0.5,
    );
    drag_move_to(&mut harness, action_rect.x + 120.0, action_rect.y + 10.0);

    assert_eq!(
        harness.attr(toast_sel, "data-swiping"),
        None,
        "Pointer down on descendant <button> must NOT start toast swipe"
    );
    assert!(
        !harness.base().has_pointer_capture(toast_nid, mouse_pid),
        "Toast <li> must NOT acquire pointer capture when pressing a descendant <button>"
    );
    // Release outside the button so click does not dismiss toast
    harness.mouse_up_at(10.0, 10.0);
    harness.pump();
    assert_eq!(harness.attr(toast_sel, "data-state").as_deref(), Some("open"));

    // -------------------------------------------------------------------------
    // Part B: Mid-drag lostpointercapture cancels swipe and snaps back cleanly
    // -------------------------------------------------------------------------
    let rect = harness.layout_rect(toast_sel);
    let start_x = rect.x + 20.0;
    let start_y = rect.y + 16.0;

    harness.mouse_down_at(start_x, start_y);
    harness.pump();
    assert!(
        harness.base().has_pointer_capture(toast_nid, mouse_pid),
        "Toast <li> must acquire native W3C pointer capture on pointerdown"
    );

    drag_move_to(&mut harness, start_x + 30.0, start_y);
    assert_eq!(
        harness.attr(toast_sel, "data-swiping").as_deref(),
        Some("true"),
        "Toast must enter swiping state during captured drag"
    );

    // Force external release of pointer capture mid-drag -> next pointer event fires lostpointercapture
    harness
        .base_mut()
        .release_pointer_capture(toast_nid, mouse_pid)
        .expect("release_pointer_capture on active capture target must succeed");
    drag_move_to(&mut harness, 5.0, 5.0);

    assert_eq!(
        harness.attr(toast_sel, "data-swiping"),
        None,
        "lostpointercapture must cancel active swipe and clear data-swiping"
    );
    assert_eq!(
        harness.attr(toast_sel, "data-swipe-out").as_deref(),
        Some("false"),
        "lostpointercapture must snap back instead of dismissing"
    );
    assert_eq!(
        harness.attr(toast_sel, "data-state").as_deref(),
        Some("open"),
        "Toast must remain open after lostpointercapture snap-back"
    );
    {
        let _guard = dioxus::core::RuntimeGuard::new(harness.doc.vdom.runtime());
        assert!(
            !TOAST_STORE.read().interaction_state.swiping,
            "TOAST_STORE.interaction_state.swiping must reset to false on lostpointercapture"
        );
    }
    harness.mouse_up_at(5.0, 5.0);
    harness.pump();
    advance_frame_and_pump(&mut harness, 0.5);

    // -------------------------------------------------------------------------
    // Part C: Out-of-bounds drag + release outside toast bounds dismisses via pointer capture
    // -------------------------------------------------------------------------
    let rect = harness.layout_rect(toast_sel);
    let down_x = rect.x + 20.0;
    let down_y = rect.y + 16.0;
    // Release coordinates well outside the toast's vertical bounds (top of page y=20.0) and >80px right
    let out_x = (down_x + 95.0).min(795.0);
    let out_y = 20.0;
    assert!(
        out_y < rect.y - 20.0,
        "Release y={out_y} must be strictly outside toast rect y={}",
        rect.y
    );

    harness.mouse_down_at(down_x, down_y);
    harness.pump();
    // Move horizontally first to lock X swipe axis, then move completely outside vertical bounds
    drag_move_to(&mut harness, out_x, down_y);
    drag_move_to(&mut harness, out_x, out_y);
    assert_eq!(
        harness.attr(toast_sel, "data-swiping").as_deref(),
        Some("true")
    );

    harness.mouse_up_at(out_x, out_y);
    harness.pump();

    assert_eq!(
        harness.attr(toast_sel, "data-swipe-out").as_deref(),
        Some("true"),
        "Out-of-bounds pointerup under pointer capture must finalize swipe-out dismissal"
    );
    assert!(
        !harness.base().has_pointer_capture(toast_nid, mouse_pid),
        "Pointer capture must be released after pointerup"
    );

    advance_frame_and_pump(&mut harness, 0.25);
    assert!(
        harness.query(toast_sel).is_none(),
        "Toast must unmount after out-of-bounds captured swipe dismissal"
    );
}




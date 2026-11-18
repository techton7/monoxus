//! Native Data and Gesture Primitives Verification Test Suite.
//!
//! Validates data and gesture primitives (Select, Toast) using
//! Blitz test harness over shared scenarios directly from `examples/playground/`.

#![allow(unused_imports, dead_code)]

use dioxus::prelude::*;
use crate::playground_fixtures::{playground_select, playground_toast, playground_tooltip};
use crate::common;
use common::native_fixture::*;
use keyboard_types::{Code, Key, Modifiers, NamedKey};
use monoxus::toast::{TOAST_STORE, ToastOptions, toast};
use std::time::Duration;





/// Test 1: Select document order, roving focus, and synchronous/closed typeahead
///
/// Verifies initial closed state, trigger mounting, opening via click,
/// content mounting and relative placement below trigger, roving focus across
/// items in true document order via arrow keys, selection commit closing the dropdown,
/// synchronous multi-character typeahead ('b' -> banana, 'bl' -> blueberry), and
/// closed-state typeahead commit ('p' -> pineapple).
#[test]
fn test_native_select_document_order_and_roving_focus() {
    let _lock = NATIVE_TEST_LOCK.lock().unwrap_or_else(|e| e.into_inner());
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
    let _lock = NATIVE_TEST_LOCK.lock().unwrap_or_else(|e| e.into_inner());

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
    let _lock = NATIVE_TEST_LOCK.lock().unwrap_or_else(|e| e.into_inner());

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
    let _lock = NATIVE_TEST_LOCK.lock().unwrap_or_else(|e| e.into_inner());
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
    let _lock = NATIVE_TEST_LOCK.lock().unwrap_or_else(|e| e.into_inner());
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
    let _lock = NATIVE_TEST_LOCK.lock().unwrap_or_else(|e| e.into_inner());

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
    let _lock = NATIVE_TEST_LOCK.lock().unwrap_or_else(|e| e.into_inner());

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
    let _lock = NATIVE_TEST_LOCK.lock().unwrap_or_else(|e| e.into_inner());

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
    let _lock = NATIVE_TEST_LOCK.lock().unwrap_or_else(|e| e.into_inner());
    use blitz_traits::events::{
        BlitzPointerEvent, BlitzPointerId, MouseEventButton, MouseEventButtons, PointerCoords,
        PointerDetails, UiEvent,
    };
    use monoxus::toast::ToastActionData;
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

fn visual_y_bounds(
    harness: &blitz_test_harness::Harness<dioxus_native_dom::DioxusDocument>,
    selector: &str,
) -> (f32, f32) {
    let node_id = harness.node(selector);
    let rect = harness.layout_rect_of(node_id);
    let doc = harness.base();
    let node = doc.get_node(node_id).expect("node must exist");
    let (scale_y, ty) = node
        .transform()
        .as_ref()
        .map(|aff| {
            let c = aff.as_coeffs();
            (c[3] as f32, c[5] as f32)
        })
        .unwrap_or((1.0, 0.0));
    let top = rect.y + ty;
    let bottom = top + rect.height * scale_y;
    (top, bottom)
}

/// Test 9: 3-toast stack collapsed clamping, hover-expanded non-overlapping card layout rects,
/// `::after` gap-bridge hover continuity, `[data-close-button]` 24x24 geometry, and timer pause.
#[test]
fn test_native_toast_stack_collapsed_clamp_hover_expand_gap_bridge_and_timer_pause() {
    let _lock = NATIVE_TEST_LOCK.lock().unwrap_or_else(|e| e.into_inner());

    let mut harness = mount_native_scenario(playground_toast::ToastPage);
    harness.set_viewport_size(800, 1400);
    harness.pump();

    {
        let _guard = dioxus::core::RuntimeGuard::new(harness.doc.vdom.runtime());
        toast::dismiss(None);
        let mut store = TOAST_STORE.write();
        store.toasts.clear();
        store.toast_heights.clear();
        store.config = monoxus::toast::ToastConfig::default();
    }
    harness.pump();

    // Dispatch 3 stacked toasts of varying line counts via #dispatch-stack-3
    click_and_pump(&mut harness, "#dispatch-stack-3");
    advance_frame_and_pump(&mut harness, 0.4);

    let sel0 = "#monoxus-toast-viewport li[data-index=\"0\"]";
    let sel1 = "#monoxus-toast-viewport li[data-index=\"1\"]";
    let sel2 = "#monoxus-toast-viewport li[data-index=\"2\"]";

    assert!(harness.query(sel0).is_some(), "Front toast must mount");
    assert!(harness.query(sel1).is_some(), "Middle toast must mount");
    assert!(harness.query(sel2).is_some(), "Back toast must mount");

    // Verify default [data-close-button] has visible 24x24 clickable geometry
    let close_rect = harness.layout_rect("#monoxus-toast-viewport li[data-index=\"0\"] [data-close-button]");
    assert!(
        (close_rect.width - 24.0).abs() <= 1.0 && (close_rect.height - 24.0).abs() <= 1.0,
        "Default [data-close-button] must render at 24x24px, got {}x{}",
        close_rect.width,
        close_rect.height
    );

    // Verify --front-toast-height, --gap, --width published on <ol>
    let vp_style = harness
        .attr("#monoxus-toast-viewport", "style")
        .unwrap_or_default();
    assert!(
        vp_style.contains("--front-toast-height:"),
        "Viewport must publish --front-toast-height after mount measurement, got: {vp_style}"
    );
    assert!(vp_style.contains("--gap: 14px;"));
    assert!(vp_style.contains("--width: 356px;"));

    // Verify collapsed back cards are clamped to front card height via var(--front-toast-height, auto)
    let r0_col = harness.layout_rect(sel0);
    let r1_col = harness.layout_rect(sel1);
    let r2_col = harness.layout_rect(sel2);
    assert!(r0_col.height > 30.0, "Front card must have real measured height");
    assert!(
        (r1_col.height - r0_col.height).abs() <= 1.5
            && (r2_col.height - r0_col.height).abs() <= 1.5,
        "Collapsed back cards must clamp to --front-toast-height ({}): got r1={}, r2={}",
        r0_col.height,
        r1_col.height,
        r2_col.height
    );

    // Record remaining duration before hover
    let rem_before_hover = {
        let _guard = dioxus::core::RuntimeGuard::new(harness.doc.vdom.runtime());
        TOAST_STORE.read().toasts[0].remaining_duration
    };

    // Hover over front toast to expand stack and pause timers
    let (cx, cy) = r0_col.center();
    harness.move_mouse_to(cx, cy);
    harness.pump();
    advance_frame_and_pump(&mut harness, 0.45);

    assert_eq!(
        harness
            .attr("#monoxus-toast-viewport", "data-expanded")
            .as_deref(),
        Some("true"),
        "Hovering toast stack must set data-expanded='true'"
    );
    {
        let _guard = dioxus::core::RuntimeGuard::new(harness.doc.vdom.runtime());
        let store = TOAST_STORE.read();
        assert!(store.expanded, "Store must be expanded while hovered");
        assert!(store.paused, "Timers must be paused while hovered");
    }

    // Advance 0.5s while hovered -> remaining duration must stay frozen
    advance_frame_and_pump(&mut harness, 0.5);
    {
        let _guard = dioxus::core::RuntimeGuard::new(harness.doc.vdom.runtime());
        let rem_during_hover = TOAST_STORE.read().toasts[0].remaining_duration;
        let drift = rem_before_hover.abs_diff(rem_during_hover);
        assert!(
            drift <= Duration::from_millis(50),
            "Remaining duration must stay paused while hovered (before={rem_before_hover:?}, during={rem_during_hover:?})"
        );
    }

    // Verify expanded cards have non-overlapping visual screen intervals separated by ~14px gap
    let (top0, bot0) = visual_y_bounds(&harness, sel0);
    let (top1, bot1) = visual_y_bounds(&harness, sel1);
    let (top2, bot2) = visual_y_bounds(&harness, sel2);

    assert!(
        bot1 <= top0 + 1.0,
        "Expanded middle card [{top1}, {bot1}] must not overlap front card [{top0}, {bot0}]"
    );
    assert!(
        bot2 <= top1 + 1.0,
        "Expanded back card [{top2}, {bot2}] must not overlap middle card [{top1}, {bot1}]"
    );
    let gap_0_1 = top0 - bot1;
    let gap_1_2 = top1 - bot2;
    assert!(
        (gap_0_1 - 14.0).abs() <= 2.5 && (gap_1_2 - 14.0).abs() <= 2.5,
        "Inter-card expanded gaps must match --gap (14px), got gap_0_1={gap_0_1}, gap_1_2={gap_1_2}"
    );

    // Move pointer into the middle of the 14px gap between card 0 and card 1 ->
    // ::after gap-bridge must hit-test as card 0 and keep hover active so stack does NOT collapse
    let gap_mid_y = (top0 + bot1) * 0.5;
    let card0_node = harness.node(sel0);
    assert_eq!(
        harness.base().element_from_point(cx, gap_mid_y),
        Some(card0_node),
        "Blitz Stylo/Taffy ::after pseudo-element gap-bridge must hit-test to front toast <li> at gap midpoint ({cx}, {gap_mid_y})"
    );
    harness.move_mouse_to(cx, gap_mid_y);
    harness.pump();
    advance_frame_and_pump(&mut harness, 0.05);

    assert_eq!(
        harness
            .attr("#monoxus-toast-viewport", "data-expanded")
            .as_deref(),
        Some("true"),
        "Expanded ::after gap-bridge must keep stack expanded when pointer traverses inter-card gap (y={gap_mid_y})"
    );

    // Move pointer outside viewport -> stack collapses and timers resume
    harness.move_mouse_to(20.0, 20.0);
    harness.pump();
    advance_frame_and_pump(&mut harness, 0.05);

    assert_eq!(
        harness
            .attr("#monoxus-toast-viewport", "data-expanded")
            .as_deref(),
        Some("false"),
        "Moving pointer outside viewport must collapse stack"
    );
    {
        let _guard = dioxus::core::RuntimeGuard::new(harness.doc.vdom.runtime());
        assert!(!TOAST_STORE.read().paused, "Timers must resume after mouseleave");
    }

    // Also verify opposite height polarity: TALL back card (4 lines) behind SHORT front card (1 line)
    // clamps down to short --front-toast-height when collapsed, then expands to full intrinsic height on hover.
    let (tall_back_id, short_front_id) = {
        let _guard = dioxus::core::RuntimeGuard::new(harness.doc.vdom.runtime());
        let mut store = TOAST_STORE.write();
        store.toasts.clear();
        store.toast_heights.clear();
        drop(store);
        let tall_back = toast::warning(
            "Multi-line tall background card",
            Some(ToastOptions {
                description: Some(
                    "Line one of lengthy diagnostic payload that wraps across several rows inside the 356px card width so its intrinsic height substantially exceeds a one-line front toast.".to_string(),
                ),
                duration: Some(Duration::from_secs(60)),
                ..Default::default()
            }),
        );
        let short_front = toast::success(
            "Short front",
            Some(ToastOptions {
                duration: Some(Duration::from_secs(60)),
                ..Default::default()
            }),
        );
        (tall_back, short_front)
    };
    harness.pump();
    advance_frame_and_pump(&mut harness, 0.05);
    advance_frame_and_pump(&mut harness, 0.05);

    let short_sel = format!("#monoxus-toast-{}", short_front_id.0);
    let tall_sel = format!("#monoxus-toast-{}", tall_back_id.0);
    let short_col = harness.layout_rect(&short_sel);
    let tall_col = harness.layout_rect(&tall_sel);
    assert!(
        (tall_col.height - short_col.height).abs() <= 1.5,
        "Tall back card must clamp down to short front card --front-toast-height ({}) when collapsed, got {}",
        short_col.height,
        tall_col.height
    );

    let (scx, scy) = short_col.center();
    harness.move_mouse_to(scx, scy);
    harness.pump();
    advance_frame_and_pump(&mut harness, 0.45);

    let short_exp = harness.layout_rect(&short_sel);
    let tall_exp = harness.layout_rect(&tall_sel);
    assert!(
        tall_exp.height > short_exp.height + 20.0,
        "Tall back card must restore full unclamped intrinsic height on hover expand (short={}, tall={})",
        short_exp.height,
        tall_exp.height
    );
    let (s_top, _) = visual_y_bounds(&harness, &short_sel);
    let (_, t_bot) = visual_y_bounds(&harness, &tall_sel);
    assert!(
        ((s_top - t_bot) - 14.0).abs() <= 2.5,
        "Tall expanded back card must sit ~14px above short front card, got gap={}",
        s_top - t_bot
    );
}

/// Test 10: Swipe immunity for `ToastType::Loading` and `dismissible: Some(false)`.
#[test]
fn test_native_toast_loading_and_non_dismissible_swipe_immunity() {
    let _lock = NATIVE_TEST_LOCK.lock().unwrap_or_else(|e| e.into_inner());

    let mut harness = mount_native_scenario(playground_toast::ToastPage);
    harness.set_viewport_size(800, 1400);
    harness.pump();

    // 1. ToastType::Loading must ignore pointer swipe and omit close button
    let loading_id = {
        let _guard = dioxus::core::RuntimeGuard::new(harness.doc.vdom.runtime());
        toast::dismiss(None);
        let mut store = TOAST_STORE.write();
        store.toasts.clear();
        store.toast_heights.clear();
        drop(store);
        toast::loading("Syncing snapshot...", None)
    };
    harness.pump();
    advance_frame_and_pump(&mut harness, 0.02);

    let loading_sel = format!("#monoxus-toast-{}", loading_id.0);
    assert_eq!(
        harness.attr(&loading_sel, "data-dismissible").as_deref(),
        Some("false")
    );
    assert!(
        harness
            .query(&format!("{loading_sel} [data-close-button]"))
            .is_none(),
        "Loading toast must not render default close button"
    );

    let rect = harness.layout_rect(&loading_sel);
    harness.mouse_down_at(rect.x + 20.0, rect.y + 18.0);
    harness.move_mouse_to(rect.x + 120.0, rect.y + 18.0);
    harness.pump();
    assert_eq!(
        harness.attr(&loading_sel, "data-swiping"),
        None,
        "Loading toast must be immune to pointer swipe"
    );
    harness.mouse_up_at(rect.x + 120.0, rect.y + 18.0);
    harness.pump();
    advance_frame_and_pump(&mut harness, 0.3);
    assert!(
        harness.query(&loading_sel).is_some(),
        "Loading toast must remain mounted after attempted swipe"
    );
    assert_eq!(
        harness.attr(&loading_sel, "data-state").as_deref(),
        Some("open")
    );

    // 2. dismissible: Some(false) must also ignore pointer swipe and omit close button
    let locked_id = {
        let _guard = dioxus::core::RuntimeGuard::new(harness.doc.vdom.runtime());
        TOAST_STORE.write().toasts.clear();
        toast::warning(
            "Mandatory security notice",
            Some(ToastOptions {
                dismissible: Some(false),
                duration: Some(Duration::from_secs(60)),
                ..Default::default()
            }),
        )
    };
    harness.pump();
    advance_frame_and_pump(&mut harness, 0.02);

    let locked_sel = format!("#monoxus-toast-{}", locked_id.0);
    assert_eq!(
        harness.attr(&locked_sel, "data-dismissible").as_deref(),
        Some("false")
    );
    assert!(
        harness
            .query(&format!("{locked_sel} [data-close-button]"))
            .is_none(),
        "dismissible: Some(false) toast must not render close button"
    );

    let locked_rect = harness.layout_rect(&locked_sel);
    harness.mouse_down_at(locked_rect.x + 20.0, locked_rect.y + 18.0);
    harness.move_mouse_to(locked_rect.x + 120.0, locked_rect.y + 18.0);
    harness.pump();
    assert_eq!(
        harness.attr(&locked_sel, "data-swiping"),
        None,
        "dismissible: false toast must be immune to pointer swipe"
    );
    harness.mouse_up_at(locked_rect.x + 120.0, locked_rect.y + 18.0);
    harness.pump();
    advance_frame_and_pump(&mut harness, 0.3);
    assert_eq!(
        harness.attr(&locked_sel, "data-state").as_deref(),
        Some("open")
    );
}

/// Test 11: Global `F8` landmark hotkey focus, `Escape` targeted & frontmost dismissal,
/// and focus restoration to `#monoxus-toast-viewport`.
#[test]
fn test_native_toast_f8_landmark_focus_and_escape_dismissal() {
    let _lock = NATIVE_TEST_LOCK.lock().unwrap_or_else(|e| e.into_inner());

    let mut harness = mount_native_scenario(playground_toast::ToastPage);
    harness.set_viewport_size(800, 1400);
    harness.pump();

    let (id_back, id_front) = {
        let _guard = dioxus::core::RuntimeGuard::new(harness.doc.vdom.runtime());
        toast::dismiss(None);
        let mut store = TOAST_STORE.write();
        store.toasts.clear();
        store.toast_heights.clear();
        drop(store);
        let b = toast::message(
            "Older toast",
            Some(ToastOptions {
                duration: Some(Duration::from_secs(60)),
                ..Default::default()
            }),
        );
        let f = toast::success(
            "Newest front toast",
            Some(ToastOptions {
                duration: Some(Duration::from_secs(60)),
                ..Default::default()
            }),
        );
        (b, f)
    };
    harness.pump();
    advance_frame_and_pump(&mut harness, 0.02);

    // 1. Pressing Escape while focused outside #monoxus-toast-viewport must NOT dismiss toasts
    focus_node(&mut harness, "#dispatch-info");
    send_key_and_pump(&mut harness, Key::Named(NamedKey::Escape), Code::Escape);
    assert_eq!(
        harness
            .attr(&format!("#monoxus-toast-{}", id_front.0), "data-state")
            .as_deref(),
        Some("open"),
        "Escape outside viewport must never intercept or dismiss toasts"
    );

    // 2. Pressing F8 focuses #monoxus-toast-viewport and expands/pauses the stack
    send_key_and_pump(&mut harness, Key::Named(NamedKey::F8), Code::F8);
    assert_native_focus_id(&harness, "#monoxus-toast-viewport");
    assert_eq!(
        harness
            .attr("#monoxus-toast-viewport", "data-expanded")
            .as_deref(),
        Some("true"),
        "Focusing viewport via F8 must expand stack"
    );

    // 3. Pressing Escape on #monoxus-toast-viewport dismisses ONLY frontmost toast (id_front)
    send_key_and_pump(&mut harness, Key::Named(NamedKey::Escape), Code::Escape);
    assert_eq!(
        harness
            .attr(&format!("#monoxus-toast-{}", id_front.0), "data-state")
            .as_deref(),
        Some("closed"),
        "Escape on viewport must dismiss frontmost active toast"
    );
    assert_eq!(
        harness
            .attr(&format!("#monoxus-toast-{}", id_back.0), "data-state")
            .as_deref(),
        Some("open"),
        "Escape on viewport must leave older toasts open"
    );
    advance_frame_and_pump(&mut harness, 0.4);

    // 4. Add two more toasts, focus middle card directly, and press Escape ->
    //    dismisses ONLY that focused card and restores focus to #monoxus-toast-viewport
    let (id_mid, id_top) = {
        let _guard = dioxus::core::RuntimeGuard::new(harness.doc.vdom.runtime());
        let m = toast::info(
            "Middle target toast",
            Some(ToastOptions {
                duration: Some(Duration::from_secs(60)),
                ..Default::default()
            }),
        );
        let t = toast::warning(
            "Top untouched toast",
            Some(ToastOptions {
                duration: Some(Duration::from_secs(60)),
                ..Default::default()
            }),
        );
        (m, t)
    };
    harness.pump();
    advance_frame_and_pump(&mut harness, 0.02);

    let mid_sel = format!("#monoxus-toast-{}", id_mid.0);
    focus_node(&mut harness, &mid_sel);
    assert_native_focus_id(&harness, &mid_sel);

    send_key_and_pump(&mut harness, Key::Named(NamedKey::Escape), Code::Escape);
    assert_eq!(
        harness.attr(&mid_sel, "data-state").as_deref(),
        Some("closed"),
        "Escape on focused toast card must dismiss that specific card"
    );
    assert_eq!(
        harness
            .attr(&format!("#monoxus-toast-{}", id_top.0), "data-state")
            .as_deref(),
        Some("open"),
        "Front toast must remain open when Escape targeted middle card"
    );
    assert_eq!(
        harness
            .attr(&format!("#monoxus-toast-{}", id_back.0), "data-state")
            .as_deref(),
        Some("open"),
        "Back toast must remain open when Escape targeted middle card"
    );
    assert_native_focus_id(&harness, "#monoxus-toast-viewport");

    // 5. Clicking [data-close-button] inside a focused toast also restores focus to #monoxus-toast-viewport
    let top_close_sel = format!("#monoxus-toast-{} [data-close-button]", id_top.0);
    focus_node(&mut harness, &top_close_sel);
    click_and_pump(&mut harness, &top_close_sel);
    assert_eq!(
        harness
            .attr(&format!("#monoxus-toast-{}", id_top.0), "data-state")
            .as_deref(),
        Some("closed")
    );
    assert_native_focus_id(&harness, "#monoxus-toast-viewport");
}

/// Test 12: Tooltip 184px grid layout wraps unbroken tokens cleanly without horizontal overflow.
#[test]
fn test_native_tooltip_184px_grid_long_token_no_horizontal_overflow() {
    let _lock = NATIVE_TEST_LOCK.lock().unwrap_or_else(|e| e.into_inner());

    #[component]
    fn ZeroDelayTooltipScenario() -> Element {
        rsx! {
            playground_tooltip::TooltipPlayground {
                delay_duration_ms: Some(0),
            }
        }
    }

    let mut harness = mount_native_scenario(ZeroDelayTooltipScenario);
    harness.set_viewport_size(800, 800);
    harness.pump();

    // Hover primary trigger (contains unbroken `playground-tooltip-provider` code token)
    let (tx1, ty1) = harness.center_of("#tooltip-trigger");
    harness.move_mouse_to(tx1, ty1);
    harness.pump();
    advance_frame_and_pump(&mut harness, 0.05);
    advance_frame_and_pump(&mut harness, 0.05);

    assert!(
        harness.query("#tooltip-content").is_some(),
        "First tooltip content must mount on hover"
    );
    let c1 = harness.layout_rect("#tooltip-content");
    assert!(
        (c1.width - 184.0).abs() <= 1.0,
        "First tooltip border-box width must stay clamped to 184px, got {}",
        c1.width
    );
    let p1 = harness.layout_rect("#tooltip-content p");
    let code1 = harness.layout_rect("#tooltip-content code");
    assert!(
        p1.x + p1.width <= c1.x + c1.width + 0.5
            && code1.x + code1.width <= c1.x + c1.width + 0.5,
        "Inner <p> ({:?}) and <code> ({:?}) must wrap within 184px tooltip ({:?})",
        p1,
        code1,
        c1
    );

    // Move to second trigger (skip-delay instant open) and verify its 184px wrapping too
    let (tx2, ty2) = harness.center_of("#tooltip-trigger-2");
    harness.move_mouse_to(tx2, ty2);
    harness.pump();
    advance_frame_and_pump(&mut harness, 0.05);
    advance_frame_and_pump(&mut harness, 0.05);

    assert!(
        harness.query("#tooltip-content-2").is_some(),
        "Second tooltip content must mount on skip-delay handoff"
    );
    let c2 = harness.layout_rect("#tooltip-content-2");
    assert!(
        (c2.width - 184.0).abs() <= 1.0,
        "Second tooltip border-box width must stay clamped to 184px, got {}",
        c2.width
    );
    let p2 = harness.layout_rect("#tooltip-content-2 p");
    assert!(
        p2.x + p2.width <= c2.x + c2.width + 0.5,
        "Second tooltip <p> ({:?}) must not overflow 184px tooltip ({:?})",
        p2,
        c2
    );
}

/// Test 13: 3-card stack with distinct natural heights measures all 3 unscaled auto heights
/// while collapsed background cards clamp to `--front-toast-height` and restore on hover expand.
#[test]
fn test_native_toast_three_card_distinct_heights_collapsed_clamp_and_hover_expand() {
    let _lock = NATIVE_TEST_LOCK.lock().unwrap_or_else(|e| e.into_inner());

    let mut harness = mount_native_scenario(playground_toast::ToastPage);
    harness.set_viewport_size(800, 1400);
    harness.pump();

    let (id_tall, id_mid, id_short) = {
        let _guard = dioxus::core::RuntimeGuard::new(harness.doc.vdom.runtime());
        toast::dismiss(None);
        let mut store = TOAST_STORE.write();
        store.toasts.clear();
        store.toast_heights.clear();
        store.config = monoxus::toast::ToastConfig::default();
        drop(store);

        let tall = toast::error(
            "Back card with extended multi-line diagnostic summary",
            Some(ToastOptions {
                description: Some(
                    "First diagnostic sentence detailing background replication state. Second sentence adding replica lag context across regions. Third sentence describing automatic retry backoff and recovery.".to_string(),
                ),
                duration: Some(Duration::from_secs(60)),
                ..Default::default()
            }),
        );
        let mid = toast::info(
            "Middle card moderate summary",
            Some(ToastOptions {
                description: Some(
                    "Two-line supporting description that makes this card taller than a one-line toast.".to_string(),
                ),
                duration: Some(Duration::from_secs(60)),
                ..Default::default()
            }),
        );
        let short = toast::success(
            "Short front",
            Some(ToastOptions {
                duration: Some(Duration::from_secs(60)),
                ..Default::default()
            }),
        );
        (tall, mid, short)
    };
    harness.pump();
    advance_frame_and_pump(&mut harness, 0.05);
    advance_frame_and_pump(&mut harness, 0.05);

    let sel_short = format!("#monoxus-toast-{}", id_short.0);
    let sel_mid = format!("#monoxus-toast-{}", id_mid.0);
    let sel_tall = format!("#monoxus-toast-{}", id_tall.0);

    for sel in [&sel_short, &sel_mid, &sel_tall] {
        assert_eq!(
            harness.attr(sel, "data-mounted").as_deref(),
            Some("true"),
            "{sel} must publish data-mounted='true' after mount"
        );
        let style = harness.attr(sel, "style").unwrap_or_default();
        assert!(
            style.contains("--swipe-amount-x: 0px; --swipe-amount-y: 0px; --drag-offset: 0px;"),
            "{sel} must unconditionally publish zeroed swipe CSS variables, got: {style}"
        );
    }

    let (h_short, h_mid, h_tall) = {
        let _guard = dioxus::core::RuntimeGuard::new(harness.doc.vdom.runtime());
        let store = TOAST_STORE.read();
        (
            store.toast_height(id_short).expect("front height recorded"),
            store.toast_height(id_mid).expect("middle height recorded"),
            store.toast_height(id_tall).expect("back height recorded"),
        )
    };
    assert!(
        h_mid > h_short + 12.0 && h_tall > h_mid + 12.0,
        "Document::measure_auto_height must record 3 distinct natural heights even while collapsed: short={h_short}, mid={h_mid}, tall={h_tall}"
    );

    let r_short_col = harness.layout_rect(&sel_short);
    let r_mid_col = harness.layout_rect(&sel_mid);
    let r_tall_col = harness.layout_rect(&sel_tall);
    assert!(
        (r_mid_col.height - r_short_col.height).abs() <= 1.5
            && (r_tall_col.height - r_short_col.height).abs() <= 1.5,
        "Collapsed background cards must clamp to --front-toast-height ({}): mid={}, tall={}",
        r_short_col.height,
        r_mid_col.height,
        r_tall_col.height
    );

    // Hover front toast -> stack expands and each card restores its distinct natural --height
    let (cx, cy) = r_short_col.center();
    harness.move_mouse_to(cx, cy);
    harness.pump();
    advance_frame_and_pump(&mut harness, 0.45);

    let r_short_exp = harness.layout_rect(&sel_short);
    let r_mid_exp = harness.layout_rect(&sel_mid);
    let r_tall_exp = harness.layout_rect(&sel_tall);
    assert!(
        (r_short_exp.height - h_short as f32).abs() <= 1.5
            && (r_mid_exp.height - h_mid as f32).abs() <= 1.5
            && (r_tall_exp.height - h_tall as f32).abs() <= 1.5,
        "Expanded layout rects must match measured natural heights: rects=({}, {}, {}), measured=({h_short}, {h_mid}, {h_tall})",
        r_short_exp.height,
        r_mid_exp.height,
        r_tall_exp.height
    );

    let (top0, _) = visual_y_bounds(&harness, &sel_short);
    let (top1, bot1) = visual_y_bounds(&harness, &sel_mid);
    let (_, bot2) = visual_y_bounds(&harness, &sel_tall);
    assert!(
        ((top0 - bot1) - 14.0).abs() <= 2.5 && ((top1 - bot2) - 14.0).abs() <= 2.5,
        "Expanded distinct-height cards must maintain 14px inter-card gaps: gap01={}, gap12={}",
        top0 - bot1,
        top1 - bot2
    );
}

/// Test 14: In-place content updates bump `content_generation` and re-measure both a collapsed
/// background card and the frontmost card (updating `--front-toast-height`).
#[test]
fn test_native_toast_in_place_content_update_remeasures_clamped_and_front_height() {
    let _lock = NATIVE_TEST_LOCK.lock().unwrap_or_else(|e| e.into_inner());

    let mut harness = mount_native_scenario(playground_toast::ToastPage);
    harness.set_viewport_size(800, 1400);
    harness.pump();

    let id_back = monoxus::toast::ToastId(9101);
    let id_front = monoxus::toast::ToastId(9102);

    {
        let _guard = dioxus::core::RuntimeGuard::new(harness.doc.vdom.runtime());
        toast::dismiss(None);
        let mut store = TOAST_STORE.write();
        store.toasts.clear();
        store.toast_heights.clear();
        store.config = monoxus::toast::ToastConfig::default();
        drop(store);

        toast::info(
            "Short back",
            Some(ToastOptions {
                id: Some(id_back),
                duration: Some(Duration::from_secs(60)),
                ..Default::default()
            }),
        );
        toast::success(
            "Short front",
            Some(ToastOptions {
                id: Some(id_front),
                duration: Some(Duration::from_secs(60)),
                ..Default::default()
            }),
        );
    }
    harness.pump();
    advance_frame_and_pump(&mut harness, 0.05);
    advance_frame_and_pump(&mut harness, 0.05);

    let sel_back = format!("#monoxus-toast-{}", id_back.0);
    let sel_front = format!("#monoxus-toast-{}", id_front.0);

    let (h_back_before, h_front_before) = {
        let _guard = dioxus::core::RuntimeGuard::new(harness.doc.vdom.runtime());
        let store = TOAST_STORE.read();
        (
            store.toast_height(id_back).expect("initial back height"),
            store.toast_height(id_front).expect("initial front height"),
        )
    };

    // 1. Update collapsed background toast in-place with multi-line description
    {
        let _guard = dioxus::core::RuntimeGuard::new(harness.doc.vdom.runtime());
        toast::warning(
            "Updated back card with multi-line details",
            Some(ToastOptions {
                id: Some(id_back),
                description: Some(
                    "Expanded payload appended in place while card sits collapsed behind the shorter front card so content_generation triggers auto-height re-measurement.".to_string(),
                ),
                duration: Some(Duration::from_secs(60)),
                ..Default::default()
            }),
        );
    }
    harness.pump();
    advance_frame_and_pump(&mut harness, 0.05);
    advance_frame_and_pump(&mut harness, 0.05);

    let (h_back_after, back_content_gen) = {
        let _guard = dioxus::core::RuntimeGuard::new(harness.doc.vdom.runtime());
        let store = TOAST_STORE.read();
        let cg = store
            .toasts
            .iter()
            .find(|t| t.id == id_back)
            .map(|t| t.content_generation)
            .unwrap_or(0);
        (store.toast_height(id_back).expect("updated back height"), cg)
    };
    assert_eq!(back_content_gen, 1, "In-place add_toast update must bump content_generation");
    assert!(
        h_back_after > h_back_before + 20.0,
        "Collapsed back card in-place update must re-measure taller auto height (before={h_back_before}, after={h_back_after})"
    );
    assert!(
        (harness.layout_rect(&sel_back).height - h_front_before as f32).abs() <= 1.5,
        "Updated collapsed back card must remain visually clamped to unchanged front card height ({h_front_before})"
    );

    // 2. Update front toast in-place via update_toast -> re-measures and updates --front-toast-height
    {
        let _guard = dioxus::core::RuntimeGuard::new(harness.doc.vdom.runtime());
        assert!(TOAST_STORE.write().update_toast(id_front, |item| {
            item.title = "Front card expanded in place".to_string();
            item.description = Some(
                "Multi-line description added to front card in place so --front-toast-height grows automatically.".to_string(),
            );
        }));
    }
    harness.pump();
    advance_frame_and_pump(&mut harness, 0.05);
    advance_frame_and_pump(&mut harness, 0.05);

    let h_front_after = {
        let _guard = dioxus::core::RuntimeGuard::new(harness.doc.vdom.runtime());
        TOAST_STORE
            .read()
            .toast_height(id_front)
            .expect("updated front height")
    };
    assert!(
        h_front_after > h_front_before + 20.0,
        "Front card update_toast must re-measure taller auto height (before={h_front_before}, after={h_front_after})"
    );
    let vp_style = harness
        .attr("#monoxus-toast-viewport", "style")
        .unwrap_or_default();
    assert!(
        vp_style.contains(&format!("--front-toast-height: {h_front_after}px;")),
        "Viewport must publish updated --front-toast-height: {h_front_after}px, got: {vp_style}"
    );
    assert!(
        (harness.layout_rect(&sel_front).height - h_front_after as f32).abs() <= 1.5
            && (harness.layout_rect(&sel_back).height - h_front_after as f32).abs() <= 1.5,
        "Front and clamped back layout heights must track updated --front-toast-height ({h_front_after})"
    );
}

/// Test 15: Pointer snapback on dismissible card, pointer click on `Loading` card, `#clear-all`
/// queue-empty interaction reset, and `F8` -> `Escape` pre-hotkey focus restoration.
#[test]
fn test_native_toast_pointer_snapback_loading_click_and_clear_all_focus_hover_reset() {
    let _lock = NATIVE_TEST_LOCK.lock().unwrap_or_else(|e| e.into_inner());

    let mut harness = mount_native_scenario(playground_toast::ToastPage);
    harness.set_viewport_size(800, 1400);
    harness.pump();

    // 1. Short drag / snapback on dismissible <li tabindex="0"> must blur card and keep focused=false
    let dismissible_id = {
        let _guard = dioxus::core::RuntimeGuard::new(harness.doc.vdom.runtime());
        toast::dismiss(None);
        let mut store = TOAST_STORE.write();
        store.toasts.clear();
        store.toast_heights.clear();
        store.config = monoxus::toast::ToastConfig::default();
        drop(store);
        toast::info(
            "Snapback focus hygiene card",
            Some(ToastOptions {
                duration: Some(Duration::from_secs(60)),
                ..Default::default()
            }),
        )
    };
    harness.pump();
    advance_frame_and_pump(&mut harness, 0.05);

    let d_sel = format!("#monoxus-toast-{}", dismissible_id.0);
    let d_nid = harness.node(&d_sel);
    let d_rect = harness.layout_rect(&d_sel);
    harness.mouse_down_at(d_rect.x + 24.0, d_rect.y + 18.0);
    harness.move_mouse_to(d_rect.x + 34.0, d_rect.y + 18.0);
    harness.mouse_up_at(d_rect.x + 34.0, d_rect.y + 18.0);
    harness.pump();
    advance_frame_and_pump(&mut harness, 0.05);
    // Move pointer away so only focus could hold expanded state if leaked
    harness.move_mouse_to(20.0, 20.0);
    harness.pump();
    advance_frame_and_pump(&mut harness, 0.05);

    assert_ne!(
        harness.base().active_focus_node_id(),
        Some(d_nid),
        "Pointer snapback on dismissible <li> must blur active focus off the card"
    );
    {
        let _guard = dioxus::core::RuntimeGuard::new(harness.doc.vdom.runtime());
        let store = TOAST_STORE.read();
        assert!(
            !store.interaction_state.focused && !store.expanded,
            "Pointer snapback must not leave interaction_state.focused or expanded stuck true"
        );
    }

    // 2. Clicking Loading <li tabindex="0"> must also blur card and never trap focus
    let loading_id = {
        let _guard = dioxus::core::RuntimeGuard::new(harness.doc.vdom.runtime());
        toast::loading("Persistent loading upload...", None)
    };
    harness.pump();
    advance_frame_and_pump(&mut harness, 0.05);

    let l_sel = format!("#monoxus-toast-{}", loading_id.0);
    let l_nid = harness.node(&l_sel);
    let l_rect = harness.layout_rect(&l_sel);
    click_at_and_pump(&mut harness, l_rect.x + 24.0, l_rect.y + 18.0);
    harness.move_mouse_to(20.0, 20.0);
    harness.pump();
    advance_frame_and_pump(&mut harness, 0.05);

    assert_ne!(
        harness.base().active_focus_node_id(),
        Some(l_nid),
        "Clicking Loading <li> must blur card rather than leaving focus trapped"
    );
    {
        let _guard = dioxus::core::RuntimeGuard::new(harness.doc.vdom.runtime());
        let store = TOAST_STORE.read();
        assert!(
            !store.interaction_state.focused && !store.expanded,
            "Clicking Loading <li> must not leave focused or expanded true"
        );
    }

    // 3. Hover stack so hovered=true & expanded=true, then click #clear-all -> resets immediately
    harness.move_mouse_to(l_rect.x + 24.0, l_rect.y + 18.0);
    harness.pump();
    advance_frame_and_pump(&mut harness, 0.05);
    {
        let _guard = dioxus::core::RuntimeGuard::new(harness.doc.vdom.runtime());
        assert!(TOAST_STORE.read().interaction_state.hovered);
        assert!(TOAST_STORE.read().expanded);
    }

    click_and_pump(&mut harness, "#clear-all");
    {
        let _guard = dioxus::core::RuntimeGuard::new(harness.doc.vdom.runtime());
        let store = TOAST_STORE.read();
        assert!(!store.has_active_toasts());
        assert!(
            !store.interaction_state.hovered
                && !store.interaction_state.focused
                && !store.expanded,
            "#clear-all emptying active queue must clear hovered, focused, and expanded state"
        );
    }
    advance_frame_and_pump(&mut harness, 0.25);

    // 4. Focus #dispatch-info, press F8, then press Escape on single active toast -> restores #dispatch-info
    let single_id = {
        let _guard = dioxus::core::RuntimeGuard::new(harness.doc.vdom.runtime());
        toast::success(
            "Single toast for F8 -> Escape focus restore",
            Some(ToastOptions {
                duration: Some(Duration::from_secs(60)),
                ..Default::default()
            }),
        )
    };
    harness.pump();
    advance_frame_and_pump(&mut harness, 0.05);

    focus_node(&mut harness, "#dispatch-info");
    assert_native_focus_id(&harness, "#dispatch-info");

    send_key_and_pump(&mut harness, Key::Named(NamedKey::F8), Code::F8);
    assert_native_focus_id(&harness, "#monoxus-toast-viewport");
    {
        let _guard = dioxus::core::RuntimeGuard::new(harness.doc.vdom.runtime());
        assert_eq!(
            TOAST_STORE.read().last_focused_element_id.as_deref(),
            Some("dispatch-info")
        );
    }

    send_key_and_pump(&mut harness, Key::Named(NamedKey::Escape), Code::Escape);
    assert_eq!(
        harness
            .attr(&format!("#monoxus-toast-{}", single_id.0), "data-state")
            .as_deref(),
        Some("closed")
    );
    assert_native_focus_id(&harness, "#dispatch-info");
    {
        let _guard = dioxus::core::RuntimeGuard::new(harness.doc.vdom.runtime());
        let store = TOAST_STORE.read();
        assert!(!store.interaction_state.focused && !store.expanded);
    }
}

/// Test 16: Cross-host `observe_visibility_change` pauses active toast timers when the native
/// document becomes hidden/occluded (`Document::set_document_hidden(true)`) and resumes on restore.
#[test]
fn test_native_toast_document_visibility_occlusion_pauses_and_resumes_timers() {
    let _lock = NATIVE_TEST_LOCK.lock().unwrap_or_else(|e| e.into_inner());

    let mut harness = mount_native_scenario(playground_toast::ToastPage);
    harness.set_viewport_size(800, 1400);
    harness.pump();

    if let Some(doc) = oxidase::dom::Document::current() {
        doc.set_document_hidden(false);
    }

    let tid = {
        let _guard = dioxus::core::RuntimeGuard::new(harness.doc.vdom.runtime());
        toast::dismiss(None);
        let mut store = TOAST_STORE.write();
        store.toasts.clear();
        store.toast_heights.clear();
        store.config = monoxus::toast::ToastConfig::default();
        store.set_page_hidden(false);
        drop(store);

        toast::info(
            "Visibility occlusion timer toast",
            Some(ToastOptions {
                duration: Some(Duration::from_millis(600)),
                ..Default::default()
            }),
        )
    };
    harness.pump();
    advance_frame_and_pump(&mut harness, 0.2);

    let toast_sel = format!("#monoxus-toast-{}", tid.0);
    assert_eq!(harness.attr(&toast_sel, "data-state").as_deref(), Some("open"));
    let rem_before_hide = {
        let _guard = dioxus::core::RuntimeGuard::new(harness.doc.vdom.runtime());
        TOAST_STORE.read().toasts[0].remaining_duration
    };
    assert!(
        rem_before_hide < Duration::from_millis(550) && rem_before_hide > Duration::from_millis(200),
        "Timer must tick down prior to occlusion, got {rem_before_hide:?}"
    );

    // Occlude/hide native document -> observe_visibility_change pauses store timers
    oxidase::dom::Document::current()
        .expect("native Document::current must be set")
        .set_document_hidden(true);
    harness.pump();

    {
        let _guard = dioxus::core::RuntimeGuard::new(harness.doc.vdom.runtime());
        let store = TOAST_STORE.read();
        assert!(
            store.interaction_state.page_hidden && store.paused,
            "Document::set_document_hidden(true) must pause toast store via observe_visibility_change"
        );
    }

    // Advance 1.0s while hidden (exceeding 600ms total lifespan) -> toast must remain open
    advance_frame_and_pump(&mut harness, 0.5);
    advance_frame_and_pump(&mut harness, 0.5);

    assert_eq!(
        harness.attr(&toast_sel, "data-state").as_deref(),
        Some("open"),
        "Toast must remain open while document is hidden/occluded"
    );
    {
        let _guard = dioxus::core::RuntimeGuard::new(harness.doc.vdom.runtime());
        let rem_while_hidden = TOAST_STORE.read().toasts[0].remaining_duration;
        assert!(
            rem_before_hide.abs_diff(rem_while_hidden) <= Duration::from_millis(50),
            "Remaining duration must stay frozen while hidden (before={rem_before_hide:?}, hidden={rem_while_hidden:?})"
        );
    }

    // Restore visibility -> timer resumes and auto-closes toast
    oxidase::dom::Document::current()
        .expect("native Document::current must be set")
        .set_document_hidden(false);
    harness.pump();

    {
        let _guard = dioxus::core::RuntimeGuard::new(harness.doc.vdom.runtime());
        let store = TOAST_STORE.read();
        assert!(
            !store.interaction_state.page_hidden && !store.paused,
            "Document::set_document_hidden(false) must resume toast store"
        );
    }

    advance_frame_and_pump(&mut harness, 0.25);
    advance_frame_and_pump(&mut harness, 0.25);

    let closed_or_unmounted = match harness.query(&toast_sel) {
        None => true,
        Some(_) => harness.attr(&toast_sel, "data-state").as_deref() == Some("closed"),
    };
    assert!(
        closed_or_unmounted,
        "Toast must auto-close after document visibility is restored and remaining duration elapses"
    );
}

/// Test 17: Concurrent 3-toast countdown in `NativeTestHarness` expires all 3 stacked toasts
/// with identical duration in the exact same virtual-frame batch (never serialized 3x).
#[test]
fn test_native_toast_concurrent_three_toast_countdown_same_frame_batch() {
    let _lock = NATIVE_TEST_LOCK.lock().unwrap_or_else(|e| e.into_inner());
    use std::sync::Arc;
    use std::sync::atomic::{AtomicUsize, Ordering};

    let mut harness = mount_native_scenario(playground_toast::ToastPage);
    harness.set_viewport_size(800, 1400);
    harness.pump();

    if let Some(doc) = oxidase::dom::Document::current() {
        doc.set_document_hidden(false);
    }

    let auto_closed = Arc::new(AtomicUsize::new(0));
    let ids = harness
        .doc
        .vdom
        .in_scope(dioxus::core::ScopeId::ROOT, || {
            toast::dismiss(None);
            let mut store = TOAST_STORE.write();
            store.toasts.clear();
            store.toast_heights.clear();
            store.config = monoxus::toast::ToastConfig::default();
            store.set_page_hidden(false);
            store.set_hovered(false);
            store.set_focused(false);
            store.set_swiping(false);
            drop(store);

            let lifespan = Duration::from_millis(300);
            let mut dispatched = Vec::with_capacity(3);
            for title in ["Batch Toast 1", "Batch Toast 2", "Batch Toast 3"] {
                let counter = auto_closed.clone();
                let cb = Callback::new(move |()| {
                    counter.fetch_add(1, Ordering::SeqCst);
                });
                dispatched.push(toast::info(
                    title,
                    Some(ToastOptions {
                        duration: Some(lifespan),
                        on_auto_close: Some(cb),
                        ..Default::default()
                    }),
                ));
            }
            dispatched
        });
    harness.pump();

    let sels: Vec<String> = ids
        .iter()
        .map(|tid| format!("#monoxus-toast-{}", tid.0))
        .collect();
    for sel in &sels {
        assert_eq!(
            harness.attr(sel, "data-state").as_deref(),
            Some("open"),
            "Dispatched stacked toast {sel} must mount open"
        );
    }

    // Advance 150ms of the 300ms lifespan: all 3 must tick down together and remain open
    advance_frame_and_pump(&mut harness, 0.15);
    assert_eq!(auto_closed.load(Ordering::SeqCst), 0);
    for sel in &sels {
        assert_eq!(harness.attr(sel, "data-state").as_deref(), Some("open"));
    }
    {
        let _guard = dioxus::core::RuntimeGuard::new(harness.doc.vdom.runtime());
        let store = TOAST_STORE.read();
        assert_eq!(store.toasts.len(), 3);
        for item in &store.toasts {
            assert!(
                item.remaining_duration > Duration::from_millis(100)
                    && item.remaining_duration < Duration::from_millis(200),
                "Each concurrent toast must deduct ~150ms in parallel, got {:?}",
                item.remaining_duration
            );
        }
    }

    // Advance 160ms (total 310ms >= 300ms): all 3 must expire in this single virtual-frame batch
    advance_frame_and_pump(&mut harness, 0.16);
    assert_eq!(
        auto_closed.load(Ordering::SeqCst),
        3,
        "All 3 stacked toasts must auto-close in the same virtual-frame batch"
    );
    {
        let _guard = dioxus::core::RuntimeGuard::new(harness.doc.vdom.runtime());
        let store = TOAST_STORE.read();
        assert!(
            !store.has_active_toasts(),
            "Zero active toasts may remain after the single expiration frame batch"
        );
    }
    for sel in &sels {
        if harness.query(sel).is_some() {
            assert_eq!(harness.attr(sel, "data-state").as_deref(), Some("closed"));
        }
    }

    // Advance past TIME_BEFORE_UNMOUNT (200ms): all 3 must evict together
    advance_frame_and_pump(&mut harness, 0.22);
    for sel in &sels {
        assert!(
            harness.query(sel).is_none(),
            "Toast {sel} must be evicted after unmount window"
        );
    }
    {
        let _guard = dioxus::core::RuntimeGuard::new(harness.doc.vdom.runtime());
        assert!(TOAST_STORE.read().toasts.is_empty());
    }
}

/// Test 18: Pointer dismissal via `[data-close-button]` (`x`) and `[data-button]` (`Action`)
/// on a multi-toast hovered stack keeps `interaction_state.focused == false`, never focuses
/// `#monoxus-toast-viewport`, collapses (`data-expanded="false"`) and unpauses on `mouseleave`,
/// and expires/unmounts remaining stacked toasts.
#[test]
fn test_native_toast_pointer_close_and_action_keep_viewport_unfocused_and_collapse_on_mouseleave() {
    let _lock = NATIVE_TEST_LOCK.lock().unwrap_or_else(|e| e.into_inner());
    use monoxus::toast::ToastActionData;
    use std::rc::Rc;
    use std::sync::atomic::{AtomicBool, Ordering};

    let mut harness = mount_native_scenario(playground_toast::ToastPage);
    harness.set_viewport_size(800, 1400);
    harness.pump();

    if let Some(doc) = oxidase::dom::Document::current() {
        doc.set_document_hidden(false);
    }

    // Verify #monoxus-toast-viewport has zero height (so focusing it with a ring would draw a flat bar)
    let vp_rect = harness.layout_rect("#monoxus-toast-viewport");
    assert_eq!(
        vp_rect.height, 0.0,
        "#monoxus-toast-viewport container height is 0px because child cards are position: absolute"
    );

    let action_clicked = Rc::new(AtomicBool::new(false));
    let action_clicked_cb = action_clicked.clone();

    let (id_back, id_mid, id_front) = {
        let _guard = dioxus::core::RuntimeGuard::new(harness.doc.vdom.runtime());
        toast::dismiss(None);
        let mut store = TOAST_STORE.write();
        store.toasts.clear();
        store.toast_heights.clear();
        store.config = monoxus::toast::ToastConfig::default();
        store.set_page_hidden(false);
        store.set_hovered(false);
        store.set_focused(false);
        store.set_swiping(false);
        drop(store);

        let b = toast::message(
            "Back stack card (auto-expires after mouseleave)",
            Some(ToastOptions {
                duration: Some(Duration::from_millis(400)),
                ..Default::default()
            }),
        );
        let m = toast::info(
            "Middle stack card with Action button",
            Some(ToastOptions {
                duration: Some(Duration::from_millis(400)),
                action: Some(
                    ToastActionData::new("Confirm", "Confirm change").with_handler(move || {
                        action_clicked_cb.store(true, Ordering::SeqCst);
                    }),
                ),
                ..Default::default()
            }),
        );
        let f = toast::success(
            "Front stack card with Close button",
            Some(ToastOptions {
                duration: Some(Duration::from_millis(400)),
                ..Default::default()
            }),
        );
        (b, m, f)
    };
    harness.pump();
    advance_frame_and_pump(&mut harness, 0.05);

    let vp_nid = harness.node("#monoxus-toast-viewport");
    let front_sel = format!("#monoxus-toast-{}", id_front.0);
    let mid_sel = format!("#monoxus-toast-{}", id_mid.0);
    let back_sel = format!("#monoxus-toast-{}", id_back.0);

    // 1. Hover #monoxus-toast-viewport -> stack expands and pauses, focused stays false
    let (fx, fy) = harness.center_of(&front_sel);
    harness.move_mouse_to(fx, fy);
    harness.pump();
    advance_frame_and_pump(&mut harness, 0.05);

    assert_eq!(
        harness
            .attr("#monoxus-toast-viewport", "data-expanded")
            .as_deref(),
        Some("true")
    );
    {
        let _guard = dioxus::core::RuntimeGuard::new(harness.doc.vdom.runtime());
        let store = TOAST_STORE.peek();
        assert!(store.interaction_state.hovered);
        assert!(!store.interaction_state.focused);
        assert!(store.expanded);
        assert!(store.paused);
    }

    // 2. Click [data-close-button] (x) on front card with pointer (holding press across multiple frames)
    let close_sel = format!("{front_sel} [data-close-button]");
    let (cx, cy) = harness.center_of(&close_sel);
    harness.mouse_down_at(cx, cy);
    advance_frame_and_pump(&mut harness, 0.05);
    advance_frame_and_pump(&mut harness, 0.05);
    harness.mouse_up_at(cx, cy);
    advance_frame_and_pump(&mut harness, 0.02);

    assert_eq!(
        harness.attr(&front_sel, "data-state").as_deref(),
        Some("closed"),
        "Clicking [data-close-button] must dismiss front card"
    );
    assert_eq!(harness.attr(&mid_sel, "data-state").as_deref(), Some("open"));
    assert_eq!(harness.attr(&back_sel, "data-state").as_deref(), Some("open"));
    assert_ne!(
        harness.base().active_focus_node_id(),
        Some(vp_nid),
        "Pointer click on [data-close-button] must NOT focus #monoxus-toast-viewport"
    );
    assert_eq!(
        harness.base().active_focus_node_id(),
        None,
        "Pointer click on [data-close-button] must blur the dismissing button instead of leaving stale focus"
    );
    {
        let _guard = dioxus::core::RuntimeGuard::new(harness.doc.vdom.runtime());
        assert!(
            !TOAST_STORE.peek().interaction_state.focused,
            "Pointer click on [data-close-button] must leave interaction_state.focused == false"
        );
    }

    // 3. Move pointer out (mouseleave) -> remaining 2-card stack collapses and unpauses
    harness.move_mouse_to(20.0, 20.0);
    harness.pump();
    advance_frame_and_pump(&mut harness, 0.02);

    assert_eq!(
        harness
            .attr("#monoxus-toast-viewport", "data-expanded")
            .as_deref(),
        Some("false"),
        "mouseleave after pointer [data-close-button] dismissal must collapse remaining 2-card stack"
    );
    {
        let _guard = dioxus::core::RuntimeGuard::new(harness.doc.vdom.runtime());
        let store = TOAST_STORE.peek();
        assert!(!store.expanded, "expanded must be false after mouseleave");
        assert!(!store.paused, "paused must be false after mouseleave");
        assert!(!store.interaction_state.focused);
    }

    // 4. Hover remaining 2-card stack and click [data-button] (Action) on middle card with pointer
    let mid_ty = harness
        .base()
        .get_node(harness.node(&mid_sel))
        .and_then(|n| n.transform().as_ref())
        .map(|aff| aff.as_coeffs()[5] as f32)
        .unwrap_or(0.0);
    let (mx, my_layout) = harness.center_of(&mid_sel);
    harness.move_mouse_to(mx, my_layout + mid_ty);
    harness.pump();
    advance_frame_and_pump(&mut harness, 0.45);

    let mid_ty_expanded = harness
        .base()
        .get_node(harness.node(&mid_sel))
        .and_then(|n| n.transform().as_ref())
        .map(|aff| aff.as_coeffs()[5] as f32)
        .unwrap_or(0.0);
    let action_sel = format!("{mid_sel} [data-button]");
    let (ax, ay_layout) = harness.center_of(&action_sel);
    click_at_and_pump(&mut harness, ax, ay_layout + mid_ty_expanded);

    assert!(
        action_clicked.load(Ordering::SeqCst),
        "Clicking [data-button] must invoke ToastAction callback"
    );
    assert_eq!(
        harness.attr(&mid_sel, "data-state").as_deref(),
        Some("closed"),
        "Clicking [data-button] without prevent_default must dismiss toast"
    );
    assert_eq!(harness.attr(&back_sel, "data-state").as_deref(), Some("open"));
    assert_ne!(
        harness.base().active_focus_node_id(),
        Some(vp_nid),
        "Pointer click on [data-button] must NOT focus #monoxus-toast-viewport"
    );
    assert_eq!(
        harness.base().active_focus_node_id(),
        None,
        "Pointer click on [data-button] must blur the dismissing action button"
    );
    {
        let _guard = dioxus::core::RuntimeGuard::new(harness.doc.vdom.runtime());
        assert!(
            !TOAST_STORE.peek().interaction_state.focused,
            "Pointer click on [data-button] must leave interaction_state.focused == false"
        );
    }

    // 5. Move pointer out (mouseleave) -> stack collapses, unpauses, and remaining toast expires & unmounts
    harness.move_mouse_to(20.0, 20.0);
    harness.pump();
    advance_frame_and_pump(&mut harness, 0.02);

    assert_eq!(
        harness
            .attr("#monoxus-toast-viewport", "data-expanded")
            .as_deref(),
        Some("false"),
        "mouseleave after pointer [data-button] dismissal must collapse viewport"
    );
    {
        let _guard = dioxus::core::RuntimeGuard::new(harness.doc.vdom.runtime());
        let store = TOAST_STORE.peek();
        assert!(!store.expanded);
        assert!(!store.paused);
    }

    // Advance past remaining ~360ms lifespan + 200ms TIME_BEFORE_UNMOUNT
    advance_frame_and_pump(&mut harness, 0.25);
    advance_frame_and_pump(&mut harness, 0.25);
    advance_frame_and_pump(&mut harness, 0.25);

    assert!(
        harness.query("#monoxus-toast-viewport li").is_none(),
        "Remaining toast(s) must expire and unmount after mouseleave resumes auto-close countdown"
    );
    {
        let _guard = dioxus::core::RuntimeGuard::new(harness.doc.vdom.runtime());
        assert!(TOAST_STORE.peek().toasts.is_empty());
    }
}




//! Native Data and Gesture Primitives Verification Test Suite.
//!
//! Validates data and gesture primitives (Select, Toast) using
//! Blitz test harness over shared scenarios directly from `examples/playground/`.

#![allow(unused_imports, dead_code)]

use dioxus::prelude::*;
mod common;
use common::native_fixture::*;
use keyboard_types::{Code, Key, Modifiers};
use monoxus::toast::{toast, ToastOptions, TOAST_STORE};
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
    std::thread::sleep(Duration::from_millis(60));
    harness.pump();
    oxidase::frame::tick(Duration::from_millis(16));
    harness.pump();

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
        initial_descendant.as_deref().unwrap_or("").contains("apple"),
        "Initial active descendant should be apple, got {initial_descendant:?}"
    );

    // Send ArrowDown: roving focus advances to next item in document tree order ("banana")
    send_key_and_pump(&mut harness, Key::ArrowDown, Code::ArrowDown);
    let after_down1 = harness.attr("#select-content", "aria-activedescendant");
    assert!(
        after_down1.as_deref().unwrap_or("").contains("banana"),
        "Roving focus should advance to banana after ArrowDown, got {after_down1:?}"
    );

    // Send ArrowDown again: roving focus advances to "blueberry"
    send_key_and_pump(&mut harness, Key::ArrowDown, Code::ArrowDown);
    let after_down2 = harness.attr("#select-content", "aria-activedescendant");
    assert!(
        after_down2.as_deref().unwrap_or("").contains("blueberry"),
        "Roving focus should advance to blueberry after second ArrowDown, got {after_down2:?}"
    );

    // Send ArrowUp: roving focus steps backward to "banana"
    send_key_and_pump(&mut harness, Key::ArrowUp, Code::ArrowUp);
    let after_up = harness.attr("#select-content", "aria-activedescendant");
    assert!(
        after_up.as_deref().unwrap_or("").contains("banana"),
        "Roving focus should move back to banana after ArrowUp, got {after_up:?}"
    );

    // Interaction 2: Send Enter to commit selection of "banana" and close dropdown
    send_key_and_pump(&mut harness, Key::Enter, Code::Enter);

    // Closed state assertions: trigger is collapsed
    assert_eq!(
        harness.attr("#select-trigger", "aria-expanded").as_deref(),
        Some("false"),
        "Select trigger should be collapsed after selection commit"
    );

    // Wait for exit presence animation/transition to unmount content
    std::thread::sleep(Duration::from_millis(550));
    harness.pump();

    assert!(
        harness.query("#select-content").is_none(),
        "Select content should unmount after committing selection"
    );

    // =========================================================================
    // Explicit order-sensitive typeahead verification:
    // a. Open dropdown on `#select-trigger`, focus `#select-content`.
    // =========================================================================
    click_and_pump(&mut harness, "#select-trigger");
    std::thread::sleep(Duration::from_millis(60));
    harness.pump();
    oxidase::frame::tick(Duration::from_millis(16));
    harness.pump();

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
    send_key_and_pump(&mut harness, Key::Enter, Code::Enter);
    assert_eq!(
        harness.attr("#select-trigger", "aria-expanded").as_deref(),
        Some("false"),
        "Dropdown should close after Enter commit"
    );
    std::thread::sleep(Duration::from_millis(550));
    harness.pump();
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
        harness.attr("#monoxus-toast-viewport li", "data-state").as_deref(),
        Some("open"),
        "Active toast should have data-state='open'"
    );

    // d. Query toast layout rect
    let rect = harness.layout_rect("#monoxus-toast-viewport li");
    assert!(rect.width > 0.0 && rect.height > 0.0, "Toast should have non-empty rect");

    // e. Synthesize real pointer drag exceeding 45px swipe threshold:
    harness.mouse_down_at(rect.x + 20.0, rect.y + 20.0);
    harness.move_mouse_to(rect.x + 100.0, rect.y + 20.0);
    harness.pump();

    // Assert transient attributes:
    assert_eq!(
        harness.attr("#monoxus-toast-viewport li", "data-swiping").as_deref(),
        Some("true"),
        "Toast should reflect data-swiping='true' while actively dragging"
    );

    harness.mouse_up_at(rect.x + 100.0, rect.y + 20.0);
    harness.pump();

    // Assert swipe out attributes:
    assert_eq!(
        harness.attr("#monoxus-toast-viewport li", "data-swipe-out").as_deref(),
        Some("true"),
        "Toast should reflect data-swipe-out='true' after release exceeding threshold"
    );
    assert_eq!(
        harness.attr("#monoxus-toast-viewport li", "data-swipe-direction").as_deref(),
        Some("right"),
        "Toast should reflect data-swipe-direction='right'"
    );

    // Advance unmount delay (200ms) via sleep + pump
    std::thread::sleep(Duration::from_millis(300));
    harness.pump();

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
        harness.attr("#monoxus-toast-viewport li", "data-state").as_deref(),
        Some("open"),
        "Timer toast should initially have data-state='open'"
    );

    // d. Advance past duration (150ms) + pump, advance past unmount delay (250ms) + pump
    std::thread::sleep(Duration::from_millis(150));
    harness.pump();

    std::thread::sleep(Duration::from_millis(250));
    harness.pump();

    // e. Assert toast is automatically evicted from DOM without any pointer interaction
    assert!(
        harness.query("#monoxus-toast-viewport li").is_none(),
        "Timer toast should be automatically evicted after duration expires"
    );
}

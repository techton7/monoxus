//! Native Data and Gesture Primitives Verification Test Suite.
//!
//! Validates data and gesture primitives (Select, Toast) using
//! Blitz test harness over shared scenarios directly from `examples/playground/`.

#![allow(unused_imports, dead_code)]

use dioxus::prelude::*;
mod common;
use common::native_fixture::*;
use keyboard_types::{Code, Key};
use monoxus::toast::{toast, ToastOptions, TOAST_STORE};
use std::time::Duration;

#[path = "../examples/playground/scenarios/select/mod.rs"]
mod playground_select;

#[path = "../examples/playground/pages/toast_page.rs"]
mod playground_toast;

static TEST_LOCK: std::sync::Mutex<()> = std::sync::Mutex::new(());

/// Test 1: Select document order and roving focus lifecycle
///
/// Verifies initial closed state, trigger mounting, opening via click,
/// content mounting and relative placement below trigger, roving focus across
/// items in true document order via arrow keys, and selection commit closing the dropdown.
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
}

/// Test 2: Toast stack layout, timer lifecycle, and dismissal
///
/// Verifies landmark viewport mounting, dispatching stacked toast notifications
/// with non-empty layout rects, and lifecycle dismissal via timer and clear all.
#[test]
fn test_native_toast_swipe_and_timer_lifecycle() {
    let _lock = TEST_LOCK.lock().unwrap_or_else(|e| e.into_inner());

    let mut harness = mount_native_scenario(playground_toast::ToastPage);

    // Clean any residual toast store state within Dioxus runtime
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

    // Interaction 1: Click button '#dispatch-stack-3' to dispatch 3 stacked toasts
    click_and_pump(&mut harness, "#dispatch-stack-3");
    let count = {
        let _guard = dioxus::core::RuntimeGuard::new(harness.doc.vdom.runtime());
        TOAST_STORE.read().toasts.len()
    };
    assert!(count >= 3, "At least 3 toasts should be stored");

    // Assert stacked toasts mount with non-empty layout rects
    assert!(
        harness.query("#monoxus-toast-viewport li").is_some(),
        "Toast items should mount after dispatch"
    );
    assert_rect_non_empty(&harness, "#monoxus-toast-viewport li");
    assert_eq!(
        harness.attr("#monoxus-toast-viewport li", "data-state").as_deref(),
        Some("open"),
        "Active toast should have data-state='open'"
    );

    // Interaction 2: Click '#clear-all' to dismiss all active toasts
    click_and_pump(&mut harness, "#clear-all");

    // Assert toasts transition to closed state
    assert_eq!(
        harness.attr("#monoxus-toast-viewport li", "data-state").as_deref(),
        Some("closed"),
        "Toast should transition to data-state='closed' upon dismissal"
    );

    // Wait for eviction after unmount transition (TIME_BEFORE_UNMOUNT = 200ms)
    std::thread::sleep(Duration::from_millis(300));
    harness.pump();
    assert!(
        harness.query("#monoxus-toast-viewport li").is_none(),
        "Toasts should be evicted from DOM after unmount delay"
    );

    // Interaction 3: Verify timer auto-close lifecycle with short-duration toast
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

    assert!(
        harness.query("#monoxus-toast-viewport li").is_some(),
        "Timer toast should mount"
    );
    assert_rect_non_empty(&harness, "#monoxus-toast-viewport li");

    // Sleep past duration (60ms) and tick pump to allow auto-close timer to trigger
    std::thread::sleep(Duration::from_millis(150));
    harness.pump();

    // Sleep past unmount transition (200ms) and pump
    std::thread::sleep(Duration::from_millis(250));
    harness.pump();

    assert!(
        harness.query("#monoxus-toast-viewport li").is_none(),
        "Timer toast should be automatically evicted after duration expires"
    );
}

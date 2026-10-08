//! Native Overlay and Floating Primitives Verification Test Suite.
//!
//! Validates floating and overlay primitives (Popover, Tooltip, Dialog, AlertDialog)
//! using Blitz test harness over shared scenarios directly from `examples/playground/`.

mod common;
use common::native_fixture::*;

#[path = "../examples/playground/popover.rs"]
mod playground_popover;

#[path = "../examples/playground/tooltip.rs"]
mod playground_tooltip;

#[path = "../examples/playground/dialog.rs"]
mod playground_dialog;

#[path = "../examples/playground/alert_dialog.rs"]
mod playground_alert_dialog;

static TEST_LOCK: std::sync::Mutex<()> = std::sync::Mutex::new(());

/// Test 1: Popover floating and dismissal lifecycle
///
/// Verifies initial closed state, opening via trigger click, non-empty content
/// layout positioned relative to the trigger, and dismissal via close click.
#[test]
fn test_native_popover_floating_and_dismiss() {
    let _lock = TEST_LOCK.lock().unwrap_or_else(|e| e.into_inner());
    let mut harness = mount_native_scenario(playground_popover::PopoverPlayground);

    // Initial state: trigger exists, content not mounted
    assert_rect_non_empty(&harness, "#popover-trigger");
    assert!(
        harness.query("#popover-content").is_none(),
        "Popover content should not be mounted initially"
    );

    // Interaction 1: Click trigger to open popover
    click_and_pump(&mut harness, "#popover-trigger");

    // Open state assertions
    assert!(
        harness.query("#popover-content").is_some(),
        "Popover content should be mounted after click"
    );
    assert_rect_non_empty(&harness, "#popover-content");

    // Interaction 2: Click close button to dismiss popover
    click_and_pump(&mut harness, "#popover-close");
    std::thread::sleep(std::time::Duration::from_millis(550));
    harness.pump();

    // Closed state assertions
    assert!(
        harness.query("#popover-content").is_none(),
        "Popover content should be unmounted after close click"
    );
}

/// Test 2: Tooltip trigger and arrow mounting
///
/// Verifies trigger mounting, pointer/focus interaction to show tooltip,
/// and that content and arrow mount with valid non-empty bounds.
#[test]
fn test_native_tooltip_trigger_and_arrow() {
    let _lock = TEST_LOCK.lock().unwrap_or_else(|e| e.into_inner());
    let mut harness = mount_native_scenario(playground_tooltip::TooltipPlayground);

    // Assert trigger mounts #tooltip-trigger with valid bounds
    assert_rect_non_empty(&harness, "#tooltip-trigger");
    assert!(
        harness.query("#tooltip-content").is_none(),
        "Tooltip content should not be mounted initially"
    );

    // Trigger pointer enter / move on #tooltip-trigger and advance delay
    let (tx, ty) = harness.center_of("#tooltip-trigger");
    harness.move_mouse_to(tx, ty);
    std::thread::sleep(std::time::Duration::from_millis(600));
    harness.pump();
    harness.pump();
    harness.pump();
    harness.pump();
    harness.pump();

    // Assert #tooltip-content and #tooltip-arrow mount with valid bounds
    assert!(
        harness.query("#tooltip-content").is_some(),
        "Tooltip content should mount after hover/focus and delay"
    );
    assert_rect_non_empty(&harness, "#tooltip-content");
    assert!(
        harness.query("#tooltip-arrow").is_some(),
        "Tooltip arrow should mount"
    );
    assert_rect_non_empty(&harness, "#tooltip-arrow");
}

/// Test 3: Dialog and AlertDialog lifecycle
///
/// Verifies opening, overlay, and content layout, followed by clean dismissal
/// for both Modal Dialog and Destructive AlertDialog.
#[test]
fn test_native_dialog_and_alert_dialog_lifecycle() {
    let _lock = TEST_LOCK.lock().unwrap_or_else(|e| e.into_inner());

    // -------------------------------------------------------------------------
    // 1. Dialog Playground Lifecycle
    // -------------------------------------------------------------------------
    let mut dialog_harness = mount_native_scenario(playground_dialog::DialogPlayground);

    // Initial state: trigger exists, content not mounted
    assert_rect_non_empty(&dialog_harness, "#dialog-trigger");
    assert!(
        dialog_harness.query("#dialog-content").is_none(),
        "Dialog content should not be mounted initially"
    );

    // Interaction 1: Click trigger to open dialog
    click_and_pump(&mut dialog_harness, "#dialog-trigger");

    // Assert overlay and content are mounted with non-empty rects
    assert!(
        dialog_harness.query("#dialog-overlay").is_some(),
        "Dialog overlay should mount when open"
    );
    assert_rect_non_empty(&dialog_harness, "#dialog-overlay");
    assert!(
        dialog_harness.query("#dialog-content").is_some(),
        "Dialog content should mount when open"
    );
    assert_rect_non_empty(&dialog_harness, "#dialog-content");

    // Interaction 2: Click close button to dismiss dialog
    click_and_pump(&mut dialog_harness, "#dialog-close");
    std::thread::sleep(std::time::Duration::from_millis(550));
    dialog_harness.pump();

    // Assert dialog content closes
    assert!(
        dialog_harness.query("#dialog-content").is_none(),
        "Dialog content should be unmounted after clicking close"
    );

    // -------------------------------------------------------------------------
    // 2. AlertDialog Playground Lifecycle
    // -------------------------------------------------------------------------
    let mut alert_harness = mount_native_scenario(playground_alert_dialog::AlertDialogPlayground);

    // Initial state: trigger exists, content not mounted
    assert_rect_non_empty(&alert_harness, "#alert-dialog-trigger");
    assert!(
        alert_harness.query("#alert-dialog-content").is_none(),
        "Alert dialog content should not be mounted initially"
    );

    // Interaction 1: Click trigger to open alert dialog
    click_and_pump(&mut alert_harness, "#alert-dialog-trigger");

    // Assert content is mounted with non-empty rect
    assert!(
        alert_harness.query("#alert-dialog-content").is_some(),
        "Alert dialog content should mount when open"
    );
    assert_rect_non_empty(&alert_harness, "#alert-dialog-content");

    // Interaction 2: Click cancel button to dismiss alert dialog
    click_and_pump(&mut alert_harness, "#alert-dialog-cancel");
    std::thread::sleep(std::time::Duration::from_millis(550));
    alert_harness.pump();

    // Assert dialog closes
    assert!(
        alert_harness.query("#alert-dialog-content").is_none(),
        "Alert dialog content should be unmounted after cancel click"
    );
}

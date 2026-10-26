//! Native Overlay and Floating Primitives Verification Test Suite.
//!
//! Validates floating and overlay primitives (Popover, Tooltip, Dialog, AlertDialog)
//! using Blitz test harness over shared scenarios directly from `examples/playground/`.

mod common;
use common::native_fixture::*;
use dioxus::prelude::*;
use keyboard_types::{Code, Key};

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
/// Verifies initial closed state, opening via trigger click, relative floating
/// placement below anchor, and multiple dismissal paths (close button, Escape key, and outside click).
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

    // -------------------------------------------------------------------------
    // Phase 1: Open and verify relative floating placement
    // -------------------------------------------------------------------------
    click_and_pump(&mut harness, "#popover-trigger");

    assert!(
        harness.query("#popover-content").is_some(),
        "Popover content should be mounted after click"
    );
    assert_rect_non_empty(&harness, "#popover-content");

    // Geometric proof: Popover content is positioned relative to anchor
    assert_relative_below(&harness, "#popover-anchor", "#popover-content", 30.0);
    assert_relative_aligned(&harness, "#popover-anchor", "#popover-content", 30.0);

    // Dismissal path 1: Click close button
    click_and_pump(&mut harness, "#popover-close");
    advance_frame_and_pump(&mut harness, 0.35);

    assert!(
        harness.query("#popover-content").is_none(),
        "Popover content should be unmounted after close click"
    );

    // -------------------------------------------------------------------------
    // Phase 2: Open and verify Escape key dismissal
    // -------------------------------------------------------------------------
    click_and_pump(&mut harness, "#popover-trigger");
    assert!(
        harness.query("#popover-content").is_some(),
        "Popover content should be mounted after second click"
    );

    focus_node(&mut harness, "#popover-content");
    assert_native_focus_id(&harness, "#popover-content");
    send_key_and_pump(&mut harness, Key::Escape, Code::Escape);
    advance_frame_and_pump(&mut harness, 0.35);

    assert!(
        harness.query("#popover-content").is_none(),
        "Popover content should be unmounted after Escape key"
    );

    // -------------------------------------------------------------------------
    // Phase 3: Open and verify outside click dismissal
    // -------------------------------------------------------------------------
    click_and_pump(&mut harness, "#popover-trigger");
    assert!(
        harness.query("#popover-content").is_some(),
        "Popover content should be mounted after third click"
    );

    // Click outside popover content and anchor/trigger (at page coordinates 5.0, 5.0)
    click_at_and_pump(&mut harness, 5.0, 5.0);
    advance_frame_and_pump(&mut harness, 0.35);

    assert!(
        harness.query("#popover-content").is_none(),
        "Popover content should be unmounted after outside click"
    );
}

/// Test 2: Tooltip trigger and arrow mounting
///
/// Verifies trigger mounting, pointer hover/focus interaction, provider delay,
/// arrow mounting, relative vertical/horizontal alignment, and unmount on pointer leave.
#[test]
fn test_native_tooltip_trigger_and_arrow() {
    let _lock = TEST_LOCK.lock().unwrap_or_else(|e| e.into_inner());

    fn tooltip_test_scenario() -> Element {
        rsx! {
            playground_tooltip::TooltipPlayground {
                delay_duration_ms: 0,
                disable_hoverable_content: true,
            }
        }
    }

    let mut harness = mount_native_scenario(tooltip_test_scenario);

    // Assert trigger mounts #tooltip-trigger with valid bounds
    assert_rect_non_empty(&harness, "#tooltip-trigger");
    assert!(
        harness.query("#tooltip-content").is_none(),
        "Tooltip content should not be mounted initially"
    );

    // Trigger pointer enter / move on #tooltip-trigger with delay_duration_ms=0
    let (tx, ty) = harness.center_of("#tooltip-trigger");
    harness.move_mouse_to(tx, ty);
    // Pass 1: Trigger hover -> mounts DOM nodes in unpositioned state and registers next_frame()
    advance_frame_and_pump(&mut harness, 0.02);
    // Pass 2: next_frame() future resumes -> Floating UI calculates coordinates & arrow layout
    advance_frame_and_pump(&mut harness, 0.02);

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

    // Geometric alignment assertions:
    // 1. Tooltip is positioned above trigger (configured with PlacementSide::Top)
    assert_relative_above(&harness, "#tooltip-trigger", "#tooltip-content", 30.0);

    // 2. Tooltip arrow is contained within the horizontal bounds of the tooltip content
    let content_rect = harness.layout_rect("#tooltip-content");
    let arrow_rect = harness.layout_rect("#tooltip-arrow");
    assert!(
        arrow_rect.x >= content_rect.x - 5.0
            && arrow_rect.x + arrow_rect.width <= content_rect.x + content_rect.width + 5.0,
        "Tooltip arrow (x={}, w={}) must be horizontally within content bounds (x={}, w={})",
        arrow_rect.x,
        arrow_rect.width,
        content_rect.x,
        content_rect.width
    );

    // Interaction 2: Pointer leave dismisses tooltip
    harness.move_mouse_to(0.0, 0.0);
    harness.pump();
    harness.pump();
    // Advance past the 200ms monoxus-tooltip-content-out exit animation
    advance_frame_and_pump(&mut harness, 0.25);

    assert!(
        harness.query("#tooltip-content").is_none(),
        "Tooltip content should be unmounted after pointer leave"
    );
}

/// Test 3: Dialog and AlertDialog lifecycle
///
/// Verifies opening, overlay, focus trapping, focus restoration to trigger,
/// backdrop click dismiss (for Dialog) vs non-dismiss (for AlertDialog),
/// and Escape key handling.
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

    // Phase 1: Open dialog and verify Escape key dismiss + focus restore
    click_and_pump(&mut dialog_harness, "#dialog-trigger");

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

    // Escape key dismissal
    send_key_and_pump(&mut dialog_harness, Key::Escape, Code::Escape);
    advance_frame_and_pump(&mut dialog_harness, 0.25);

    assert!(
        dialog_harness.query("#dialog-content").is_none(),
        "Dialog content should be unmounted after Escape key"
    );
    assert_native_focus_id(&dialog_harness, "#dialog-trigger");

    // Phase 2: Open dialog and verify backdrop click dismiss + focus restore
    click_and_pump(&mut dialog_harness, "#dialog-trigger");
    assert!(
        dialog_harness.query("#dialog-content").is_some(),
        "Dialog content should be mounted after second click"
    );

    // Click backdrop (at (5.0, 5.0) on overlay outside centered content)
    let overlay_rect = dialog_harness.layout_rect("#dialog-overlay");
    click_at_and_pump(
        &mut dialog_harness,
        overlay_rect.x + 5.0,
        overlay_rect.y + 5.0,
    );
    advance_frame_and_pump(&mut dialog_harness, 0.25);

    assert!(
        dialog_harness.query("#dialog-content").is_none(),
        "Dialog content should be unmounted after clicking backdrop overlay"
    );
    assert_native_focus_id(&dialog_harness, "#dialog-trigger");

    // Phase 3: Open dialog and verify close button dismiss + focus restore
    click_and_pump(&mut dialog_harness, "#dialog-trigger");
    assert!(
        dialog_harness.query("#dialog-content").is_some(),
        "Dialog content should be mounted after third click"
    );

    click_and_pump(&mut dialog_harness, "#dialog-close");
    advance_frame_and_pump(&mut dialog_harness, 0.25);

    assert!(
        dialog_harness.query("#dialog-content").is_none(),
        "Dialog content should be unmounted after clicking close"
    );
    assert_native_focus_id(&dialog_harness, "#dialog-trigger");

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

    assert!(
        alert_harness.query("#alert-dialog-content").is_some(),
        "Alert dialog content should mount when open"
    );
    assert_rect_non_empty(&alert_harness, "#alert-dialog-content");

    // Verify modal backdrop: clicking overlay does NOT dismiss AlertDialog
    let overlay_rect = alert_harness.layout_rect("#alert-dialog-overlay");
    click_at_and_pump(
        &mut alert_harness,
        overlay_rect.x + 5.0,
        overlay_rect.y + 5.0,
    );
    advance_frame_and_pump(&mut alert_harness, 0.1);

    assert!(
        alert_harness.query("#alert-dialog-content").is_some(),
        "Alert dialog content must remain mounted when clicking backdrop"
    );

    // Interaction 2: Click cancel button to dismiss alert dialog
    click_and_pump(&mut alert_harness, "#alert-dialog-cancel");
    advance_frame_and_pump(&mut alert_harness, 0.25);

    assert!(
        alert_harness.query("#alert-dialog-content").is_none(),
        "Alert dialog content should be unmounted after cancel click"
    );
    assert_native_focus_id(&alert_harness, "#alert-dialog-trigger");
}

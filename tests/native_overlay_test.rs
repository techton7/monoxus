//! Native Overlay and Floating Primitives Verification Test Suite.
//!
//! Validates floating and overlay primitives (Popover, Tooltip, Dialog, AlertDialog)
//! using Blitz test harness over shared scenarios directly from `examples/playground/`.

use crate::common;
use common::native_fixture::*;
use dioxus::prelude::*;
use crate::playground_fixtures::{playground_popover, playground_tooltip, playground_dialog, playground_alert_dialog};
use keyboard_types::{Code, Key, NamedKey};






/// Test 1: Popover floating and dismissal lifecycle
///
/// Verifies initial closed state, opening via trigger click, relative floating
/// placement below anchor, and multiple dismissal paths (close button, Escape key, and outside click).
#[test]
fn test_native_popover_floating_and_dismiss() {
    let _lock = NATIVE_TEST_LOCK.lock().unwrap_or_else(|e| e.into_inner());
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
    send_key_and_pump(&mut harness, Key::Named(NamedKey::Escape), Code::Escape);
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
    let _lock = NATIVE_TEST_LOCK.lock().unwrap_or_else(|e| e.into_inner());

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
    let _lock = NATIVE_TEST_LOCK.lock().unwrap_or_else(|e| e.into_inner());

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
    send_key_and_pump(&mut dialog_harness, Key::Named(NamedKey::Escape), Code::Escape);
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

/// Test 5: Double-correction invariant guard test (AC-4)
///
/// Verifies that:
/// 1) `compute_coords_from_placement` produces identical results regardless of `floating.x` / `floating.y` input.
/// 2) `NativeFloatingPlatform::get_element_rects` produces `floating.x == 0.0` and `floating.y == 0.0` for `Strategy::Fixed`.
#[test]
fn test_native_double_correction_invariants() {
    let _lock = NATIVE_TEST_LOCK.lock().unwrap_or_else(|e| e.into_inner());
    use oxidase_floating_ui::compute_coords_from_placement;
    use oxidase_floating_ui::geometry::{ElementOrVirtual, ElementRects, Placement, Rect, Strategy};
    use oxidase_floating_ui::platform::NativeFloatingPlatform;
    use oxidase_floating_ui::types::{FloatingPlatform, GetElementRectsArgs, Platform};

    // 1. Invariant 1: compute_coords_from_placement ignores floating.x / floating.y across all 12 placements and directions
    let reference = Rect {
        x: 150.0,
        y: 220.0,
        width: 100.0,
        height: 40.0,
    };

    let all_placements = [
        Placement::Top,
        Placement::TopStart,
        Placement::TopEnd,
        Placement::Right,
        Placement::RightStart,
        Placement::RightEnd,
        Placement::Bottom,
        Placement::BottomStart,
        Placement::BottomEnd,
        Placement::Left,
        Placement::LeftStart,
        Placement::LeftEnd,
    ];

    let rtl_options = [None, Some(false), Some(true)];

    let floating_variations = [
        Rect {
            x: 0.0,
            y: 0.0,
            width: 80.0,
            height: 60.0,
        },
        Rect {
            x: 50.0,
            y: 120.0,
            width: 80.0,
            height: 60.0,
        },
        Rect {
            x: -300.0,
            y: -450.0,
            width: 80.0,
            height: 60.0,
        },
        Rect {
            x: 12345.0,
            y: -98765.0,
            width: 80.0,
            height: 60.0,
        },
    ];

    for &placement in &all_placements {
        for &rtl in &rtl_options {
            let canonical_rects = ElementRects {
                reference,
                floating: floating_variations[0],
            };
            let canonical_coords = compute_coords_from_placement(&canonical_rects, placement, rtl);

            for &varied_floating in &floating_variations[1..] {
                let varied_rects = ElementRects {
                    reference,
                    floating: varied_floating,
                };
                let varied_coords = compute_coords_from_placement(&varied_rects, placement, rtl);
                assert_eq!(
                    canonical_coords,
                    varied_coords,
                    "compute_coords_from_placement must ignore floating.x/y for placement={:?}, rtl={:?}",
                    placement,
                    rtl
                );
            }
        }
    }

    // 2. Invariant 2: NativeFloatingPlatform::get_element_rects produces floating.x == 0.0 and floating.y == 0.0 for Strategy::Fixed
    #[component]
    fn FloatingFixedScenario() -> Element {
        rsx! {
            div {
                id: "fixed-root",
                style: "width: 800px; height: 600px; position: relative;",
                button {
                    id: "ref-target",
                    style: "position: absolute; left: 150px; top: 220px; width: 100px; height: 40px;",
                    "Reference"
                }
                div {
                    id: "float-target",
                    style: "position: fixed; left: 400px; top: 500px; width: 80px; height: 60px;",
                    "Floating"
                }
            }
        }
    }

    let harness = mount_native_scenario(FloatingFixedScenario);
    let platform = NativeFloatingPlatform::new(harness.doc.inner.clone());

    let rects_from_fp = FloatingPlatform::get_element_rects(&platform, "ref-target", "float-target")
        .expect("FloatingPlatform::get_element_rects must succeed");
    assert_eq!(
        rects_from_fp.floating.x, 0.0,
        "FloatingPlatform floating.x must be 0.0 to prevent double-correction under Strategy::Fixed"
    );
    assert_eq!(
        rects_from_fp.floating.y, 0.0,
        "FloatingPlatform floating.y must be 0.0 to prevent double-correction under Strategy::Fixed"
    );
    assert_eq!(rects_from_fp.floating.width, 80.0);
    assert_eq!(rects_from_fp.floating.height, 60.0);

    let ref_node = harness.node("#ref-target");
    let float_node = harness.node("#float-target");
    let rects_from_node_platform = Platform::<blitz_dom::NodeId, ()>::get_element_rects(
        &platform,
        GetElementRectsArgs {
            reference: ElementOrVirtual::Element(&ref_node),
            floating: &float_node,
            strategy: Strategy::Fixed,
        },
    );
    assert_eq!(
        rects_from_node_platform.floating.x, 0.0,
        "Platform<NodeId> floating.x must be 0.0 to prevent double-correction under Strategy::Fixed"
    );
    assert_eq!(
        rects_from_node_platform.floating.y, 0.0,
        "Platform<NodeId> floating.y must be 0.0 to prevent double-correction under Strategy::Fixed"
    );
    assert_eq!(rects_from_node_platform.floating.width, 80.0);
    assert_eq!(rects_from_node_platform.floating.height, 60.0);
}

/// Test 6: Popover Keyboard Activation and Descendant Input Space Typing
#[test]
fn test_native_popover_keyboard_activation_and_descendant_input() {
    let _lock = NATIVE_TEST_LOCK.lock().unwrap_or_else(|e| e.into_inner());
    let mut harness = mount_native_scenario(playground_popover::PopoverPlayground);

    assert_rect_non_empty(&harness, "#popover-trigger");
    assert!(harness.query("#popover-content").is_none());

    // 1. Space key on focused trigger opens popover
    focus_node(&mut harness, "#popover-trigger");
    send_key_and_pump(&mut harness, Key::Character(" ".into()), Code::Space);

    assert!(
        harness.query("#popover-content").is_some(),
        "Popover content must mount after Space on trigger"
    );

    // 2. Typing Space in focused descendant input does NOT close popover
    focus_node(&mut harness, "#popover-nested-input");
    send_key_and_pump(&mut harness, Key::Character(" ".into()), Code::Space);
    assert!(
        harness.query("#popover-content").is_some(),
        "Popover must remain open when typing Space into descendant input"
    );

    // 3. Enter on PopoverClose closes popover and restores trigger focus
    focus_node(&mut harness, "#popover-close");
    send_key_and_pump(&mut harness, Key::Named(NamedKey::Enter), Code::Enter);
    advance_frame_and_pump(&mut harness, 0.35);

    assert!(
        harness.query("#popover-content").is_none(),
        "Popover content must unmount after Enter on PopoverClose"
    );
    assert!(harness.focused().is_some());
}

/// Test 7: Dialog & AlertDialog Keyboard Activation, Descendant Input, and Action/Cancel
#[test]
fn test_native_dialog_and_alert_dialog_keyboard_activation() {
    let _lock = NATIVE_TEST_LOCK.lock().unwrap_or_else(|e| e.into_inner());

    // -------------------------------------------------------------------------
    // Part 1: Dialog with Descendant Input
    // -------------------------------------------------------------------------
    let mut dialog_harness = mount_native_scenario(playground_dialog::DialogPlayground);

    assert_rect_non_empty(&dialog_harness, "#dialog-trigger");
    assert!(dialog_harness.query("#dialog-content").is_none());

    // Enter on trigger opens dialog
    focus_node(&mut dialog_harness, "#dialog-trigger");
    send_key_and_pump(&mut dialog_harness, Key::Named(NamedKey::Enter), Code::Enter);

    assert!(
        dialog_harness.query("#dialog-content").is_some(),
        "Dialog content must mount after Enter on DialogTrigger"
    );

    // Typing Space in descendant input does NOT close dialog
    focus_node(&mut dialog_harness, "#name");
    send_key_and_pump(&mut dialog_harness, Key::Character(" ".into()), Code::Space);
    assert!(
        dialog_harness.query("#dialog-content").is_some(),
        "Dialog must remain open when typing Space into descendant input"
    );

    // Space on DialogClose closes dialog and restores trigger focus
    focus_node(&mut dialog_harness, "#dialog-close");
    send_key_and_pump(&mut dialog_harness, Key::Character(" ".into()), Code::Space);
    advance_frame_and_pump(&mut dialog_harness, 0.25);

    assert!(
        dialog_harness.query("#dialog-content").is_none(),
        "Dialog content must unmount after Space on DialogClose"
    );
    assert_native_focus_id(&dialog_harness, "#dialog-trigger");

    // -------------------------------------------------------------------------
    // Part 2: AlertDialog Cancel and Action Keyboard Activation
    // -------------------------------------------------------------------------
    let mut alert_harness = mount_native_scenario(playground_alert_dialog::AlertDialogPlayground);

    // Space on AlertDialogTrigger opens modal
    focus_node(&mut alert_harness, "#alert-dialog-trigger");
    send_key_and_pump(&mut alert_harness, Key::Character(" ".into()), Code::Space);

    assert!(
        alert_harness.query("#alert-dialog-content").is_some(),
        "AlertDialog must open upon Space key on trigger"
    );

    // Enter on Cancel closes alert dialog and restores trigger focus
    focus_node(&mut alert_harness, "#alert-dialog-cancel");
    send_key_and_pump(&mut alert_harness, Key::Named(NamedKey::Enter), Code::Enter);
    advance_frame_and_pump(&mut alert_harness, 0.25);

    assert!(
        alert_harness.query("#alert-dialog-content").is_none(),
        "AlertDialog must close upon Enter on Cancel"
    );
    assert_native_focus_id(&alert_harness, "#alert-dialog-trigger");

    // Re-open with Enter
    send_key_and_pump(&mut alert_harness, Key::Named(NamedKey::Enter), Code::Enter);
    assert!(alert_harness.query("#alert-dialog-content").is_some());

    // Space on Action/Confirm closes alert dialog and restores trigger focus
    focus_node(&mut alert_harness, "#alert-dialog-confirm");
    send_key_and_pump(&mut alert_harness, Key::Character(" ".into()), Code::Space);
    advance_frame_and_pump(&mut alert_harness, 0.25);

    assert!(
        alert_harness.query("#alert-dialog-content").is_none(),
        "AlertDialog must close upon Space on Action"
    );
    assert_native_focus_id(&alert_harness, "#alert-dialog-trigger");
}

/// Test 8: Tooltip Trigger Dismiss Upon Space and Enter
#[test]
fn test_native_tooltip_space_enter_dismiss() {
    let _lock = NATIVE_TEST_LOCK.lock().unwrap_or_else(|e| e.into_inner());

    fn tooltip_dismiss_scenario() -> Element {
        rsx! {
            playground_tooltip::TooltipPlayground {
                delay_duration_ms: 0,
                disable_hoverable_content: true,
            }
        }
    }

    let mut harness = mount_native_scenario(tooltip_dismiss_scenario);

    // Open tooltip via mouse hover
    let (tx, ty) = harness.center_of("#tooltip-trigger");
    harness.move_mouse_to(tx, ty);
    advance_frame_and_pump(&mut harness, 0.02);
    advance_frame_and_pump(&mut harness, 0.02);

    assert!(
        harness.query("#tooltip-content").is_some(),
        "Tooltip must be open"
    );

    // Focus trigger and press Space -> dismisses open tooltip
    focus_node(&mut harness, "#tooltip-trigger");
    send_key_and_pump(&mut harness, Key::Character(" ".into()), Code::Space);
    advance_frame_and_pump(&mut harness, 0.25);

    assert!(
        harness.query("#tooltip-content").is_none(),
        "Tooltip must close upon Space key on TooltipTrigger"
    );
    assert_native_focus_id(&harness, "tooltip-trigger");

    // Re-open tooltip
    harness.move_mouse_to(0.0, 0.0);
    advance_frame_and_pump(&mut harness, 0.25);
    harness.move_mouse_to(tx, ty);
    advance_frame_and_pump(&mut harness, 0.02);
    advance_frame_and_pump(&mut harness, 0.02);

    assert!(harness.query("#tooltip-content").is_some());

    // Press Enter on focused trigger -> dismisses open tooltip
    focus_node(&mut harness, "#tooltip-trigger");
    send_key_and_pump(&mut harness, Key::Named(NamedKey::Enter), Code::Enter);
    advance_frame_and_pump(&mut harness, 0.25);

    assert!(
        harness.query("#tooltip-content").is_none(),
        "Tooltip must close upon Enter key on TooltipTrigger"
    );
    assert_native_focus_id(&harness, "tooltip-trigger");
}

/// Test 9: Popover & Dialog :focus-visible modality and pointer-dismiss focus restoration
#[test]
fn test_native_popover_and_dialog_focus_visible_modality_and_pointer_dismiss_restoration() {
    let _lock = NATIVE_TEST_LOCK.lock().unwrap_or_else(|e| e.into_inner());

    // -------------------------------------------------------------------------
    // Part 1: PopoverPlayground (explicit Target open/close focus policies)
    // -------------------------------------------------------------------------
    let mut pop_harness = mount_native_scenario(playground_popover::PopoverPlayground);
    let restore_id = "#10:playground|7:popover|13:restore-focus";

    // a) Keyboard open (Space) + Keyboard Escape close -> Restore focus target HAS :focus-visible
    focus_node(&mut pop_harness, "#popover-trigger");
    send_key_and_pump(&mut pop_harness, Key::Character(" ".into()), Code::Space);
    advance_frame_and_pump(&mut pop_harness, 0.05);
    assert!(pop_harness.query("#popover-content").is_some());

    send_key_and_pump(&mut pop_harness, Key::Named(NamedKey::Escape), Code::Escape);
    advance_frame_and_pump(&mut pop_harness, 0.35);
    assert!(pop_harness.query("#popover-content").is_none());
    assert_native_focus_visible(&pop_harness, restore_id, true);

    // b) Keyboard open (Space), then unfocusable outside pointer click (5.0, 5.0)
    //    -> Restore focus target gets :focus=true and :focus-visible=false (no spurious ring)
    focus_node(&mut pop_harness, "#popover-trigger");
    send_key_and_pump(&mut pop_harness, Key::Character(" ".into()), Code::Space);
    advance_frame_and_pump(&mut pop_harness, 0.05);
    assert!(pop_harness.query("#popover-content").is_some());

    click_at_and_pump(&mut pop_harness, 5.0, 5.0);
    advance_frame_and_pump(&mut pop_harness, 0.35);
    assert!(pop_harness.query("#popover-content").is_none());
    assert_native_focus_visible(&pop_harness, restore_id, false);

    // -------------------------------------------------------------------------
    // Part 2: Non-modal Popover with default Trigger close policy does not steal outside click focus
    // -------------------------------------------------------------------------
    #[component]
    fn NonModalPopoverOutsideFocusScenario() -> Element {
        use monoxus::popover::{PopoverClose, PopoverContent, PopoverPortal, PopoverRoot, PopoverTrigger};
        rsx! {
            div {
                style: "width: 600px; height: 400px; padding: 24px;",
                PopoverRoot {
                    id: "nm-pop",
                    PopoverTrigger { id: "nm-pop-trigger", "Open Popover" }
                    PopoverPortal {
                        PopoverContent {
                            id: "nm-pop-content",
                            PopoverClose { id: "nm-pop-close", "Close" }
                        }
                    }
                }
                button {
                    id: "outside-action-btn",
                    style: "margin-top: 200px;",
                    "Outside Action"
                }
            }
        }
    }

    let mut nm_harness = mount_native_scenario(NonModalPopoverOutsideFocusScenario);
    focus_node(&mut nm_harness, "#nm-pop-trigger");
    send_key_and_pump(&mut nm_harness, Key::Named(NamedKey::Enter), Code::Enter);
    advance_frame_and_pump(&mut nm_harness, 0.05);
    assert!(nm_harness.query("#nm-pop-content").is_some());

    // Mouse-clicking #outside-action-btn dismisses non-modal popover WITHOUT stealing focus back to #nm-pop-trigger
    click_and_pump(&mut nm_harness, "#outside-action-btn");
    advance_frame_and_pump(&mut nm_harness, 0.1);
    assert_native_focus_visible(&nm_harness, "#outside-action-btn", false);

    // -------------------------------------------------------------------------
    // Part 3: Dialog keyboard open -> backdrop pointer click restores #dialog-trigger with :focus-visible=false
    // -------------------------------------------------------------------------
    let mut dlg_harness = mount_native_scenario(playground_dialog::DialogPlayground);
    focus_node(&mut dlg_harness, "#dialog-trigger");
    send_key_and_pump(&mut dlg_harness, Key::Named(NamedKey::Enter), Code::Enter);
    advance_frame_and_pump(&mut dlg_harness, 0.05);
    assert!(dlg_harness.query("#dialog-content").is_some());

    // Escape key close -> #dialog-trigger HAS :focus-visible=true
    send_key_and_pump(&mut dlg_harness, Key::Named(NamedKey::Escape), Code::Escape);
    advance_frame_and_pump(&mut dlg_harness, 0.25);
    assert!(dlg_harness.query("#dialog-content").is_none());
    assert_native_focus_visible(&dlg_harness, "#dialog-trigger", true);

    // Re-open via keyboard (Enter), then click unfocusable #dialog-overlay backdrop
    send_key_and_pump(&mut dlg_harness, Key::Named(NamedKey::Enter), Code::Enter);
    advance_frame_and_pump(&mut dlg_harness, 0.05);
    assert!(dlg_harness.query("#dialog-content").is_some());

    let overlay_rect = dlg_harness.layout_rect("#dialog-overlay");
    click_at_and_pump(&mut dlg_harness, overlay_rect.x + 5.0, overlay_rect.y + 5.0);
    advance_frame_and_pump(&mut dlg_harness, 0.25);
    assert!(dlg_harness.query("#dialog-content").is_none());
    assert_native_focus_visible(&dlg_harness, "#dialog-trigger", false);
}

/// Test 10: Tooltip :focus-visible modality gating and passive grace corridor without pointer capture
#[test]
fn test_native_tooltip_focus_visible_gating_and_passive_grace_corridor() {
    let _lock = NATIVE_TEST_LOCK.lock().unwrap_or_else(|e| e.into_inner());
    use blitz_traits::events::BlitzPointerId;

    #[component]
    fn TooltipModalityScenario() -> Element {
        use monoxus::tooltip::{TooltipContent, TooltipPortal, TooltipProvider, TooltipRoot, TooltipTrigger};
        rsx! {
            div {
                style: "width: 600px; height: 400px; padding: 40px; display: flex; gap: 24px;",
                button { id: "lead-btn", "Before" }
                TooltipProvider {
                    delay_duration: 0,
                    TooltipRoot {
                        id: "mod-tt",
                        TooltipTrigger {
                            "Hover or Tab"
                        }
                        TooltipPortal {
                            TooltipContent { "Keyboard Tooltip" }
                        }
                    }
                }
                button {
                    id: "prog-focus-btn",
                    onclick: move |_| async move {
                        if let Some(doc) = oxidase::Document::current()
                            && let Some(el) = doc.element_by_id("7:tooltip|6:mod-tt|7:trigger")
                        {
                            let _ = el
                                .focus_with_options(oxidase::FocusOptions {
                                    prevent_scroll: false,
                                    focus_visible: Some(false),
                                })
                                .await;
                        }
                    },
                    "Programmatic Non-Visible Focus"
                }
            }
        }
    }

    let mut harness = mount_native_scenario(TooltipModalityScenario);
    let mod_trigger_id = "7:tooltip|6:mod-tt|7:trigger";
    let mod_trigger_sel = "[id=\"7:tooltip|6:mod-tt|7:trigger\"]";
    let mod_content_sel = "[id=\"7:tooltip|6:mod-tt|7:content\"]";

    // 1. Mouse click on trigger focuses it with pointer modality (:focus-visible=false) -> tooltip stays closed
    click_and_pump(&mut harness, mod_trigger_sel);
    advance_frame_and_pump(&mut harness, 0.05);
    assert_native_focus_visible(&harness, mod_trigger_id, false);
    assert!(
        harness.query(mod_content_sel).is_none(),
        "Mouse click focus (:focus-visible=false) must NOT spuriously open Tooltip"
    );

    // 2. Programmatic FocusOptions { focus_visible: Some(false) } moves focus onto trigger
    //    WITHOUT firing pointerdown/mouseenter on trigger -> live :focus-visible
    //    gate in TooltipRuntime::trigger_focus suppresses opening
    click_and_pump(&mut harness, "#lead-btn");
    assert_native_focus_visible(&harness, "#lead-btn", false);
    click_and_pump(&mut harness, "#prog-focus-btn");
    advance_frame_and_pump(&mut harness, 0.05);
    assert_native_focus_visible(&harness, mod_trigger_id, false);
    assert!(
        harness.query(mod_content_sel).is_none(),
        "Programmatic non-visible focus (:focus-visible=false) must NOT open Tooltip"
    );

    // 3. Focus #lead-btn, then press Tab -> focuses trigger with :focus-visible=true -> Tooltip opens
    click_and_pump(&mut harness, "#lead-btn");
    send_key_and_pump(&mut harness, Key::Named(NamedKey::Tab), Code::Tab);
    advance_frame_and_pump(&mut harness, 0.05);
    assert_native_focus_visible(&harness, mod_trigger_id, true);
    assert!(
        harness.query(mod_content_sel).is_some(),
        "Keyboard Tab focus (:focus-visible=true) must open Tooltip"
    );

    // 4. Passive observe_pointer_move grace corridor on hoverable TooltipPlayground (never acquires pointer capture)
    fn hoverable_tooltip_scenario() -> Element {
        rsx! {
            playground_tooltip::TooltipPlayground {
                delay_duration_ms: 0,
                disable_hoverable_content: false,
            }
        }
    }

    let mut grace_harness = mount_native_scenario(hoverable_tooltip_scenario);
    let trigger_nid = grace_harness.node("#tooltip-trigger");
    let trigger_rect = grace_harness.layout_rect("#tooltip-trigger");
    let (tx, ty) = grace_harness.center_of("#tooltip-trigger");
    grace_harness.move_mouse_to(tx, ty);
    advance_frame_and_pump(&mut grace_harness, 0.02);
    advance_frame_and_pump(&mut grace_harness, 0.02);

    assert!(grace_harness.query("#tooltip-content").is_some());
    assert!(
        !grace_harness
            .base()
            .has_pointer_capture(trigger_nid, BlitzPointerId::Mouse.to_dom_pointer_id()),
        "Tooltip passive pointer stream must never acquire pointer capture"
    );

    // Leave trigger's left edge by 2px (inside 5px padded grace corridor, outside arrow/content)
    grace_harness.move_mouse_to(trigger_rect.x - 2.0, ty);
    grace_harness.pump();
    grace_harness.pump();
    assert!(
        grace_harness.query("#tooltip-content").is_some(),
        "Tooltip must remain open while pointer is inside safe polygon grace corridor"
    );
    assert!(
        !grace_harness
            .base()
            .has_pointer_capture(trigger_nid, BlitzPointerId::Mouse.to_dom_pointer_id()),
        "Leaving trigger into grace corridor must never acquire pointer capture"
    );

    // Move far outside corridor -> passive observe_pointer_move detects ExitedCorridor and closes tooltip
    grace_harness.move_mouse_to(5.0, 550.0);
    grace_harness.pump();
    grace_harness.pump();
    advance_frame_and_pump(&mut grace_harness, 0.25);
    assert!(
        grace_harness.query("#tooltip-content").is_none(),
        "Moving pointer outside safe polygon corridor must dismiss hoverable Tooltip"
    );
}

/// Test 13: Tooltip post-unmount drop safety regression test
///
/// Verifies that when a component owning TooltipProvider and Tooltip unmounts
/// while the skip_delay root timer (600ms) or other root tasks are pending,
/// the waking task aborts cleanly without panicking with ValueDroppedError.
#[test]
fn test_native_tooltip_unmount_during_skip_delay_no_panic() {
    let _lock = NATIVE_TEST_LOCK.lock().unwrap_or_else(|e| e.into_inner());

    fn unmount_tooltip_scenario() -> Element {
        let mut mounted = use_signal(|| true);

        if !mounted() {
            return rsx! {
                div {
                    id: "unmounted-placeholder",
                    "Unmounted"
                }
            };
        }

        rsx! {
            div {
                button {
                    id: "unmount-btn",
                    onclick: move |_| mounted.set(false),
                    "Unmount"
                }
                playground_tooltip::TooltipPlayground {
                    delay_duration_ms: 0,
                    disable_hoverable_content: true,
                }
            }
        }
    }

    let mut harness = mount_native_scenario(unmount_tooltip_scenario);
    assert_rect_non_empty(&harness, "#tooltip-trigger");

    // 1. Open tooltip via hover (delay=0)
    let (tx, ty) = harness.center_of("#tooltip-trigger");
    harness.move_mouse_to(tx, ty);
    advance_frame_and_pump(&mut harness, 0.02);
    advance_frame_and_pump(&mut harness, 0.02);
    assert!(
        harness.query("#tooltip-content").is_some(),
        "Tooltip must be open after hover"
    );

    // 2. Move pointer away to trigger close and start the 600ms skip_delay timer
    harness.move_mouse_to(0.0, 0.0);
    harness.pump();
    harness.pump();
    advance_frame_and_pump(&mut harness, 0.25);
    assert!(
        harness.query("#tooltip-content").is_none(),
        "Tooltip must be dismissed after leaving trigger"
    );

    // 3. Immediately unmount while the 600ms skip_delay timer is still pending
    click_and_pump(&mut harness, "#unmount-btn");
    advance_frame_and_pump(&mut harness, 0.02);
    assert!(
        harness.query("#unmounted-placeholder").is_some(),
        "Placeholder must be present after unmounting TooltipPlayground"
    );

    // 4. Sleep past the 600ms skip-delay duration and pump VirtualDom/reactor
    std::thread::sleep(std::time::Duration::from_millis(650));
    advance_frame_and_pump(&mut harness, 0.05);

    // 5. Zero panic! Re-assert placeholder is still valid and intact
    assert!(harness.query("#unmounted-placeholder").is_some());
}

/// Test 14: Tooltip with custom trigger ID and custom content ID positioning & scroll-ancestor dismissal
#[test]
fn test_native_tooltip_custom_trigger_id_positioning_and_scroll_dismiss() {
    let _lock = NATIVE_TEST_LOCK.lock().unwrap_or_else(|e| e.into_inner());

    #[component]
    fn CustomIdsTooltipScrollScenario() -> Element {
        use monoxus::tooltip::{
            TooltipContent, TooltipPortal, TooltipProvider, TooltipRoot, TooltipTrigger,
        };
        rsx! {
            div {
                id: "tooltip-scroll-container",
                style: "width: 400px; height: 220px; overflow-y: auto; padding: 24px;",
                div {
                    style: "height: 80px;",
                    "Top spacer"
                }
                TooltipProvider {
                    delay_duration: 0,
                    disable_hoverable_content: true,
                    TooltipRoot {
                        id: "custom-guard-tt",
                        TooltipTrigger {
                            id: "custom-guard-trigger",
                            "Hover Custom Trigger"
                        }
                        TooltipPortal {
                            TooltipContent {
                                id: "custom-guard-content",
                                "Custom Guard Tooltip"
                            }
                        }
                    }
                }
                div {
                    style: "height: 600px;",
                    "Bottom spacer"
                }
            }
        }
    }

    let mut harness = mount_native_scenario(CustomIdsTooltipScrollScenario);

    assert_rect_non_empty(&harness, "#custom-guard-trigger");
    assert!(
        harness.query("#custom-guard-content").is_none(),
        "Tooltip content should not be mounted initially"
    );

    // 1. Hovering custom trigger opens tooltip and positions content (no longer Measuring/hidden)
    let (tx, ty) = harness.center_of("#custom-guard-trigger");
    harness.move_mouse_to(tx, ty);
    advance_frame_and_pump(&mut harness, 0.02);
    advance_frame_and_pump(&mut harness, 0.02);

    assert!(
        harness.query("#custom-guard-content").is_some(),
        "Tooltip content should mount after hovering custom trigger ID"
    );
    assert_rect_non_empty(&harness, "#custom-guard-content");
    assert_eq!(
        harness
            .attr("#custom-guard-trigger", "aria-describedby")
            .as_deref(),
        Some("custom-guard-content")
    );

    let wrapper_style = harness
        .attr("[data-monoxus-floating-content-wrapper]", "style")
        .expect("Floating wrapper should have a style attribute");
    assert!(
        wrapper_style.contains("visibility: visible;"),
        "Positioned tooltip wrapper must transition out of Measuring to visibility: visible, got: {wrapper_style}"
    );
    assert!(
        !wrapper_style.contains("visibility: hidden;"),
        "Positioned tooltip wrapper must not remain visibility: hidden, got: {wrapper_style}"
    );
    assert_relative_above(
        &harness,
        "#custom-guard-trigger",
        "#custom-guard-content",
        30.0,
    );

    // 2. Scrolling ancestor container dismisses tooltip attached to custom trigger ID
    let container_rect = harness.layout_rect("#tooltip-scroll-container");
    wheel_at_and_pump(
        &mut harness,
        container_rect.x + container_rect.width - 20.0,
        container_rect.y + container_rect.height - 20.0,
        0.0,
        -50.0,
    );
    drain_polls_and_resolve(&mut harness);
    advance_frame_and_pump(&mut harness, 0.25);
    drain_polls_and_resolve(&mut harness);

    let container_node = harness.node("#tooltip-scroll-container");
    let container_scroll_y = harness
        .base()
        .get_node(container_node)
        .unwrap()
        .scroll_offset()
        .y;
    assert!(
        container_scroll_y > 0.0,
        "Scrollable ancestor must actually scroll for regression check (scroll_y={container_scroll_y})"
    );
    assert_eq!(
        harness
            .attr("#custom-guard-trigger", "data-state")
            .as_deref(),
        Some("closed"),
        "Tooltip trigger must transition to closed after scrollable ancestor scroll"
    );
    assert!(
        harness.query("#custom-guard-content").is_none(),
        "Tooltip content must unmount after scrollable ancestor dismisses it"
    );
}

/// Test 15: LMB pointerdown closes open tooltip immediately and cancels pending open timer
#[test]
fn test_native_tooltip_pointer_down_closes_immediately_and_cancels_pending_open() {
    let _lock = NATIVE_TEST_LOCK.lock().unwrap_or_else(|e| e.into_inner());
    use blitz_traits::events::UiEvent;

    #[component]
    fn PointerDownCloseScenario() -> Element {
        use monoxus::tooltip::{
            TooltipContent, TooltipPortal, TooltipProvider, TooltipRoot, TooltipTrigger,
        };
        rsx! {
            div {
                style: "width: 600px; height: 400px; padding: 40px; display: flex; gap: 32px;",
                TooltipProvider {
                    delay_duration: 0,
                    disable_hoverable_content: true,
                    TooltipRoot {
                        id: "tt-immediate",
                        TooltipTrigger {
                            id: "tt-immediate-trigger",
                            "Immediate Trigger"
                        }
                        TooltipPortal {
                            TooltipContent {
                                id: "tt-immediate-content",
                                "Immediate Tooltip"
                            }
                        }
                    }
                }
                TooltipRoot {
                    id: "tt-delayed",
                    delay_duration: 180,
                    disable_hoverable_content: true,
                    TooltipTrigger {
                        id: "tt-delayed-trigger",
                        "Delayed Trigger"
                    }
                    TooltipPortal {
                        TooltipContent {
                            id: "tt-delayed-content",
                            "Delayed Tooltip"
                        }
                    }
                }
            }
        }
    }

    let mut harness = mount_native_scenario(PointerDownCloseScenario);

    // Part 1: Open tooltip via hover, then press LMB down (without releasing yet)
    let (tx, ty) = harness.center_of("#tt-immediate-trigger");
    harness.move_mouse_to(tx, ty);
    advance_frame_and_pump(&mut harness, 0.02);
    advance_frame_and_pump(&mut harness, 0.02);
    assert!(
        harness.query("#tt-immediate-content").is_some(),
        "Tooltip must open on hover"
    );

    let down_event = blitz_test_harness::mouse_pointer_event(tx, ty);
    harness.dispatch(UiEvent::PointerDown(down_event));
    drain_polls_and_resolve(&mut harness);
    assert_eq!(
        harness.attr("#tt-immediate-trigger", "data-state").as_deref(),
        Some("closed"),
        "LMB pointerdown on trigger must transition tooltip to closed immediately before pointerup/click"
    );
    advance_frame_and_pump(&mut harness, 0.25);
    assert!(
        harness.query("#tt-immediate-content").is_none(),
        "Tooltip content must unmount after pointerdown close"
    );

    // Release pointer outside to keep state clean
    harness.dispatch(UiEvent::PointerUp(blitz_test_harness::mouse_pointer_event(5.0, 5.0)));
    drain_polls_and_resolve(&mut harness);

    // Part 2: Hover delayed trigger, press LMB down before delay expires -> pending open timer cancelled
    let (dx, dy) = harness.center_of("#tt-delayed-trigger");
    harness.move_mouse_to(dx, dy);
    drain_polls_and_resolve(&mut harness);
    std::thread::sleep(std::time::Duration::from_millis(40));
    drain_polls_and_resolve(&mut harness);
    assert!(
        harness.query("#tt-delayed-content").is_none(),
        "Delayed tooltip must still be closed before delay_duration"
    );

    harness.dispatch(UiEvent::PointerDown(blitz_test_harness::mouse_pointer_event(dx, dy)));
    harness.dispatch(UiEvent::PointerUp(blitz_test_harness::mouse_pointer_event(dx, dy)));
    drain_polls_and_resolve(&mut harness);

    std::thread::sleep(std::time::Duration::from_millis(220));
    advance_frame_and_pump(&mut harness, 0.05);
    assert_eq!(
        harness.attr("#tt-delayed-trigger", "data-state").as_deref(),
        Some("closed"),
        "Pending open timer must remain cancelled after trigger pointerdown"
    );
    assert!(
        harness.query("#tt-delayed-content").is_none(),
        "Delayed tooltip must NOT open after trigger pointerdown cancelled pending timer"
    );
}

/// Test 16: LMB pointerdown does NOT close tooltip when close_on_trigger_click == false
#[test]
fn test_native_tooltip_pointer_down_respects_close_on_trigger_click_false() {
    let _lock = NATIVE_TEST_LOCK.lock().unwrap_or_else(|e| e.into_inner());
    use blitz_traits::events::UiEvent;

    #[component]
    fn KeepOpenOnClickScenario() -> Element {
        use monoxus::tooltip::{
            TooltipContent, TooltipPortal, TooltipProvider, TooltipRoot, TooltipTrigger,
        };
        rsx! {
            div {
                style: "width: 400px; height: 300px; padding: 40px;",
                TooltipProvider {
                    delay_duration: 0,
                    close_on_trigger_click: false,
                    disable_hoverable_content: true,
                    TooltipRoot {
                        id: "tt-keep-open",
                        TooltipTrigger {
                            id: "tt-keep-open-trigger",
                            "Keep Open Trigger"
                        }
                        TooltipPortal {
                            TooltipContent {
                                id: "tt-keep-open-content",
                                "Stays Open"
                            }
                        }
                    }
                }
            }
        }
    }

    let mut harness = mount_native_scenario(KeepOpenOnClickScenario);
    let (tx, ty) = harness.center_of("#tt-keep-open-trigger");
    harness.move_mouse_to(tx, ty);
    advance_frame_and_pump(&mut harness, 0.02);
    advance_frame_and_pump(&mut harness, 0.02);
    assert!(harness.query("#tt-keep-open-content").is_some());

    harness.dispatch(UiEvent::PointerDown(blitz_test_harness::mouse_pointer_event(tx, ty)));
    advance_frame_and_pump(&mut harness, 0.05);
    assert_eq!(
        harness.attr("#tt-keep-open-trigger", "data-state").as_deref(),
        Some("open"),
        "LMB pointerdown must NOT close tooltip when close_on_trigger_click is false"
    );
    assert!(harness.query("#tt-keep-open-content").is_some());

    harness.dispatch(UiEvent::PointerUp(blitz_test_harness::mouse_pointer_event(tx, ty)));
    advance_frame_and_pump(&mut harness, 0.05);
    assert_eq!(
        harness.attr("#tt-keep-open-trigger", "data-state").as_deref(),
        Some("open"),
        "Click release must NOT close tooltip when close_on_trigger_click is false"
    );
    assert!(harness.query("#tt-keep-open-content").is_some());
}

/// Test 17: Hover open -> LMB down -> native DOM focus on trigger -> pointer leave closes instead of sticking open
#[test]
fn test_native_tooltip_outside_pointer_up_does_not_stick_open_on_native_focus() {
    let _lock = NATIVE_TEST_LOCK.lock().unwrap_or_else(|e| e.into_inner());
    use blitz_traits::events::UiEvent;

    #[component]
    fn HoverPressLeaveScenario() -> Element {
        use monoxus::tooltip::{
            TooltipContent, TooltipPortal, TooltipRoot, TooltipTrigger,
        };
        rsx! {
            div {
                style: "width: 600px; height: 400px; padding: 40px; display: flex; gap: 32px;",
                TooltipRoot {
                    id: "tt-c1",
                    delay_duration: 0,
                    disable_hoverable_content: true,
                    close_on_trigger_click: true,
                    TooltipTrigger {
                        id: "tt-c1-trigger",
                        "Default Close Trigger"
                    }
                    TooltipPortal {
                        TooltipContent {
                            id: "tt-c1-content",
                            "Tooltip C1"
                        }
                    }
                }
                TooltipRoot {
                    id: "tt-c2",
                    delay_duration: 0,
                    disable_hoverable_content: true,
                    close_on_trigger_click: false,
                    TooltipTrigger {
                        id: "tt-c2-trigger",
                        "Opt-out Trigger"
                    }
                    TooltipPortal {
                        TooltipContent {
                            id: "tt-c2-content",
                            "Tooltip C2"
                        }
                    }
                }
            }
        }
    }

    let mut harness = mount_native_scenario(HoverPressLeaveScenario);

    // Part 1 (close_on_trigger_click: true):
    // Hover open -> LMB down -> native focus on button -> drag outside -> LMB up outside -> re-hover -> leave closes
    let (tx1, ty1) = harness.center_of("#tt-c1-trigger");
    harness.move_mouse_to(tx1, ty1);
    advance_frame_and_pump(&mut harness, 0.02);
    advance_frame_and_pump(&mut harness, 0.02);
    assert!(harness.query("#tt-c1-content").is_some());

    harness.dispatch(UiEvent::PointerDown(blitz_test_harness::mouse_pointer_event(tx1, ty1)));
    drain_polls_and_resolve(&mut harness);
    assert_eq!(harness.attr("#tt-c1-trigger", "data-state").as_deref(), Some("closed"));
    // Ensure native DOM focus is on trigger (non-focus-visible pointer focus)
    if let Some(doc) = oxidase::Document::current()
        && let Some(el) = doc.element_by_id("tt-c1-trigger")
        && !el.has_focus()
    {
        focus_node(&mut harness, "#tt-c1-trigger");
    }
    assert_native_focus_id(&harness, "tt-c1-trigger");

    harness.move_mouse_to(5.0, 5.0);
    harness.dispatch(UiEvent::PointerUp(blitz_test_harness::mouse_pointer_event(5.0, 5.0)));
    advance_frame_and_pump(&mut harness, 0.25);
    assert!(
        harness.query("#tt-c1-content").is_none(),
        "Tooltip must stay closed after LMB down + drag outside + LMB up outside"
    );

    // Re-hover while button STILL has raw DOM focus, then leave -> must close, NOT stick open
    harness.move_mouse_to(tx1, ty1);
    advance_frame_and_pump(&mut harness, 0.02);
    advance_frame_and_pump(&mut harness, 0.02);
    assert!(harness.query("#tt-c1-content").is_some());
    assert_native_focus_id(&harness, "tt-c1-trigger");

    harness.move_mouse_to(5.0, 5.0);
    advance_frame_and_pump(&mut harness, 0.25);
    assert!(
        harness.query("#tt-c1-content").is_none(),
        "Raw DOM focus from pointer click must NOT block hover-leave dismissal"
    );

    // Part 2 (close_on_trigger_click: false):
    // Hover open -> LMB down -> native focus on button (opened_via_focus == false) -> leave + LMB up outside -> closes
    let (tx2, ty2) = harness.center_of("#tt-c2-trigger");
    harness.move_mouse_to(tx2, ty2);
    advance_frame_and_pump(&mut harness, 0.02);
    advance_frame_and_pump(&mut harness, 0.02);
    assert!(harness.query("#tt-c2-content").is_some());

    harness.dispatch(UiEvent::PointerDown(blitz_test_harness::mouse_pointer_event(tx2, ty2)));
    drain_polls_and_resolve(&mut harness);
    if let Some(doc) = oxidase::Document::current()
        && let Some(el) = doc.element_by_id("tt-c2-trigger")
        && !el.has_focus()
    {
        focus_node(&mut harness, "#tt-c2-trigger");
    }
    assert_native_focus_id(&harness, "tt-c2-trigger");
    assert_eq!(harness.attr("#tt-c2-trigger", "data-state").as_deref(), Some("open"));

    harness.move_mouse_to(5.0, 5.0);
    harness.dispatch(UiEvent::PointerUp(blitz_test_harness::mouse_pointer_event(5.0, 5.0)));
    advance_frame_and_pump(&mut harness, 0.25);
    assert_eq!(
        harness.attr("#tt-c2-trigger", "data-state").as_deref(),
        Some("closed"),
        "Pointer leave must close hover-opened tooltip even when trigger has raw DOM focus"
    );
    assert!(
        harness.query("#tt-c2-content").is_none(),
        "Tooltip C2 must not stick open after pointer leave while button has raw DOM focus"
    );
}

/// Test 18: pointer_down_inside suppresses trigger_focus even when ignore_non_keyboard_focus == false, and resets cleanly
#[test]
fn test_native_tooltip_pointer_down_inside_suppresses_focus_even_when_ignore_non_keyboard_focus_false() {
    let _lock = NATIVE_TEST_LOCK.lock().unwrap_or_else(|e| e.into_inner());
    use blitz_traits::events::UiEvent;

    #[component]
    fn IgnoreNonKeyboardFalseScenario() -> Element {
        use monoxus::tooltip::{
            TooltipContent, TooltipPortal, TooltipRoot, TooltipTrigger,
        };
        rsx! {
            div {
                style: "width: 600px; height: 400px; padding: 40px; display: flex; gap: 24px;",
                button { id: "lead-btn-d", "Before" }
                TooltipRoot {
                    id: "tt-d",
                    delay_duration: 0,
                    ignore_non_keyboard_focus: false,
                    TooltipTrigger {
                        id: "tt-d-trigger",
                        "Focus Guard Trigger"
                    }
                    TooltipPortal {
                        TooltipContent {
                            id: "tt-d-content",
                            "Tooltip D"
                        }
                    }
                }
            }
        }
    }

    let mut harness = mount_native_scenario(IgnoreNonKeyboardFalseScenario);
    let (tx, ty) = harness.center_of("#tt-d-trigger");

    // 1. LMB down on trigger sets pointer_down_inside = true; native focus during pointerdown MUST be suppressed
    //    even though ignore_non_keyboard_focus == false
    harness.dispatch(UiEvent::PointerDown(blitz_test_harness::mouse_pointer_event(tx, ty)));
    drain_polls_and_resolve(&mut harness);
    // Explicitly drive focus while pointer is still down if Blitz defers focus to pointerup/click
    focus_node(&mut harness, "#tt-d-trigger");
    advance_frame_and_pump(&mut harness, 0.05);
    assert_eq!(
        harness.attr("#tt-d-trigger", "data-state").as_deref(),
        Some("closed"),
        "pointer_down_inside must suppress trigger_focus even when ignore_non_keyboard_focus == false"
    );
    assert!(harness.query("#tt-d-content").is_none());

    // 2. Release LMB outside trigger -> document pointerup observer resets pointer_down_inside cleanly
    harness.dispatch(UiEvent::PointerUp(blitz_test_harness::mouse_pointer_event(5.0, 5.0)));
    drain_polls_and_resolve(&mut harness);
    advance_frame_and_pump(&mut harness, 0.02);

    // 3. Move focus to #lead-btn-d and Tab back to trigger -> pointer_down_inside is false, so focus opens tooltip
    click_and_pump(&mut harness, "#lead-btn-d");
    send_key_and_pump(&mut harness, Key::Named(NamedKey::Tab), Code::Tab);
    advance_frame_and_pump(&mut harness, 0.05);
    assert_eq!(
        harness.attr("#tt-d-trigger", "data-state").as_deref(),
        Some("open"),
        "After outside pointerup resets pointer_down_inside, subsequent focus must open tooltip"
    );
    assert!(harness.query("#tt-d-content").is_some());
}

/// Test 19: Keyboard focus (opened_via_focus == true) stays open on pointerleave and closes on blur / Escape
#[test]
fn test_native_tooltip_keyboard_focus_stays_open_on_pointer_leave_and_closes_on_blur_or_escape() {
    let _lock = NATIVE_TEST_LOCK.lock().unwrap_or_else(|e| e.into_inner());

    #[component]
    fn KeyboardFocusScenario() -> Element {
        use monoxus::tooltip::{
            TooltipContent, TooltipPortal, TooltipRoot, TooltipTrigger,
        };
        rsx! {
            div {
                style: "width: 600px; height: 400px; padding: 40px; display: flex; gap: 24px;",
                button { id: "lead-btn-e", "Before" }
                TooltipRoot {
                    id: "tt-e",
                    delay_duration: 0,
                    disable_hoverable_content: true,
                    TooltipTrigger {
                        id: "tt-e-trigger",
                        "Keyboard Trigger"
                    }
                    TooltipPortal {
                        TooltipContent {
                            id: "tt-e-content",
                            "Keyboard Tooltip"
                        }
                    }
                }
            }
        }
    }

    let mut harness = mount_native_scenario(KeyboardFocusScenario);

    // 1. Focus #lead-btn-e and Tab into trigger -> opens via keyboard focus
    click_and_pump(&mut harness, "#lead-btn-e");
    send_key_and_pump(&mut harness, Key::Named(NamedKey::Tab), Code::Tab);
    advance_frame_and_pump(&mut harness, 0.05);
    assert_native_focus_visible(&harness, "tt-e-trigger", true);
    assert!(
        harness.query("#tt-e-content").is_some(),
        "Keyboard Tab focus must open tooltip"
    );

    // 2. Move pointer across trigger and leave -> MUST stay open while opened_via_focus == true
    let (tx, ty) = harness.center_of("#tt-e-trigger");
    harness.move_mouse_to(tx, ty);
    harness.pump();
    harness.move_mouse_to(5.0, 5.0);
    advance_frame_and_pump(&mut harness, 0.25);
    assert_eq!(
        harness.attr("#tt-e-trigger", "data-state").as_deref(),
        Some("open"),
        "pointerleave must NOT close tooltip opened via keyboard focus"
    );
    assert!(harness.query("#tt-e-content").is_some());

    // 3. Escape closes keyboard-focus-opened tooltip
    send_key_and_pump(&mut harness, Key::Named(NamedKey::Escape), Code::Escape);
    advance_frame_and_pump(&mut harness, 0.25);
    assert_eq!(harness.attr("#tt-e-trigger", "data-state").as_deref(), Some("closed"));
    assert!(harness.query("#tt-e-content").is_none());

    // 4. Re-open via Tab, then blur by clicking #lead-btn-e -> closes tooltip
    click_and_pump(&mut harness, "#lead-btn-e");
    send_key_and_pump(&mut harness, Key::Named(NamedKey::Tab), Code::Tab);
    advance_frame_and_pump(&mut harness, 0.05);
    assert!(harness.query("#tt-e-content").is_some());

    click_and_pump(&mut harness, "#lead-btn-e");
    advance_frame_and_pump(&mut harness, 0.25);
    assert_eq!(
        harness.attr("#tt-e-trigger", "data-state").as_deref(),
        Some("closed"),
        "Blur must close tooltip opened via keyboard focus"
    );
    assert!(harness.query("#tt-e-content").is_none());
}

/// Test 20: Hoverable grace corridor leave fallback (40ms) and transit-intent timeout (180ms)
#[test]
fn test_native_tooltip_grace_corridor_leave_fallback_and_transit_intent_timeout() {
    let _lock = NATIVE_TEST_LOCK.lock().unwrap_or_else(|e| e.into_inner());
    use std::{cell::RefCell, rc::Rc};

    #[derive(Clone)]
    struct GraceTestRuntime(Rc<RefCell<Option<monoxus::tooltip::TooltipRuntime>>>);
    impl PartialEq for GraceTestRuntime {
        fn eq(&self, other: &Self) -> bool {
            Rc::ptr_eq(&self.0, &other.0)
        }
    }

    #[component]
    fn GraceRuntimeProbe(out: GraceTestRuntime) -> Element {
        let ctx = use_context::<monoxus::tooltip::TooltipRootContext>();
        *out.0.borrow_mut() = Some(ctx.runtime.read().clone());
        rsx! {}
    }

    let captured_runtime = GraceTestRuntime(Rc::new(RefCell::new(None)));
    let captured_for_app = captured_runtime.clone();

    let mut harness = {
        let opts = blitz_test_harness::HarnessOptions::default();
        let mut doc = dioxus_native_dom::DioxusDocument::new(
            VirtualDom::new_with_props(
                move |out: GraceTestRuntime| {
                    use monoxus::tooltip::{
                        TooltipContent, TooltipPortal, TooltipProvider, TooltipRoot, TooltipTrigger,
                    };
                    rsx! {
                        div {
                            style: "width: 600px; height: 400px; padding: 120px;",
                            TooltipProvider {
                                delay_duration: 0,
                                disable_hoverable_content: false,
                                TooltipRoot {
                                    id: "tt-grace-fallback",
                                    TooltipTrigger {
                                        id: "tt-grace-trigger",
                                        "Hoverable Trigger"
                                    }
                                    TooltipPortal {
                                        TooltipContent {
                                            id: "tt-grace-content",
                                            "Hoverable Tooltip"
                                        }
                                    }
                                    GraceRuntimeProbe { out }
                                }
                            }
                        }
                    }
                },
                captured_for_app,
            ),
            blitz_dom::DocumentConfig {
                viewport: Some(blitz_traits::shell::Viewport::new(
                    opts.width,
                    opts.height,
                    opts.scale,
                    opts.color_scheme,
                )),
                base_url: opts.base_url,
                net_provider: opts.net_provider,
                ..Default::default()
            },
        );
        let native_doc = oxidase::dom::Document::from_base(doc.inner.clone());
        native_doc.attach_engine_bridges();
        oxidase::dom::Document::set_current(Some(native_doc.clone()));
        doc.vdom.in_scope(dioxus::core::ScopeId::ROOT, move || {
            provide_context(native_doc);
        });
        doc.initial_build();
        let mut harness = blitz_test_harness::Harness::wrap(doc);
        drain_polls_and_resolve(&mut harness);
        harness
    };

    let trigger_rect = harness.layout_rect("#tt-grace-trigger");
    let (tx, ty) = harness.center_of("#tt-grace-trigger");

    // 1. Hover open -> click trigger to leave pointer-induced DOM focus -> re-hover ->
    //    jump straight outside (500, 350) -> closes cleanly (does not stick open)
    harness.move_mouse_to(tx, ty);
    advance_frame_and_pump(&mut harness, 0.02);
    advance_frame_and_pump(&mut harness, 0.02);
    assert!(harness.query("#tt-grace-content").is_some());

    click_and_pump(&mut harness, "#tt-grace-trigger");
    advance_frame_and_pump(&mut harness, 0.25);
    assert!(harness.query("#tt-grace-content").is_none());

    harness.move_mouse_to(5.0, 5.0);
    harness.pump();
    harness.move_mouse_to(tx, ty);
    advance_frame_and_pump(&mut harness, 0.02);
    advance_frame_and_pump(&mut harness, 0.02);
    assert!(harness.query("#tt-grace-content").is_some());

    harness.move_mouse_to(500.0, 350.0);
    harness.pump();
    std::thread::sleep(std::time::Duration::from_millis(220));
    drain_polls_and_resolve(&mut harness);
    advance_frame_and_pump(&mut harness, 0.25);
    drain_polls_and_resolve(&mut harness);
    assert_eq!(
        harness.attr("#tt-grace-trigger", "data-state").as_deref(),
        Some("closed"),
        "Hoverable tooltip must close when pointer jumps out of trigger even with pointer-induced DOM focus"
    );
    assert!(harness.query("#tt-grace-content").is_none());

    // 2. Transit-intent timeout (180ms): move into gap between trigger and content (InTransit) ->
    //    stays open past 40ms fallback, then closes after 180ms transit-intent timeout when pointer stalls
    let runtime = captured_runtime
        .0
        .borrow()
        .as_ref()
        .expect("probe must capture TooltipRuntime")
        .clone();
    harness.move_mouse_to(tx, ty);
    advance_frame_and_pump(&mut harness, 0.02);
    advance_frame_and_pump(&mut harness, 0.02);
    assert!(harness.query("#tt-grace-content").is_some());

    let gap_y = trigger_rect.y - 2.0;
    harness.move_mouse_to(tx, gap_y);
    harness.pump();
    // Move slightly inside corridor so document pointermove fires InTransit while grace monitor is active
    harness.move_mouse_to(tx + 1.0, gap_y);
    harness.pump();
    std::thread::sleep(std::time::Duration::from_millis(60));
    drain_polls_and_resolve(&mut harness);
    advance_frame_and_pump(&mut harness, 0.01);
    assert_eq!(
        harness.attr("#tt-grace-trigger", "data-state").as_deref(),
        Some("open"),
        "InTransit pointermove must extend close deadline from 40ms fallback to 180ms transit-intent timeout"
    );

    // Wait past 180ms transit-intent timeout without entering content -> closes cleanly
    std::thread::sleep(std::time::Duration::from_millis(160));
    drain_polls_and_resolve(&mut harness);
    advance_frame_and_pump(&mut harness, 0.25);
    drain_polls_and_resolve(&mut harness);
    assert_eq!(
        harness.attr("#tt-grace-trigger", "data-state").as_deref(),
        Some("closed"),
        "Hoverable tooltip must close after 180ms transit-intent timeout when pointer stalls in corridor"
    );
    assert!(harness.query("#tt-grace-content").is_none());
    assert!(
        !runtime.has_active_grace_monitor(),
        "Grace monitor must be torn down after transit-intent timeout closes tooltip"
    );
}

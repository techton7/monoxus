//! Native Overlay and Floating Primitives Verification Test Suite.
//!
//! Validates floating and overlay primitives (Popover, Tooltip, Dialog, AlertDialog)
//! using Blitz test harness over shared scenarios directly from `examples/playground/`.

mod common;
use common::native_fixture::*;
use dioxus::prelude::*;
use keyboard_types::{Code, Key, NamedKey};

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
    let _lock = TEST_LOCK.lock().unwrap_or_else(|e| e.into_inner());
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
    let _lock = TEST_LOCK.lock().unwrap_or_else(|e| e.into_inner());
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
    let _lock = TEST_LOCK.lock().unwrap_or_else(|e| e.into_inner());

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
    let _lock = TEST_LOCK.lock().unwrap_or_else(|e| e.into_inner());

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
    let _lock = TEST_LOCK.lock().unwrap_or_else(|e| e.into_inner());

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
    use blitz_traits::events::BlitzPointerId;

    let _lock = TEST_LOCK.lock().unwrap_or_else(|e| e.into_inner());

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
    let _lock = TEST_LOCK.lock().unwrap_or_else(|e| e.into_inner());

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

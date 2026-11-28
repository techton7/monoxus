use std::{cell::RefCell, rc::Rc};

use dioxus::{core::ScopeId, prelude::*};

use super::{
    DismissBehavior, DismissCallbacks, DismissConfig, DismissEvent, DismissLayer, FloatingLayer,
    FloatingReadiness, FocusGuards, FocusScope, GeometryVars, OutsidePressPolicy,
    OverlayCoordinator, OverlayLayerSpec, PlacementAlign, PlacementSide, PortalHost, Presence,
    PresenceController, PresenceState, Rect, Size, active_overlay_layers_snapshot,
    dispatch_central_escape_event_for_test, ensure_active_overlay_layer,
    is_overlay_escape_consumed, is_topmost_overlay_layer, mark_overlay_escape_consumed,
    register_overlay_escape_handler, reset_overlay_coordinator_for_test, sync_overlay_layer_spec,
    unregister_active_overlay_layer, use_dismissable_layer, use_overlay_layer_registration,
};
use crate::foundation::shared::Direction;

#[test]
fn portal_hosts_resolve_local_then_inherited_then_default_host() {
    let inherited = PortalHost::named("dialog-root");

    assert_eq!(
        PortalHost::resolve(Some(PortalHost::named("popover-root")), Some(&inherited)),
        PortalHost::named("popover-root"),
    );
    assert_eq!(PortalHost::resolve(None, Some(&inherited)), inherited);
    assert_eq!(PortalHost::resolve(None, None), PortalHost::default_host());
    assert!(PortalHost::default_host().is_default_host());
    assert!(PortalHost::inline().is_inline());
}

#[test]
fn retained_presence_waits_for_completion_before_unmounting() {
    let mut presence = Presence::new(true).with_retained_mount(true);

    assert_eq!(presence.sync(false), PresenceState::Suspended);
    assert!(presence.is_mounted());
    assert!(presence.complete_unmount());
    assert_eq!(presence.state(), PresenceState::Unmounted);
    assert!(!presence.is_mounted());
    assert!(!presence.complete_unmount());

    assert_eq!(presence.sync(true), PresenceState::Mounted);
    assert!(presence.is_mounted());
}

#[test]
fn presence_controller_preserves_close_cycles_and_rejects_stale_completion_after_reopen() {
    let mut controller = PresenceController::new(true).with_retained_mount(true);

    let close = controller.sync(false);
    let first_cycle = close.started_close_cycle().unwrap();
    assert_eq!(close.state(), PresenceState::Suspended);
    assert!(close.should_render());
    assert_eq!(close.active_close_cycle(), Some(first_cycle));
    assert_eq!(controller.active_close_cycle(), Some(first_cycle));

    let rerender = controller.sync(false);
    assert_eq!(rerender.state(), PresenceState::Suspended);
    assert_eq!(rerender.started_close_cycle(), None);
    assert_eq!(rerender.active_close_cycle(), Some(first_cycle));

    let reopen = controller.sync(true);
    assert_eq!(reopen.state(), PresenceState::Mounted);
    assert_eq!(reopen.invalidated_close_cycle(), Some(first_cycle));
    assert_eq!(controller.active_close_cycle(), None);

    let second_cycle = controller.sync(false).started_close_cycle().unwrap();
    assert_ne!(second_cycle, first_cycle);
    assert!(!controller.complete_close_cycle(first_cycle));
    assert_eq!(controller.state(), PresenceState::Suspended);
    assert!(controller.complete_close_cycle(second_cycle));
    assert_eq!(controller.state(), PresenceState::Unmounted);
    assert!(!controller.should_render());
}

#[test]
fn focus_scope_tracks_activation_restore_and_branch_state() {
    let mut scope = FocusScope::new("content")
        .with_trap_focus(true)
        .with_loop_focus(true);
    scope.capture_restore_target(Some("trigger"));
    scope.set_autofocus_target(Some("content"));

    assert!(scope.register_branch("portal"));
    assert!(scope.activate().is_some());
    assert!(scope.is_active());
    assert!(scope.is_parent_paused());
    assert!(scope.focus("portal"));
    assert_eq!(scope.last_focused(), Some(&"portal"));
    assert_eq!(scope.restore_focus(), Some("trigger"));
    assert!(!scope.focus("outside"));
    assert_eq!(scope.deactivate(), Some("trigger"));
    assert!(!scope.is_active());
    assert!(!scope.is_parent_paused());
}

#[test]
fn focus_guards_reference_count_their_lifecycle() {
    let mut guards = FocusGuards::new("before", "after");

    assert_eq!(guards.retain_count(), 0);
    assert!(!guards.is_installed());
    assert_eq!(guards.retain(), 1);
    assert!(guards.is_installed());
    assert_eq!(guards.release(), 0);
    assert!(!guards.is_installed());
}

#[test]
fn dismiss_layers_account_for_stack_position_branch_exceptions_and_modal_state() {
    let mut dialog = DismissLayer::new("dialog").with_modal(true);
    let mut popover = DismissLayer::new("popover");
    let stack = ["dialog", "popover"];

    assert!(dialog.register_branch("dialog-portal"));
    assert!(dialog.contains(&"dialog-portal"));
    assert!(dialog.blocks_outside_interaction());
    assert!(dialog.should_dismiss_escape(&["dialog"]));
    assert!(!dialog.should_dismiss_escape(&stack));
    assert!(popover.register_branch("popover-branch"));
    assert!(popover.should_dismiss_escape(&["popover"]));
    assert!(popover.should_dismiss_outside_pointer(Some(&"outside"), &["popover"]));
    assert!(!popover.should_dismiss_outside_pointer(Some(&"popover-branch"), &["popover"]));
    assert!(popover.should_dismiss_outside_focus(Some(&"outside"), &["popover"]));
}

#[test]
fn floating_layers_publish_namespaced_geometry_variables() {
    let layer = FloatingLayer::new(PlacementSide::Bottom)
        .with_align(PlacementAlign::Start)
        .with_direction(Direction::Rtl)
        .with_side_offset(8.0)
        .with_align_offset(4.0)
        .with_available_space(Size::new(120.0, 80.0))
        .with_namespace("dialog");
    let geometry = layer.geometry_vars(Rect::new(10.0, 20.0, 40.0, 16.0), Size::new(30.0, 12.0));

    assert_eq!(layer.data_side(), "bottom");
    assert_eq!(layer.data_align(), "start");
    assert_eq!(geometry.namespace(), "dialog");
    assert_eq!(geometry.x(), 24.0);
    assert_eq!(geometry.y(), 44.0);
    assert_eq!(geometry.transform_origin_x(), 26.0);
    assert_eq!(geometry.transform_origin_y(), 0.0);
    assert_eq!(geometry.available_width(), 120.0);
    assert_eq!(geometry.available_height(), 80.0);
    assert_eq!(geometry.get("--ui-dialog-floating-x"), Some(24.0));
    assert_eq!(geometry.get("--ui-dialog-available-width"), Some(120.0));
    assert_eq!(
        geometry.iter().collect::<Vec<_>>(),
        vec![
            ("--ui-dialog-floating-x".to_string(), 24.0),
            ("--ui-dialog-floating-y".to_string(), 44.0),
            ("--ui-dialog-transform-origin-x".to_string(), 26.0),
            ("--ui-dialog-transform-origin-y".to_string(), 0.0),
            ("--ui-dialog-available-width".to_string(), 120.0),
            ("--ui-dialog-available-height".to_string(), 80.0),
            ("--ui-dialog-anchor-width".to_string(), 40.0),
            ("--ui-dialog-anchor-height".to_string(), 16.0),
            ("--ui-dialog-content-width".to_string(), 30.0),
            ("--ui-dialog-content-height".to_string(), 12.0),
        ],
    );

    let monoxus_geometry = layer
        .with_css_prefix("monoxus")
        .geometry_vars(Rect::new(10.0, 20.0, 40.0, 16.0), Size::new(30.0, 12.0));
    assert_eq!(
        monoxus_geometry.get("--monoxus-dialog-floating-x"),
        Some(24.0)
    );
    assert_eq!(
        monoxus_geometry.get("--monoxus-dialog-available-width"),
        Some(120.0)
    );
    assert_eq!(
        GeometryVars::variable_name_with_prefix("monoxus", "dialog", "floating-x"),
        "--monoxus-dialog-floating-x"
    );
}

#[test]
fn floating_readiness_exposes_measurement_and_positioning_labels() {
    let measuring = FloatingReadiness::default();

    assert_eq!(measuring, FloatingReadiness::Measuring);
    assert_eq!(measuring.as_str(), "measuring");
    assert_eq!(measuring.positioning_state(), "unpositioned");
    assert!(!measuring.is_positioned());

    let ready = FloatingReadiness::from_is_positioned(true);
    assert_eq!(ready, FloatingReadiness::Ready);
    assert_eq!(ready.as_str(), "ready");
    assert_eq!(ready.positioning_state(), "positioned");
    assert!(ready.is_positioned());

    assert_eq!(
        FloatingReadiness::from_is_positioned(false),
        FloatingReadiness::Measuring,
    );

    let unpositioned = FloatingReadiness::unpositioned_style();
    assert!(unpositioned.contains("left: 0px"));
    assert!(unpositioned.contains("top: 0px"));
    assert!(unpositioned.contains("min-width: max-content"));
    assert!(unpositioned.contains("width: max-content"));
    assert!(unpositioned.contains("visibility: hidden"));
    assert!(unpositioned.contains("pointer-events: none"));

    assert_eq!(measuring.position_style(None), unpositioned);
    assert_eq!(ready.position_style(None), unpositioned);

    let placement = FloatingLayer::new(PlacementSide::Bottom)
        .with_align(PlacementAlign::Start)
        .with_direction(Direction::Rtl)
        .with_side_offset(8.0)
        .with_align_offset(4.0)
        .with_available_space(Size::new(120.0, 80.0))
        .position(Rect::new(10.0, 20.0, 40.0, 16.0), Size::new(30.0, 12.0));
    let positioned = ready.position_style(Some(&placement));
    assert_eq!(
        positioned,
        "position: fixed; left: 0px; top: 0px; transform: translate(24px, 44px); visibility: visible; pointer-events: auto;",
    );
}

#[test]
fn geometry_vars_publish_css_values_and_compatibility_aliases() {
    let default_geometry = FloatingLayer::new(PlacementSide::Bottom)
        .with_align(PlacementAlign::Start)
        .with_direction(Direction::Rtl)
        .with_side_offset(8.0)
        .with_align_offset(4.0)
        .with_available_space(Size::new(120.0, 80.0))
        .with_namespace("dialog")
        .geometry_vars(Rect::new(10.0, 20.0, 40.0, 16.0), Size::new(30.0, 12.0));

    assert_eq!(
        default_geometry.css_get("--ui-dialog-floating-x"),
        Some("24px".to_string())
    );
    assert_eq!(
        default_geometry.css_get("--ui-dialog-transform-origin-y"),
        Some("0px".to_string())
    );

    let geometry = default_geometry.with_css_prefix("monoxus");

    assert_eq!(
        geometry.css_get("--monoxus-dialog-floating-x"),
        Some("24px".to_string())
    );
    assert_eq!(
        geometry.css_get("--monoxus-dialog-transform-origin-y"),
        Some("0px".to_string())
    );
    assert_eq!(geometry.transform_origin_css_value(), "26px 0px");
    assert_eq!(
        geometry.css_iter().collect::<Vec<_>>(),
        vec![
            (
                "--monoxus-dialog-floating-x".to_string(),
                "24px".to_string()
            ),
            (
                "--monoxus-dialog-floating-y".to_string(),
                "44px".to_string()
            ),
            (
                "--monoxus-dialog-transform-origin-x".to_string(),
                "26px".to_string(),
            ),
            (
                "--monoxus-dialog-transform-origin-y".to_string(),
                "0px".to_string(),
            ),
            (
                "--monoxus-dialog-available-width".to_string(),
                "120px".to_string(),
            ),
            (
                "--monoxus-dialog-available-height".to_string(),
                "80px".to_string(),
            ),
            (
                "--monoxus-dialog-anchor-width".to_string(),
                "40px".to_string(),
            ),
            (
                "--monoxus-dialog-anchor-height".to_string(),
                "16px".to_string(),
            ),
            (
                "--monoxus-dialog-content-width".to_string(),
                "30px".to_string(),
            ),
            (
                "--monoxus-dialog-content-height".to_string(),
                "12px".to_string(),
            ),
        ],
    );
    assert_eq!(
        geometry
            .compatibility_alias_iter("radix-popover", "trigger")
            .collect::<Vec<_>>(),
        vec![
            (
                "--radix-popover-content-transform-origin".to_string(),
                "26px 0px".to_string(),
            ),
            (
                "--radix-popover-content-available-width".to_string(),
                "120px".to_string(),
            ),
            (
                "--radix-popover-content-available-height".to_string(),
                "80px".to_string(),
            ),
            (
                "--radix-popover-trigger-width".to_string(),
                "40px".to_string(),
            ),
            (
                "--radix-popover-trigger-height".to_string(),
                "16px".to_string(),
            ),
        ],
    );
}

#[test]
fn floating_layers_flip_to_the_opposite_side_and_publish_runtime_arrow_geometry() {
    let layer = FloatingLayer::new(PlacementSide::Top)
        .with_align(PlacementAlign::Start)
        .with_side_offset(8.0)
        .with_namespace("tooltip");
    let placement = layer.position_with_available_size(
        Rect::new(24.0, 8.0, 120.0, 38.0),
        Size::new(184.0, 72.0),
        Size::new(240.0, 140.0),
    );

    assert_eq!(placement.side(), PlacementSide::Bottom);
    assert_eq!(placement.align(), PlacementAlign::Start);
    assert_eq!(placement.geometry().x(), 24.0);
    assert_eq!(placement.geometry().y(), 54.0);
    assert_eq!(placement.arrow().x(), Some(60.0));
    assert_eq!(placement.arrow().y(), None);
    assert!(!placement.arrow().hidden());
    assert!(!placement.reference_hidden());
}

#[test]
fn floating_layers_hide_the_arrow_when_the_live_geometry_cannot_center_it() {
    let layer = FloatingLayer::new(PlacementSide::Bottom)
        .with_align(PlacementAlign::Center)
        .with_side_offset(8.0);
    let placement = layer.position_with_available_size(
        Rect::new(0.0, 30.0, 16.0, 20.0),
        Size::new(100.0, 40.0),
        Size::new(120.0, 120.0),
    );

    assert_eq!(placement.side(), PlacementSide::Bottom);
    assert!(placement.arrow().hidden());
    assert_eq!(placement.arrow().x(), None);
    assert_eq!(placement.arrow().y(), None);
    assert!(!placement.reference_hidden());
}

#[test]
fn floating_layers_can_hide_when_the_reference_detaches_from_the_viewport() {
    let visible = FloatingLayer::new(PlacementSide::Bottom).position_with_available_size(
        Rect::new(24.0, -8.0, 120.0, 38.0),
        Size::new(184.0, 72.0),
        Size::new(240.0, 140.0),
    );
    let hidden = FloatingLayer::new(PlacementSide::Bottom)
        .with_hide_when_detached(true)
        .position_with_available_size(
            Rect::new(24.0, -80.0, 120.0, 38.0),
            Size::new(184.0, 72.0),
            Size::new(240.0, 140.0),
        );

    assert!(!visible.reference_hidden());
    assert!(hidden.reference_hidden());
    assert!(hidden.arrow().hidden());
    assert_eq!(hidden.arrow().x(), None);
    assert_eq!(hidden.arrow().y(), None);
}

#[test]
fn floating_layers_keep_attached_popovers_offscreen_with_their_anchor() {
    let placement = FloatingLayer::new(PlacementSide::Bottom)
        .with_align(PlacementAlign::Start)
        .with_side_offset(12.0)
        .position_with_available_size(
            Rect::new(24.0, -198.0, 120.0, 35.0),
            Size::new(184.0, 209.0),
            Size::new(1280.0, 720.0),
        );

    assert_eq!(placement.side(), PlacementSide::Bottom);
    assert!(placement.geometry().y() < 0.0);
    assert_eq!(placement.geometry().y(), -151.0);
    assert!(!placement.reference_hidden());
}

// ---------------------------------------------------------------------------
// T2.3 OverlayCoordinator & Centralized Escape Regression Suite
// ---------------------------------------------------------------------------

#[test]
fn fm_esc_1_nested_child_and_parent_escape_dismisses_only_child_regardless_of_registration_order() {
    reset_overlay_coordinator_for_test();

    // Case A: Child Escape handler registered BEFORE Parent Escape handler (arbitrary order),
    // while coordinator stack open_order is [parent, child].
    ensure_active_overlay_layer("fm-esc-1-parent");
    ensure_active_overlay_layer("fm-esc-1-child");

    let parent_hits = Rc::new(RefCell::new(0usize));
    let child_hits = Rc::new(RefCell::new(0usize));

    register_overlay_escape_handler(
        "fm-esc-1-child",
        OverlayLayerSpec::new("fm-esc-1-child").with_parent(Some("fm-esc-1-parent")),
        {
            let child_hits = Rc::clone(&child_hits);
            move || {
                *child_hits.borrow_mut() += 1;
            }
        },
    );
    register_overlay_escape_handler("fm-esc-1-parent", OverlayLayerSpec::new("fm-esc-1-parent"), {
        let parent_hits = Rc::clone(&parent_hits);
        move || {
            *parent_hits.borrow_mut() += 1;
        }
    });

    let outcome = dispatch_central_escape_event_for_test();
    assert!(outcome.consumed);
    assert_eq!(outcome.dismissed_layers, vec!["fm-esc-1-child".to_string()]);
    assert_eq!(*child_hits.borrow(), 1);
    assert_eq!(*parent_hits.borrow(), 0);

    // Case B: Parent registered before Child, still dismisses ONLY Child.
    reset_overlay_coordinator_for_test();
    ensure_active_overlay_layer("fm-esc-1b-parent");
    ensure_active_overlay_layer("fm-esc-1b-child");

    let parent_hits_b = Rc::new(RefCell::new(0usize));
    let child_hits_b = Rc::new(RefCell::new(0usize));

    register_overlay_escape_handler(
        "fm-esc-1b-parent",
        OverlayLayerSpec::new("fm-esc-1b-parent"),
        {
            let parent_hits_b = Rc::clone(&parent_hits_b);
            move || {
                *parent_hits_b.borrow_mut() += 1;
            }
        },
    );
    register_overlay_escape_handler(
        "fm-esc-1b-child",
        OverlayLayerSpec::new("fm-esc-1b-child").with_parent(Some("fm-esc-1b-parent")),
        {
            let child_hits_b = Rc::clone(&child_hits_b);
            move || {
                *child_hits_b.borrow_mut() += 1;
            }
        },
    );

    let outcome_b = dispatch_central_escape_event_for_test();
    assert!(outcome_b.consumed);
    assert_eq!(
        outcome_b.dismissed_layers,
        vec!["fm-esc-1b-child".to_string()]
    );
    assert_eq!(*child_hits_b.borrow(), 1);
    assert_eq!(*parent_hits_b.borrow(), 0);

    reset_overlay_coordinator_for_test();
}

#[derive(Clone, Props)]
struct NestedEscapeProps {
    parent_dismissed: Rc<RefCell<usize>>,
    child_dismissed: Rc<RefCell<usize>>,
    unregister_child_sync: bool,
}

impl PartialEq for NestedEscapeProps {
    fn eq(&self, other: &Self) -> bool {
        Rc::ptr_eq(&self.parent_dismissed, &other.parent_dismissed)
            && Rc::ptr_eq(&self.child_dismissed, &other.child_dismissed)
            && self.unregister_child_sync == other.unregister_child_sync
    }
}

#[component]
fn NestedEscapeScenario(props: NestedEscapeProps) -> Element {
    let mut parent_open = use_signal(|| true);
    let mut child_open = use_signal(|| true);

    let parent_counter = Rc::clone(&props.parent_dismissed);
    let _parent = use_dismissable_layer(
        parent_open(),
        DismissConfig::new("fm-esc-nested-parent").with_modal(true),
        DismissCallbacks {
            on_dismiss: Some(EventHandler::new(move |evt: DismissEvent| {
                if matches!(evt, DismissEvent::Escape) {
                    *parent_counter.borrow_mut() += 1;
                    parent_open.set(false);
                }
            })),
            ..Default::default()
        },
    );

    let child_counter = Rc::clone(&props.child_dismissed);
    let unreg_sync = props.unregister_child_sync;
    let _child = use_dismissable_layer(
        child_open(),
        DismissConfig::new("fm-esc-nested-child"),
        DismissCallbacks {
            on_dismiss: Some(EventHandler::new(move |evt: DismissEvent| {
                if matches!(evt, DismissEvent::Escape) {
                    *child_counter.borrow_mut() += 1;
                    if unreg_sync {
                        unregister_active_overlay_layer("fm-esc-nested-child");
                    }
                    child_open.set(false);
                }
            })),
            ..Default::default()
        },
    );

    rsx! {}
}

#[test]
fn fm_esc_2_capture_phase_escape_consumption_blocks_same_turn_target_bubble_dismissal() {
    reset_overlay_coordinator_for_test();

    let parent_dismissed = Rc::new(RefCell::new(0usize));
    let child_dismissed = Rc::new(RefCell::new(0usize));

    let mut dom = VirtualDom::new_with_props(
        NestedEscapeScenario,
        NestedEscapeProps {
            parent_dismissed: Rc::clone(&parent_dismissed),
            child_dismissed: Rc::clone(&child_dismissed),
            unregister_child_sync: true,
        },
    );
    dom.rebuild_in_place();

    dom.in_scope(ScopeId::ROOT, || {
        assert!(!is_overlay_escape_consumed());
        assert_eq!(
            active_overlay_layers_snapshot(),
            vec![
                "fm-esc-nested-parent".to_string(),
                "fm-esc-nested-child".to_string()
            ]
        );

        // Capture-phase Escape dismisses Child (Combobox) and synchronously unregisters it.
        let outcome = dispatch_central_escape_event_for_test();
        assert!(outcome.consumed);
        assert_eq!(
            outcome.dismissed_layers,
            vec!["fm-esc-nested-child".to_string()]
        );
        assert_eq!(*child_dismissed.borrow(), 1);
        assert_eq!(*parent_dismissed.borrow(), 0);

        // Even though Parent (Dialog) is now topmost in the same synchronous turn,
        // `is_overlay_escape_consumed()` MUST remain `true` to block Target/Bubble `onkeydown`.
        assert!(is_topmost_overlay_layer("fm-esc-nested-parent"));
        assert!(is_overlay_escape_consumed());

        let mut bubble_dismissed_parent = false;
        if !is_overlay_escape_consumed() && is_topmost_overlay_layer("fm-esc-nested-parent") {
            bubble_dismissed_parent = true;
        }
        assert!(!bubble_dismissed_parent);
    });

    drop(dom);
    reset_overlay_coordinator_for_test();
}

#[test]
fn fm_esc_3_headless_vdom_drop_and_new_runtime_never_leak_stale_escape_consumption() {
    reset_overlay_coordinator_for_test();

    // 1. VirtualDom drops without any frame tick or async poll.
    let mut dom1 = VirtualDom::new(|| rsx! {});
    dom1.rebuild_in_place();
    dom1.in_scope(ScopeId::ROOT, || {
        mark_overlay_escape_consumed();
        assert!(is_overlay_escape_consumed());
    });
    drop(dom1);
    assert!(
        !is_overlay_escape_consumed(),
        "dropping VirtualDom without frame tick must invalidate Escape consumption"
    );

    // 2. Switching to a different active Runtime without dropping dom_a or ticking frames.
    let mut dom_a = VirtualDom::new(|| rsx! {});
    dom_a.rebuild_in_place();
    let mut dom_b = VirtualDom::new(|| rsx! {});
    dom_b.rebuild_in_place();

    dom_a.in_scope(ScopeId::ROOT, || {
        mark_overlay_escape_consumed();
        assert!(is_overlay_escape_consumed());
    });
    dom_b.in_scope(ScopeId::ROOT, || {
        assert!(
            !is_overlay_escape_consumed(),
            "new Runtime must never observe stale Escape consumption from a prior Runtime"
        );
    });

    drop(dom_b);
    drop(dom_a);
    reset_overlay_coordinator_for_test();
}

#[test]
fn fm_esc_4_back_to_back_escape_events_without_frame_ticks_dismiss_child_then_parent() {
    reset_overlay_coordinator_for_test();

    let parent_dismissed = Rc::new(RefCell::new(0usize));
    let child_dismissed = Rc::new(RefCell::new(0usize));

    let mut dom = VirtualDom::new_with_props(
        NestedEscapeScenario,
        NestedEscapeProps {
            parent_dismissed: Rc::clone(&parent_dismissed),
            child_dismissed: Rc::clone(&child_dismissed),
            unregister_child_sync: false,
        },
    );
    dom.rebuild_in_place();

    assert_eq!(
        active_overlay_layers_snapshot(),
        vec![
            "fm-esc-nested-parent".to_string(),
            "fm-esc-nested-child".to_string()
        ]
    );

    // Escape #1 without any frame tick -> dismisses Child only.
    let first = dispatch_central_escape_event_for_test();
    assert!(first.consumed);
    assert_eq!(
        first.dismissed_layers,
        vec!["fm-esc-nested-child".to_string()]
    );
    assert_eq!(*child_dismissed.borrow(), 1);
    assert_eq!(*parent_dismissed.borrow(), 0);

    // Re-render without ticking frames so Child unregisters on `active == false`.
    dom.render_immediate(&mut dioxus::core::NoOpMutations);
    assert_eq!(
        active_overlay_layers_snapshot(),
        vec!["fm-esc-nested-parent".to_string()]
    );

    // Escape #2 immediately (still zero frame ticks) -> advances Escape epoch and dismisses Parent.
    let second = dispatch_central_escape_event_for_test();
    assert!(second.consumed);
    assert_eq!(
        second.dismissed_layers,
        vec!["fm-esc-nested-parent".to_string()]
    );
    assert_eq!(*child_dismissed.borrow(), 1);
    assert_eq!(*parent_dismissed.borrow(), 1);

    dom.render_immediate(&mut dioxus::core::NoOpMutations);
    assert!(active_overlay_layers_snapshot().is_empty());

    drop(dom);
    reset_overlay_coordinator_for_test();
}

#[component]
fn ParentRemountProbe() -> Element {
    use_overlay_layer_registration("remount-parent", true);
    rsx! {}
}

#[test]
fn parent_remount_order_does_not_steal_topmost_escape_order_from_mounted_child() {
    reset_overlay_coordinator_for_test();

    let parent_hits = Rc::new(RefCell::new(0usize));
    let child_hits = Rc::new(RefCell::new(0usize));

    register_overlay_escape_handler("remount-parent", OverlayLayerSpec::new("remount-parent"), {
        let parent_hits = Rc::clone(&parent_hits);
        move || {
            *parent_hits.borrow_mut() += 1;
        }
    });
    register_overlay_escape_handler(
        "remount-child",
        OverlayLayerSpec::new("remount-child").with_parent(Some("remount-parent")),
        {
            let child_hits = Rc::clone(&child_hits);
            move || {
                *child_hits.borrow_mut() += 1;
            }
        },
    );

    assert_eq!(
        active_overlay_layers_snapshot(),
        vec!["remount-parent".to_string(), "remount-child".to_string()]
    );

    // Simulate parent remount via `ensure_active_overlay_layer`, `sync_overlay_layer_spec`,
    // and `use_overlay_layer_registration` inside a fresh VirtualDom render.
    ensure_active_overlay_layer("remount-parent");
    sync_overlay_layer_spec(OverlayLayerSpec::new("remount-parent").with_modal(true));
    let mut remount_dom = VirtualDom::new(ParentRemountProbe);
    remount_dom.rebuild_in_place();

    assert_eq!(
        active_overlay_layers_snapshot(),
        vec!["remount-parent".to_string(), "remount-child".to_string()],
        "parent remount must not steal topmost LIFO position from mounted child"
    );
    assert!(is_topmost_overlay_layer("remount-child"));
    assert!(!is_topmost_overlay_layer("remount-parent"));

    let outcome = dispatch_central_escape_event_for_test();
    assert_eq!(outcome.dismissed_layers, vec!["remount-child".to_string()]);
    assert_eq!(*child_hits.borrow(), 1);
    assert_eq!(*parent_hits.borrow(), 0);

    drop(remount_dom);
    reset_overlay_coordinator_for_test();
}

#[test]
fn sloppy_pointer_down_defer_otherwise_ignore_and_rename_layer_branch_merge() {
    use ui_sdk::kernel::ChangeSource;

    // 1. `Sloppy` outside `pointerdown` (`prevent_default()`) does not double-fire on trailing `click`.
    let mut sloppy_coord = OverlayCoordinator::new();
    sloppy_coord.open_or_promote(
        OverlayLayerSpec::new("sloppy-layer")
            .with_outside_press_policy(OutsidePressPolicy::Sloppy),
    );
    let pd = sloppy_coord.handle_pointer_down(Some("outside-btn"));
    assert!(pd.consumed);
    assert!(!pd.suppressed);
    assert_eq!(pd.dismissed_layers, vec!["sloppy-layer".to_string()]);

    // Consumer prevented default so `sloppy-layer` stays open; trailing mouse `click` must not double-fire.
    let trailing_click =
        sloppy_coord.handle_outside_click(Some("outside-btn"), ChangeSource::Mouse);
    assert!(trailing_click.dismissed_layers.is_empty());
    assert!(trailing_click.consumed);
    assert!(trailing_click.suppressed);

    // 2. `DeferOtherwiseIgnore` skips child (`dismiss=false`) and walks ancestor (`bubbles=true`).
    let mut defer_coord = OverlayCoordinator::new();
    defer_coord.open_or_promote(
        OverlayLayerSpec::new("defer-parent")
            .with_escape_behavior(DismissBehavior::Close)
            .with_outside_press_behavior(DismissBehavior::Close)
            .with_outside_press_policy(OutsidePressPolicy::Sloppy),
    );
    defer_coord.open_or_promote(
        OverlayLayerSpec::new("defer-child")
            .with_parent(Some("defer-parent"))
            .with_modal(true)
            .with_escape_behavior(DismissBehavior::DeferOtherwiseIgnore)
            .with_outside_press_behavior(DismissBehavior::DeferOtherwiseIgnore),
    );

    let esc = defer_coord.handle_escape();
    assert!(esc.consumed);
    assert_eq!(esc.dismissed_layers, vec!["defer-parent".to_string()]);

    let out_pd = defer_coord.handle_pointer_down(Some("outside-btn"));
    assert!(out_pd.consumed);
    assert_eq!(out_pd.dismissed_layers, vec!["defer-parent".to_string()]);

    let out_click = defer_coord.handle_outside_click(Some("outside-btn"), ChangeSource::Mouse);
    assert!(out_click.dismissed_layers.is_empty());
    assert!(out_click.suppressed);

    // 3. `rename_layer` merges unique branches into existing target layer cleanly.
    let mut branch_coord = OverlayCoordinator::new();
    branch_coord.open_or_promote(OverlayLayerSpec::new("old-layer"));
    branch_coord.open_or_promote(OverlayLayerSpec::new("target-layer"));
    assert!(branch_coord.register_branch("old-layer", "branch-only-old"));
    assert!(branch_coord.register_branch("old-layer", "branch-shared"));
    assert!(branch_coord.register_branch("target-layer", "branch-shared"));
    assert!(branch_coord.register_branch("target-layer", "branch-only-target"));

    assert!(branch_coord.rename_layer("old-layer", "target-layer"));
    assert_eq!(
        branch_coord.active_layers_snapshot(),
        vec!["target-layer".to_string()]
    );
    assert!(branch_coord.contains_in_tree("target-layer", "branch-only-old"));
    assert!(branch_coord.contains_in_tree("target-layer", "branch-shared"));
    assert!(branch_coord.contains_in_tree("target-layer", "branch-only-target"));
    assert!(branch_coord.contains_path_in_tree(
        "target-layer",
        &["unrelated".to_string(), "branch-only-old".to_string()]
    ));
}

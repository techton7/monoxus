use super::{
    ARROW_EDGE_MARGIN, CardinalSide, DismissLayer, FloatingLayer, FloatingReadiness, FocusGuards,
    FocusScope, PlacementAlign, PlacementSide, Point2D, PortalHost, Presence, PresenceController,
    PresenceState, PureOverlayPositionConfig, Rect, Rect2D, Size, StickyStrategy,
    compute_pure_overlay_position, create_grace_area, get_convex_hull, get_exit_side_from_rect,
    get_padded_exit_points, is_point_in_polygon, is_point_on_segment, resolve_inline_align,
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
    assert!(!PortalHost::default_host().is_inline());
    assert!(PortalHost::default_host().is_portalled());
    assert_eq!(PortalHost::default_host().host_attr_value(), "default");
    assert_eq!(PortalHost::default_host().as_str(), "default");
    assert_eq!(PortalHost::default_host().id(), None);

    let inline = PortalHost::inline();
    assert!(inline.is_inline());
    assert!(!inline.is_portalled());
    assert_eq!(inline.host_attr_value(), "inline");
    assert_eq!(inline.as_str(), "inline");
    assert_eq!(inline.id(), None);

    let named = PortalHost::named("custom-portal");
    assert!(!named.is_inline());
    assert!(named.is_portalled());
    assert_eq!(named.host_attr_value(), "custom-portal");
    assert_eq!(named.as_str(), "custom-portal");
    assert_eq!(named.id(), Some("custom-portal"));
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
    assert_eq!(geometry.get("--monoxus-dialog-floating-x"), Some(24.0));
    assert_eq!(
        geometry.get("--monoxus-dialog-available-width"),
        Some(120.0)
    );
    assert_eq!(
        geometry.iter().collect::<Vec<_>>(),
        vec![
            ("--monoxus-dialog-floating-x".to_string(), 24.0),
            ("--monoxus-dialog-floating-y".to_string(), 44.0),
            ("--monoxus-dialog-transform-origin-x".to_string(), 26.0),
            ("--monoxus-dialog-transform-origin-y".to_string(), 0.0),
            ("--monoxus-dialog-available-width".to_string(), 120.0),
            ("--monoxus-dialog-available-height".to_string(), 80.0),
            ("--monoxus-dialog-anchor-width".to_string(), 40.0),
            ("--monoxus-dialog-anchor-height".to_string(), 16.0),
            ("--monoxus-dialog-content-width".to_string(), 30.0),
            ("--monoxus-dialog-content-height".to_string(), 12.0),
        ],
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
}

#[test]
fn geometry_vars_publish_css_values_and_compatibility_aliases() {
    let geometry = FloatingLayer::new(PlacementSide::Bottom)
        .with_align(PlacementAlign::Start)
        .with_direction(Direction::Rtl)
        .with_side_offset(8.0)
        .with_align_offset(4.0)
        .with_available_space(Size::new(120.0, 80.0))
        .with_namespace("dialog")
        .geometry_vars(Rect::new(10.0, 20.0, 40.0, 16.0), Size::new(30.0, 12.0));

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
fn floating_layers_flip_left_right_collision() {
    let layer = FloatingLayer::new(PlacementSide::Left)
        .with_side_offset(4.0)
        .with_avoid_collisions(true);
    let placement = layer.position_with_available_size(
        Rect::new(10.0, 100.0, 50.0, 50.0),
        Size::new(80.0, 40.0),
        Size::new(300.0, 300.0),
    );

    assert_eq!(placement.side(), PlacementSide::Right);
    assert_eq!(placement.geometry().x(), 64.0);
}

#[test]
fn floating_layers_boundary_and_limit_shift_clamping() {
    let layer = FloatingLayer::new(PlacementSide::Top)
        .with_align(PlacementAlign::Start)
        .with_side_offset(5.0)
        .with_collision_padding(10.0)
        .with_sticky(StickyStrategy::Partial);

    let placement = layer.position_in_boundary(
        Rect::new(0.0, 100.0, 30.0, 30.0),
        Size::new(80.0, 40.0),
        Rect::new(0.0, 0.0, 200.0, 200.0),
    );

    assert_eq!(placement.geometry().x(), 10.0);
    assert_eq!(placement.geometry().y(), 55.0);
}

#[test]
fn floating_layers_rtl_inline_align_flip() {
    assert_eq!(
        resolve_inline_align(PlacementAlign::Start, Direction::Rtl),
        PlacementAlign::End
    );
    assert_eq!(
        resolve_inline_align(PlacementAlign::End, Direction::Rtl),
        PlacementAlign::Start
    );
    assert_eq!(
        resolve_inline_align(PlacementAlign::Center, Direction::Rtl),
        PlacementAlign::Center
    );
    assert_eq!(
        resolve_inline_align(PlacementAlign::Start, Direction::Ltr),
        PlacementAlign::Start
    );
}

#[test]
fn floating_layers_arrow_margin_and_hidden_guard() {
    assert_eq!(ARROW_EDGE_MARGIN, 12.0);

    let layer = FloatingLayer::new(PlacementSide::Bottom);
    let placement = layer.position_with_available_size(
        Rect::new(50.0, 20.0, 40.0, 20.0),
        Size::new(100.0, 30.0),
        Size::new(300.0, 300.0),
    );
    assert!(!placement.arrow().hidden());
    assert_eq!(placement.arrow().x(), Some(50.0));

    let narrow_placement = layer.position_with_available_size(
        Rect::new(50.0, 20.0, 20.0, 20.0),
        Size::new(20.0, 30.0),
        Size::new(300.0, 300.0),
    );
    assert!(narrow_placement.arrow().hidden());
    assert_eq!(narrow_placement.arrow().x(), None);
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

#[test]
fn compute_pure_overlay_position_evaluates_correctly() {
    let config = PureOverlayPositionConfig {
        anchor: Rect::new(100.0, 100.0, 40.0, 20.0),
        content: Size::new(60.0, 30.0),
        boundary: Some(Rect::new(0.0, 0.0, 500.0, 500.0)),
        side: PlacementSide::Bottom,
        align: PlacementAlign::Center,
        direction: Direction::Ltr,
        side_offset: 4.0,
        align_offset: 0.0,
        avoid_collisions: true,
        collision_padding: 8.0,
        arrow_padding: 12.0,
        sticky: StickyStrategy::Partial,
        hide_when_detached: true,
        namespace: Some("test-popover"),
    };

    let placement = compute_pure_overlay_position(&config);
    assert_eq!(placement.side(), PlacementSide::Bottom);
    assert_eq!(placement.align(), PlacementAlign::Center);
    assert_eq!(placement.geometry().x(), 90.0);
    assert_eq!(placement.geometry().y(), 124.0);
    assert!(!placement.reference_hidden());
    assert!(!placement.arrow().hidden());
}

#[test]
fn grace_rect2d_corners_contains_and_from_rect() {
    let rect = Rect2D::new(10.0, 20.0, 100.0, 50.0);
    assert_eq!(rect.left(), 10.0);
    assert_eq!(rect.top(), 20.0);
    assert_eq!(rect.right(), 110.0);
    assert_eq!(rect.bottom(), 70.0);

    let corners = rect.corners();
    assert_eq!(
        corners,
        [
            Point2D::new(10.0, 20.0),
            Point2D::new(110.0, 20.0),
            Point2D::new(110.0, 70.0),
            Point2D::new(10.0, 70.0),
        ]
    );

    assert!(rect.contains_point(Point2D::new(50.0, 40.0)));
    assert!(rect.contains_point(Point2D::new(10.0, 20.0)));
    assert!(rect.contains_point(Point2D::new(110.0, 70.0)));
    assert!(!rect.contains_point(Point2D::new(9.9, 40.0)));
    assert!(!rect.contains_point(Point2D::new(110.1, 40.0)));

    let pure_rect = Rect::new(10.0, 20.0, 100.0, 50.0);
    let from_rect: Rect2D = pure_rect.into();
    assert_eq!(rect, from_rect);
}

#[test]
fn grace_convex_hull_and_is_point_in_polygon() {
    let points = [
        Point2D::new(0.0, 0.0),
        Point2D::new(10.0, 0.0),
        Point2D::new(10.0, 10.0),
        Point2D::new(0.0, 10.0),
        Point2D::new(5.0, 5.0),
    ];

    let hull = get_convex_hull(&points);
    assert_eq!(hull.len(), 4);

    assert!(is_point_in_polygon(Point2D::new(5.0, 5.0), &hull));
    assert!(is_point_in_polygon(Point2D::new(0.0, 5.0), &hull));
    assert!(!is_point_in_polygon(Point2D::new(15.0, 5.0), &hull));
    assert!(is_point_on_segment(
        Point2D::new(5.0, 0.0),
        Point2D::new(0.0, 0.0),
        Point2D::new(10.0, 0.0),
        1e-9
    ));
}

#[test]
fn grace_corridor_creation() {
    let trigger = Rect2D::new(100.0, 100.0, 100.0, 50.0);
    let tooltip = Rect2D::new(100.0, 160.0, 100.0, 50.0);

    let exit_point = Point2D::new(150.0, 151.0);
    assert_eq!(
        get_exit_side_from_rect(exit_point, &trigger),
        CardinalSide::Bottom
    );

    let padded = get_padded_exit_points(exit_point, CardinalSide::Bottom, 5.0);
    assert_eq!(padded.len(), 3);

    let corridor = create_grace_area(exit_point, &trigger, &tooltip, Some(5.0));
    assert!(corridor.len() >= 4);

    assert!(is_point_in_polygon(Point2D::new(150.0, 155.0), &corridor));
}

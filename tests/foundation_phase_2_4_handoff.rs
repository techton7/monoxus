use std::{cell::RefCell, rc::Rc};

use monoxus::foundation::{
    compose::{
        AsChildSlot, EventHandlerOptions, RefHandler, Slottable, compose_event_handlers,
        compose_refs,
    },
    overlay::{
        DismissLayer, FloatingLayer, FocusGuardSide, FocusGuards, FocusScope, GeometryVars,
        PlacementAlign, PlacementSide, PortalHost, Presence, PresenceState, Rect, Size,
    },
    shared::{
        CollectionRegistry, Direction, Orientation, RovingFocusController, RovingFocusKey,
        ScopeHandle, use_direction, use_stable_id,
    },
    state::{self, DataState},
};

#[derive(Default)]
struct TestEvent {
    calls: Vec<&'static str>,
    default_prevented: bool,
}

fn is_default_prevented(event: &TestEvent) -> bool {
    event.default_prevented
}

#[test]
fn projected_compound_lane_reuses_public_state_composition_and_utility_exports() {
    let _ = state::use_controllable_state::<bool, fn(bool)>;
    let _ = state::use_controllable_state_reducer::<bool, bool, fn(&bool, bool) -> bool, fn(bool)>;
    let _ = use_stable_id;

    let scope = ScopeHandle::root("menu").child("root");
    let trigger_id = scope.qualify("trigger");
    let content_id = scope.qualify("content");

    assert_ne!(trigger_id, content_id);
    assert_eq!(scope.token(), "4:menu|4:root");
    assert_eq!(DataState::Open.as_str(), "open");

    let (projected_trigger, projected_content) = AsChildSlot::new(trigger_id.clone())
        .with_slottable(Slottable::new(content_id.clone()).map(|id| format!("{id}-panel")));
    assert_eq!(projected_trigger, trigger_id);
    assert_eq!(projected_content, format!("{content_id}-panel"));

    let mut event = TestEvent::default();
    let mut handlers = compose_event_handlers(
        Some(|event: &mut TestEvent| {
            event.calls.push("consumer");
            event.default_prevented = true;
        }),
        Some(|event: &mut TestEvent| event.calls.push("internal")),
        EventHandlerOptions::cancelable(is_default_prevented),
    );
    handlers(&mut event);
    assert_eq!(event.calls, vec!["consumer"]);

    let seen = Rc::new(RefCell::new(Vec::new()));
    let refs: Vec<Option<RefHandler<String>>> = vec![
        Some(Box::new({
            let seen = Rc::clone(&seen);
            move |value| seen.borrow_mut().push(format!("trigger:{value}"))
        })),
        Some(Box::new({
            let seen = Rc::clone(&seen);
            move |value| seen.borrow_mut().push(format!("content:{value}"))
        })),
    ];
    let mut composed_refs = compose_refs(refs);
    composed_refs(scope.token());
    assert_eq!(
        *seen.borrow(),
        vec![
            String::from("trigger:4:menu|4:root"),
            String::from("content:4:menu|4:root"),
        ],
    );

    let mut registry = CollectionRegistry::new();
    assert!(registry.register("trigger", true));
    assert!(registry.register("separator", false));
    assert!(registry.register("content", true));

    let controller = RovingFocusController::new(use_direction(None, Some(Direction::Rtl)))
        .with_orientation(Orientation::Horizontal)
        .with_looping(true);

    assert_eq!(
        controller.navigate_by_key(
            &registry,
            Some(&"trigger"),
            RovingFocusKey::ArrowLeft,
            |focusable| *focusable,
        ),
        Some("content"),
    );
}

#[test]
fn positioned_overlay_lane_publishes_wrapper_safe_overlay_hooks_and_geometry() {
    let inherited_host = PortalHost::default_host();
    let host = PortalHost::resolve(Some(PortalHost::named("layers")), Some(&inherited_host));

    assert!(host.is_portalled());
    assert_eq!(host.id(), Some("layers"));

    let mut presence = Presence::new(true).with_retained_mount(true);
    assert_eq!(presence.sync(false), PresenceState::Suspended);
    assert!(presence.is_mounted());
    assert!(presence.complete_unmount());
    assert_eq!(presence.state(), PresenceState::Unmounted);

    let mut focus_scope = FocusScope::new("content")
        .with_trap_focus(true)
        .with_loop_focus(true);
    assert!(focus_scope.register_branch("item"));
    focus_scope.set_autofocus_target(Some("item"));
    focus_scope.capture_restore_target(Some("trigger"));

    assert_eq!(focus_scope.activate(), Some("item"));
    assert!(focus_scope.focus("item"));
    assert_eq!(focus_scope.deactivate(), Some("trigger"));

    let mut guards = FocusGuards::new("before", "after");
    assert_eq!(guards.retain(), 1);
    assert_eq!(guards.retain(), 2);
    assert!(guards.is_installed());
    assert_eq!(guards.side_of(&"after"), Some(FocusGuardSide::After));
    assert_eq!(guards.release(), 1);

    let mut layer = DismissLayer::new("content").with_modal(true);
    assert!(layer.register_branch("item"));
    let stack = vec!["background", "content"];

    assert!(layer.blocks_outside_interaction());
    assert!(layer.should_dismiss_outside_pointer(Some(&"outside"), &stack));
    assert!(!layer.should_dismiss_outside_focus(Some(&"item"), &stack));

    let floating = FloatingLayer::new(PlacementSide::Bottom)
        .with_align(PlacementAlign::Start)
        .with_direction(use_direction(None, Some(Direction::Rtl)))
        .with_side_offset(4.0)
        .with_available_space(Size::new(200.0, 120.0))
        .with_namespace("popover");

    let geometry = floating.geometry_vars(Rect::new(80.0, 20.0, 40.0, 10.0), Size::new(20.0, 30.0));

    assert_eq!(DataState::Open.as_str(), "open");
    assert_eq!(floating.data_side(), "bottom");
    assert_eq!(floating.data_align(), "start");
    assert_eq!(geometry.namespace(), "popover");
    assert_eq!(
        GeometryVars::variable_name("popover", "floating-x"),
        "--monoxus-popover-floating-x",
    );
    assert_eq!(geometry.get("--monoxus-popover-floating-x"), Some(100.0));
    assert_eq!(geometry.get("--monoxus-popover-floating-y"), Some(34.0));
    assert_eq!(
        geometry.get("--monoxus-popover-transform-origin-x"),
        Some(20.0)
    );
    assert_eq!(
        geometry.get("--monoxus-popover-transform-origin-y"),
        Some(0.0)
    );
}

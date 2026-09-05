use std::{cell::RefCell, rc::Rc};

use monoxus::{
    foundation::{
        compose::{EventHandlerOptions, RefHandler, Slottable},
        overlay::{FloatingLayer, PlacementAlign, PlacementSide, PortalHost, Rect, Size},
        shared::ScopeHandle,
        state::{self, DataState},
    },
    popover::{
        Popover, PopoverCloseFocusPolicy, PopoverOpenFocusPolicy, PopoverOutsideDismissBehavior,
        PopoverOutsideInteractionPolicy, PopoverPart, PopoverRuntime, PopoverScrollLockPolicy,
        PopoverStateRequest, compose_part_event_handlers as compose_popover_event_handlers,
        compose_part_refs as compose_popover_refs, project_as_child as project_popover_as_child,
        use_popover_runtime,
    },
    tooltip::{
        Tooltip, TooltipPart, TooltipProvider, TooltipRuntime, TooltipStateRequest,
        compose_part_event_handlers as compose_tooltip_event_handlers,
        compose_part_refs as compose_tooltip_refs, project_as_child as project_tooltip_as_child,
        use_tooltip_provider_runtime, use_tooltip_runtime,
    },
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
fn positioned_overlay_family_boundary_matches_the_phase_3_2_inventory() {
    let popover_parts: Vec<_> = Popover::parts().iter().map(PopoverPart::as_str).collect();
    let tooltip_parts: Vec<_> = Tooltip::parts().iter().map(TooltipPart::as_str).collect();

    assert_eq!(
        popover_parts,
        vec![
            "root", "trigger", "portal", "content", "arrow", "anchor", "close"
        ],
    );
    assert_eq!(
        tooltip_parts,
        vec!["root", "trigger", "portal", "content", "arrow", "provider"],
    );

    for omitted in [
        "overlay",
        "title",
        "description",
        "header",
        "footer",
        "hover-card",
        "context-menu",
        "dropdown-menu",
        "select",
        "combobox",
    ] {
        assert!(!popover_parts.contains(&omitted));
        assert!(!tooltip_parts.contains(&omitted));
    }
}

#[test]
fn popover_reuses_the_shared_state_composition_and_overlay_backbone() {
    let _ = use_popover_runtime::<fn(bool)>;
    let _ = PopoverRuntime::mount_anchor;
    let _ = PopoverRuntime::placement;
    let _ = PopoverRuntime::outside_pointer_down;
    let _ = PopoverRuntime::outside_focus_in;
    let _ = PopoverRuntime::escape_keydown;

    let _ = state::use_controllable_state::<bool, fn(bool)>;
    let _ = state::use_controllable_state_reducer::<bool, bool, fn(&bool, bool) -> bool, fn(bool)>;

    let scope = ScopeHandle::root("popover").child("root");
    let branch_id = scope.qualify("branch");
    let restore_id = scope.qualify("restore");
    let mut popover = Popover::new(scope.clone(), true)
        .with_portal_host(PortalHost::named("layers"))
        .with_floating(
            FloatingLayer::new(PlacementSide::Right)
                .with_align(PlacementAlign::End)
                .with_side_offset(6.0)
                .with_available_space(Size::new(180.0, 140.0)),
        )
        .with_modal(true)
        .with_open_focus_policy(PopoverOpenFocusPolicy::Target(branch_id.clone()))
        .with_close_focus_policy(PopoverCloseFocusPolicy::Target(restore_id.clone()))
        .with_scroll_lock_policy(PopoverScrollLockPolicy::enabled().with_restore_delay(Some(12)))
        .with_outside_interaction_policy(PopoverOutsideInteractionPolicy::new(
            PopoverOutsideDismissBehavior::Ignore,
            PopoverOutsideDismissBehavior::Dismiss,
        ));
    let relationships = popover.relationships().clone();
    let geometry = popover.geometry_vars(Rect::new(40.0, 20.0, 30.0, 10.0), Size::new(50.0, 20.0));
    let root = popover.root();
    let trigger = popover.trigger();
    let anchor = popover.anchor();
    let portal = popover.portal();
    let content = popover.content();
    let arrow = popover.arrow();
    let close = popover.close();

    assert_eq!(relationships.scope(), &scope);
    assert_eq!(relationships.root_id(), scope.token());
    assert_eq!(relationships.trigger_id(), scope.qualify("trigger"));
    assert_eq!(relationships.anchor_id(), scope.qualify("anchor"));
    assert_eq!(relationships.content_id(), scope.qualify("content"));
    assert_eq!(relationships.arrow_id(), scope.qualify("arrow"));
    assert_eq!(relationships.close_id(), scope.qualify("close"));

    assert_eq!(popover.data_state(), DataState::Open);
    assert_eq!(root.id(), relationships.root_id());
    assert_eq!(root.data_state(), &DataState::Open);
    assert_eq!(trigger.id(), relationships.trigger_id());
    assert_eq!(trigger.aria_controls(), relationships.content_id());
    assert!(trigger.aria_expanded());
    assert_eq!(trigger.data_state(), &DataState::Open);
    assert_eq!(trigger.open_request(), PopoverStateRequest::Toggle);
    assert!(!trigger.open_request().next_open(true));
    assert_eq!(trigger.open_request().data_state(true), DataState::Closed);
    assert_eq!(anchor.id(), relationships.anchor_id());
    assert_eq!(portal.host(), &PortalHost::named("layers"));
    assert_eq!(content.id(), relationships.content_id());
    assert_eq!(content.role(), "dialog");
    assert!(content.aria_modal());
    assert_eq!(content.data_state(), &DataState::Open);
    assert_eq!(content.data_side(), "right");
    assert_eq!(content.data_align(), "end");
    assert_eq!(arrow.id(), relationships.arrow_id());
    assert_eq!(arrow.data_state(), &DataState::Open);
    assert_eq!(arrow.data_side(), "right");
    assert_eq!(arrow.data_align(), "end");
    assert_eq!(close.id(), relationships.close_id());
    assert_eq!(close.data_state(), &DataState::Open);
    assert_eq!(close.close_request(), PopoverStateRequest::Close);
    assert!(!close.close_request().next_open(true));

    assert_eq!(
        popover.lifecycle().portal_host(),
        &PortalHost::named("layers")
    );
    assert!(popover.lifecycle().presence().is_mounted());
    assert!(popover.lifecycle().is_modal());
    assert_eq!(
        popover.lifecycle().open_focus_policy(),
        &PopoverOpenFocusPolicy::Target(branch_id.clone()),
    );
    assert_eq!(
        popover.lifecycle().close_focus_policy(),
        &PopoverCloseFocusPolicy::Target(restore_id.clone()),
    );
    assert_eq!(
        popover.lifecycle().scroll_lock_policy(),
        &PopoverScrollLockPolicy::enabled().with_restore_delay(Some(12)),
    );
    assert_eq!(
        popover.lifecycle().outside_interaction_policy(),
        &PopoverOutsideInteractionPolicy::new(
            PopoverOutsideDismissBehavior::Ignore,
            PopoverOutsideDismissBehavior::Dismiss,
        ),
    );
    assert_eq!(
        popover.lifecycle().focus_scope().root(),
        relationships.content_id()
    );
    assert!(popover.lifecycle().focus_scope().traps_focus());
    assert!(popover.lifecycle().focus_scope().loops_focus());
    assert_eq!(
        popover.lifecycle().focus_scope().autofocus_target(),
        Some(&branch_id),
    );
    assert_eq!(
        popover.lifecycle().focus_scope().restore_target(),
        Some(&restore_id),
    );
    assert_eq!(
        popover.lifecycle().dismiss_layer().id(),
        relationships.content_id()
    );
    assert!(
        popover
            .lifecycle()
            .dismiss_layer()
            .blocks_outside_interaction()
    );
    assert_eq!(popover.lifecycle().floating().data_side(), "right");
    assert_eq!(popover.lifecycle().floating().data_align(), "end");

    let lifecycle = popover.lifecycle_mut();
    assert!(lifecycle.register_branch(branch_id.clone()));
    assert_eq!(
        lifecycle.focus_scope_mut().activate(),
        Some(branch_id.clone())
    );
    assert_eq!(lifecycle.focus_scope_mut().deactivate(), Some(restore_id));
    assert_eq!(lifecycle.focus_guards_mut().retain(), 1);

    let stack = vec![
        String::from("background"),
        relationships.content_id().to_owned(),
    ];
    assert!(
        !lifecycle
            .dismiss_layer()
            .should_dismiss_outside_pointer(Some(&String::from("outside")), &stack)
    );
    assert!(
        lifecycle
            .dismiss_layer()
            .should_dismiss_outside_focus(Some(&String::from("outside")), &stack)
    );

    assert_eq!(geometry.namespace(), "popover");
    assert_eq!(geometry.get("--monoxus-popover-floating-x"), Some(76.0));
    assert_eq!(geometry.get("--monoxus-popover-floating-y"), Some(10.0));
    assert_eq!(
        geometry.get("--monoxus-popover-transform-origin-x"),
        Some(0.0),
    );
    assert_eq!(
        geometry.get("--monoxus-popover-transform-origin-y"),
        Some(20.0),
    );

    let (projected_trigger, projected_content) = project_popover_as_child(
        relationships.trigger_id().to_owned(),
        Slottable::new(relationships.content_id().to_owned()).map(|id| format!("{id}-surface")),
    );
    assert_eq!(projected_trigger, relationships.trigger_id());
    assert_eq!(
        projected_content,
        format!("{}-surface", relationships.content_id())
    );

    let mut event = TestEvent::default();
    let mut handlers = compose_popover_event_handlers(
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
    let mut composed_refs = compose_popover_refs(refs);
    composed_refs(relationships.content_id().to_owned());
    assert_eq!(
        *seen.borrow(),
        vec![
            format!("trigger:{}", relationships.content_id()),
            format!("content:{}", relationships.content_id()),
        ],
    );
}

#[test]
fn tooltip_reuses_the_shared_state_composition_and_overlay_backbone() {
    let _ = use_tooltip_provider_runtime;
    let _ = use_tooltip_runtime::<fn(bool)>;
    let _ = TooltipRuntime::mount_trigger;
    let _ = TooltipRuntime::mount_content;
    let _ = TooltipRuntime::placement;
    let _ = TooltipRuntime::escape_keydown;

    let provider = TooltipProvider::new(ScopeHandle::root("tooltip").child("provider"))
        .with_delay_duration_ms(600)
        .with_skip_delay_duration_ms(250)
        .with_disable_hoverable_content(true)
        .with_close_on_trigger_click(false)
        .with_ignore_non_keyboard_focus(true);
    let scope = ScopeHandle::root("tooltip").child("root");
    let mut tooltip = Tooltip::new(scope.clone(), true)
        .with_provider(provider.clone())
        .with_portal_host(PortalHost::inline())
        .with_floating(
            FloatingLayer::new(PlacementSide::Top)
                .with_align(PlacementAlign::Start)
                .with_side_offset(8.0)
                .with_available_space(Size::new(160.0, 120.0)),
        );
    let relationships = tooltip.relationships().clone();
    let geometry = tooltip.geometry_vars(Rect::new(60.0, 50.0, 40.0, 20.0), Size::new(50.0, 24.0));
    let root = tooltip.root();
    let trigger = tooltip.trigger();
    let portal = tooltip.portal();
    let content = tooltip.content();
    let arrow = tooltip.arrow();

    assert_eq!(relationships.scope(), &scope);
    assert_eq!(relationships.root_id(), scope.token());
    assert_eq!(relationships.trigger_id(), scope.qualify("trigger"));
    assert_eq!(relationships.content_id(), scope.qualify("content"));
    assert_eq!(relationships.arrow_id(), scope.qualify("arrow"));

    assert_eq!(provider.delay_duration_ms(), 600);
    assert_eq!(provider.skip_delay_duration_ms(), 250);
    assert!(provider.disable_hoverable_content());
    assert!(!provider.close_on_trigger_click());
    assert!(provider.ignore_non_keyboard_focus());

    assert_eq!(tooltip.data_state(), DataState::Open);
    assert_eq!(root.id(), relationships.root_id());
    assert_eq!(root.data_state(), &DataState::Open);
    assert_eq!(trigger.id(), relationships.trigger_id());
    assert_eq!(trigger.aria_describedby(), Some(relationships.content_id()));
    assert_eq!(trigger.provider_id(), Some(provider.id()));
    assert_eq!(trigger.data_state(), &DataState::Open);
    assert_eq!(trigger.open_request(), TooltipStateRequest::Open);
    assert_eq!(trigger.close_request(), TooltipStateRequest::Close);
    assert!(trigger.open_request().next_open());
    assert_eq!(trigger.close_request().data_state(), DataState::Closed);
    assert!(portal.host().is_inline());
    assert_eq!(content.id(), relationships.content_id());
    assert_eq!(content.role(), "tooltip");
    assert_eq!(content.data_state(), &DataState::Open);
    assert_eq!(content.data_side(), "top");
    assert_eq!(content.data_align(), "start");
    assert!(content.autofocus_suppressed());
    assert_eq!(arrow.id(), relationships.arrow_id());
    assert_eq!(arrow.data_state(), &DataState::Open);
    assert_eq!(arrow.data_side(), "top");
    assert_eq!(arrow.data_align(), "start");

    assert!(tooltip.lifecycle().presence().is_mounted());
    assert_eq!(
        tooltip.lifecycle().dismiss_layer().id(),
        relationships.content_id()
    );
    assert!(
        !tooltip
            .lifecycle()
            .dismiss_layer()
            .blocks_outside_interaction()
    );
    assert_eq!(tooltip.lifecycle().floating().data_side(), "top");
    assert_eq!(tooltip.lifecycle().floating().data_align(), "start");
    assert!(tooltip.lifecycle().content_autofocus_suppressed());
    assert_eq!(tooltip.provider(), Some(&provider));

    let lifecycle = tooltip.lifecycle_mut();
    assert!(lifecycle.register_branch(scope.qualify("content-branch")));

    let stack = vec![relationships.content_id().to_owned()];
    assert!(
        !lifecycle
            .dismiss_layer()
            .should_dismiss_outside_focus(Some(&scope.qualify("content-branch")), &stack)
    );
    assert!(
        lifecycle
            .dismiss_layer()
            .should_dismiss_outside_pointer(Some(&String::from("outside")), &stack)
    );

    assert_eq!(geometry.namespace(), "tooltip");
    assert_eq!(geometry.get("--monoxus-tooltip-floating-x"), Some(60.0));
    assert_eq!(geometry.get("--monoxus-tooltip-floating-y"), Some(18.0));
    assert_eq!(
        geometry.get("--monoxus-tooltip-transform-origin-x"),
        Some(0.0),
    );
    assert_eq!(
        geometry.get("--monoxus-tooltip-transform-origin-y"),
        Some(24.0),
    );

    let (projected_trigger, projected_content) = project_tooltip_as_child(
        relationships.trigger_id().to_owned(),
        Slottable::new(relationships.content_id().to_owned()).map(|id| format!("{id}-tooltip")),
    );
    assert_eq!(projected_trigger, relationships.trigger_id());
    assert_eq!(
        projected_content,
        format!("{}-tooltip", relationships.content_id())
    );

    let mut event = TestEvent::default();
    let mut handlers = compose_tooltip_event_handlers(
        Some(|event: &mut TestEvent| event.calls.push("consumer")),
        Some(|event: &mut TestEvent| event.calls.push("internal")),
        EventHandlerOptions::always_invoke_internal(),
    );
    handlers(&mut event);
    assert_eq!(event.calls, vec!["consumer", "internal"]);

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
    let mut composed_refs = compose_tooltip_refs(refs);
    composed_refs(relationships.trigger_id().to_owned());
    assert_eq!(
        *seen.borrow(),
        vec![
            format!("trigger:{}", relationships.trigger_id()),
            format!("content:{}", relationships.trigger_id()),
        ],
    );
}

#[test]
fn floating_layers_flip_and_publish_runtime_arrow_contracts() {
    let flipped = FloatingLayer::new(PlacementSide::Top)
        .with_align(PlacementAlign::Start)
        .with_side_offset(8.0)
        .position_with_available_size(
            Rect::new(24.0, 8.0, 120.0, 38.0),
            Size::new(184.0, 72.0),
            Size::new(240.0, 140.0),
        );
    let constrained = FloatingLayer::new(PlacementSide::Bottom)
        .with_align(PlacementAlign::Center)
        .with_side_offset(8.0)
        .position_with_available_size(
            Rect::new(0.0, 30.0, 16.0, 20.0),
            Size::new(100.0, 40.0),
            Size::new(120.0, 120.0),
        );

    assert_eq!(flipped.side(), PlacementSide::Bottom);
    assert_eq!(flipped.align(), PlacementAlign::Start);
    assert_eq!(flipped.geometry().x(), 24.0);
    assert_eq!(flipped.geometry().y(), 54.0);
    assert_eq!(flipped.arrow().x(), Some(60.0));
    assert_eq!(flipped.arrow().y(), None);
    assert!(!flipped.arrow().hidden());
    assert!(!flipped.reference_hidden());

    assert_eq!(constrained.side(), PlacementSide::Bottom);
    assert!(constrained.arrow().hidden());
    assert_eq!(constrained.arrow().x(), None);
    assert_eq!(constrained.arrow().y(), None);
    assert!(!constrained.reference_hidden());
}

#[test]
fn floating_layers_can_flag_detached_references_for_hidden_popovers() {
    let hidden = FloatingLayer::new(PlacementSide::Bottom)
        .with_hide_when_detached(true)
        .position_with_available_size(
            Rect::new(24.0, -80.0, 120.0, 38.0),
            Size::new(184.0, 72.0),
            Size::new(240.0, 140.0),
        );
    let still_visible = FloatingLayer::new(PlacementSide::Bottom)
        .with_hide_when_detached(true)
        .position_with_available_size(
            Rect::new(24.0, -8.0, 120.0, 38.0),
            Size::new(184.0, 72.0),
            Size::new(240.0, 140.0),
        );

    assert!(hidden.reference_hidden());
    assert!(hidden.arrow().hidden());
    assert_eq!(hidden.arrow().x(), None);
    assert_eq!(hidden.arrow().y(), None);
    assert!(!still_visible.reference_hidden());
}

#[test]
fn floating_layers_keep_attached_popovers_following_anchors_offscreen() {
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

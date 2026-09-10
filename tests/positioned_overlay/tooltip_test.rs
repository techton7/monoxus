use std::{cell::RefCell, rc::Rc};

use monoxus::{
    foundation::{
        compose::{EventHandlerOptions, RefHandler, Slottable},
        overlay::{FloatingLayer, PlacementAlign, PlacementSide, PortalHost, Rect, Size},
        shared::ScopeHandle,
        state::DataState,
    },
    tooltip::{
        Tooltip, TooltipProvider, TooltipRuntime, TooltipStateRequest,
        compose_part_event_handlers as compose_tooltip_event_handlers,
        compose_part_refs as compose_tooltip_refs, project_as_child as project_tooltip_as_child,
        use_tooltip_provider_runtime, use_tooltip_runtime,
    },
};

use super::fixtures::TestEvent;

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

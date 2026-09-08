use std::{cell::RefCell, rc::Rc};

use monoxus::{
    foundation::{
        compose::{EventHandlerOptions, RefHandler, Slottable},
        overlay::{FloatingLayer, PlacementAlign, PlacementSide, PortalHost, Rect, Size},
        shared::ScopeHandle,
        state::{self, DataState},
    },
    popover::{
        compose_part_event_handlers as compose_popover_event_handlers,
        compose_part_refs as compose_popover_refs, project_as_child as project_popover_as_child,
        use_popover_runtime, Popover, PopoverCloseFocusPolicy, PopoverOpenFocusPolicy,
        PopoverOutsideDismissBehavior, PopoverOutsideInteractionPolicy, PopoverRuntime,
        PopoverScrollLockPolicy, PopoverStateRequest,
    },
};

use super::fixtures::{is_default_prevented, TestEvent};

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

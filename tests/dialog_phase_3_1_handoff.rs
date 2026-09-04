use std::{cell::RefCell, rc::Rc};

use monoxus::{
    alert_dialog::{AlertDialog, AlertDialogPart},
    dialog::{
        Dialog, DialogPart, DialogStateRequest, compose_part_event_handlers, compose_part_refs,
        project_as_child,
    },
    foundation::{
        compose::{EventHandlerOptions, RefHandler, Slottable},
        overlay::{PortalHost, PresenceState},
        shared::ScopeHandle,
        state::{self, DataState},
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
fn dialog_family_publishes_stable_relationships_and_wrapper_safe_attributes() {
    let _ = state::use_controllable_state::<bool, fn(bool)>;
    let _ = state::use_controllable_state_reducer::<bool, bool, fn(&bool, bool) -> bool, fn(bool)>;

    let scope = ScopeHandle::root("dialog").child("root");
    let dialog = Dialog::new(scope.clone(), true);
    let relationships = dialog.relationships();

    let part_names: Vec<_> = Dialog::parts().iter().map(DialogPart::as_str).collect();
    assert_eq!(
        part_names,
        vec![
            "root",
            "trigger",
            "portal",
            "overlay",
            "content",
            "title",
            "description",
            "close",
        ],
    );

    assert_eq!(relationships.scope(), &scope);
    assert_eq!(relationships.root_id(), scope.token());
    assert_eq!(relationships.trigger_id(), scope.qualify("trigger"));
    assert_eq!(relationships.overlay_id(), scope.qualify("overlay"));
    assert_eq!(relationships.content_id(), scope.qualify("content"));
    assert_eq!(relationships.title_id(), scope.qualify("title"));
    assert_eq!(relationships.description_id(), scope.qualify("description"));
    assert_eq!(relationships.close_id(), scope.qualify("close"));

    assert_eq!(dialog.data_state(), DataState::Open);
    assert_eq!(dialog.root().data_state(), &DataState::Open);

    let trigger = dialog.trigger();
    assert_eq!(trigger.id(), relationships.trigger_id());
    assert_eq!(trigger.aria_controls(), relationships.content_id());
    assert!(trigger.aria_expanded());
    assert_eq!(trigger.open_request(), DialogStateRequest::Open);
    assert_eq!(trigger.data_state(), &DataState::Open);

    let portal = dialog.portal();
    assert_eq!(portal.host(), &PortalHost::default_host());

    let overlay = dialog.overlay();
    assert_eq!(overlay.id(), relationships.overlay_id());
    assert_eq!(overlay.data_state(), &DataState::Open);

    let content = dialog.content();
    assert_eq!(content.id(), relationships.content_id());
    assert_eq!(content.role(), "dialog");
    assert!(content.aria_modal());
    assert_eq!(content.aria_labelledby(), relationships.title_id());
    assert_eq!(content.aria_describedby(), relationships.description_id());
    assert_eq!(content.data_state(), &DataState::Open);

    assert_eq!(dialog.title().id(), relationships.title_id());
    assert_eq!(dialog.description().id(), relationships.description_id());
    assert_eq!(dialog.close().id(), relationships.close_id());
    assert_eq!(dialog.close().close_request(), DialogStateRequest::Close);

    let (projected_trigger, projected_content) = project_as_child(
        relationships.trigger_id().to_owned(),
        Slottable::new(relationships.content_id().to_owned()).map(|id| format!("{id}-slot")),
    );
    assert_eq!(projected_trigger, relationships.trigger_id());
    assert_eq!(
        projected_content,
        format!("{}-slot", relationships.content_id())
    );

    let mut event = TestEvent::default();
    let mut handlers = compose_part_event_handlers(
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
    let mut composed_refs = compose_part_refs(refs);
    composed_refs(dialog.relationships().content_id().to_owned());
    assert_eq!(
        *seen.borrow(),
        vec![
            format!("trigger:{}", dialog.relationships().content_id()),
            format!("content:{}", dialog.relationships().content_id()),
        ],
    );
}

#[test]
fn dialog_family_reuses_portal_presence_focus_and_dismiss_foundations() {
    let mut dialog = Dialog::new(ScopeHandle::root("dialog").child("overlay"), true)
        .with_portal_host(PortalHost::named("layers"));
    let content_id = dialog.relationships().content_id().to_owned();
    let trigger_id = dialog.relationships().trigger_id().to_owned();
    let branch_id = format!("{content_id}-branch");
    let outside_id = String::from("outside");
    let stack = vec![String::from("background"), content_id.clone()];

    let lifecycle = dialog.lifecycle_mut();
    lifecycle.capture_restore_target(Some(trigger_id.clone()));
    lifecycle.set_autofocus_target(Some(branch_id.clone()));

    assert_eq!(lifecycle.portal_host(), &PortalHost::named("layers"));
    assert_eq!(lifecycle.presence().state(), PresenceState::Mounted);
    assert!(lifecycle.register_branch(branch_id.clone()));
    assert_eq!(lifecycle.focus_scope().root(), &content_id);
    assert_eq!(lifecycle.dismiss_layer().id(), &content_id);
    assert_eq!(
        lifecycle.focus_scope_mut().activate(),
        Some(branch_id.clone())
    );
    assert!(lifecycle.focus_scope_mut().focus(branch_id.clone()));
    assert_eq!(lifecycle.focus_scope_mut().deactivate(), Some(trigger_id));

    assert_eq!(
        lifecycle.presence_mut().sync(false),
        PresenceState::Suspended
    );
    assert!(lifecycle.presence().is_mounted());
    assert!(lifecycle.presence_mut().complete_unmount());

    assert_eq!(lifecycle.focus_guards_mut().retain(), 1);
    assert!(lifecycle.focus_guards().is_installed());
    assert!(lifecycle.dismiss_layer().blocks_outside_interaction());
    assert!(lifecycle.dismiss_layer().should_dismiss_escape(&stack));
    assert!(
        lifecycle
            .dismiss_layer()
            .should_dismiss_outside_pointer(Some(&outside_id), &stack)
    );
    assert!(
        !lifecycle
            .dismiss_layer()
            .should_dismiss_outside_focus(Some(&branch_id), &stack)
    );
}

#[test]
fn alert_dialog_extends_the_same_modal_lane_with_action_and_cancel_semantics() {
    let alert = AlertDialog::new(ScopeHandle::root("alert-dialog").child("root"), false);

    let part_names: Vec<_> = AlertDialog::parts()
        .iter()
        .map(AlertDialogPart::as_str)
        .collect();
    assert_eq!(
        part_names,
        vec![
            "root",
            "trigger",
            "portal",
            "overlay",
            "content",
            "title",
            "description",
            "close",
            "action",
            "cancel",
        ],
    );

    assert_eq!(alert.data_state(), DataState::Closed);
    assert_eq!(alert.content().role(), "alertdialog");
    assert_eq!(alert.content().id(), alert.relationships().content_id());
    assert_eq!(alert.action().id(), alert.action_id());
    assert_eq!(alert.cancel().id(), alert.cancel_id());
    assert_eq!(alert.action().data_state(), &DataState::Closed);
    assert_eq!(alert.cancel().data_state(), &DataState::Closed);
    assert_eq!(alert.close().close_request(), DialogStateRequest::Close);
    assert_eq!(alert.action().close_request(), DialogStateRequest::Close);
    assert_eq!(alert.cancel().close_request(), DialogStateRequest::Close);
    assert!(!alert.action().close_request().next_open());
    assert_eq!(
        alert.action().close_request().data_state(),
        alert.cancel().close_request().data_state(),
    );
}

#[test]
fn dialog_family_boundary_defers_product_helpers_by_omission() {
    let dialog_parts: Vec<_> = Dialog::parts().iter().map(DialogPart::as_str).collect();
    let alert_parts: Vec<_> = AlertDialog::parts()
        .iter()
        .map(AlertDialogPart::as_str)
        .collect();

    for omitted in [
        "header",
        "footer",
        "media",
        "sheet",
        "drawer",
        "command-dialog",
    ] {
        assert!(!dialog_parts.contains(&omitted));
        assert!(!alert_parts.contains(&omitted));
    }
}

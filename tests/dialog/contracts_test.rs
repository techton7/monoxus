use std::{cell::RefCell, rc::Rc};

use monoxus::{
    alert_dialog::AlertDialog,
    dialog::{
        compose_part_event_handlers, compose_part_refs, project_as_child, Dialog,
        DialogCloseFocusPolicy, DialogMode, DialogOpenFocusPolicy, DialogOutsideInteractionPolicy,
        DialogPart, DialogScrollLockPolicy, DialogStateRequest,
    },
    foundation::{
        compose::{EventHandlerOptions, RefHandler, Slottable},
        overlay::PortalHost,
        shared::ScopeHandle,
        state::{self, DataState},
    },
};

use super::fixtures::{is_default_prevented, TestEvent};

#[test]
fn dialog_family_publishes_stable_relationships_and_wrapper_safe_attributes() {
    let _ = state::use_controllable_state::<bool, fn(bool)>;
    let _ = state::use_controllable_state_reducer::<bool, bool, fn(&bool, bool) -> bool, fn(bool)>;

    let scope = ScopeHandle::root("dialog").child("root");
    let dialog = Dialog::new(scope.clone(), true);
    let relationships = dialog.relationships();
    let trigger_id = relationships.trigger_id().to_owned();

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
    assert_eq!(dialog.lifecycle().mode(), DialogMode::Modal);
    assert_eq!(
        dialog.lifecycle().open_focus_policy(),
        &DialogOpenFocusPolicy::FirstFocusable,
    );
    assert_eq!(
        dialog.lifecycle().close_focus_policy(),
        &DialogCloseFocusPolicy::Trigger,
    );
    assert_eq!(
        dialog.lifecycle().scroll_lock_policy(),
        &DialogScrollLockPolicy::enabled(),
    );
    assert_eq!(
        dialog.lifecycle().outside_interaction_policy(),
        &DialogOutsideInteractionPolicy::modal_default(),
    );
    assert_eq!(
        dialog.lifecycle().focus_scope().restore_target(),
        Some(&trigger_id),
    );
    assert!(dialog.lifecycle().focus_scope().autofocus_enabled());
    assert!(dialog.lifecycle().focus_scope().loops_focus());
    assert_eq!(dialog.lifecycle().focus_scope().autofocus_target(), None);

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
fn dialog_family_boundary_defers_product_helpers_by_omission() {
    let dialog_parts: Vec<_> = Dialog::parts().iter().map(DialogPart::as_str).collect();
    let alert_parts: Vec<_> = AlertDialog::parts()
        .iter()
        .map(monoxus::alert_dialog::AlertDialogPart::as_str)
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

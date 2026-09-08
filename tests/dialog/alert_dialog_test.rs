use monoxus::{
    alert_dialog::{AlertDialog, AlertDialogPart},
    dialog::{
        DialogCloseFocusPolicy, DialogMode, DialogOpenFocusPolicy, DialogOutsideInteractionPolicy,
        DialogScrollLockPolicy, DialogStateRequest,
    },
    foundation::{shared::ScopeHandle, state::DataState},
};

#[test]
fn alert_dialog_extends_the_same_modal_lane_with_alert_specific_restrictions() {
    let mut alert = AlertDialog::new(ScopeHandle::root("alert-dialog").child("root"), false);
    let outside_id = String::from("outside");
    let content_id = alert.relationships().content_id().to_owned();
    let action_id = alert.action_id().to_owned();
    let cancel_id = alert.cancel_id().to_owned();
    let stack = vec![String::from("background"), content_id.clone()];

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
    assert!(alert.content().aria_modal());
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
    assert_eq!(alert.lifecycle().mode(), DialogMode::Modal);
    assert_eq!(
        alert.lifecycle().open_focus_policy(),
        &DialogOpenFocusPolicy::Target(cancel_id.clone()),
    );
    assert_eq!(
        alert.lifecycle().close_focus_policy(),
        &DialogCloseFocusPolicy::Trigger,
    );
    assert_eq!(
        alert.lifecycle().scroll_lock_policy(),
        &DialogScrollLockPolicy::enabled(),
    );
    assert!(alert.lifecycle().focus_scope().loops_focus());
    assert_eq!(
        alert.lifecycle().outside_interaction_policy(),
        &DialogOutsideInteractionPolicy::alert_default(),
    );

    let lifecycle = alert.lifecycle_mut();
    assert_eq!(
        lifecycle.focus_scope().branches(),
        &[action_id, cancel_id.clone()]
    );
    assert_eq!(lifecycle.focus_scope_mut().activate(), Some(cancel_id));
    assert!(lifecycle.dismiss_layer().should_dismiss_escape(&stack));
    assert!(
        !lifecycle
            .dismiss_layer()
            .should_dismiss_outside_pointer(Some(&outside_id), &stack)
    );
    assert!(
        !lifecycle
            .dismiss_layer()
            .should_dismiss_outside_focus(Some(&outside_id), &stack)
    );
}

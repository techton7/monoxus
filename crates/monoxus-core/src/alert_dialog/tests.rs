use crate::dialog::{DialogOpenFocusPolicy, DialogOutsideInteractionPolicy};
use crate::foundation::shared::ScopeHandle;

use super::*;

#[test]
fn alert_dialog_has_exact_ten_parts_and_alert_role() {
    assert_eq!(
        AlertDialog::parts()
            .iter()
            .map(AlertDialogPart::as_str)
            .collect::<Vec<_>>(),
        [
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
        ]
    );
    let alert = AlertDialog::new(ScopeHandle::root("alert-dialog").child("main"), true);
    assert_eq!(alert.content().role(), "alertdialog");
    assert!(alert.content().aria_modal());
}

#[test]
fn alert_dialog_targets_cancel_and_suppresses_outside_dismiss() {
    let scope = ScopeHandle::root("alert-dialog").child("destructive");
    let alert = AlertDialog::new(scope.clone(), true);
    let cancel_id = scope.qualify("cancel");
    assert_eq!(alert.cancel_id(), cancel_id);
    assert_eq!(
        alert.lifecycle().open_focus_policy(),
        &DialogOpenFocusPolicy::Target(cancel_id)
    );
    assert_eq!(
        alert.lifecycle().outside_interaction_policy(),
        &DialogOutsideInteractionPolicy::alert_default()
    );
    assert!(
        !alert
            .lifecycle()
            .outside_interaction_policy()
            .pointer_down_outside()
            .dismisses()
    );
    assert!(
        !alert
            .lifecycle()
            .dismiss_layer()
            .should_dismiss_outside_pointer(None, &[alert.relationships().content_id().to_owned()])
    );
    assert!(!alert.action().close_request().next_open());
    assert!(!alert.cancel().close_request().next_open());
    assert_eq!(alert.action().data_disabled(), None);
    assert!(!alert.cancel().is_disabled());
}

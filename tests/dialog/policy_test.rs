use monoxus::{
    dialog::{
        Dialog, DialogCloseFocusPolicy, DialogMode, DialogOpenFocusPolicy,
        DialogOutsideDismissBehavior, DialogOutsideInteractionPolicy, DialogScrollLockPolicy,
    },
    foundation::shared::ScopeHandle,
};

#[test]
fn dialog_family_exposes_explicit_non_modal_and_focus_policy_overrides() {
    let scope = ScopeHandle::root("dialog").child("non-modal");
    let mut dialog = Dialog::new_non_modal(scope.clone(), true)
        .with_open_focus_policy(DialogOpenFocusPolicy::Target(scope.qualify("branch")))
        .with_close_focus_policy(DialogCloseFocusPolicy::Target(scope.qualify("restore")))
        .with_scroll_lock_policy(DialogScrollLockPolicy::enabled().with_restore_delay(Some(24)))
        .with_outside_interaction_policy(DialogOutsideInteractionPolicy::new(
            DialogOutsideDismissBehavior::Ignore,
            DialogOutsideDismissBehavior::Dismiss,
        ));
    let content_id = dialog.relationships().content_id().to_owned();
    let branch_id = scope.qualify("branch");
    let restore_id = scope.qualify("restore");
    let outside_id = String::from("outside");
    let stack = vec![String::from("background"), content_id.clone()];

    assert_eq!(dialog.lifecycle().mode(), DialogMode::NonModal);
    assert_eq!(
        dialog.lifecycle().open_focus_policy(),
        &DialogOpenFocusPolicy::Target(branch_id.clone()),
    );
    assert_eq!(
        dialog.lifecycle().close_focus_policy(),
        &DialogCloseFocusPolicy::Target(restore_id.clone()),
    );
    assert_eq!(
        dialog.lifecycle().scroll_lock_policy(),
        &DialogScrollLockPolicy::enabled().with_restore_delay(Some(24)),
    );
    assert_eq!(
        dialog.lifecycle().outside_interaction_policy(),
        &DialogOutsideInteractionPolicy::new(
            DialogOutsideDismissBehavior::Ignore,
            DialogOutsideDismissBehavior::Dismiss,
        ),
    );
    assert!(!dialog.content().aria_modal());

    let lifecycle = dialog.lifecycle_mut();
    assert!(lifecycle.register_branch(branch_id.clone()));
    assert!(!lifecycle.focus_scope().traps_focus());
    assert!(lifecycle.focus_scope().loops_focus());
    assert!(!lifecycle.dismiss_layer().blocks_outside_interaction());
    assert_eq!(
        lifecycle.focus_scope_mut().activate(),
        Some(branch_id.clone())
    );
    assert_eq!(lifecycle.focus_scope_mut().deactivate(), Some(restore_id));
    assert!(
        !lifecycle
            .dismiss_layer()
            .should_dismiss_outside_pointer(Some(&outside_id), &stack)
    );
    assert!(
        lifecycle
            .dismiss_layer()
            .should_dismiss_outside_focus(Some(&outside_id), &stack)
    );
}

#[test]
fn dialog_family_supports_open_focus_suppression_without_manual_close_focus_wiring() {
    let mut dialog = Dialog::new(ScopeHandle::root("dialog").child("suppressed-focus"), true)
        .with_open_focus_policy(DialogOpenFocusPolicy::Suppress);
    let trigger_id = dialog.relationships().trigger_id().to_owned();

    assert_eq!(
        dialog.lifecycle().open_focus_policy(),
        &DialogOpenFocusPolicy::Suppress,
    );
    assert!(!dialog.lifecycle().focus_scope().autofocus_enabled());
    assert_eq!(
        dialog.lifecycle().focus_scope().restore_target(),
        Some(&trigger_id)
    );

    let lifecycle = dialog.lifecycle_mut();
    assert_eq!(lifecycle.focus_scope_mut().activate(), None);
    assert_eq!(lifecycle.focus_scope_mut().deactivate(), Some(trigger_id));
}

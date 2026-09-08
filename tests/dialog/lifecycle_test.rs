use monoxus::{
    dialog::Dialog,
    foundation::{
        overlay::{PortalHost, PresenceState},
        shared::ScopeHandle,
    },
};

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

use crate::foundation::{overlay::PortalHost, shared::ScopeHandle};

use super::*;

#[test]
fn part_inventory_is_exact_and_relationships_are_deterministic() {
    assert_eq!(
        Dialog::parts()
            .iter()
            .map(DialogPart::as_str)
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
        ]
    );

    let scope = ScopeHandle::root("dialog").child("main");
    let rel = DialogRelationships::new(scope.clone());
    assert_eq!(rel.root_id(), scope.token());
    assert_eq!(rel.trigger_id(), scope.qualify("trigger"));
    assert_eq!(rel.content_id(), scope.qualify("content"));
    assert_eq!(rel.title_id(), scope.qualify("title"));
    assert_eq!(rel.description_id(), scope.qualify("description"));
    assert_eq!(
        rel.focus_guard_before_id(),
        scope.qualify("focus-guard-before")
    );
    assert_eq!(
        rel.focus_guard_after_id(),
        scope.qualify("focus-guard-after")
    );
}

#[test]
fn attributes_and_modal_policies_publish_closed_and_open_contracts() {
    let scope = ScopeHandle::root("dialog").child("policies");
    let closed = Dialog::new(scope.clone(), false);
    assert_eq!(closed.trigger().aria_controls(), None);
    assert_eq!(closed.trigger().data_disabled(), None);
    assert!(!closed.trigger().aria_expanded());
    assert!(closed.lifecycle().mode().is_modal());
    assert!(
        closed
            .lifecycle()
            .dismiss_layer()
            .should_dismiss_escape(&[closed.relationships().content_id().to_owned()])
    );
    assert!(
        closed
            .lifecycle()
            .outside_interaction_policy()
            .pointer_down_outside()
            .dismisses()
    );
    assert!(
        !closed
            .lifecycle()
            .outside_interaction_policy()
            .focus_outside()
            .dismisses()
    );

    let open = Dialog::new_non_modal(scope, true);
    assert_eq!(
        open.trigger().aria_controls(),
        Some(open.relationships().content_id())
    );
    assert!(!open.content().aria_modal());
    assert!(
        open.lifecycle()
            .outside_interaction_policy()
            .focus_outside()
            .dismisses()
    );
}

#[test]
fn focus_policies_and_trigger_overrides_work_via_public_api() {
    let scope = ScopeHandle::root("dialog").child("lifecycle");
    let dialog = Dialog::new(scope, true)
        .with_portal_host(PortalHost::named("layers"))
        .with_open_focus_policy(DialogOpenFocusPolicy::Target("cancel".into()))
        .with_close_focus_policy(DialogCloseFocusPolicy::Target("trigger".into()));

    assert_eq!(
        dialog.lifecycle().open_focus_policy(),
        &DialogOpenFocusPolicy::Target("cancel".into())
    );
    assert_eq!(
        dialog.lifecycle().focus_scope().restore_target(),
        Some(&"trigger".to_string())
    );
    assert_eq!(dialog.portal().host(), &PortalHost::named("layers"));

    let trigger = dialog
        .trigger()
        .with_target_content_id("custom-content")
        .with_aria_controls(Some("custom-content".to_string()));
    assert_eq!(trigger.target_content_id(), "custom-content");
    assert_eq!(trigger.aria_controls(), Some("custom-content"));
}

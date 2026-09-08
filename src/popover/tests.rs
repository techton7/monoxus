use crate::foundation::shared::ScopeHandle;

use super::{
    runtime::{popover_document_path_is_inside, popover_document_path_is_outside},
    state::Popover,
};

#[test]
fn popover_document_path_detection_treats_runtime_surfaces_as_inside() {
    let scope = ScopeHandle::root("playground").child("popover-test");
    let mut popover = Popover::new(scope.clone(), true).with_modal(true);
    let branch_id = scope.qualify("branch");
    let focus_before = popover.lifecycle().focus_guards().before().clone();

    assert!(popover.lifecycle_mut().register_branch(branch_id.clone()));

    assert!(popover_document_path_is_inside(
        &popover,
        &[branch_id.clone()]
    ));
    assert!(popover_document_path_is_inside(
        &popover,
        &[popover.relationships().trigger_id().to_owned()]
    ));
    assert!(popover_document_path_is_inside(
        &popover,
        &[popover.relationships().anchor_id().to_owned()]
    ));
    assert!(popover_document_path_is_inside(
        &popover,
        &[popover.relationships().content_id().to_owned()]
    ));
    assert!(popover_document_path_is_inside(&popover, &[focus_before]));
    assert!(popover_document_path_is_outside(
        &popover,
        &[String::from("outside-target")]
    ));
}

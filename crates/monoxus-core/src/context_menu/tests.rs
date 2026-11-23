use crate::foundation::{shared::ScopeHandle, state::DataState};

use super::*;

#[test]
fn context_menu_defaults_to_modal_and_reanchors_virtual_point_while_open() {
    let rel = ContextMenuRelationships::new(ScopeHandle::root("context-menu").child("cm-1"));
    let mut state = ContextMenuState::new(rel, false);
    assert!(state.is_modal());
    assert!(state.content().loop_focus());
    assert_eq!(state.anchor_point(), None);

    let first = VirtualAnchorPoint::new(120.0, 240.0);
    assert_eq!(
        state.open_at_point(first, false, false),
        ContextMenuTriggerAction::OpenAtPoint { point: first }
    );
    assert!(state.is_open());
    assert_eq!(state.anchor_point(), Some(first));

    // Subsequent right-click while already open updates anchor coordinates directly (`CM-3`)
    let second = VirtualAnchorPoint::new(300.0, 410.0);
    assert_eq!(
        state.open_at_point(second, false, false),
        ContextMenuTriggerAction::OpenAtPoint { point: second }
    );
    assert!(state.is_open());
    assert_eq!(state.anchor_point(), Some(second));
}

#[test]
fn context_menu_trigger_attrs_and_nested_claim_and_long_press_rules() {
    let rel = ContextMenuRelationships::new(ScopeHandle::root("context-menu").child("cm-2"));
    let mut state = ContextMenuState::new(rel, false);

    let attrs = state.trigger_attrs(false);
    assert_eq!(attrs.style, "-webkit-touch-callout: none;");
    assert_eq!(attrs.data_state, DataState::Closed);
    assert_eq!(attrs.data_disabled, None);
    assert_eq!(attrs.part_attr, "data-monoxus-context-menu-trigger");

    // Non-mouse pointer starts 700ms long-press unless disabled; mouse pointer does not (`CM-5`)
    assert!(state.should_start_long_press("touch", false));
    assert!(state.should_start_long_press("pen", false));
    assert!(!state.should_start_long_press("mouse", false));
    assert!(!state.should_start_long_press("touch", true));

    // Innermost descendant claim blocks outer ancestor trigger from opening (`CM-7`)
    assert_eq!(
        state.open_at_point(VirtualAnchorPoint::new(10.0, 20.0), false, true),
        ContextMenuTriggerAction::Ignore
    );
    assert!(!state.is_open());
}

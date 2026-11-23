use crate::foundation::{shared::ScopeHandle, state::DataState};
use crate::menu::MenuKey;

use super::*;

#[test]
fn dropdown_menu_defaults_to_modal_and_wires_trigger_content_aria() {
    let rel = DropdownMenuRelationships::new(ScopeHandle::root("dropdown-menu").child("dm-1"));
    let closed_state = DropdownMenuState::new(rel.clone(), false);
    assert!(closed_state.is_modal());
    assert!(closed_state.content().loop_focus());
    assert_eq!(closed_state.data_state(), DataState::Closed);

    let trigger_closed = closed_state.trigger_attrs(false);
    assert_eq!(trigger_closed.aria_haspopup, "menu");
    assert!(!trigger_closed.aria_expanded);
    assert_eq!(trigger_closed.aria_controls, None);
    assert_eq!(trigger_closed.part_attr, "data-monoxus-dropdown-menu-trigger");

    let open_state = DropdownMenuState::new(rel.clone(), true);
    let trigger_open = open_state.trigger_attrs(false);
    assert!(trigger_open.aria_expanded);
    assert_eq!(
        trigger_open.aria_controls.as_deref(),
        Some(rel.content_id())
    );

    let content = open_state.content_attrs();
    assert_eq!(content.role, "menu");
    assert_eq!(content.aria_labelledby.as_deref(), Some(rel.trigger_id()));
    assert_eq!(content.part_attr, "data-monoxus-dropdown-menu-content");
}

#[test]
fn dropdown_menu_trigger_keyboard_and_disabled_policies() {
    let rel = DropdownMenuRelationships::new(ScopeHandle::root("dropdown-menu").child("dm-2"));
    let state = DropdownMenuState::new(rel, false);

    assert_eq!(
        state.resolve_trigger_key(MenuKey::Enter, false),
        DropdownMenuTriggerAction::OpenAndFocusFirst
    );
    assert_eq!(
        state.resolve_trigger_key(MenuKey::Space, false),
        DropdownMenuTriggerAction::OpenAndFocusFirst
    );
    assert_eq!(
        state.resolve_trigger_key(MenuKey::ArrowDown, false),
        DropdownMenuTriggerAction::OpenAndFocusFirst
    );
    assert_eq!(
        state.resolve_trigger_key(MenuKey::ArrowUp, false),
        DropdownMenuTriggerAction::Ignore
    );

    // Disabled trigger ignores both pointer and keyboard triggers
    assert_eq!(
        state.resolve_trigger_pointer_down(true),
        DropdownMenuTriggerAction::Ignore
    );
    assert_eq!(
        state.resolve_trigger_key(MenuKey::Enter, true),
        DropdownMenuTriggerAction::Ignore
    );
}

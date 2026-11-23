use crate::foundation::{
    shared::{Direction, ScopeHandle},
    state::DataState,
};
use crate::menu::MenuKey;

use super::*;

fn sample_menubar() -> MenubarState {
    let rel = MenubarRelationships::new(ScopeHandle::root("menubar").child("mb-1"));
    let mut state = MenubarState::new(rel);
    state.register_menu(MenubarMenuEntry::new("file", "t-file", "c-file", false, 0));
    state.register_menu(MenubarMenuEntry::new("edit", "t-edit", "c-edit", true, 1));
    state.register_menu(MenubarMenuEntry::new("view", "t-view", "c-view", false, 2));
    state.register_menu(MenubarMenuEntry::new("help", "t-help", "c-help", false, 3));
    state
}

#[test]
fn menubar_root_and_trigger_attributes_follow_apg_contract() {
    let mut state = sample_menubar();
    let root = state.root_attrs();
    assert_eq!(root.role, "menubar");
    assert_eq!(root.aria_orientation, "horizontal");
    assert_eq!(root.part_attr, "data-monoxus-menubar-root");

    let t_file = state.trigger_attrs("file", "t-file".into(), "c-file", false);
    assert_eq!(t_file.role, "menuitem");
    assert_eq!(t_file.tabindex, 0);
    assert!(!t_file.aria_expanded);
    assert_eq!(t_file.aria_controls, None);

    let t_view = state.trigger_attrs("view", "t-view".into(), "c-view", false);
    assert_eq!(t_view.tabindex, -1);

    let (prev, skip_exit) = state.open_menu("view");
    assert_eq!(prev, None);
    assert!(!skip_exit);
    let t_view_open = state.trigger_attrs("view", "t-view".into(), "c-view", false);
    assert!(t_view_open.aria_expanded);
    assert_eq!(t_view_open.aria_controls.as_deref(), Some("c-view"));
    assert_eq!(t_view_open.data_state, DataState::Open);
    assert_eq!(t_view_open.data_highlighted, Some(""));
}

#[test]
fn menubar_hover_swap_fast_animation_skip_and_trigger_arrows() {
    let mut state = sample_menubar();

    // Hovering trigger while closed does NOT open any menu (`MB-3`)
    assert!(state.on_trigger_pointer_enter("view").is_none());

    // ArrowRight while closed moves roving focus without opening (`MB-2`), skipping disabled "edit"
    assert_eq!(
        state.on_trigger_key("file", MenuKey::ArrowRight),
        MenubarNavigationStep::FocusTrigger {
            value: "view".into(),
            trigger_id: "t-view".into()
        }
    );

    // Open "file", then hover "view" -> swaps and marks skip_exit_animation (`MB-5`)
    state.open_menu("file");
    let hovered = state.on_trigger_pointer_enter("view").unwrap();
    assert_eq!(hovered.value, "view");
    let (prev, skip_exit) = state.open_menu("view");
    assert_eq!(prev.as_deref(), Some("file"));
    assert!(skip_exit);

    // ArrowLeft while open wraps backward from "file" -> "help" and opens it (`MB-2`, `3B D-5`)
    assert_eq!(
        state.on_trigger_key("file", MenuKey::ArrowLeft),
        MenubarNavigationStep::FocusAndOpenMenu {
            value: "help".into(),
            trigger_id: "t-help".into()
        }
    );
}

#[test]
fn menubar_cross_menu_arrow_rules_and_rtl_support() {
    let mut state = sample_menubar();
    state.open_menu("file");

    // ArrowRight inside open content on regular item -> next menu ("view") (`MB-6`)
    assert_eq!(
        state.resolve_cross_menu_arrow("file", MenuKey::ArrowRight, false, false),
        MenubarCrossMenuResult::SwitchToMenu {
            value: "view".into(),
            trigger_id: "t-view".into()
        }
    );

    // ArrowRight on a SubTrigger opens the submenu instead of switching menubar menus
    assert_eq!(
        state.resolve_cross_menu_arrow("file", MenuKey::ArrowRight, true, false),
        MenubarCrossMenuResult::None
    );

    // ArrowRight inside a deep submenu on a leaf item switches to next menubar menu (`MB-6`)
    assert_eq!(
        state.resolve_cross_menu_arrow("file", MenuKey::ArrowRight, false, true),
        MenubarCrossMenuResult::SwitchToMenu {
            value: "view".into(),
            trigger_id: "t-view".into()
        }
    );

    // ArrowLeft inside a deep submenu closes ONLY the submenu, NOT switching menubar menus (`MB-6`)
    assert_eq!(
        state.resolve_cross_menu_arrow("file", MenuKey::ArrowLeft, false, true),
        MenubarCrossMenuResult::None
    );

    // ArrowLeft at top-level switches to prev menubar menu ("help" via loop)
    assert_eq!(
        state.resolve_cross_menu_arrow("file", MenuKey::ArrowLeft, false, false),
        MenubarCrossMenuResult::SwitchToMenu {
            value: "help".into(),
            trigger_id: "t-help".into()
        }
    );

    // RTL flips ArrowLeft -> forward ("view")
    let rtl_state = state.with_direction(Direction::Rtl);
    assert_eq!(
        rtl_state.resolve_cross_menu_arrow("file", MenuKey::ArrowLeft, false, false),
        MenubarCrossMenuResult::SwitchToMenu {
            value: "view".into(),
            trigger_id: "t-view".into()
        }
    );
}

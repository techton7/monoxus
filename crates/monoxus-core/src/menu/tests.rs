use crate::foundation::{
    overlay::{FloatingLayer, PlacementSide, Point2D, Rect, Rect2D, Size},
    shared::{Direction, ScopeHandle},
    state::DataState,
};

use super::*;

fn sample_items() -> Vec<MenuItemEntry> {
    vec![
        MenuItemEntry::new("i-1", "Cut", false, MenuItemKind::Item, 0),
        MenuItemEntry::new("i-2", "Copy", true, MenuItemKind::Item, 1),
        MenuItemEntry::new("i-3", "Paste", false, MenuItemKind::Item, 2),
        MenuItemEntry::new("i-4", "Print", false, MenuItemKind::Item, 3),
        MenuItemEntry::new("i-5", "More Tools", false, MenuItemKind::SubTrigger, 4),
    ]
}

#[test]
fn menu_relationships_and_group_ids_are_deterministic() {
    let scope = ScopeHandle::root("menu").child("root-1");
    let rel = MenuRelationships::new(scope.clone());
    assert_eq!(rel.trigger_id(), scope.qualify("trigger"));
    assert_eq!(rel.content_id(), scope.qualify("content"));
    assert_eq!(rel.wrapper_id(), scope.qualify("wrapper"));

    let group_scope = scope.child("g1");
    let group = rel.group(group_scope.clone());
    assert_eq!(group.group_id(), group_scope.qualify("group"));
    assert_eq!(group.heading_id(), group_scope.qualify("group-heading"));

    let sub_scope = scope.child("sub1");
    let sub = rel.sub(sub_scope.clone());
    assert_eq!(sub.trigger_id(), sub_scope.qualify("sub-trigger"));
    assert_eq!(sub.content_id(), sub_scope.qualify("sub-content"));
}

#[test]
fn menu_navigation_skips_disabled_and_loops_by_default() {
    let items = sample_items();

    // First ArrowDown highlights first enabled item ("i-1")
    let step = resolve_menu_navigation(&items, None, MenuKey::ArrowDown, Direction::Ltr, true, true, false, false);
    assert_eq!(
        step,
        MenuNavigationStep::HighlightItem {
            id: "i-1".to_owned()
        }
    );

    // ArrowDown from "i-1" skips disabled "i-2" and moves to "i-3"
    let step = resolve_menu_navigation(
        &items,
        Some("i-1"),
        MenuKey::ArrowDown,
        Direction::Ltr,
        true,
        true,
        false,
        false,
    );
    assert_eq!(
        step,
        MenuNavigationStep::HighlightItem {
            id: "i-3".to_owned()
        }
    );

    // ArrowUp from first enabled wraps to last enabled when loop_focus = true
    let step = resolve_menu_navigation(
        &items,
        Some("i-1"),
        MenuKey::ArrowUp,
        Direction::Ltr,
        true,
        true,
        false,
        false,
    );
    assert_eq!(
        step,
        MenuNavigationStep::HighlightItem {
            id: "i-5".to_owned()
        }
    );

    // loop_focus = false stops at boundaries
    let step = resolve_menu_navigation(
        &items,
        Some("i-5"),
        MenuKey::ArrowDown,
        Direction::Ltr,
        false,
        true,
        false,
        false,
    );
    assert_eq!(step, MenuNavigationStep::None);
}

#[test]
fn menu_rtl_and_ltr_submenu_arrow_directions_and_modal_tab_behavior() {
    let items = sample_items();

    assert_eq!(
        resolve_menu_navigation(
            &items,
            Some("i-5"),
            MenuKey::ArrowRight,
            Direction::Ltr,
            true,
            true,
            false,
            false
        ),
        MenuNavigationStep::OpenSubmenuAndFocusFirst {
            sub_trigger_id: "i-5".to_owned()
        }
    );

    assert_eq!(
        resolve_menu_navigation(
            &items,
            Some("i-5"),
            MenuKey::ArrowLeft,
            Direction::Rtl,
            true,
            true,
            false,
            false
        ),
        MenuNavigationStep::OpenSubmenuAndFocusFirst {
            sub_trigger_id: "i-5".to_owned()
        }
    );

    assert_eq!(
        resolve_menu_navigation(
            &items,
            Some("i-1"),
            MenuKey::ArrowLeft,
            Direction::Ltr,
            true,
            true,
            true,
            false
        ),
        MenuNavigationStep::CloseSubmenuAndFocusTrigger
    );

    assert_eq!(
        resolve_menu_navigation(
            &items,
            Some("i-1"),
            MenuKey::ArrowRight,
            Direction::Rtl,
            true,
            true,
            true,
            false
        ),
        MenuNavigationStep::CloseSubmenuAndFocusTrigger
    );

    assert_eq!(
        resolve_menu_navigation(&items, None, MenuKey::Tab, Direction::Ltr, true, true, false, false),
        MenuNavigationStep::TabTrap
    );

    assert_eq!(
        resolve_menu_navigation(
            &items,
            None,
            MenuKey::Tab,
            Direction::Ltr,
            true,
            false,
            false,
            true
        ),
        MenuNavigationStep::TabOut { forward: false }
    );
}

#[test]
fn typeahead_buffer_handles_single_char_cycling_multi_char_and_space_guard() {
    let items = sample_items();
    let mut buffer = TypeaheadBuffer::default();

    // Empty buffer ignores space so space activates items instead
    assert_eq!(buffer.push_char(' ', 100, &items, Some("i-1")), None);
    assert!(!buffer.is_active(100));

    // Pressing 'p' highlights Paste ("i-3"), pressing 'p' again cycles to Print ("i-4")
    assert_eq!(
        buffer.push_char('p', 200, &items, None),
        Some("i-3".to_owned())
    );
    assert!(buffer.is_active(250));
    assert_eq!(
        buffer.push_char('p', 300, &items, Some("i-3")),
        Some("i-4".to_owned())
    );
    // Wraps back to "i-3"
    assert_eq!(
        buffer.push_char('p', 400, &items, Some("i-4")),
        Some("i-3".to_owned())
    );

    // Timeout resets buffer at 1000ms and lets multi-char query including space match "More Tools"
    assert!(buffer.expire_if_elapsed(1400));
    assert_eq!(
        buffer.push_char('m', 1500, &items, Some("i-3")),
        Some("i-5".to_owned())
    );
    assert_eq!(
        buffer.push_char('o', 1600, &items, Some("i-5")),
        Some("i-5".to_owned())
    );
    assert_eq!(
        buffer.push_char('r', 1700, &items, Some("i-5")),
        Some("i-5".to_owned())
    );
    assert_eq!(
        buffer.push_char('e', 1800, &items, Some("i-5")),
        Some("i-5".to_owned())
    );
    assert_eq!(
        buffer.push_char(' ', 1850, &items, Some("i-5")),
        Some("i-5".to_owned())
    );
    assert_eq!(
        buffer.push_char('t', 1900, &items, Some("i-5")),
        Some("i-5".to_owned())
    );
}

#[test]
fn submenu_intent_corridor_supports_bidirectional_travel_and_stall_timeout() {
    let mut intent = SubmenuIntentMachine::default();
    let trigger_rect = Rect2D::new(0.0, 40.0, 120.0, 32.0);
    let content_rect = Rect2D::new(124.0, 40.0, 140.0, 160.0);

    intent.begin_trigger_to_sub(
        Point2D::new(118.0, 56.0),
        trigger_rect,
        content_rect,
        PlacementSide::Right,
        1000,
    );
    assert_eq!(intent.direction(), SubmenuCorridorDirection::TriggerToContent);

    // Diagonal point moving toward subcontent stays inside corridor
    assert_eq!(
        intent.evaluate_pointer_move(
            Point2D::new(122.0, 72.0),
            1020,
            false,
            false,
            false
        ),
        SubmenuPointerEvaluation::InCorridor
    );

    // Stalling beyond 100ms expires corridor
    assert_eq!(
        intent.check_timeout(1130),
        SubmenuPointerEvaluation::Stalled
    );
    assert!(!intent.is_active());

    // Reverse corridor: moving back from subcontent toward trigger
    intent.begin_content_to_trigger(
        Point2D::new(126.0, 60.0),
        content_rect,
        trigger_rect,
        PlacementSide::Right,
        2000,
    );
    assert_eq!(intent.direction(), SubmenuCorridorDirection::ContentToTrigger);
    assert_eq!(
        intent.evaluate_pointer_move(
            Point2D::new(100.0, 56.0),
            2020,
            false,
            true,
            false
        ),
        SubmenuPointerEvaluation::ReachedTarget
    );
}

#[test]
fn checkbox_group_and_attrs_and_css_vars_follow_contract() {
    let selected = vec!["bold".to_owned()];
    let toggled = toggle_checkbox_group_value(&selected, "italic");
    assert_eq!(toggled, vec!["bold".to_owned(), "italic".to_owned()]);
    assert_eq!(
        checkbox_group_item_state(&toggled, "italic"),
        CheckedState::Checked
    );
    assert_eq!(
        toggle_checkbox_group_value(&toggled, "bold"),
        vec!["italic".to_owned()]
    );

    let indeterminate_attrs = MenuCheckboxItemAttributes::from_parts(
        "cb-1".to_owned(),
        CheckedState::Indeterminate,
        false,
        true,
        MenuVariant::DropdownMenu,
    );
    assert_eq!(indeterminate_attrs.aria_checked, "mixed");
    assert_eq!(indeterminate_attrs.data_state, DataState::Indeterminate);
    assert_eq!(indeterminate_attrs.data_highlighted, Some(""));
    assert_eq!(indeterminate_attrs.tabindex, 0);
    assert_eq!(
        indeterminate_attrs.part_attr,
        "data-monoxus-dropdown-menu-checkbox-item"
    );

    let disabled_attrs = MenuItemAttributes::from_parts(
        "item-disabled".to_owned(),
        true,
        true,
        MenuVariant::ContextMenu,
    );
    assert_eq!(disabled_attrs.aria_disabled, Some("true"));
    assert_eq!(disabled_attrs.data_disabled, Some(""));
    assert_eq!(disabled_attrs.data_highlighted, None);
    assert_eq!(disabled_attrs.tabindex, -1);

    let layer = FloatingLayer::new(PlacementSide::Bottom).with_namespace("dropdown-menu");
    let geometry = layer.geometry_vars(Rect::new(10.0, 20.0, 80.0, 30.0), Size::new(160.0, 120.0));
    let vars = menu_css_vars(&geometry, MenuVariant::DropdownMenu);
    assert!(vars.iter().any(|(k, _)| k == "--monoxus-dropdown-menu-content-transform-origin"));
    assert!(vars.iter().any(|(k, _)| k == "--monoxus-dropdown-menu-content-available-width"));
    assert!(vars.iter().any(|(k, _)| k == "--monoxus-dropdown-menu-anchor-width"));
    assert!(vars.iter().any(|(k, _)| k == "--radix-dropdown-menu-content-transform-origin"));
    assert!(vars.iter().any(|(k, _)| k == "--radix-dropdown-menu-trigger-width"));
}

#[test]
fn pointer_scroll_guard_cancels_movement_beyond_threshold() {
    let snap = PressedItemSnapshot::new("item-1", 100.0, 100.0);
    assert!(!snap.moved_beyond(103.5, 101.0, MENU_POINTER_SCROLL_GUARD_PX));
    assert!(snap.moved_beyond(104.0, 100.0, MENU_POINTER_SCROLL_GUARD_PX));
    assert!(snap.moved_beyond(100.0, 95.0, MENU_POINTER_SCROLL_GUARD_PX));
}

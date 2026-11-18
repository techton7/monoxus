use std::cell::RefCell;
use std::rc::Rc;

use dioxus::prelude::*;
use monoxus::{
    foundation::shared::ScopeHandle,
    toggle_group::{
        ToggleGroup, ToggleGroupItem, ToggleGroupMode, ToggleGroupRoot,
    },
};

#[test]
fn test_single_selection_and_deselection_reducer() {
    let scope = ScopeHandle::root("tg-selection-single");
    let mut group = ToggleGroup::new(scope, ToggleGroupMode::Single);

    assert_eq!(group.single_value(), "");
    assert!(!group.is_item_pressed("a"));

    // Select "a"
    let val = group.toggle_single("a");
    assert_eq!(val, "a");
    assert_eq!(group.single_value(), "a");
    assert!(group.is_item_pressed("a"));
    assert_eq!(group.current_tab_stop(), Some("a"));

    // Select "b" (switches to "b")
    let val2 = group.toggle_single("b");
    assert_eq!(val2, "b");
    assert_eq!(group.single_value(), "b");
    assert!(!group.is_item_pressed("a"));
    assert!(group.is_item_pressed("b"));

    // Re-click "b" (TG-2: deselects to "")
    let val3 = group.toggle_single("b");
    assert_eq!(val3, "");
    assert_eq!(group.single_value(), "");
    assert!(!group.is_item_pressed("b"));
}

#[test]
fn test_multiple_selection_reducer() {
    let scope = ScopeHandle::root("tg-selection-multiple");
    let mut group = ToggleGroup::new(scope, ToggleGroupMode::Multiple);

    assert!(group.multiple_values().is_empty());

    // Toggle "a"
    let list1 = group.toggle_multiple("a");
    assert_eq!(list1, vec!["a".to_string()]);
    assert!(group.is_item_pressed("a"));

    // Toggle "b"
    let list2 = group.toggle_multiple("b");
    assert_eq!(list2, vec!["a".to_string(), "b".to_string()]);
    assert!(group.is_item_pressed("a"));
    assert!(group.is_item_pressed("b"));

    // Re-toggle "a" (removes "a")
    let list3 = group.toggle_multiple("a");
    assert_eq!(list3, vec!["b".to_string()]);
    assert!(!group.is_item_pressed("a"));
    assert!(group.is_item_pressed("b"));

    // Re-toggle "b" (removes "b")
    let list4 = group.toggle_multiple("b");
    assert!(list4.is_empty());
    assert!(!group.is_item_pressed("b"));
}

#[component]
fn ControlledSingleToggleGroup(
    value: Rc<RefCell<String>>,
    on_change_calls: Rc<RefCell<Vec<String>>>,
) -> Element {
    let current_val = value.borrow().clone();
    let on_change = on_change_calls.clone();

    rsx! {
        ToggleGroupRoot {
            r#type: ToggleGroupMode::Single,
            value: current_val,
            on_value_change: move |next: String| {
                on_change.borrow_mut().push(next);
            },
            ToggleGroupItem { value: "1".to_string(), "One" }
            ToggleGroupItem { value: "2".to_string(), "Two" }
        }
    }
}

#[test]
fn test_controlled_single_toggle_group_vdom() {
    let value = Rc::new(RefCell::new("1".to_string()));
    let calls = Rc::new(RefCell::new(Vec::new()));

    let mut dom = VirtualDom::new_with_props(
        ControlledSingleToggleGroup,
        ControlledSingleToggleGroupProps {
            value: value.clone(),
            on_change_calls: calls.clone(),
        },
    );
    dom.rebuild_in_place();

    assert_eq!(calls.borrow().len(), 0);
}

#[component]
fn ControlledMultipleToggleGroup(
    values: Rc<RefCell<Vec<String>>>,
    on_change_calls: Rc<RefCell<Vec<Vec<String>>>>,
) -> Element {
    let current_vals = values.borrow().clone();
    let on_change = on_change_calls.clone();

    rsx! {
        ToggleGroupRoot {
            r#type: ToggleGroupMode::Multiple,
            values: current_vals,
            on_values_change: move |next: Vec<String>| {
                on_change.borrow_mut().push(next);
            },
            ToggleGroupItem { value: "1".to_string(), "One" }
            ToggleGroupItem { value: "2".to_string(), "Two" }
        }
    }
}

#[test]
fn test_controlled_multiple_toggle_group_vdom() {
    let values = Rc::new(RefCell::new(vec!["1".to_string()]));
    let calls = Rc::new(RefCell::new(Vec::new()));

    let mut dom = VirtualDom::new_with_props(
        ControlledMultipleToggleGroup,
        ControlledMultipleToggleGroupProps {
            values: values.clone(),
            on_change_calls: calls.clone(),
        },
    );
    dom.rebuild_in_place();

    assert_eq!(calls.borrow().len(), 0);
}

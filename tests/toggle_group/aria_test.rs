use dioxus::prelude::*;
use monoxus::{
    foundation::{shared::Direction, state::DataState},
    toggle_group::{
        ToggleGroupMode, ToggleGroupOrientation, ToggleGroupRoot, ToggleGroupItem,
        ToggleGroup, TOGGLE_GROUP_PARTS,
    },
};

#[test]
fn test_toggle_group_part_inventory() {
    assert_eq!(TOGGLE_GROUP_PARTS.len(), 2);
    assert_eq!(TOGGLE_GROUP_PARTS[0].as_str(), "root");
    assert_eq!(TOGGLE_GROUP_PARTS[1].as_str(), "item");
}

#[test]
fn test_toggle_group_single_aria_pairing() {
    use monoxus::foundation::shared::ScopeHandle;
    let scope = ScopeHandle::root("tg-aria-single");
    let group = ToggleGroup::new(scope.clone(), ToggleGroupMode::Single)
        .with_single_value("item-1");

    let root_attrs = group.root_attributes(None);
    assert_eq!(root_attrs.role(), "radiogroup");
    assert_eq!(root_attrs.dir(), "ltr");
    assert_eq!(root_attrs.data_orientation(), "horizontal");
    assert_eq!(root_attrs.data_disabled(), None);
    assert_eq!(root_attrs.data_toggle_group_root(), "");

    // Item 1 (pressed)
    let item1_attrs = group.item_attributes("item-1", None, false, Some("item-1"));
    assert_eq!(item1_attrs.role(), Some("radio"));
    assert_eq!(item1_attrs.aria_checked(), Some("true"));
    assert_eq!(item1_attrs.aria_pressed(), None); // TG-5: aria-pressed MUST be None in Single mode!
    assert_eq!(item1_attrs.data_state(), DataState::On);
    assert_eq!(item1_attrs.data_disabled(), None);
    assert_eq!(item1_attrs.tabindex(), 0);

    // Item 2 (unpressed)
    let item2_attrs = group.item_attributes("item-2", None, false, Some("item-1"));
    assert_eq!(item2_attrs.role(), Some("radio"));
    assert_eq!(item2_attrs.aria_checked(), Some("false"));
    assert_eq!(item2_attrs.aria_pressed(), None);
    assert_eq!(item2_attrs.data_state(), DataState::Off);
    assert_eq!(item2_attrs.data_disabled(), None);
    assert_eq!(item2_attrs.tabindex(), -1);
}

#[test]
fn test_toggle_group_multiple_aria_pairing() {
    use monoxus::foundation::shared::ScopeHandle;
    let scope = ScopeHandle::root("tg-aria-multiple");
    let group = ToggleGroup::new(scope.clone(), ToggleGroupMode::Multiple)
        .with_multiple_values(vec!["item-1".to_string()]);

    let root_attrs = group.root_attributes(None);
    assert_eq!(root_attrs.role(), "group"); // TG-6: role="group" in Multiple mode!
    assert_eq!(root_attrs.dir(), "ltr");
    assert_eq!(root_attrs.data_orientation(), "horizontal");
    assert_eq!(root_attrs.data_disabled(), None);

    // Item 1 (pressed)
    let item1_attrs = group.item_attributes("item-1", None, false, Some("item-1"));
    assert_eq!(item1_attrs.role(), None); // TG-6: role MUST be None in Multiple mode!
    assert_eq!(item1_attrs.aria_checked(), None); // TG-6: aria-checked MUST be None in Multiple mode!
    assert_eq!(item1_attrs.aria_pressed(), Some("true"));
    assert_eq!(item1_attrs.data_state(), DataState::On);
    assert_eq!(item1_attrs.tabindex(), 0);

    // Item 2 (unpressed)
    let item2_attrs = group.item_attributes("item-2", None, false, Some("item-1"));
    assert_eq!(item2_attrs.role(), None);
    assert_eq!(item2_attrs.aria_checked(), None);
    assert_eq!(item2_attrs.aria_pressed(), Some("false"));
    assert_eq!(item2_attrs.data_state(), DataState::Off);
    assert_eq!(item2_attrs.tabindex(), -1);
}

#[test]
fn test_toggle_group_direction_and_disabled_emission() {
    use monoxus::foundation::shared::ScopeHandle;
    let scope = ScopeHandle::root("tg-aria-dir-dis");
    let group = ToggleGroup::new(scope, ToggleGroupMode::Single)
        .with_direction(Direction::Rtl)
        .with_orientation(ToggleGroupOrientation::Vertical)
        .with_disabled(true);

    let root_attrs = group.root_attributes(None);
    assert_eq!(root_attrs.dir(), "rtl");
    assert_eq!(root_attrs.data_orientation(), "vertical");
    assert_eq!(root_attrs.data_disabled(), Some(""));

    let item_attrs = group.item_attributes("item-1", None, false, None);
    assert_eq!(item_attrs.data_disabled(), Some(""));
    assert!(item_attrs.is_disabled());
    assert_eq!(item_attrs.tabindex(), -1);
}

#[component]
fn ToggleGroupSampleAriaApp() -> Element {
    rsx! {
        ToggleGroupRoot {
            id: "aria-root",
            r#type: ToggleGroupMode::Single,
            default_value: "bold".to_string(),
            ToggleGroupItem { value: "bold".to_string(), "B" }
            ToggleGroupItem { value: "italic".to_string(), "I" }
            ToggleGroupItem { value: "underline".to_string(), disabled: true, "U" }
        }
    }
}

#[test]
fn test_toggle_group_vdom_aria_tree_mounts() {
    let mut dom = VirtualDom::new(ToggleGroupSampleAriaApp);
    dom.rebuild_in_place();
}

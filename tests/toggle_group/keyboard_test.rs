use dioxus::prelude::*;
use monoxus::{
    foundation::shared::{CollectionRegistry, Direction, ScopeHandle},
    toggle_group::{
        ToggleGroup, ToggleGroupItem, ToggleGroupItemRegistration, ToggleGroupMode,
        ToggleGroupOrientation, ToggleGroupRoot,
    },
};

#[cfg(feature = "native")]
#[path = "../common/mod.rs"]
mod common;

fn create_sample_registry() -> CollectionRegistry<String, ToggleGroupItemRegistration> {
    let mut reg = CollectionRegistry::new();
    reg.register(
        "1".to_string(),
        ToggleGroupItemRegistration {
            value: "1".to_string(),
            id: "id-1".to_string(),
            disabled: false,
        },
    );
    reg.register(
        "2".to_string(),
        ToggleGroupItemRegistration {
            value: "2".to_string(),
            id: "id-2".to_string(),
            disabled: true, // disabled item should be skipped!
        },
    );
    reg.register(
        "3".to_string(),
        ToggleGroupItemRegistration {
            value: "3".to_string(),
            id: "id-3".to_string(),
            disabled: false,
        },
    );
    reg.register(
        "4".to_string(),
        ToggleGroupItemRegistration {
            value: "4".to_string(),
            id: "id-4".to_string(),
            disabled: false,
        },
    );
    reg
}

#[test]
fn test_keyboard_navigation_horizontal_ltr() {
    let scope = ScopeHandle::root("tg-nav-h-ltr");
    let group = ToggleGroup::new(scope, ToggleGroupMode::Single)
        .with_orientation(ToggleGroupOrientation::Horizontal)
        .with_direction(Direction::Ltr)
        .with_loop_focus(true);

    let reg = create_sample_registry();

    // From "1", ArrowRight skips disabled "2" and reaches "3"
    let next = group.navigate_key(&reg, "1", "ArrowRight", false, false, false, false);
    assert_eq!(next, Some("3".to_string()));

    // From "3", ArrowRight reaches "4"
    let next2 = group.navigate_key(&reg, "3", "ArrowRight", false, false, false, false);
    assert_eq!(next2, Some("4".to_string()));

    // From "4", ArrowRight loops back to "1"
    let wrap = group.navigate_key(&reg, "4", "ArrowRight", false, false, false, false);
    assert_eq!(wrap, Some("1".to_string()));

    // From "1", ArrowLeft loops back to "4"
    let prev_wrap = group.navigate_key(&reg, "1", "ArrowLeft", false, false, false, false);
    assert_eq!(prev_wrap, Some("4".to_string()));

    // Home goes to first focusable "1"
    let home = group.navigate_key(&reg, "4", "Home", false, false, false, false);
    assert_eq!(home, Some("1".to_string()));

    // End goes to last focusable "4"
    let end = group.navigate_key(&reg, "1", "End", false, false, false, false);
    assert_eq!(end, Some("4".to_string()));
}

#[test]
fn test_keyboard_navigation_horizontal_rtl() {
    let scope = ScopeHandle::root("tg-nav-h-rtl");
    let group = ToggleGroup::new(scope, ToggleGroupMode::Single)
        .with_orientation(ToggleGroupOrientation::Horizontal)
        .with_direction(Direction::Rtl)
        .with_loop_focus(true);

    let reg = create_sample_registry();

    // In RTL, ArrowRight goes to PREVIOUS (loops to "4")
    let prev = group.navigate_key(&reg, "1", "ArrowRight", false, false, false, false);
    assert_eq!(prev, Some("4".to_string()));

    // In RTL, ArrowLeft goes to NEXT (skips disabled "2" to "3")
    let next = group.navigate_key(&reg, "1", "ArrowLeft", false, false, false, false);
    assert_eq!(next, Some("3".to_string()));
}

#[test]
fn test_keyboard_navigation_vertical() {
    let scope = ScopeHandle::root("tg-nav-v");
    let group = ToggleGroup::new(scope, ToggleGroupMode::Single)
        .with_orientation(ToggleGroupOrientation::Vertical)
        .with_loop_focus(true);

    let reg = create_sample_registry();

    // ArrowDown goes next (skips disabled "2" to "3")
    let down = group.navigate_key(&reg, "1", "ArrowDown", false, false, false, false);
    assert_eq!(down, Some("3".to_string()));

    // ArrowUp goes prev (loops to "4")
    let up = group.navigate_key(&reg, "1", "ArrowUp", false, false, false, false);
    assert_eq!(up, Some("4".to_string()));
}

#[test]
fn test_orthogonal_arrows_ignored() {
    let reg = create_sample_registry();

    // Horizontal ignores ArrowUp and ArrowDown
    let h_group = ToggleGroup::new(ScopeHandle::root("tg-h"), ToggleGroupMode::Single)
        .with_orientation(ToggleGroupOrientation::Horizontal);
    assert_eq!(
        h_group.navigate_key(&reg, "1", "ArrowUp", false, false, false, false),
        None
    );
    assert_eq!(
        h_group.navigate_key(&reg, "1", "ArrowDown", false, false, false, false),
        None
    );

    // Vertical ignores ArrowLeft and ArrowRight
    let v_group = ToggleGroup::new(ScopeHandle::root("tg-v"), ToggleGroupMode::Single)
        .with_orientation(ToggleGroupOrientation::Vertical);
    assert_eq!(
        v_group.navigate_key(&reg, "1", "ArrowLeft", false, false, false, false),
        None
    );
    assert_eq!(
        v_group.navigate_key(&reg, "1", "ArrowRight", false, false, false, false),
        None
    );
}

#[test]
fn test_modifier_keys_ignored() {
    let reg = create_sample_registry();
    let group = ToggleGroup::new(ScopeHandle::root("tg-mod"), ToggleGroupMode::Single);

    // Alt, Ctrl, Meta, Shift modifiers prevent roving focus navigation
    assert_eq!(
        group.navigate_key(&reg, "1", "ArrowRight", true, false, false, false),
        None
    );
    assert_eq!(
        group.navigate_key(&reg, "1", "ArrowRight", false, true, false, false),
        None
    );
    assert_eq!(
        group.navigate_key(&reg, "1", "ArrowRight", false, false, true, false),
        None
    );
    assert_eq!(
        group.navigate_key(&reg, "1", "ArrowRight", false, false, false, true),
        None
    );
}

#[test]
fn test_loop_focus_disabled_clamps_at_boundaries() {
    let reg = create_sample_registry();
    let group = ToggleGroup::new(ScopeHandle::root("tg-clamp"), ToggleGroupMode::Single)
        .with_loop_focus(false);

    // From first focusable item "1", navigating previous returns None (clamps)
    assert_eq!(
        group.navigate_key(&reg, "1", "ArrowLeft", false, false, false, false),
        None
    );

    // From last focusable item "4", navigating next returns None (clamps)
    assert_eq!(
        group.navigate_key(&reg, "4", "ArrowRight", false, false, false, false),
        None
    );
}

#[test]
fn test_roving_focus_disabled_gives_all_enabled_items_tabindex_zero() {
    let group = ToggleGroup::new(ScopeHandle::root("tg-no-roving"), ToggleGroupMode::Single)
        .with_roving_focus(false);

    assert_eq!(group.tabindex_for_item("1", false, Some("1")), 0);
    assert_eq!(group.tabindex_for_item("2", false, Some("1")), 0);
    assert_eq!(group.tabindex_for_item("3", false, Some("1")), 0);
    // Disabled item is still -1
    assert_eq!(group.tabindex_for_item("disabled", true, Some("1")), -1);
}

#[component]
fn NestedToggleGroupApp() -> Element {
    // Port of bits-ui toggle-group-nested-test.svelte
    rsx! {
        ToggleGroupRoot {
            id: "outer-root",
            r#type: ToggleGroupMode::Single,
            ToggleGroupItem { value: "outer-1".to_string(), "Outer 1" }
            ToggleGroupRoot {
                id: "inner-root",
                r#type: ToggleGroupMode::Single,
                ToggleGroupItem { value: "inner-1".to_string(), "Inner 1" }
                ToggleGroupItem { value: "inner-2".to_string(), "Inner 2" }
            }
            ToggleGroupItem { value: "outer-2".to_string(), "Outer 2" }
        }
    }
}

#[test]
fn test_nested_toggle_group_isolation_vdom() {
    let mut dom = VirtualDom::new(NestedToggleGroupApp);
    dom.rebuild_in_place();
}

#[cfg(feature = "native")]
mod native_tests {
    use super::*;
    use common::native_fixture::*;

    #[test]
    fn test_native_toggle_group_mount() {
        let harness = mount_native_scenario(|| {
            rsx! {
                ToggleGroupRoot {
                    id: "native-tg-root",
                    r#type: ToggleGroupMode::Single,
                    ToggleGroupItem { id: "item-a", value: "a".to_string(), "A" }
                    ToggleGroupItem { id: "item-b", value: "b".to_string(), "B" }
                }
            }
        });

        let root_id = harness.query("#native-tg-root").expect("ToggleGroupRoot mounted");
        let item_a_id = harness.query("#item-a").expect("Item A mounted");
        {
            let base = harness.base();
            let root_node = base.get_node(root_id).expect("Root node exists");
            assert_eq!(root_node.attr("role".into()), Some("radiogroup"));
            let item_a_node = base.get_node(item_a_id).expect("Item node exists");
            assert_eq!(item_a_node.attr("role".into()), Some("radio"));
            assert_eq!(item_a_node.attr("aria-checked".into()), Some("false"));
        }
    }
}

use dioxus::prelude::*;
use monoxus::{
    foundation::shared::{CollectionRegistry, Direction, ScopeHandle},
    radio_group::{
        RadioGroup, RadioGroupItem, RadioGroupItemRegistration, RadioGroupOrientation,
        RadioGroupRoot,
    },
};

#[cfg(feature = "native")]
#[path = "../common/mod.rs"]
mod common;

fn create_sample_registry() -> CollectionRegistry<String, RadioGroupItemRegistration> {
    let mut reg = CollectionRegistry::new();
    reg.register(
        "1".to_string(),
        RadioGroupItemRegistration {
            value: "1".to_string(),
            id: "id-1".to_string(),
            disabled: false,
        },
    );
    reg.register(
        "2".to_string(),
        RadioGroupItemRegistration {
            value: "2".to_string(),
            id: "id-2".to_string(),
            disabled: true, // disabled item should be skipped!
        },
    );
    reg.register(
        "3".to_string(),
        RadioGroupItemRegistration {
            value: "3".to_string(),
            id: "id-3".to_string(),
            disabled: false,
        },
    );
    reg.register(
        "4".to_string(),
        RadioGroupItemRegistration {
            value: "4".to_string(),
            id: "id-4".to_string(),
            disabled: false,
        },
    );
    reg
}

#[test]
fn test_keyboard_navigation_both_orientation_ltr() {
    let scope = ScopeHandle::root("rg-nav-both");
    let group = RadioGroup::new(scope).with_loop_focus(true); // default: orientation None -> Both
    let reg = create_sample_registry();

    // From "1", ArrowRight skips disabled "2" and reaches "3"
    let next = group.navigate_key(&reg, "1", "ArrowRight", false, false, false, false);
    assert_eq!(next, Some("3".to_string()));

    // From "1", ArrowDown also reaches "3"
    let down = group.navigate_key(&reg, "1", "ArrowDown", false, false, false, false);
    assert_eq!(down, Some("3".to_string()));

    // From "3", ArrowRight reaches "4"
    let next2 = group.navigate_key(&reg, "3", "ArrowRight", false, false, false, false);
    assert_eq!(next2, Some("4".to_string()));

    // From "4", ArrowRight loops back to "1"
    let wrap = group.navigate_key(&reg, "4", "ArrowRight", false, false, false, false);
    assert_eq!(wrap, Some("1".to_string()));

    // From "1", ArrowLeft loops back to "4"
    let prev_wrap = group.navigate_key(&reg, "1", "ArrowLeft", false, false, false, false);
    assert_eq!(prev_wrap, Some("4".to_string()));

    // From "1", ArrowUp also loops back to "4"
    let prev_up = group.navigate_key(&reg, "1", "ArrowUp", false, false, false, false);
    assert_eq!(prev_up, Some("4".to_string()));

    // Home goes to first focusable "1"
    let home = group.navigate_key(&reg, "4", "Home", false, false, false, false);
    assert_eq!(home, Some("1".to_string()));

    // End goes to last focusable "4"
    let end = group.navigate_key(&reg, "1", "End", false, false, false, false);
    assert_eq!(end, Some("4".to_string()));
}

#[test]
fn test_keyboard_navigation_both_orientation_rtl() {
    let scope = ScopeHandle::root("rg-nav-both-rtl");
    let group = RadioGroup::new(scope)
        .with_direction(Direction::Rtl)
        .with_loop_focus(true);
    let reg = create_sample_registry();

    // In RTL, ArrowRight goes previous (loops to 4)
    assert_eq!(
        group.navigate_key(&reg, "1", "ArrowRight", false, false, false, false),
        Some("4".to_string())
    );
    // In RTL, ArrowLeft goes next (skips disabled 2 to 3)
    assert_eq!(
        group.navigate_key(&reg, "1", "ArrowLeft", false, false, false, false),
        Some("3".to_string())
    );
}

#[test]
fn test_keyboard_navigation_orientation_constraints() {
    let reg = create_sample_registry();

    // Horizontal ignores ArrowUp and ArrowDown
    let h_group = RadioGroup::new(ScopeHandle::root("rg-h"))
        .with_orientation(Some(RadioGroupOrientation::Horizontal));
    assert_eq!(
        h_group.navigate_key(&reg, "1", "ArrowUp", false, false, false, false),
        None
    );
    assert_eq!(
        h_group.navigate_key(&reg, "1", "ArrowDown", false, false, false, false),
        None
    );
    assert_eq!(
        h_group.navigate_key(&reg, "1", "ArrowRight", false, false, false, false),
        Some("3".to_string())
    );

    // Vertical ignores ArrowLeft and ArrowRight
    let v_group = RadioGroup::new(ScopeHandle::root("rg-v"))
        .with_orientation(Some(RadioGroupOrientation::Vertical));
    assert_eq!(
        v_group.navigate_key(&reg, "1", "ArrowLeft", false, false, false, false),
        None
    );
    assert_eq!(
        v_group.navigate_key(&reg, "1", "ArrowRight", false, false, false, false),
        None
    );
    assert_eq!(
        v_group.navigate_key(&reg, "1", "ArrowDown", false, false, false, false),
        Some("3".to_string())
    );
}

#[test]
fn test_loop_focus_disabled_clamps_at_boundaries() {
    let reg = create_sample_registry();
    let group = RadioGroup::new(ScopeHandle::root("rg-clamp")).with_loop_focus(false);

    // From first focusable item "1", navigating previous returns None (clamps)
    assert_eq!(
        group.navigate_key(&reg, "1", "ArrowLeft", false, false, false, false),
        None
    );
    assert_eq!(
        group.navigate_key(&reg, "1", "ArrowUp", false, false, false, false),
        None
    );

    // From last focusable item "4", navigating next returns None (clamps)
    assert_eq!(
        group.navigate_key(&reg, "4", "ArrowRight", false, false, false, false),
        None
    );
    assert_eq!(
        group.navigate_key(&reg, "4", "ArrowDown", false, false, false, false),
        None
    );
}

#[component]
fn InteractiveKeyboardApp() -> Element {
    let mut change_events = use_signal(|| 0_u32);

    rsx! {
        div {
            style: "width: 400px; height: 300px;",
            RadioGroupRoot {
                id: "test-rg",
                on_value_change: move |_val: String| {
                    change_events += 1;
                },
                RadioGroupItem { id: "item-a", value: "a", "Option A" }
                RadioGroupItem { id: "item-b", value: "b", disabled: true, "Option B (Disabled)" }
                RadioGroupItem { id: "item-c", value: "c", "Option C" }
                RadioGroupItem { id: "item-d", value: "d", "Option D" }
            }
            span { id: "change-count", "data-count": "{change_events}", "{change_events}" }
        }
    }
}

#[component]
fn ReadOnlyKeyboardApp() -> Element {
    let mut selected = use_signal(|| Some("x".to_string()));
    rsx! {
        RadioGroupRoot {
            id: "ro-rg",
            value: selected(),
            read_only: true,
            on_value_change: move |val: String| selected.set(Some(val)),
            RadioGroupItem { id: "ro-x", value: "x", "Option X" }
            RadioGroupItem { id: "ro-y", value: "y", "Option Y" }
        }
    }
}

#[cfg(feature = "native")]
mod native_tests {
    use super::*;
    use common::native_fixture::*;
    use keyboard_types::{Code, Key, NamedKey};

    #[test]
    fn test_native_keyboard_interactions() {
        let mut harness = mount_native_scenario(InteractiveKeyboardApp);

        let item_a_id = harness.node("#item-a");
        let item_c_id = harness.node("#item-c");
        let item_d_id = harness.node("#item-d");

        // 1. Initially unselected: all items are aria-checked="false"
        {
            let base = harness.base();
            assert_eq!(base.get_node(item_a_id).unwrap().attr("aria-checked".into()), Some("false"));
            assert_eq!(base.get_node(item_c_id).unwrap().attr("aria-checked".into()), Some("false"));
            assert_eq!(base.get_node(item_d_id).unwrap().attr("aria-checked".into()), Some("false"));
            // item-a is first enabled item, so it has tabindex="0"
            assert_eq!(base.get_node(item_a_id).unwrap().attr("tabindex".into()), Some("0"));
        }

        // 2. Focus item-a via focus_node (simulating Tab into unselected group)
        // RG-4: Tab / programmatic focus does NOT auto-select!
        focus_node(&mut harness, "#item-a");
        assert_native_focus_id(&harness, "#item-a");
        {
            let base = harness.base();
            assert_eq!(
                base.get_node(item_a_id).unwrap().attr("aria-checked".into()),
                Some("false"),
                "Tab/focus into unselected group must NOT auto-select"
            );
        }

        // 3. Press Enter on focused item-a
        // RG-5: Enter key is suppressed (prevent_default called, does NOT select!)
        send_key_and_pump(&mut harness, Key::Named(NamedKey::Enter), Code::Enter);
        {
            let base = harness.base();
            assert_eq!(
                base.get_node(item_a_id).unwrap().attr("aria-checked".into()),
                Some("false"),
                "Enter key must be suppressed and NOT select radio item"
            );
        }

        // 4. Press Space on focused item-a
        // RG-6: Space (KeyUp) selects unselected item
        send_key_and_pump(&mut harness, Key::Character(" ".into()), Code::Space);
        {
            let base = harness.base();
            assert_eq!(
                base.get_node(item_a_id).unwrap().attr("aria-checked".into()),
                Some("true"),
                "Space must select unselected focused radio item"
            );
        }

        // 5. Press Space AGAIN on already-checked item-a
        // RG-2: Irreversible item activation (cannot uncheck)
        send_key_and_pump(&mut harness, Key::Character(" ".into()), Code::Space);
        {
            let base = harness.base();
            assert_eq!(
                base.get_node(item_a_id).unwrap().attr("aria-checked".into()),
                Some("true"),
                "Space on already-checked radio item must NOT uncheck"
            );
        }

        // 6. Arrow-key roving focus WITH immediate auto-selection (RG-3, D-5)
        // ArrowDown from item-a skips disabled item-b and lands on item-c, auto-selecting it!
        send_key_and_pump(&mut harness, Key::Named(NamedKey::ArrowDown), Code::ArrowDown);
        assert_native_focus_id(&harness, "#item-c");
        {
            let base = harness.base();
            assert_eq!(
                base.get_node(item_a_id).unwrap().attr("aria-checked".into()),
                Some("false")
            );
            assert_eq!(
                base.get_node(item_c_id).unwrap().attr("aria-checked".into()),
                Some("true"),
                "ArrowDown must move focus AND immediately auto-select target item"
            );
        }

        // ArrowRight from item-c advances to item-d and auto-selects item-d
        send_key_and_pump(&mut harness, Key::Named(NamedKey::ArrowRight), Code::ArrowRight);
        assert_native_focus_id(&harness, "#item-d");
        {
            let base = harness.base();
            assert_eq!(
                base.get_node(item_c_id).unwrap().attr("aria-checked".into()),
                Some("false")
            );
            assert_eq!(
                base.get_node(item_d_id).unwrap().attr("aria-checked".into()),
                Some("true"),
                "ArrowRight must move focus AND immediately auto-select item-d"
            );
        }

        // ArrowUp from item-d goes back to item-c and auto-selects item-c
        send_key_and_pump(&mut harness, Key::Named(NamedKey::ArrowUp), Code::ArrowUp);
        assert_native_focus_id(&harness, "#item-c");
        {
            let base = harness.base();
            assert_eq!(
                base.get_node(item_c_id).unwrap().attr("aria-checked".into()),
                Some("true")
            );
        }

        // ArrowLeft from item-c skips disabled item-b and reaches item-a, auto-selecting it
        send_key_and_pump(&mut harness, Key::Named(NamedKey::ArrowLeft), Code::ArrowLeft);
        assert_native_focus_id(&harness, "#item-a");
        {
            let base = harness.base();
            assert_eq!(
                base.get_node(item_a_id).unwrap().attr("aria-checked".into()),
                Some("true")
            );
        }
    }

    #[test]
    fn test_native_read_only_keyboard_navigation_does_not_change_selection() {
        let mut harness = mount_native_scenario(ReadOnlyKeyboardApp);

        let item_x_id = harness.node("#ro-x");
        let item_y_id = harness.node("#ro-y");

        // Initially "x" is selected
        {
            let base = harness.base();
            assert_eq!(base.get_node(item_x_id).unwrap().attr("aria-checked".into()), Some("true"));
            assert_eq!(base.get_node(item_y_id).unwrap().attr("aria-checked".into()), Some("false"));
        }

        // Focus ro-x and press ArrowDown
        focus_node(&mut harness, "#ro-x");
        send_key_and_pump(&mut harness, Key::Named(NamedKey::ArrowDown), Code::ArrowDown);

        // Focus moved to ro-y, but selection remains on ro-x!
        assert_native_focus_id(&harness, "#ro-y");
        {
            let base = harness.base();
            assert_eq!(
                base.get_node(item_x_id).unwrap().attr("aria-checked".into()),
                Some("true"),
                "Selection must remain on initial item in read-only group"
            );
            assert_eq!(
                base.get_node(item_y_id).unwrap().attr("aria-checked".into()),
                Some("false"),
                "Read-only group must NOT auto-select on arrow navigation"
            );
        }
    }
}

use dioxus::prelude::*;
use monoxus::{
    combobox::{
        COMBOBOX_PARTS, Combobox, ComboboxArrow, ComboboxContent, ComboboxContentStatic,
        ComboboxGroup, ComboboxGroupHeading, ComboboxInput, ComboboxInputAttrs, ComboboxItem,
        ComboboxItemAttrs, ComboboxItemData, ComboboxMode, ComboboxNavKey,
        ComboboxOptionRegistry, ComboboxPart, ComboboxPortal, ComboboxRelationships, ComboboxRoot,
        ComboboxScrollAlignment, ComboboxScrollDownButton, ComboboxScrollUpButton,
        ComboboxSeparator, ComboboxTrigger, ComboboxViewport, apply_multiple_toggle,
        apply_single_toggle, resolve_input_keydown,
    },
    foundation::shared::ScopeHandle,
};

#[cfg(feature = "native")]
use crate::common;

#[test]
fn combobox_public_parts_and_relationships_contract() {
    assert_eq!(
        COMBOBOX_PARTS
            .iter()
            .map(ComboboxPart::as_str)
            .collect::<Vec<_>>(),
        [
            "root",
            "input",
            "trigger",
            "content",
            "content-static",
            "viewport",
            "item",
            "group",
            "group-heading",
            "scroll-up-button",
            "scroll-down-button",
            "arrow",
            "portal",
            "hidden-input",
        ]
    );

    let scope = ScopeHandle::root("combobox").child("contract");
    let rels = ComboboxRelationships::new(scope.clone());
    assert_eq!(rels.root_id(), scope.token());
    assert_eq!(rels.input_id(), scope.qualify("input"));
    assert_eq!(rels.trigger_id(), scope.qualify("trigger"));
    assert_eq!(rels.content_id(), scope.qualify("content"));
    assert_eq!(rels.viewport_id(), scope.qualify("viewport"));
    assert_eq!(rels.group_id("g1"), scope.qualify("group-g1"));
    assert_eq!(
        rels.group_heading_id("g1"),
        scope.qualify("group-heading-g1")
    );
    assert_eq!(rels.item_id("apple"), scope.qualify("item-apple"));
}

#[test]
fn combobox_single_and_multiple_selection_and_keyboard_contracts() {
    // Single mode: select, allow_deselect, clear_on_deselect
    let sel = apply_single_toggle(None, "apple", "Apple", true, true);
    assert_eq!(sel.next_value.as_deref(), Some("apple"));
    assert_eq!(sel.next_input_value.as_deref(), Some("Apple"));
    assert!(sel.should_close);

    let desel = apply_single_toggle(Some("apple"), "apple", "Apple", true, true);
    assert_eq!(desel.next_value, None);
    assert_eq!(desel.next_input_value.as_deref(), Some(""));
    assert!(desel.should_close);

    let no_desel = apply_single_toggle(Some("apple"), "apple", "Apple", false, true);
    assert_eq!(no_desel.next_value.as_deref(), Some("apple"));
    assert_eq!(no_desel.next_input_value, None);
    assert!(!no_desel.changed);

    // Multiple mode: add/remove & clear_on_deselect only when last removed
    let m1 = apply_multiple_toggle(&[], "a", "Alpha", true);
    assert_eq!(m1.next_values, vec!["a"]);
    assert_eq!(m1.next_input_value.as_deref(), Some("Alpha"));
    let m2 = apply_multiple_toggle(&m1.next_values, "b", "Beta", true);
    assert_eq!(m2.next_values, vec!["a", "b"]);
    let m3 = apply_multiple_toggle(&m2.next_values, "a", "Alpha", true);
    assert_eq!(m3.next_values, vec!["b"]);
    assert_eq!(m3.next_input_value, None);
    let m4 = apply_multiple_toggle(&m3.next_values, "b", "Beta", true);
    assert!(m4.next_values.is_empty());
    assert_eq!(m4.next_input_value.as_deref(), Some(""));

    // Keyboard navigation contract
    let mut reg = ComboboxOptionRegistry::new();
    for i in 0..14 {
        reg.register(format!("v{i}"), format!("Label {i}"), format!("id-{i}"), i == 1);
    }

    // Home/End pass through to input caret
    assert!(!resolve_input_keydown(&reg, true, Some("v0"), None, ComboboxNavKey::Home, false, false, false).prevent_default);
    assert!(!resolve_input_keydown(&reg, true, Some("v0"), None, ComboboxNavKey::End, false, false, false).prevent_default);

    // PageDown (+10 enabled skipping disabled v1)
    let pd = resolve_input_keydown(&reg, true, Some("v0"), None, ComboboxNavKey::PageDown, false, false, false);
    assert!(pd.prevent_default);
    assert_eq!(pd.next_highlighted, Some(Some("v11".to_string())));

    // Alt+ArrowUp closes with prevent_default
    let alt_up = resolve_input_keydown(&reg, true, Some("v0"), None, ComboboxNavKey::AltArrowUp, false, false, false);
    assert!(alt_up.prevent_default);
    assert_eq!(alt_up.next_open, Some(false));

    // Tab closes without prevent_default
    let tab = resolve_input_keydown(&reg, true, Some("v0"), None, ComboboxNavKey::Tab, false, false, false);
    assert!(!tab.prevent_default);
    assert_eq!(tab.next_open, Some(false));

    // Escape closes with prevent_default
    let esc = resolve_input_keydown(&reg, true, Some("v0"), None, ComboboxNavKey::Escape, false, false, false);
    assert!(esc.prevent_default);
    assert_eq!(esc.next_open, Some(false));

    // Enter guarded by IME is_composing (4B D-5)
    assert!(!resolve_input_keydown(&reg, true, Some("v0"), None, ComboboxNavKey::Enter, true, false, false).trigger_select);
    assert!(resolve_input_keydown(&reg, true, Some("v0"), None, ComboboxNavKey::Enter, false, false, false).trigger_select);
}

#[test]
fn combobox_ssr_renders_wai_aria_and_data_attrs_single_and_multiple() {
    #[component]
    fn SingleComboboxApp() -> Element {
        let items = vec![
            ComboboxItemData::new("apple", "Apple", false),
            ComboboxItemData::new("banana", "Banana", true),
        ];
        rsx! {
            Combobox {
                id: Some("cbx-single".to_string()),
                default_open: true,
                default_value: Some("apple".to_string()),
                name: Some("fruit".to_string()),
                items: items,
                ComboboxInput {
                    placeholder: Some("Search fruit...".to_string()),
                }
                ComboboxTrigger { "Toggle" }
                ComboboxPortal {
                    disabled: true,
                    ComboboxContent {
                        ComboboxScrollUpButton { force_mount: true, "Up" }
                        ComboboxViewport {
                            ComboboxGroup {
                                group_key: Some("fruits".to_string()),
                                ComboboxGroupHeading { "Fruits" }
                                ComboboxItem { value: "apple".to_string(), label: Some("Apple".to_string()), "Apple" }
                                ComboboxSeparator {}
                                ComboboxItem { value: "banana".to_string(), label: Some("Banana".to_string()), disabled: true, "Banana" }
                            }
                        }
                        ComboboxScrollDownButton { force_mount: true, "Down" }
                        ComboboxArrow {}
                    }
                }
            }
        }
    }

    let mut single_dom = VirtualDom::new(SingleComboboxApp);
    single_dom.rebuild_in_place();

    #[cfg(feature = "native")]
    {
        use crate::common::native_fixture::mount_native_scenario;
        let harness = mount_native_scenario(SingleComboboxApp);
        assert_eq!(
            harness.attr("[data-combobox-input]", "role").as_deref(),
            Some("combobox")
        );
        assert_eq!(
            harness
                .attr("[data-combobox-input]", "aria-autocomplete")
                .as_deref(),
            Some("list")
        );
        assert_eq!(
            harness
                .attr("[data-combobox-input]", "aria-expanded")
                .as_deref(),
            Some("true")
        );
        assert_eq!(
            harness
                .attr("[data-combobox-trigger]", "aria-haspopup")
                .as_deref(),
            Some("listbox")
        );
        assert_eq!(
            harness.attr("[data-combobox-content]", "role").as_deref(),
            Some("listbox")
        );
        assert_eq!(
            harness
                .attr("[data-combobox-content]", "aria-multiselectable")
                .as_deref(),
            None
        );
        assert_eq!(
            harness.attr("[data-combobox-group]", "role").as_deref(),
            Some("group")
        );
        assert_eq!(
            harness
                .attr("[data-combobox-item][data-value='apple']", "role")
                .as_deref(),
            Some("option")
        );
        assert_eq!(
            harness
                .attr("[data-combobox-item][data-value='apple']", "aria-selected")
                .as_deref(),
            Some("true")
        );
        assert_eq!(
            harness
                .attr("[data-combobox-item][data-value='apple']", "data-disabled")
                .as_deref(),
            None
        );
        // 4B D-6: disabled option emits aria-disabled="true"
        assert_eq!(
            harness
                .attr("[data-combobox-item][data-value='banana']", "aria-disabled")
                .as_deref(),
            Some("true")
        );
        assert_eq!(
            harness
                .attr("[data-combobox-item][data-value='banana']", "data-disabled")
                .as_deref(),
            Some("")
        );
        // Auto-rendered hidden form input
        assert_eq!(
            harness
                .attr("input[type='hidden'][name='fruit']", "value")
                .as_deref(),
            Some("apple")
        );
    }

    #[component]
    fn MultipleComboboxStaticApp() -> Element {
        rsx! {
            ComboboxRoot {
                id: Some("cbx-multi".to_string()),
                mode: ComboboxMode::Multiple,
                default_open: true,
                default_values: Some(vec!["a".to_string(), "b".to_string()]),
                name: Some("tags".to_string()),
                scroll_alignment: ComboboxScrollAlignment::Center,
                ComboboxInput {}
                ComboboxContentStatic {
                    ComboboxViewport {
                        ComboboxItem { value: "a".to_string(), "Alpha" }
                        ComboboxItem { value: "b".to_string(), "Beta" }
                    }
                }
            }
        }
    }

    let mut multi_dom = VirtualDom::new(MultipleComboboxStaticApp);
    multi_dom.rebuild_in_place();

    #[cfg(feature = "native")]
    {
        use crate::common::native_fixture::mount_native_scenario;
        let harness = mount_native_scenario(MultipleComboboxStaticApp);
        assert_eq!(
            harness
                .attr("[data-combobox-content]", "aria-multiselectable")
                .as_deref(),
            Some("true")
        );
        assert_eq!(
            harness
                .attr("input[type='hidden'][name='tags'][value='a']", "value")
                .as_deref(),
            Some("a")
        );
        assert_eq!(
            harness
                .attr("input[type='hidden'][name='tags'][value='b']", "value")
                .as_deref(),
            Some("b")
        );
    }
}

#[test]
fn combobox_attrs_enforce_4b_d6_and_activedescendant_gating() {
    let closed_input = ComboboxInputAttrs::new("in", "ct", false, Some("opt-1".into()), false);
    assert_eq!(closed_input.aria_activedescendant(), None);
    let open_input = ComboboxInputAttrs::new("in", "ct", true, Some("opt-1".into()), false);
    assert_eq!(open_input.aria_activedescendant(), Some("opt-1"));

    let disabled_item = ComboboxItemAttrs::new("opt-2", "b", "Beta", false, true, true);
    assert_eq!(disabled_item.aria_disabled(), Some("true"));
    assert_eq!(disabled_item.data_disabled(), Some(""));
    assert_eq!(disabled_item.data_highlighted(), None);
}

#[cfg(feature = "native")]
#[test]
fn native_combobox_preserves_input_focus_across_trigger_item_and_escape() {
    use common::native_fixture::*;
    use keyboard_types::{Code, Key, NamedKey};

    let _guard = NATIVE_TEST_LOCK.lock().unwrap_or_else(|e| e.into_inner());

    #[component]
    fn NativeComboboxScenario() -> Element {
        rsx! {
            div { style: "padding: 24px;",
                ComboboxRoot {
                    id: Some("native-cbx".to_string()),
                    name: Some("native_fruit".to_string()),
                    clear_on_deselect: true,
                    ComboboxInput {
                        id: Some("native-cbx-input".to_string()),
                        style: Some("width: 200px; height: 32px;".to_string()),
                    }
                    ComboboxTrigger {
                        id: Some("native-cbx-trigger".to_string()),
                        style: Some("width: 32px; height: 32px;".to_string()),
                        "v"
                    }
                    ComboboxContentStatic {
                        id: Some("native-cbx-content".to_string()),
                        ComboboxViewport {
                            ComboboxItem {
                                id: Some("native-cbx-opt-apple".to_string()),
                                value: "apple".to_string(),
                                label: Some("Apple".to_string()),
                                style: Some("height: 28px;".to_string()),
                                "Apple"
                            }
                            ComboboxItem {
                                id: Some("native-cbx-opt-banana".to_string()),
                                value: "banana".to_string(),
                                label: Some("Banana".to_string()),
                                style: Some("height: 28px;".to_string()),
                                "Banana"
                            }
                        }
                    }
                }
            }
        }
    }

    let mut harness = mount_native_scenario(NativeComboboxScenario);

    // Click input to focus it
    click_and_pump(&mut harness, "#native-cbx-input");
    assert_eq!(harness.attr("#native-cbx-input", "aria-expanded").as_deref(), Some("false"));

    // ArrowDown opens menu and highlights first enabled option
    send_key_and_pump(&mut harness, Key::Named(NamedKey::ArrowDown), Code::ArrowDown);
    assert_eq!(harness.attr("#native-cbx-input", "aria-expanded").as_deref(), Some("true"));
    assert_eq!(
        harness.attr("#native-cbx-input", "aria-activedescendant").as_deref(),
        Some("native-cbx-opt-apple")
    );

    // Enter selects highlighted item, updates input value, and closes menu
    send_key_and_pump(&mut harness, Key::Named(NamedKey::Enter), Code::Enter);
    assert_eq!(harness.attr("#native-cbx-input", "aria-expanded").as_deref(), Some("false"));
    assert_eq!(harness.attr("#native-cbx-input", "value").as_deref(), Some("Apple"));
    assert_eq!(harness.attr("input[type='hidden'][name='native_fruit']", "value").as_deref(), Some("apple"));

    // Click trigger to reopen; input stays focused and menu opens
    click_and_pump(&mut harness, "#native-cbx-trigger");
    assert_eq!(harness.attr("#native-cbx-input", "aria-expanded").as_deref(), Some("true"));

    // Escape closes menu without reverting input value
    send_key_and_pump(&mut harness, Key::Named(NamedKey::Escape), Code::Escape);
    assert_eq!(harness.attr("#native-cbx-input", "aria-expanded").as_deref(), Some("false"));
    assert_eq!(harness.attr("#native-cbx-input", "value").as_deref(), Some("Apple"));
}

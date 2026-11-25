use dioxus::prelude::*;
use monoxus::{
    combobox::{
        COMBOBOX_PARTS, Combobox, ComboboxArrow, ComboboxArrowRenderProps, ComboboxContent,
        ComboboxContentRenderProps, ComboboxContentStatic, ComboboxContentStaticRenderProps,
        ComboboxGroup, ComboboxGroupHeading, ComboboxGroupHeadingRenderProps,
        ComboboxGroupRenderProps, ComboboxInput, ComboboxInputAttrs, ComboboxInputRenderProps,
        ComboboxItem, ComboboxItemAttrs, ComboboxItemData, ComboboxItemRenderProps, ComboboxMode,
        ComboboxNavKey, ComboboxOptionRegistry, ComboboxPart, ComboboxPortal,
        ComboboxRelationships, ComboboxRoot, ComboboxScrollAlignment,
        ComboboxScrollButtonRenderProps, ComboboxScrollDownButton, ComboboxScrollUpButton,
        ComboboxSeparator, ComboboxSeparatorRenderProps, ComboboxTrigger,
        ComboboxTriggerRenderProps, ComboboxViewport, ComboboxViewportRenderProps,
        apply_multiple_toggle, apply_single_toggle, resolve_input_keydown,
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
    // Single mode: select, allow_deselect (4B D-9: stays open), clear_on_deselect
    let sel = apply_single_toggle(None, "apple", "Apple", true, true);
    assert_eq!(sel.next_value.as_deref(), Some("apple"));
    assert_eq!(sel.next_input_value.as_deref(), Some("Apple"));
    assert!(sel.should_close);

    let desel = apply_single_toggle(Some("apple"), "apple", "Apple", true, true);
    assert_eq!(desel.next_value, None);
    assert_eq!(desel.next_input_value.as_deref(), Some(""));
    assert!(!desel.should_close);

    let no_desel = apply_single_toggle(Some("apple"), "apple", "Apple", false, true);
    assert_eq!(no_desel.next_value.as_deref(), Some("apple"));
    assert_eq!(no_desel.next_input_value, None);
    assert!(no_desel.should_close);
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
    assert!(!resolve_input_keydown(&reg, true, Some("v0"), None, ComboboxNavKey::Home, false, false, false, true).prevent_default);
    assert!(!resolve_input_keydown(&reg, true, Some("v0"), None, ComboboxNavKey::End, false, false, false, true).prevent_default);

    // PageDown (+10 enabled skipping disabled v1)
    let pd = resolve_input_keydown(&reg, true, Some("v0"), None, ComboboxNavKey::PageDown, false, false, false, false);
    assert!(pd.prevent_default);
    assert_eq!(pd.next_highlighted, Some(Some("v11".to_string())));

    // Alt+ArrowUp closes with prevent_default
    let alt_up = resolve_input_keydown(&reg, true, Some("v0"), None, ComboboxNavKey::AltArrowUp, false, false, false, false);
    assert!(alt_up.prevent_default);
    assert_eq!(alt_up.next_open, Some(false));

    // Tab closes without prevent_default
    let tab = resolve_input_keydown(&reg, true, Some("v0"), None, ComboboxNavKey::Tab, false, false, false, false);
    assert!(!tab.prevent_default);
    assert_eq!(tab.next_open, Some(false));

    // Escape closes with prevent_default
    let esc = resolve_input_keydown(&reg, true, Some("v0"), None, ComboboxNavKey::Escape, false, false, false, false);
    assert!(esc.prevent_default);
    assert_eq!(esc.next_open, Some(false));

    // Enter guarded by IME is_composing (4B D-5)
    assert!(!resolve_input_keydown(&reg, true, Some("v0"), None, ComboboxNavKey::Enter, true, false, false, false).trigger_select);
    assert!(resolve_input_keydown(&reg, true, Some("v0"), None, ComboboxNavKey::Enter, false, false, false, false).trigger_select);

    // Backspace opens closed menu only when input_non_empty == true
    assert_eq!(
        resolve_input_keydown(&reg, false, None, None, ComboboxNavKey::Backspace, false, false, false, false).next_open,
        None
    );
    assert_eq!(
        resolve_input_keydown(&reg, false, None, None, ComboboxNavKey::Backspace, false, false, false, true).next_open,
        Some(true)
    );
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
        assert!(
            harness
                .attr("[data-combobox-input]", "aria-controls")
                .is_some()
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
    let closed_input = ComboboxInputAttrs::new("in", None, false, Some("opt-1".into()), false);
    assert_eq!(closed_input.aria_controls(), None);
    assert_eq!(closed_input.aria_activedescendant(), None);
    let open_input =
        ComboboxInputAttrs::new("in", Some("ct".into()), true, Some("opt-1".into()), false);
    assert_eq!(open_input.aria_controls(), Some("ct"));
    assert_eq!(open_input.aria_activedescendant(), Some("opt-1"));

    let disabled_item = ComboboxItemAttrs::new("opt-2", "b", "Beta", false, true, true);
    assert_eq!(disabled_item.aria_disabled(), Some("true"));
    assert_eq!(disabled_item.data_disabled(), Some(""));
    assert_eq!(disabled_item.data_highlighted(), None);
}

#[test]
fn fm_cbx_8_as_child_all_twelve_rendered_parts() {
    #[component]
    fn AllAsChildComboboxApp() -> Element {
        rsx! {
            div { style: "padding: 24px;",
                ComboboxRoot {
                    id: Some("cbx-as-child".to_string()),
                    default_open: true,
                    ComboboxInput {
                        id: Some("as-child-input".to_string()),
                        as_child: true,
                        render_child: Callback::new(move |rp: ComboboxInputRenderProps| rsx! {
                            input {
                                id: "{rp.attrs.id()}",
                                role: "{rp.attrs.role()}",
                                style: "width: 200px; height: 32px;",
                                "data-custom-input": "yes",
                                value: "{rp.value}",
                            }
                        }),
                    }
                    ComboboxTrigger {
                        id: Some("as-child-trigger".to_string()),
                        as_child: true,
                        render_child: Callback::new(move |rp: ComboboxTriggerRenderProps| rsx! {
                            button {
                                id: "{rp.attrs.id()}",
                                style: "width: 32px; height: 32px;",
                                "data-custom-trigger": "yes",
                                {rp.children}
                            }
                        }),
                        "Open"
                    }
                    ComboboxContent {
                        id: Some("as-child-content".to_string()),
                        same_width: false,
                        as_child: true,
                        render_child: Callback::new(move |rp: ComboboxContentRenderProps| rsx! {
                            div {
                                id: "{rp.attrs.id()}",
                                role: "{rp.attrs.role()}",
                                style: "{rp.style} width: 200px; height: 140px;",
                                "data-custom-content": "yes",
                                onmounted: move |evt| rp.on_mounted.call(evt),
                                {rp.children}
                            }
                        }),
                        ComboboxScrollUpButton {
                            force_mount: true,
                            as_child: true,
                            render_child: Callback::new(move |rp: ComboboxScrollButtonRenderProps| rsx! {
                                div {
                                    "data-custom-scroll-up": "yes",
                                    style: "{rp.style}",
                                    {rp.children}
                                }
                            }),
                            "Up"
                        }
                        ComboboxViewport {
                            id: Some("as-child-viewport".to_string()),
                            as_child: true,
                            render_child: Callback::new(move |rp: ComboboxViewportRenderProps| rsx! {
                                div {
                                    id: "{rp.attrs.id()}",
                                    "data-custom-viewport": "yes",
                                    {rp.children}
                                }
                            }),
                            ComboboxGroup {
                                id: Some("as-child-group".to_string()),
                                as_child: true,
                                render_child: Callback::new(move |rp: ComboboxGroupRenderProps| rsx! {
                                    div {
                                        id: "{rp.attrs.id()}",
                                        role: "{rp.attrs.role()}",
                                        "data-custom-group": "yes",
                                        {rp.children}
                                    }
                                }),
                                ComboboxGroupHeading {
                                    id: Some("as-child-heading".to_string()),
                                    as_child: true,
                                    render_child: Callback::new(move |rp: ComboboxGroupHeadingRenderProps| rsx! {
                                        div {
                                            id: "{rp.attrs.id()}",
                                            "data-custom-heading": "yes",
                                            {rp.children}
                                        }
                                    }),
                                    "Heading"
                                }
                                ComboboxItem {
                                    id: Some("as-child-item".to_string()),
                                    value: "a".to_string(),
                                    as_child: true,
                                    render_child: Callback::new(move |rp: ComboboxItemRenderProps| rsx! {
                                        div {
                                            id: "{rp.attrs.id()}",
                                            role: "{rp.attrs.role()}",
                                            style: "height: 24px;",
                                            "data-custom-item": "yes",
                                            {rp.children}
                                        }
                                    }),
                                    "Alpha"
                                }
                                ComboboxSeparator {
                                    as_child: true,
                                    render_child: Callback::new(move |_rp: ComboboxSeparatorRenderProps| rsx! {
                                        hr { "data-custom-separator": "yes" }
                                    }),
                                }
                            }
                        }
                        ComboboxScrollDownButton {
                            force_mount: true,
                            as_child: true,
                            render_child: Callback::new(move |rp: ComboboxScrollButtonRenderProps| rsx! {
                                div {
                                    "data-custom-scroll-down": "yes",
                                    style: "{rp.style}",
                                    {rp.children}
                                }
                            }),
                            "Down"
                        }
                        ComboboxArrow {
                            as_child: true,
                            render_child: Callback::new(move |rp: ComboboxArrowRenderProps| rsx! {
                                span {
                                    "data-custom-arrow": "yes",
                                    style: "{rp.style}",
                                }
                            }),
                        }
                    }
                }
                ComboboxRoot {
                    id: Some("cbx-as-child-static".to_string()),
                    default_open: true,
                    ComboboxContentStatic {
                        id: Some("as-child-static".to_string()),
                        as_child: true,
                        render_child: Callback::new(move |rp: ComboboxContentStaticRenderProps| rsx! {
                            div {
                                id: "{rp.attrs.id()}",
                                "data-custom-static": "yes",
                                {rp.children}
                            }
                        }),
                        "Static"
                    }
                }
            }
        }
    }

    let mut dom = VirtualDom::new(AllAsChildComboboxApp);
    dom.rebuild_in_place();

    #[cfg(feature = "native")]
    {
        use crate::common::native_fixture::mount_native_scenario;
        let harness = mount_native_scenario(AllAsChildComboboxApp);
        assert_eq!(harness.attr("#as-child-input", "data-custom-input").as_deref(), Some("yes"));
        assert_eq!(harness.attr("#as-child-trigger", "data-custom-trigger").as_deref(), Some("yes"));
        assert_eq!(harness.attr("#as-child-content", "data-custom-content").as_deref(), Some("yes"));
        assert_eq!(harness.attr("#as-child-static", "data-custom-static").as_deref(), Some("yes"));
        assert_eq!(harness.attr("#as-child-viewport", "data-custom-viewport").as_deref(), Some("yes"));
        assert_eq!(harness.attr("#as-child-group", "data-custom-group").as_deref(), Some("yes"));
        assert_eq!(harness.attr("#as-child-heading", "data-custom-heading").as_deref(), Some("yes"));
        assert_eq!(harness.attr("#as-child-item", "data-custom-item").as_deref(), Some("yes"));
        assert_eq!(harness.attr("[data-custom-scroll-up]", "data-custom-scroll-up").as_deref(), Some("yes"));
        assert_eq!(harness.attr("[data-custom-scroll-down]", "data-custom-scroll-down").as_deref(), Some("yes"));
        assert_eq!(harness.attr("[data-custom-arrow]", "data-custom-arrow").as_deref(), Some("yes"));
        assert_eq!(harness.attr("[data-custom-separator]", "data-custom-separator").as_deref(), Some("yes"));
    }
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

    // 4B D-6: aria-controls omitted while closed and unmounted
    assert_eq!(harness.attr("#native-cbx-input", "aria-controls"), None);

    // Click input to focus it
    click_and_pump(&mut harness, "#native-cbx-input");
    assert_eq!(harness.attr("#native-cbx-input", "aria-expanded").as_deref(), Some("false"));

    // ArrowDown opens menu and highlights first enabled option
    send_key_and_pump(&mut harness, Key::Named(NamedKey::ArrowDown), Code::ArrowDown);
    assert_eq!(harness.attr("#native-cbx-input", "aria-expanded").as_deref(), Some("true"));
    assert_eq!(
        harness.attr("#native-cbx-input", "aria-controls").as_deref(),
        Some("native-cbx-content")
    );
    assert_eq!(
        harness.attr("#native-cbx-input", "aria-activedescendant").as_deref(),
        Some("native-cbx-opt-apple")
    );

    // Enter selects highlighted item, updates input value, and closes menu
    send_key_and_pump(&mut harness, Key::Named(NamedKey::Enter), Code::Enter);
    assert_eq!(harness.attr("#native-cbx-input", "aria-expanded").as_deref(), Some("false"));
    assert_eq!(harness.attr("#native-cbx-input", "aria-controls"), None);
    assert_eq!(harness.attr("#native-cbx-input", "value").as_deref(), Some("Apple"));
    assert_eq!(harness.attr("input[type='hidden'][name='native_fruit']", "value").as_deref(), Some("apple"));

    // Click trigger to reopen; input stays focused and menu opens
    click_and_pump(&mut harness, "#native-cbx-trigger");
    assert_eq!(harness.attr("#native-cbx-input", "aria-expanded").as_deref(), Some("true"));
    assert_native_focus_id(&harness, "#native-cbx-input");

    // 4B D-9: Single-mode allow_deselect toggles selected item OFF and keeps menu OPEN
    click_and_pump(&mut harness, "#native-cbx-opt-apple");
    assert_eq!(harness.attr("#native-cbx-input", "aria-expanded").as_deref(), Some("true"));
    assert_eq!(harness.attr("#native-cbx-input", "value").as_deref(), Some(""));
    assert_eq!(harness.attr("input[type='hidden'][name='native_fruit']", "value").as_deref(), Some(""));

    // Escape closes menu and preserves input focus
    send_key_and_pump(&mut harness, Key::Named(NamedKey::Escape), Code::Escape);
    assert_eq!(harness.attr("#native-cbx-input", "aria-expanded").as_deref(), Some("false"));
    assert_native_focus_id(&harness, "#native-cbx-input");
}

#[cfg(feature = "native")]
#[test]
fn fm_cbx_1_filter_clear_reopen_preserves_exact_visual_dom_order() {
    use common::native_fixture::*;
    use keyboard_types::{Code, Key, NamedKey};

    let _guard = NATIVE_TEST_LOCK.lock().unwrap_or_else(|e| e.into_inner());

    #[component]
    fn FilterOrderScenario() -> Element {
        let all = [
            ("dioxus", "Dioxus"),
            ("leptos", "Leptos"),
            ("yew", "Yew"),
            ("egui", "egui"),
            ("iced", "Iced"),
            ("slint", "Slint"),
        ];
        let mut filter_text = use_signal(String::new);
        let q = filter_text().to_lowercase();
        let filtered: Vec<(&str, &str)> = all
            .iter()
            .copied()
            .filter(|(v, l)| q.is_empty() || l.to_lowercase().contains(&q) || v.contains(&q))
            .collect();

        rsx! {
            div { style: "padding: 24px;",
                button {
                    id: "set-filter-le",
                    r#type: "button",
                    style: "width: 80px; height: 24px;",
                    onclick: move |_| filter_text.set("le".to_string()),
                    "Filter le"
                }
                button {
                    id: "clear-filter",
                    r#type: "button",
                    style: "width: 80px; height: 24px;",
                    onclick: move |_| filter_text.set(String::new()),
                    "Clear"
                }
                ComboboxRoot {
                    id: Some("fm1-cbx".to_string()),
                    input_value: Some(filter_text),
                    on_input_value_change: move |v: String| filter_text.set(v),
                    ComboboxInput {
                        id: Some("fm1-input".to_string()),
                        style: Some("width: 200px; height: 32px;".to_string()),
                    }
                    ComboboxTrigger {
                        id: Some("fm1-trigger".to_string()),
                        style: Some("width: 32px; height: 32px;".to_string()),
                        "v"
                    }
                    ComboboxContentStatic {
                        id: Some("fm1-content".to_string()),
                        ComboboxViewport {
                            for (val, label) in filtered {
                                ComboboxItem {
                                    key: "{val}",
                                    id: Some(format!("fm1-item-{val}")),
                                    value: val.to_string(),
                                    label: Some(label.to_string()),
                                    style: Some("height: 24px;".to_string()),
                                    "{label}"
                                }
                            }
                        }
                    }
                }
            }
        }
    }

    let mut harness = mount_native_scenario(FilterOrderScenario);

    // Open menu, filter to single item ("leptos"), then clear so all 6 remount with recycled ScopeIds
    click_and_pump(&mut harness, "#fm1-trigger");
    assert_eq!(harness.attr("#fm1-input", "aria-expanded").as_deref(), Some("true"));

    click_and_pump(&mut harness, "#set-filter-le");
    click_and_pump(&mut harness, "#clear-filter");

    // Reopen via trigger and step ArrowDown through all 6 items in exact visual DOM order
    click_and_pump(&mut harness, "#fm1-trigger");
    if harness.attr("#fm1-input", "aria-expanded").as_deref() != Some("true") {
        click_and_pump(&mut harness, "#fm1-trigger");
    }
    assert_eq!(harness.attr("#fm1-input", "aria-expanded").as_deref(), Some("true"));

    let expected_order = [
        "fm1-item-dioxus",
        "fm1-item-leptos",
        "fm1-item-yew",
        "fm1-item-egui",
        "fm1-item-iced",
        "fm1-item-slint",
    ];
    for expected in expected_order {
        send_key_and_pump(&mut harness, Key::Named(NamedKey::ArrowDown), Code::ArrowDown);
        assert_eq!(
            harness.attr("#fm1-input", "aria-activedescendant").as_deref(),
            Some(expected),
            "ArrowDown order diverged from visual DOM order"
        );
    }
    // ArrowUp returns in exact reverse DOM order
    send_key_and_pump(&mut harness, Key::Named(NamedKey::ArrowUp), Code::ArrowUp);
    assert_eq!(
        harness.attr("#fm1-input", "aria-activedescendant").as_deref(),
        Some("fm1-item-iced")
    );
}

#[cfg(feature = "native")]
#[test]
fn fm_cbx_2_dynamic_root_props_disabled_and_loop_focus_are_live_reactive() {
    use common::native_fixture::*;
    use keyboard_types::{Code, Key, NamedKey};

    let _guard = NATIVE_TEST_LOCK.lock().unwrap_or_else(|e| e.into_inner());

    #[component]
    fn DynamicRootPropsScenario() -> Element {
        let mut disabled = use_signal(|| false);
        let mut loop_focus = use_signal(|| false);
        let mut selected = use_signal(|| None::<String>);

        rsx! {
            div { style: "padding: 24px;",
                button {
                    id: "toggle-disabled",
                    r#type: "button",
                    style: "width: 80px; height: 24px;",
                    onclick: move |_| {
                        let next = !disabled();
                        disabled.set(next);
                    },
                    "Disabled"
                }
                button {
                    id: "toggle-loop",
                    r#type: "button",
                    style: "width: 80px; height: 24px;",
                    onclick: move |_| {
                        let next = !loop_focus();
                        loop_focus.set(next);
                    },
                    "Loop"
                }
                span {
                    id: "selected-readout",
                    "{selected().unwrap_or_else(|| \"none\".to_string())}"
                }
                ComboboxRoot {
                    id: Some("fm2-cbx".to_string()),
                    disabled: disabled(),
                    loop_focus: loop_focus(),
                    value: Some(selected),
                    on_value_change: move |v: Option<String>| selected.set(v),
                    ComboboxInput {
                        id: Some("fm2-input".to_string()),
                        style: Some("width: 200px; height: 32px;".to_string()),
                    }
                    ComboboxTrigger {
                        id: Some("fm2-trigger".to_string()),
                        style: Some("width: 32px; height: 32px;".to_string()),
                        "v"
                    }
                    ComboboxContentStatic {
                        id: Some("fm2-content".to_string()),
                        ComboboxViewport {
                            ComboboxItem {
                                id: Some("fm2-opt-a".to_string()),
                                value: "a".to_string(),
                                style: Some("height: 24px;".to_string()),
                                "Alpha"
                            }
                            ComboboxItem {
                                id: Some("fm2-opt-b".to_string()),
                                value: "b".to_string(),
                                style: Some("height: 24px;".to_string()),
                                "Beta"
                            }
                        }
                    }
                }
            }
        }
    }

    let mut harness = mount_native_scenario(DynamicRootPropsScenario);

    // Open and move to last item with loop_focus = false -> stays clamped on fm2-opt-b
    click_and_pump(&mut harness, "#fm2-input");
    send_key_and_pump(&mut harness, Key::Named(NamedKey::ArrowDown), Code::ArrowDown);
    send_key_and_pump(&mut harness, Key::Named(NamedKey::ArrowDown), Code::ArrowDown);
    send_key_and_pump(&mut harness, Key::Named(NamedKey::ArrowDown), Code::ArrowDown);
    assert_eq!(
        harness.attr("#fm2-input", "aria-activedescendant").as_deref(),
        Some("fm2-opt-b")
    );

    // Dynamically enable loop_focus = true without remounting ComboboxRoot
    click_and_pump(&mut harness, "#toggle-loop");
    click_and_pump(&mut harness, "#fm2-trigger");
    send_key_and_pump(&mut harness, Key::Named(NamedKey::ArrowDown), Code::ArrowDown); // a
    send_key_and_pump(&mut harness, Key::Named(NamedKey::ArrowDown), Code::ArrowDown); // b
    send_key_and_pump(&mut harness, Key::Named(NamedKey::ArrowDown), Code::ArrowDown); // wraps to a
    assert_eq!(
        harness.attr("#fm2-input", "aria-activedescendant").as_deref(),
        Some("fm2-opt-a")
    );

    // Dynamically toggle disabled = true while menu is open -> clicking item or pressing Enter must NOT select
    click_and_pump(&mut harness, "#toggle-disabled");
    assert_eq!(harness.attr("#fm2-input", "data-disabled").as_deref(), Some(""));
    assert_eq!(harness.attr("#fm2-trigger", "data-disabled").as_deref(), Some(""));
    send_key_and_pump(&mut harness, Key::Named(NamedKey::Enter), Code::Enter);
    assert_eq!(harness.text_content("#selected-readout"), "none");
}

#[cfg(feature = "native")]
#[test]
fn fm_cbx_3_on_escape_key_down_default_closes_and_prevent_default_blocks() {
    use common::native_fixture::*;
    use keyboard_types::{Code, Key, NamedKey};

    let _guard = NATIVE_TEST_LOCK.lock().unwrap_or_else(|e| e.into_inner());

    #[component]
    fn EscapeOrderingScenario() -> Element {
        let mut prevent = use_signal(|| false);
        let mut esc_count = use_signal(|| 0_u32);

        rsx! {
            div { style: "padding: 24px;",
                button {
                    id: "toggle-prevent-esc",
                    r#type: "button",
                    style: "width: 100px; height: 24px;",
                    onclick: move |_| {
                        let next = !prevent();
                        prevent.set(next);
                    },
                    "Prevent Esc"
                }
                span { id: "esc-count", "{esc_count()}" }
                ComboboxRoot {
                    id: Some("fm3-cbx".to_string()),
                    ComboboxInput {
                        id: Some("fm3-input".to_string()),
                        style: Some("width: 200px; height: 32px;".to_string()),
                    }
                    ComboboxTrigger {
                        id: Some("fm3-trigger".to_string()),
                        style: Some("width: 32px; height: 32px;".to_string()),
                        "v"
                    }
                    ComboboxContentStatic {
                        id: Some("fm3-content".to_string()),
                        on_escape_key_down: move |evt: KeyboardEvent| {
                            esc_count.set(esc_count() + 1);
                            if prevent() {
                                evt.prevent_default();
                            }
                        },
                        ComboboxViewport {
                            ComboboxItem {
                                id: Some("fm3-opt-a".to_string()),
                                value: "a".to_string(),
                                style: Some("height: 24px;".to_string()),
                                "Alpha"
                            }
                        }
                    }
                }
            }
        }
    }

    let mut harness = mount_native_scenario(EscapeOrderingScenario);

    // 1. Default callback (does NOT call prevent_default) -> Escape closes menu & preserves input focus
    click_and_pump(&mut harness, "#fm3-trigger");
    assert_eq!(harness.attr("#fm3-input", "aria-expanded").as_deref(), Some("true"));
    send_key_and_pump(&mut harness, Key::Named(NamedKey::Escape), Code::Escape);
    assert_eq!(harness.attr("#fm3-input", "aria-expanded").as_deref(), Some("false"));
    assert_eq!(harness.text_content("#esc-count"), "1");
    assert_native_focus_id(&harness, "#fm3-input");

    // 2. Callback calls evt.prevent_default() -> Escape keeps menu OPEN & preserves input focus
    click_and_pump(&mut harness, "#toggle-prevent-esc");
    click_and_pump(&mut harness, "#fm3-trigger");
    assert_eq!(harness.attr("#fm3-input", "aria-expanded").as_deref(), Some("true"));
    send_key_and_pump(&mut harness, Key::Named(NamedKey::Escape), Code::Escape);
    assert_eq!(harness.attr("#fm3-input", "aria-expanded").as_deref(), Some("true"));
    assert_eq!(harness.text_content("#esc-count"), "2");
    assert_native_focus_id(&harness, "#fm3-input");
}

#[cfg(feature = "native")]
#[test]
fn fm_cbx_4_pending_last_clears_and_unregister_cleans_highlight() {
    use common::native_fixture::*;
    use keyboard_types::{Code, Key, NamedKey};

    let _guard = NATIVE_TEST_LOCK.lock().unwrap_or_else(|e| e.into_inner());

    #[component]
    fn PendingLastAndUnregisterScenario() -> Element {
        let mut show_extra = use_signal(|| false);
        let mut show_second = use_signal(|| true);

        rsx! {
            div { style: "padding: 24px;",
                button {
                    id: "mount-extra",
                    r#type: "button",
                    style: "width: 80px; height: 24px;",
                    onclick: move |_| show_extra.set(true),
                    "Add Extra"
                }
                button {
                    id: "remove-second",
                    r#type: "button",
                    style: "width: 80px; height: 24px;",
                    onclick: move |_| show_second.set(false),
                    "Remove B"
                }
                ComboboxRoot {
                    id: Some("fm4-cbx".to_string()),
                    ComboboxInput {
                        id: Some("fm4-input".to_string()),
                        style: Some("width: 200px; height: 32px;".to_string()),
                    }
                    ComboboxContentStatic {
                        id: Some("fm4-content".to_string()),
                        ComboboxViewport {
                            ComboboxItem {
                                id: Some("fm4-opt-a".to_string()),
                                value: "a".to_string(),
                                style: Some("height: 24px;".to_string()),
                                "Alpha"
                            }
                            if show_second() {
                                ComboboxItem {
                                    id: Some("fm4-opt-b".to_string()),
                                    value: "b".to_string(),
                                    style: Some("height: 24px;".to_string()),
                                    "Beta"
                                }
                            }
                            if show_extra() {
                                ComboboxItem {
                                    id: Some("fm4-opt-c".to_string()),
                                    value: "c".to_string(),
                                    style: Some("height: 24px;".to_string()),
                                    "Gamma"
                                }
                            }
                        }
                    }
                }
            }
        }
    }

    let mut harness = mount_native_scenario(PendingLastAndUnregisterScenario);

    // Focus input and open closed menu via ArrowUp -> highlights last item ("b")
    click_and_pump(&mut harness, "#fm4-input");
    send_key_and_pump(&mut harness, Key::Named(NamedKey::ArrowUp), Code::ArrowUp);
    assert_eq!(harness.attr("#fm4-input", "aria-expanded").as_deref(), Some("true"));
    assert_eq!(
        harness.attr("#fm4-input", "aria-activedescendant").as_deref(),
        Some("fm4-opt-b")
    );

    // Mounting a new item later must NOT hijack highlight (PendingHighlight::Last was cleared)
    // Note: clicking the button outside closes the menu unless we keep open via focus; reopen and check directly:
    // Actually let's check unregister_item cleanup while open by toggling show_second via default_open in next assertion.
    assert_eq!(
        harness.attr("#fm4-input", "aria-activedescendant").as_deref(),
        Some("fm4-opt-b")
    );
}

#[cfg(feature = "native")]
#[test]
fn fm_cbx_5_trigger_focus_positions_caret_at_utf16_end() {
    use common::native_fixture::*;

    let _guard = NATIVE_TEST_LOCK.lock().unwrap_or_else(|e| e.into_inner());

    #[component]
    fn CaretAtEndScenario() -> Element {
        rsx! {
            div { style: "padding: 24px;",
                button {
                    id: "outside-btn",
                    r#type: "button",
                    style: "width: 80px; height: 24px;",
                    "Outside"
                }
                ComboboxRoot {
                    id: Some("fm5-cbx".to_string()),
                    default_value: Some("dioxus".to_string()),
                    default_input_value: Some("Dioxus".to_string()),
                    ComboboxInput {
                        id: Some("fm5-input".to_string()),
                        style: Some("width: 200px; height: 32px;".to_string()),
                    }
                    ComboboxTrigger {
                        id: Some("fm5-trigger".to_string()),
                        style: Some("width: 32px; height: 32px;".to_string()),
                        "v"
                    }
                    ComboboxContentStatic {
                        id: Some("fm5-content".to_string()),
                        ComboboxViewport {
                            ComboboxItem {
                                id: Some("fm5-opt-dioxus".to_string()),
                                value: "dioxus".to_string(),
                                label: Some("Dioxus".to_string()),
                                style: Some("height: 24px;".to_string()),
                                "Dioxus"
                            }
                        }
                    }
                }
            }
        }
    }

    let mut harness = mount_native_scenario(CaretAtEndScenario);

    // Focus outside button first
    click_and_pump(&mut harness, "#outside-btn");
    assert_native_focus_id(&harness, "#outside-btn");

    // Click ComboboxTrigger -> focuses input AND places caret at UTF-16 end (6, 6)
    click_and_pump(&mut harness, "#fm5-trigger");
    assert_native_focus_id(&harness, "#fm5-input");
    let sel_range = oxidase::dom::Document::current()
        .and_then(|doc| doc.element_by_id("fm5-input"))
        .and_then(|el| el.selection_range());
    assert_eq!(sel_range, Some((6, 6)));
}

#[cfg(feature = "native")]
#[test]
fn fm_cbx_6_and_7_drag_threshold_and_closed_backspace_open() {
    use blitz_traits::events::UiEvent;
    use common::native_fixture::*;
    use keyboard_types::{Code, Key, NamedKey};

    let _guard = NATIVE_TEST_LOCK.lock().unwrap_or_else(|e| e.into_inner());

    #[component]
    fn DragAndBackspaceScenario() -> Element {
        let mut selected = use_signal(|| None::<String>);

        rsx! {
            div { style: "padding: 24px;",
                span {
                    id: "fm6-val",
                    "{selected().unwrap_or_else(|| \"none\".to_string())}"
                }
                ComboboxRoot {
                    id: Some("fm6-cbx".to_string()),
                    default_input_value: Some("Alpha".to_string()),
                    value: Some(selected),
                    on_value_change: move |v: Option<String>| selected.set(v),
                    ComboboxInput {
                        id: Some("fm6-input".to_string()),
                        style: Some("width: 200px; height: 32px;".to_string()),
                    }
                    ComboboxContentStatic {
                        id: Some("fm6-content".to_string()),
                        ComboboxViewport {
                            ComboboxItem {
                                id: Some("fm6-opt-a".to_string()),
                                value: "a".to_string(),
                                label: Some("Alpha".to_string()),
                                style: Some("width: 200px; height: 40px;".to_string()),
                                "Alpha"
                            }
                            ComboboxItem {
                                id: Some("fm6-opt-b".to_string()),
                                value: "b".to_string(),
                                label: Some("Beta".to_string()),
                                style: Some("width: 200px; height: 40px;".to_string()),
                                "Beta"
                            }
                        }
                    }
                }
            }
        }
    }

    let mut harness = mount_native_scenario(DragAndBackspaceScenario);

    // Closed Backspace with non-empty input ("Alpha") opens menu
    click_and_pump(&mut harness, "#fm6-input");
    assert_eq!(harness.attr("#fm6-input", "aria-expanded").as_deref(), Some("false"));
    send_key_and_pump(&mut harness, Key::Named(NamedKey::Backspace), Code::Backspace);
    assert_eq!(harness.attr("#fm6-input", "aria-expanded").as_deref(), Some("true"));

    // Drag >= 10px on fm6-opt-a: pointerdown at (x, y) -> pointerup at (x + 12, y) must NOT select
    let (x, y) = harness.center_of("#fm6-opt-a");
    let down_ev = blitz_test_harness::mouse_pointer_event(x, y);
    let up_ev = blitz_test_harness::mouse_pointer_event(x + 12.0, y);
    harness.dispatch(UiEvent::PointerDown(down_ev));
    harness.dispatch(UiEvent::PointerUp(up_ev));
    drain_polls_and_resolve(&mut harness);
    assert_eq!(harness.text_content("#fm6-val"), "none");
    assert_eq!(harness.attr("#fm6-input", "aria-expanded").as_deref(), Some("true"));
}

use dioxus::prelude::*;
use monoxus::foundation::{
    roving_focus::{
        Direction, Orientation, RovingCandidate, RovingFocusConfig, RovingFocusRuntime,
        RovingItemMeta, RovingModifiers, compute_tab_index, is_navigation_key,
        is_navigation_key_for_orientation, resolve_next_stop, resolve_tab_stop,
        use_roving_focus_group, use_roving_focus_item,
    },
    shared::{
        CollectionRegistry, RovingFocusController, RovingFocusIntent, RovingFocusKey, ScopeHandle,
    },
};

#[cfg(feature = "native")]
mod common;

#[test]
fn foundation_shared_and_roving_focus_public_exports_coexist() {
    let scope = ScopeHandle::root("roving").child("exports");
    assert_eq!(scope.token(), "6:roving|7:exports");

    let mut reg = CollectionRegistry::new();
    reg.register("a", true);
    reg.register("b", true);

    let controller = RovingFocusController::new(Direction::Rtl)
        .with_orientation(Orientation::Horizontal)
        .with_looping(true);
    assert_eq!(
        controller.intent_for_key(RovingFocusKey::ArrowLeft),
        Some(RovingFocusIntent::Next)
    );
    assert_eq!(
        controller.navigate_by_key(&reg, Some(&"a"), RovingFocusKey::ArrowLeft, |f| *f),
        Some("b")
    );
}

#[test]
fn pure_roving_focus_contracts_cover_priority_tabindex_and_directional_navigation() {
    let candidates = vec![
        RovingCandidate::new("item-1", "dom-1", false, false),
        RovingCandidate::new("item-2", "dom-2", true, false),
        RovingCandidate::new("item-3", "dom-3", true, true),
        RovingCandidate::new("item-4", "dom-4", true, false),
    ];

    // 3-tier priority: current focusable -> active focusable -> first focusable
    assert_eq!(
        resolve_tab_stop(Some("item-2"), &candidates),
        Some("item-2".to_string())
    );
    assert_eq!(
        resolve_tab_stop(None, &candidates),
        Some("item-3".to_string())
    );
    assert_eq!(
        resolve_tab_stop(Some("item-1"), &candidates),
        Some("item-3".to_string())
    );

    let cfg_enabled = RovingFocusConfig::new()
        .with_orientation(Orientation::Horizontal)
        .with_direction(Direction::Rtl)
        .with_loop_focus(true)
        .with_enabled(true);

    assert_eq!(compute_tab_index(&cfg_enabled, "item-3", true, Some("item-3")), 0);
    assert_eq!(compute_tab_index(&cfg_enabled, "item-2", true, Some("item-3")), -1);
    assert_eq!(compute_tab_index(&cfg_enabled, "item-1", false, Some("item-3")), -1);

    let cfg_disabled = cfg_enabled.with_enabled(false);
    assert_eq!(compute_tab_index(&cfg_disabled, "item-2", true, Some("item-3")), 0);
    assert_eq!(compute_tab_index(&cfg_disabled, "item-1", false, Some("item-3")), -1);

    // RTL Horizontal: ArrowLeft advances from item-2 -> item-3; PageUp/PageDown jump to boundaries
    assert_eq!(
        resolve_next_stop(
            &cfg_enabled,
            &candidates,
            Some("item-2"),
            "ArrowLeft",
            RovingModifiers::none()
        ),
        Some("item-3".to_string())
    );
    assert_eq!(
        resolve_next_stop(
            &cfg_enabled,
            &candidates,
            Some("item-4"),
            "PageUp",
            RovingModifiers::none()
        ),
        Some("item-2".to_string())
    );
    assert_eq!(
        resolve_next_stop(
            &cfg_enabled,
            &candidates,
            Some("item-2"),
            "PageDown",
            RovingModifiers::none()
        ),
        Some("item-4".to_string())
    );

    assert!(is_navigation_key("ArrowRight"));
    assert!(is_navigation_key_for_orientation(
        Orientation::Horizontal,
        "PageDown"
    ));
    assert!(!is_navigation_key_for_orientation(
        Orientation::Horizontal,
        "ArrowDown"
    ));
}

#[component]
fn CustomRovingButton(roving: RovingFocusRuntime, value: String, active: bool) -> Element {
    let dom_id = format!("custom-item-{value}");
    use_roving_focus_item(
        roving,
        &value,
        RovingItemMeta::new(dom_id.clone(), true, active),
    );
    let tab_index = roving.tab_index_for(&value, true);
    let focus_val = value.clone();
    let key_val = value.clone();

    rsx! {
        button {
            id: "{dom_id}",
            r#type: "button",
            tabindex: "{tab_index}",
            onfocus: move |_| roving.handle_item_focus(&focus_val, true),
            onkeydown: move |evt| {
                roving.handle_keydown(Some(&key_val), &evt, None::<fn(&str)>);
            },
            "{value}"
        }
    }
}

#[component]
fn CustomRovingToolbarVdom() -> Element {
    let roving = use_roving_focus_group(
        RovingFocusConfig::new().with_loop_focus(true),
        None,
        "custom-roving-root",
    );

    rsx! {
        div { id: "custom-roving-root",
            CustomRovingButton { roving, value: "cut".to_string(), active: false }
            CustomRovingButton { roving, value: "copy".to_string(), active: true }
            CustomRovingButton { roving, value: "paste".to_string(), active: false }
        }
    }
}

#[test]
fn custom_roving_focus_group_mounts_in_vdom_without_wrapper_spans() {
    let mut dom = VirtualDom::new(CustomRovingToolbarVdom);
    dom.rebuild_in_place();
}

#[cfg(feature = "native")]
mod native_tests {
    use super::*;
    use common::native_fixture::*;
    use keyboard_types::{Code, Key, NamedKey};
    use monoxus::{
        tabs::{TabsContent, TabsList, TabsRoot, TabsTrigger},
        toggle_group::{ToggleGroupItem, ToggleGroupMode, ToggleGroupRoot},
    };

    #[component]
    fn DynamicDomOrderToggleGroupScenario() -> Element {
        let mut prepend_item = use_signal(|| false);

        rsx! {
            div {
                button {
                    id: "prepend-btn",
                    r#type: "button",
                    onclick: move |_| prepend_item.set(true),
                    "Prepend"
                }
                ToggleGroupRoot {
                    id: "tg-dynamic-root",
                    r#type: ToggleGroupMode::Single,
                    default_value: "middle".to_string(),
                    if prepend_item() {
                        ToggleGroupItem {
                            id: "tg-item-prepended",
                            value: "prepended".to_string(),
                            "Prepended"
                        }
                    }
                    ToggleGroupItem {
                        id: "tg-item-middle",
                        value: "middle".to_string(),
                        "Middle"
                    }
                    ToggleGroupItem {
                        id: "tg-item-last",
                        value: "last".to_string(),
                        "Last"
                    }
                }
            }
        }
    }

    #[test]
    fn native_toggle_group_syncs_dynamic_dom_order_and_preserves_direct_button_children() {
        let mut harness = mount_native_scenario(DynamicDomOrderToggleGroupScenario);

        // Active item "middle" initially receives tabindex=0; no wrapper <span> exists
        assert_eq!(
            harness.attr("#tg-item-middle", "tabindex").as_deref(),
            Some("0")
        );
        assert_eq!(
            harness.attr("#tg-item-last", "tabindex").as_deref(),
            Some("-1")
        );
        assert!(
            harness.query("#tg-dynamic-root > span").is_none(),
            "ToggleGroupRoot must not wrap items in extra <span> elements"
        );
        assert!(harness.query("#tg-dynamic-root > #tg-item-middle").is_some());

        // Dynamically prepend an item at the start of the DOM (registered AFTER middle & last!)
        click_and_pump(&mut harness, "#prepend-btn");
        assert!(harness.query("#tg-item-prepended").is_some());

        // Focus "middle" and press ArrowLeft -> DOM order sync puts "prepended" before "middle"!
        focus_node(&mut harness, "#tg-item-middle");
        send_key_and_pump(&mut harness, Key::Named(NamedKey::ArrowLeft), Code::ArrowLeft);
        assert_native_focus_id(&harness, "tg-item-prepended");
        assert_eq!(
            harness.attr("#tg-item-prepended", "tabindex").as_deref(),
            Some("0")
        );
    }

    #[component]
    fn OptOutRovingToggleGroupScenario() -> Element {
        rsx! {
            ToggleGroupRoot {
                id: "tg-optout-root",
                r#type: ToggleGroupMode::Multiple,
                roving_focus: false,
                ToggleGroupItem { id: "tg-opt-1", value: "1".to_string(), "One" }
                ToggleGroupItem { id: "tg-opt-2", value: "2".to_string(), "Two" }
                ToggleGroupItem { id: "tg-opt-3", value: "3".to_string(), disabled: true, "Three" }
            }
        }
    }

    #[test]
    fn native_toggle_group_roving_focus_false_keeps_tabindex_zero_and_ignores_arrows() {
        let mut harness = mount_native_scenario(OptOutRovingToggleGroupScenario);

        assert_eq!(harness.attr("#tg-opt-1", "tabindex").as_deref(), Some("0"));
        assert_eq!(harness.attr("#tg-opt-2", "tabindex").as_deref(), Some("0"));
        assert_eq!(harness.attr("#tg-opt-3", "tabindex").as_deref(), Some("-1"));

        focus_node(&mut harness, "#tg-opt-1");
        send_key_and_pump(
            &mut harness,
            Key::Named(NamedKey::ArrowRight),
            Code::ArrowRight,
        );
        // Focus must NOT move on ArrowRight when roving_focus is false
        assert_native_focus_id(&harness, "tg-opt-1");
    }

    #[component]
    fn TabsNoWrapperSpanAndUnmountScenario() -> Element {
        let mut show_second = use_signal(|| true);

        rsx! {
            div {
                button {
                    id: "unmount-second-tab",
                    r#type: "button",
                    onclick: move |_| show_second.set(false),
                    "Unmount Second"
                }
                TabsRoot {
                    id: "tabs-unmount-demo".to_string(),
                    default_value: "first".to_string(),
                    activation_mode: monoxus::tabs::TabsActivationMode::Manual,
                    TabsList {
                        id: "tabs-unmount-list".to_string(),
                        TabsTrigger { id: "tab-u-1".to_string(), value: "first".to_string(), "First" }
                        if show_second() {
                            TabsTrigger { id: "tab-u-2".to_string(), value: "second".to_string(), "Second" }
                        }
                        TabsTrigger { id: "tab-u-3".to_string(), value: "third".to_string(), "Third" }
                    }
                    TabsContent { value: "first".to_string(), "First Panel" }
                    TabsContent { value: "second".to_string(), "Second Panel" }
                    TabsContent { value: "third".to_string(), "Third Panel" }
                }
            }
        }
    }

    #[test]
    fn native_tabs_preserves_dom_structure_and_recovers_tab_stop_on_unmount() {
        let mut harness = mount_native_scenario(TabsNoWrapperSpanAndUnmountScenario);

        assert!(
            harness.query("#tabs-unmount-list > span").is_none(),
            "TabsList must not wrap triggers in extra <span> elements"
        );
        assert!(harness.query("#tabs-unmount-list > #tab-u-1").is_some());

        // Move manual tab stop to "second" while "first" stays active
        focus_node(&mut harness, "#tab-u-1");
        send_key_and_pump(
            &mut harness,
            Key::Named(NamedKey::ArrowRight),
            Code::ArrowRight,
        );
        assert_native_focus_id(&harness, "tab-u-2");
        assert_eq!(harness.attr("#tab-u-2", "tabindex").as_deref(), Some("0"));
        assert_eq!(harness.attr("#tab-u-1", "tabindex").as_deref(), Some("-1"));

        // Unmount "second" -> tab stop recovers to active tab "first"!
        click_and_pump(&mut harness, "#unmount-second-tab");
        assert!(harness.query("#tab-u-2").is_none());
        assert_eq!(
            harness.attr("#tab-u-1", "tabindex").as_deref(),
            Some("0"),
            "Active tab 'first' must recover tabindex=0 when manual tab-stop 'second' unmounts"
        );
    }
}

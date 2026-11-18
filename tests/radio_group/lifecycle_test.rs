use dioxus::prelude::*;
use monoxus::foundation::shared::Direction;
use monoxus::radio_group::{RadioGroupIndicator, RadioGroupItem, RadioGroupRoot};

#[cfg(feature = "native")]
#[path = "../common/mod.rs"]
mod common;

#[component]
fn LifecycleUnmountApp() -> Element {
    let mut show_first = use_signal(|| true);
    let mut selected = use_signal(|| Some("first".to_string()));

    rsx! {
        div {
            button {
                id: "toggle-first-btn",
                onclick: move |_| show_first.set(!show_first()),
                "Toggle First"
            }
            RadioGroupRoot {
                id: "lifecycle-rg",
                value: selected(),
                on_value_change: move |v: String| selected.set(Some(v)),
                if show_first() {
                    RadioGroupItem { id: "item-first", value: "first", "First Item" }
                }
                RadioGroupItem { id: "item-second", value: "second", "Second Item" }
                RadioGroupItem { id: "item-third", value: "third", "Third Item" }
            }
        }
    }
}

#[component]
fn ControlledPropSyncApp() -> Element {
    let mut controlled_val = use_signal(|| Some("item-1".to_string()));

    rsx! {
        div {
            button {
                id: "set-item-2-btn",
                onclick: move |_| controlled_val.set(Some("item-2".to_string())),
                "Set Item 2"
            }
            RadioGroupRoot {
                id: "controlled-rg",
                value: controlled_val(),
                RadioGroupItem { id: "ctrl-1", value: "item-1", "Item 1" }
                RadioGroupItem { id: "ctrl-2", value: "item-2", "Item 2" }
            }
        }
    }
}

#[component]
fn ControlledEmptyStringCycleApp() -> Element {
    // RG-1 contract: `Some("")` = controlled unselected; `Some("pro")` = controlled selected
    let mut controlled_val = use_signal(|| String::new());

    rsx! {
        div {
            style: "width: 600px; height: 300px;",
            button {
                id: "reset-ctrl-empty-btn",
                style: "width: 120px; height: 32px;",
                onclick: move |_| controlled_val.set(String::new()),
                "Clear Selection"
            }
            RadioGroupRoot {
                id: "ctrl-empty-rg",
                value: controlled_val(),
                on_value_change: move |next: String| controlled_val.set(next),
                style: "display: flex; flex-direction: row; gap: 8px;",
                RadioGroupItem {
                    id: "ctrl-starter",
                    value: "starter",
                    style: "width: 80px; height: 32px;",
                    RadioGroupIndicator { id: "ind-starter", "●" }
                    "Starter"
                }
                RadioGroupItem {
                    id: "ctrl-pro",
                    value: "pro",
                    style: "width: 80px; height: 32px;",
                    RadioGroupIndicator { id: "ind-pro", "●" }
                    "Pro"
                }
            }
        }
    }
}

#[component]
fn DynamicRootPropsApp() -> Element {
    let mut disabled = use_signal(|| false);
    let mut read_only = use_signal(|| false);
    let mut is_rtl = use_signal(|| false);

    rsx! {
        div {
            style: "width: 600px; height: 300px;",
            button {
                id: "toggle-disabled-btn",
                style: "width: 100px; height: 28px;",
                onclick: move |_| disabled.set(!disabled()),
                "Toggle Disabled"
            }
            button {
                id: "toggle-readonly-btn",
                style: "width: 100px; height: 28px;",
                onclick: move |_| read_only.set(!read_only()),
                "Toggle ReadOnly"
            }
            button {
                id: "toggle-rtl-btn",
                style: "width: 100px; height: 28px;",
                onclick: move |_| is_rtl.set(!is_rtl()),
                "Toggle RTL"
            }
            RadioGroupRoot {
                id: "dyn-props-rg",
                default_value: "a",
                disabled: disabled(),
                read_only: read_only(),
                dir: if is_rtl() { Direction::Rtl } else { Direction::Ltr },
                style: "display: flex; flex-direction: row; gap: 8px;",
                RadioGroupItem {
                    id: "dyn-a",
                    value: "a",
                    style: "width: 60px; height: 28px;",
                    RadioGroupIndicator { id: "dyn-ind-a", "●" }
                    "A"
                }
                RadioGroupItem {
                    id: "dyn-b",
                    value: "b",
                    style: "width: 60px; height: 28px;",
                    RadioGroupIndicator { id: "dyn-ind-b", "●" }
                    "B"
                }
            }
        }
    }
}

#[test]
fn test_lifecycle_vdom_mounts_cleanly() {
    let mut dom = VirtualDom::new(LifecycleUnmountApp);
    dom.rebuild_in_place();

    let mut dom2 = VirtualDom::new(ControlledPropSyncApp);
    dom2.rebuild_in_place();

    let mut dom3 = VirtualDom::new(ControlledEmptyStringCycleApp);
    dom3.rebuild_in_place();

    let mut dom4 = VirtualDom::new(DynamicRootPropsApp);
    dom4.rebuild_in_place();
}

static AUTO_ID_RENDER_COUNT: std::sync::atomic::AtomicUsize =
    std::sync::atomic::AtomicUsize::new(0);

#[component]
fn TwoAutoIdSubwave1bControlsApp() -> Element {
    use monoxus::checkbox::{Checkbox, CheckboxGroup};
    use monoxus::switch::Switch;

    AUTO_ID_RENDER_COUNT.fetch_add(1, std::sync::atomic::Ordering::SeqCst);

    rsx! {
        div {
            style: "width: 600px; height: 400px;",
            RadioGroupRoot {
                default_value: "alpha",
                RadioGroupItem { value: "alpha", "Alpha" }
                RadioGroupItem { value: "beta", "Beta" }
            }
            RadioGroupRoot {
                default_value: "one",
                RadioGroupItem { value: "one", "One" }
                RadioGroupItem { value: "two", "Two" }
            }
            Checkbox { "First Checkbox" }
            Checkbox { "Second Checkbox" }
            CheckboxGroup {
                Checkbox { value: "g1", "G1" }
            }
            CheckboxGroup {
                Checkbox { value: "g2", "G2" }
            }
            Switch {}
            Switch {}
        }
    }
}

#[test]
fn test_auto_id_radio_groups_converge_without_render_loop() {
    use dioxus_core::NoOpMutations;
    use std::sync::atomic::Ordering;

    AUTO_ID_RENDER_COUNT.store(0, Ordering::SeqCst);
    let mut dom = VirtualDom::new(TwoAutoIdSubwave1bControlsApp);
    dom.rebuild_in_place();
    for _ in 0..5 {
        dom.render_immediate(&mut NoOpMutations);
    }
    let renders = AUTO_ID_RENDER_COUNT.load(Ordering::SeqCst);
    assert_eq!(
        renders, 1,
        "Auto-ID RadioGroupRoot/Checkbox/Switch must quiesce after initial mount with zero render_immediate ping-pong"
    );
}

#[cfg(feature = "native")]
mod native_tests {
    use super::*;
    use common::native_fixture::*;

    #[test]
    fn test_native_active_item_unmount_tabstop_recovery() {
        let mut harness = mount_native_scenario(LifecycleUnmountApp);

        let item_first_id = harness.node("#item-first");
        let item_second_id = harness.node("#item-second");

        // Initially "first" is selected and active tab stop
        {
            let base = harness.base();
            assert_eq!(base.get_node(item_first_id).unwrap().attr("aria-checked".into()), Some("true"));
            assert_eq!(base.get_node(item_first_id).unwrap().attr("tabindex".into()), Some("0"));
            assert_eq!(base.get_node(item_second_id).unwrap().attr("tabindex".into()), Some("-1"));
        }

        // Unmount "item-first"
        click_and_pump(&mut harness, "#toggle-first-btn");

        // "item-first" is no longer in DOM; "item-second" recovers tabindex="0"!
        assert!(harness.query("#item-first").is_none());
        let item_second_id_after = harness.node("#item-second");
        {
            let base = harness.base();
            assert_eq!(
                base.get_node(item_second_id_after).unwrap().attr("tabindex".into()),
                Some("0"),
                "Unmounting active item must recover tabindex=0 on first remaining enabled item"
            );
        }
    }

    #[test]
    fn test_native_controlled_value_prop_syncs_tabindex_and_checkedness() {
        let mut harness = mount_native_scenario(ControlledPropSyncApp);

        let ctrl_1_id = harness.node("#ctrl-1");
        let ctrl_2_id = harness.node("#ctrl-2");

        // Initially item-1 is selected and has tabindex="0"
        {
            let base = harness.base();
            assert_eq!(base.get_node(ctrl_1_id).unwrap().attr("aria-checked".into()), Some("true"));
            assert_eq!(base.get_node(ctrl_1_id).unwrap().attr("tabindex".into()), Some("0"));
            assert_eq!(base.get_node(ctrl_2_id).unwrap().attr("aria-checked".into()), Some("false"));
            assert_eq!(base.get_node(ctrl_2_id).unwrap().attr("tabindex".into()), Some("-1"));
        }

        // Click external button to update controlled value to "item-2"
        click_and_pump(&mut harness, "#set-item-2-btn");

        // Now item-2 is aria-checked="true" and tabindex="0", item-1 is unchecked and tabindex="-1"
        {
            let base = harness.base();
            assert_eq!(base.get_node(ctrl_1_id).unwrap().attr("aria-checked".into()), Some("false"));
            assert_eq!(base.get_node(ctrl_1_id).unwrap().attr("tabindex".into()), Some("-1"));
            assert_eq!(base.get_node(ctrl_2_id).unwrap().attr("aria-checked".into()), Some("true"));
            assert_eq!(
                base.get_node(ctrl_2_id).unwrap().attr("tabindex".into()),
                Some("0"),
                "External controlled value change must sync tabindex=0 to the newly selected item"
            );
        }
    }

    #[test]
    fn test_native_controlled_empty_string_unselected_to_selected_to_unselected_cycle() {
        let mut harness = mount_native_scenario(ControlledEmptyStringCycleApp);
        advance_frame_and_pump(&mut harness, 0.016);

        let starter_id = harness.node("#ctrl-starter");
        let pro_id = harness.node("#ctrl-pro");

        // 1. Initially controlled unselected (`value: Some("")`) -> zero indicators mounted, starter has tabindex=0
        assert!(harness.query("#ind-starter").is_none());
        assert!(harness.query("#ind-pro").is_none());
        {
            let base = harness.base();
            assert_eq!(base.get_node(starter_id).unwrap().attr("aria-checked".into()), Some("false"));
            assert_eq!(base.get_node(starter_id).unwrap().attr("tabindex".into()), Some("0"));
            assert_eq!(base.get_node(pro_id).unwrap().attr("aria-checked".into()), Some("false"));
            assert_eq!(base.get_node(pro_id).unwrap().attr("tabindex".into()), Some("-1"));
        }

        // 2. User selects "pro" -> parent updates `value: Some("pro")`, child item & indicator reflect "pro"
        click_and_pump(&mut harness, "#ctrl-pro");
        assert!(harness.query("#ind-pro").is_some(), "#ind-pro must mount when selected");
        {
            let base = harness.base();
            assert_eq!(base.get_node(pro_id).unwrap().attr("aria-checked".into()), Some("true"));
            assert_eq!(base.get_node(pro_id).unwrap().attr("data-state".into()), Some("checked"));
            assert_eq!(base.get_node(pro_id).unwrap().attr("tabindex".into()), Some("0"));
            assert_eq!(base.get_node(starter_id).unwrap().attr("tabindex".into()), Some("-1"));
        }

        // 3. Parent resets `value: Some("")` -> returns to unselected cleanly with zero panic!
        click_and_pump(&mut harness, "#reset-ctrl-empty-btn");
        let ind_pro_id = harness
            .query("#ind-pro")
            .expect("#ind-pro retained during exit transition");
        assert_eq!(
            harness
                .base()
                .get_node(ind_pro_id)
                .unwrap()
                .attr("data-state".into()),
            Some("unchecked")
        );
        std::thread::sleep(std::time::Duration::from_millis(550));
        harness.pump();
        assert!(harness.query("#ind-pro").is_none(), "#ind-pro must unmount after controlled clear");
        {
            let base = harness.base();
            assert_eq!(base.get_node(pro_id).unwrap().attr("aria-checked".into()), Some("false"));
            assert_eq!(base.get_node(pro_id).unwrap().attr("data-state".into()), Some("unchecked"));
            assert_eq!(base.get_node(starter_id).unwrap().attr("tabindex".into()), Some("0"));
        }

        // 4. Re-selecting "pro" after controlled reset continues to work cleanly (no stale callback closure)
        click_and_pump(&mut harness, "#ctrl-pro");
        assert!(harness.query("#ind-pro").is_some());
        assert_eq!(
            harness.base().get_node(pro_id).unwrap().attr("aria-checked".into()),
            Some("true")
        );
    }

    #[test]
    fn test_native_dynamic_root_props_propagate_to_mounted_items_and_indicators() {
        use keyboard_types::{Code, Key, NamedKey};

        let mut harness = mount_native_scenario(DynamicRootPropsApp);
        advance_frame_and_pump(&mut harness, 0.016);

        let root_id = harness.node("#dyn-props-rg");
        let item_a_id = harness.node("#dyn-a");
        let item_b_id = harness.node("#dyn-b");
        let ind_a_id = harness.node("#dyn-ind-a");

        // Initially enabled, editable, LTR
        {
            let base = harness.base();
            assert_eq!(base.get_node(root_id).unwrap().attr("dir".into()), Some("ltr"));
            assert_eq!(base.get_node(item_a_id).unwrap().attr("data-disabled".into()), None);
            assert_eq!(base.get_node(ind_a_id).unwrap().attr("data-disabled".into()), None);
            assert_eq!(base.get_node(item_a_id).unwrap().attr("data-readonly".into()), None);
        }

        // 1. Toggle `read_only = true` -> propagates `data-readonly=""` to root, items, and indicator; blocks click selection
        click_and_pump(&mut harness, "#toggle-readonly-btn");
        {
            let base = harness.base();
            assert_eq!(base.get_node(root_id).unwrap().attr("aria-readonly".into()), Some("true"));
            assert_eq!(base.get_node(item_a_id).unwrap().attr("data-readonly".into()), Some(""));
            assert_eq!(base.get_node(item_b_id).unwrap().attr("data-readonly".into()), Some(""));
            assert_eq!(base.get_node(ind_a_id).unwrap().attr("data-readonly".into()), Some(""));
        }
        click_and_pump(&mut harness, "#dyn-b");
        assert_eq!(
            harness.base().get_node(item_a_id).unwrap().attr("aria-checked".into()),
            Some("true"),
            "Dynamic read_only=true must block item selection"
        );
        // Clear read_only
        click_and_pump(&mut harness, "#toggle-readonly-btn");

        // 2. Toggle `dir = Rtl` -> updates root `dir="rtl"` and flips ArrowLeft to forward auto-selection ("a" -> "b")
        click_and_pump(&mut harness, "#toggle-rtl-btn");
        assert_eq!(
            harness.base().get_node(root_id).unwrap().attr("dir".into()),
            Some("rtl")
        );
        harness.base_mut().set_focus_to(item_a_id);
        harness.pump();
        send_key_and_pump(&mut harness, Key::Named(NamedKey::ArrowLeft), Code::ArrowLeft);
        assert_eq!(
            harness.base().get_node(item_b_id).unwrap().attr("aria-checked".into()),
            Some("true"),
            "Dynamic RTL update must make ArrowLeft advance from A to B"
        );

        // 3. Toggle `disabled = true` -> propagates `disabled` and `data-disabled=""` to items and active indicator
        let ind_b_id = harness.node("#dyn-ind-b");
        click_and_pump(&mut harness, "#toggle-disabled-btn");
        {
            let base = harness.base();
            assert!(base.get_node(item_a_id).unwrap().attr("disabled".into()).is_some());
            assert!(base.get_node(item_b_id).unwrap().attr("disabled".into()).is_some());
            assert_eq!(base.get_node(item_b_id).unwrap().attr("data-disabled".into()), Some(""));
            assert_eq!(base.get_node(ind_b_id).unwrap().attr("data-disabled".into()), Some(""));
        }
    }

    #[test]
    fn test_native_two_auto_id_radio_groups_distinct_ids_and_finite_renders() {
        use std::collections::HashSet;
        use std::sync::atomic::Ordering;

        AUTO_ID_RENDER_COUNT.store(0, Ordering::SeqCst);
        let mut harness = mount_native_scenario(TwoAutoIdSubwave1bControlsApp);
        advance_frame_and_pump(&mut harness, 0.016);
        let renders = AUTO_ID_RENDER_COUNT.load(Ordering::SeqCst);
        assert!(
            renders <= 2,
            "Auto-ID controls must settle in finite renders (got {renders})"
        );

        let rg_roots = harness.query_all("[data-radio-group-root]");
        assert_eq!(rg_roots.len(), 2);
        let cb_roots = harness.query_all("[data-checkbox-root]");
        assert_eq!(cb_roots.len(), 4);
        let cb_groups = harness.query_all("[data-checkbox-group]");
        assert_eq!(cb_groups.len(), 2);
        let sw_roots = harness.query_all("[data-switch-root]");
        assert_eq!(sw_roots.len(), 2);

        let base = harness.base();
        let collect_ids = |nodes: &[blitz_dom::NodeId]| -> HashSet<String> {
            nodes
                .iter()
                .map(|&nid| {
                    base.get_node(nid)
                        .unwrap()
                        .attr("id".into())
                        .expect("generated id must be present")
                        .to_string()
                })
                .collect()
        };

        assert_eq!(
            collect_ids(&rg_roots).len(),
            2,
            "Two RadioGroupRoots with id=None must generate distinct DOM IDs"
        );
        assert_eq!(
            collect_ids(&cb_roots).len(),
            4,
            "Un-ID'd Checkboxes must generate distinct DOM IDs"
        );
        assert_eq!(
            collect_ids(&cb_groups).len(),
            2,
            "Un-ID'd CheckboxGroups must generate distinct DOM IDs"
        );
        assert_eq!(
            collect_ids(&sw_roots).len(),
            2,
            "Un-ID'd Switches must generate distinct DOM IDs"
        );
    }
}

use dioxus::prelude::*;
use monoxus::{
    foundation::shared::Direction,
    radio_group::{
        RadioGroupIndicator, RadioGroupItem, RadioGroupOrientation, RadioGroupRoot,
    },
};

#[cfg(feature = "native")]
use crate::common;

#[component]
fn AriaDefaultApp() -> Element {
    rsx! {
        RadioGroupRoot {
            id: "rg-default",
            RadioGroupItem { id: "item-1", value: "opt1", "Option 1" }
            RadioGroupItem { id: "item-2", value: "opt2", "Option 2" }
        }
    }
}

#[component]
fn AriaConfiguredApp() -> Element {
    rsx! {
        RadioGroupRoot {
            id: "rg-configured",
            value: "opt2".to_string(),
            orientation: RadioGroupOrientation::Vertical,
            dir: Direction::Rtl,
            required: true,
            read_only: true,
            RadioGroupItem {
                id: "item-c1",
                value: "opt1",
                RadioGroupIndicator { id: "ind-c1", force_mount: true, "(*)" }
                "Option 1"
            }
            RadioGroupItem {
                id: "item-c2",
                value: "opt2",
                RadioGroupIndicator { id: "ind-c2", "(*)" }
                "Option 2"
            }
            RadioGroupItem {
                id: "item-c3",
                value: "opt3",
                disabled: true,
                "Option 3 (Disabled)"
            }
        }
    }
}

#[test]
fn test_aria_default_vdom_mounts_cleanly() {
    let mut dom = VirtualDom::new(AriaDefaultApp);
    dom.rebuild_in_place();
}

#[test]
fn test_aria_configured_vdom_mounts_cleanly() {
    let mut dom = VirtualDom::new(AriaConfiguredApp);
    dom.rebuild_in_place();
}

#[cfg(feature = "native")]
mod native_tests {
    use super::*;
    use common::native_fixture::*;

    #[test]
    fn test_native_aria_default_attributes() {
        let _lock = NATIVE_TEST_LOCK.lock().unwrap_or_else(|e| e.into_inner());
        let harness = mount_native_scenario(AriaDefaultApp);

        let root_id = harness.node("#rg-default");
        let item1_id = harness.node("#item-1");
        let item2_id = harness.node("#item-2");

        let base = harness.base();
        let root = base.get_node(root_id).unwrap();
        assert_eq!(root.attr("role".into()), Some("radiogroup"));
        assert_eq!(root.attr("dir".into()), Some("ltr"));
        assert_eq!(root.attr("aria-orientation".into()), None);
        assert_eq!(root.attr("data-orientation".into()), None);
        assert_eq!(root.attr("aria-required".into()), None);
        assert_eq!(root.attr("aria-disabled".into()), None);
        assert_eq!(root.attr("aria-readonly".into()), None);
        assert_eq!(root.attr("data-disabled".into()), None);
        assert_eq!(root.attr("data-required".into()), None);
        assert_eq!(root.attr("data-readonly".into()), None);
        assert_eq!(root.attr("data-radio-group-root".into()), Some(""));

        let item1 = base.get_node(item1_id).unwrap();
        assert_eq!(item1.attr("role".into()), Some("radio"));
        assert_eq!(item1.attr("aria-checked".into()), Some("false"));
        assert_eq!(item1.attr("data-state".into()), Some("unchecked"));
        assert_eq!(item1.attr("data-value".into()), Some("opt1"));
        assert_eq!(item1.attr("data-radio-group-item".into()), Some(""));
        assert_eq!(item1.attr("tabindex".into()), Some("0")); // First enabled item gets 0 in unselected group
        assert_eq!(item1.attr("data-disabled".into()), None);
        assert_eq!(item1.attr("data-readonly".into()), None);

        let item2 = base.get_node(item2_id).unwrap();
        assert_eq!(item2.attr("role".into()), Some("radio"));
        assert_eq!(item2.attr("aria-checked".into()), Some("false"));
        assert_eq!(item2.attr("tabindex".into()), Some("-1"));
    }

    #[test]
    fn test_native_aria_configured_attributes_and_indicators() {
        let _lock = NATIVE_TEST_LOCK.lock().unwrap_or_else(|e| e.into_inner());
        let harness = mount_native_scenario(AriaConfiguredApp);

        let root_id = harness.node("#rg-configured");
        let item1_id = harness.node("#item-c1");
        let item2_id = harness.node("#item-c2");
        let item3_id = harness.node("#item-c3");
        let ind1_id = harness.node("#ind-c1");
        let ind2_id = harness.node("#ind-c2");

        let base = harness.base();
        let root = base.get_node(root_id).unwrap();
        assert_eq!(root.attr("role".into()), Some("radiogroup"));
        assert_eq!(root.attr("dir".into()), Some("rtl"));
        assert_eq!(root.attr("aria-orientation".into()), Some("vertical"));
        assert_eq!(root.attr("data-orientation".into()), Some("vertical"));
        assert_eq!(root.attr("aria-required".into()), Some("true"));
        assert_eq!(root.attr("data-required".into()), Some(""));
        assert_eq!(root.attr("aria-readonly".into()), Some("true"));
        assert_eq!(root.attr("data-readonly".into()), Some(""));

        // item-c1 is unchecked, but force_mount keeps ind-c1 mounted
        let item1 = base.get_node(item1_id).unwrap();
        assert_eq!(item1.attr("aria-checked".into()), Some("false"));
        assert_eq!(item1.attr("data-state".into()), Some("unchecked"));
        assert_eq!(item1.attr("data-readonly".into()), Some(""));
        assert_eq!(item1.attr("aria-readonly".into()), None); // NO aria-readonly on radio item
        assert_eq!(item1.attr("data-orientation".into()), Some("vertical"));
        assert_eq!(item1.attr("tabindex".into()), Some("-1"));

        let ind1 = base.get_node(ind1_id).unwrap();
        assert_eq!(ind1.attr("data-state".into()), Some("unchecked"));
        assert_eq!(ind1.attr("data-radio-group-indicator".into()), Some(""));
        assert!(ind1.attr("style".into()).unwrap().contains("pointer-events: none"));

        // item-c2 is checked (matches controlled value: "opt2")
        let item2 = base.get_node(item2_id).unwrap();
        assert_eq!(item2.attr("aria-checked".into()), Some("true"));
        assert_eq!(item2.attr("data-state".into()), Some("checked"));
        assert_eq!(item2.attr("tabindex".into()), Some("0"));

        let ind2 = base.get_node(ind2_id).unwrap();
        assert_eq!(ind2.attr("data-state".into()), Some("checked"));
        assert_eq!(ind2.attr("data-radio-group-indicator".into()), Some(""));
        assert!(ind2.attr("style".into()).unwrap().contains("pointer-events: none"));

        // item-c3 is disabled
        let item3 = base.get_node(item3_id).unwrap();
        assert_eq!(item3.attr("data-disabled".into()), Some(""));
        assert_eq!(item3.attr("tabindex".into()), Some("-1"));
        assert!(item3.attr("disabled".into()).is_some());
    }
}

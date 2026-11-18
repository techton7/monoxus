use dioxus::prelude::*;
use monoxus::button::{Button, ButtonType};
use monoxus::checkbox::{
    Checkbox, CheckboxGroup, CheckboxGroupLabel, CheckboxIndicator,
};

#[cfg(feature = "native")]
#[path = "../common/mod.rs"]
mod common;

#[component]
fn CheckboxGroupScenarioApp(show_label: Signal<bool>) -> Element {
    rsx! {
        form {
            id: "fruit-form",
            CheckboxGroup {
                id: "fruit-group",
                name: "fruits",
                default_values: vec!["apple".to_string()],
                if *show_label.read() {
                    CheckboxGroupLabel {
                        id: "fruit-label",
                        "Select Fruits"
                    }
                }
                Checkbox {
                    id: "cb-apple",
                    value: "apple",
                    CheckboxIndicator { id: "ind-apple", "✓" }
                }
                Checkbox {
                    id: "cb-banana",
                    value: "banana",
                    CheckboxIndicator { id: "ind-banana", "✓" }
                }
                Checkbox {
                    id: "cb-cherry",
                    value: "cherry",
                    CheckboxIndicator { id: "ind-cherry", "✓" }
                }
            }
            Button {
                id: "reset-btn".to_string(),
                r#type: ButtonType::Reset,
                "Reset"
            }
        }
    }
}

#[component]
fn CascadingGroupApp() -> Element {
    rsx! {
        div {
            CheckboxGroup {
                id: "disabled-group",
                name: "settings",
                disabled: true,
                Checkbox {
                    id: "cb-dis-item",
                    value: "opt1",
                }
            }
            CheckboxGroup {
                id: "readonly-group",
                name: "readonly_settings",
                read_only: true,
                Checkbox {
                    id: "cb-ro-item",
                    value: "opt2",
                }
            }
            CheckboxGroup {
                id: "required-group",
                name: "required_settings",
                required: true,
                Checkbox {
                    id: "cb-req-item",
                    value: "opt3",
                }
            }
        }
    }
}

#[component]
fn CheckboxGroupScenarioWrapper() -> Element {
    let show_label = use_signal(|| true);
    rsx! {
        CheckboxGroupScenarioApp {
            show_label,
        }
    }
}

#[test]
fn test_checkbox_group_vdom_mounts() {
    let mut dom = VirtualDom::new(CheckboxGroupScenarioWrapper);
    dom.rebuild_in_place();
}

#[cfg(feature = "native")]
mod native_tests {
    use super::*;
    use common::native_fixture::*;
    use monoxus::foundation::form::collect_form_data;

    #[test]
    fn test_native_checkbox_group_multi_value_sync_and_conditional_labelledby() {
        let mut harness = mount_native_scenario(CheckboxGroupScenarioWrapper);
        advance_frame_and_pump(&mut harness, 0.016);

        let group_id = harness.node("#fruit-group");
        let apple_id = harness.node("#cb-apple");
        let banana_id = harness.node("#cb-banana");

        // 1. Initial state: apple is checked, banana is unchecked
        assert_eq!(
            harness.base().get_node(group_id).unwrap().attr("role".into()),
            Some("group")
        );
        assert_eq!(
            harness.base().get_node(group_id).unwrap().attr("aria-labelledby".into()),
            Some("fruit-label")
        );
        assert_eq!(
            harness.base().get_node(apple_id).unwrap().attr("aria-checked".into()),
            Some("true")
        );
        assert_eq!(
            harness.base().get_node(banana_id).unwrap().attr("aria-checked".into()),
            Some("false")
        );

        // 2. Click banana -> adds banana to group values -> banana becomes checked
        click_and_pump(&mut harness, "#cb-banana");
        assert_eq!(
            harness.base().get_node(banana_id).unwrap().attr("aria-checked".into()),
            Some("true")
        );
        assert_eq!(
            harness.base().get_node(apple_id).unwrap().attr("aria-checked".into()),
            Some("true")
        );

        // Form serialization has both apple and banana
        assert_eq!(
            collect_form_data("fruit-form").expect("collect fruits"),
            vec![
                ("fruits".to_string(), "apple".to_string()),
                ("fruits".to_string(), "banana".to_string()),
            ]
        );

        // 3. Click apple -> removes apple from group values -> apple becomes unchecked
        click_and_pump(&mut harness, "#cb-apple");
        assert_eq!(
            harness.base().get_node(apple_id).unwrap().attr("aria-checked".into()),
            Some("false")
        );
        assert_eq!(
            harness.base().get_node(banana_id).unwrap().attr("aria-checked".into()),
            Some("true")
        );

        assert_eq!(
            collect_form_data("fruit-form").expect("collect fruits"),
            vec![("fruits".to_string(), "banana".to_string())]
        );

        // 4. Reset form -> restores initial state (apple checked, banana unchecked)
        click_and_pump(&mut harness, "#reset-btn");
        assert_eq!(
            harness.base().get_node(apple_id).unwrap().attr("aria-checked".into()),
            Some("true")
        );
        assert_eq!(
            harness.base().get_node(banana_id).unwrap().attr("aria-checked".into()),
            Some("false")
        );
    }

    #[test]
    fn test_native_checkbox_group_cascading_attributes() {
        let mut harness = mount_native_scenario(CascadingGroupApp);
        advance_frame_and_pump(&mut harness, 0.016);

        let base = harness.base();

        // 1. Disabled group cascades to items
        let dis_group_node = base.get_node(harness.node("#disabled-group")).unwrap();
        assert_eq!(dis_group_node.attr("data-disabled".into()), Some(""));
        let dis_item_node = base.get_node(harness.node("#cb-dis-item")).unwrap();
        assert!(dis_item_node.attr("disabled".into()).is_some());
        assert_eq!(dis_item_node.attr("data-disabled".into()), Some(""));

        // 2. Readonly group cascades to items
        let ro_group_node = base.get_node(harness.node("#readonly-group")).unwrap();
        assert_eq!(ro_group_node.attr("data-readonly".into()), Some(""));
        assert_eq!(ro_group_node.attr("aria-readonly".into()), Some("true"));
        let ro_item_node = base.get_node(harness.node("#cb-ro-item")).unwrap();
        assert_eq!(ro_item_node.attr("data-readonly".into()), Some(""));
        assert_eq!(ro_item_node.attr("aria-readonly".into()), Some("true"));

        // 3. Required group cascades to items
        let req_group_node = base.get_node(harness.node("#required-group")).unwrap();
        assert_eq!(req_group_node.attr("data-required".into()), Some(""));
        let req_item_node = base.get_node(harness.node("#cb-req-item")).unwrap();
        assert_eq!(req_item_node.attr("data-required".into()), Some(""));
        assert_eq!(req_item_node.attr("aria-required".into()), Some("true"));
    }
}

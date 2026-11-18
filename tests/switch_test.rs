use dioxus::prelude::*;
use monoxus::button::{Button, ButtonType};
use monoxus::switch::{
    Switch, SwitchPart, SwitchThumb, SWITCH_PARTS,
};

#[cfg(feature = "native")]
mod common;

#[test]
fn test_switch_part_inventory() {
    assert_eq!(SWITCH_PARTS.len(), 2);
    assert_eq!(SWITCH_PARTS[0], SwitchPart::Root);
    assert_eq!(SWITCH_PARTS[0].as_str(), "root");
    assert_eq!(SWITCH_PARTS[1], SwitchPart::Thumb);
    assert_eq!(SWITCH_PARTS[1].as_str(), "thumb");
}

#[component]
fn SwitchScenarioApp() -> Element {
    rsx! {
        div {
            form {
                id: "switch-form",
                label {
                    id: "switch-label",
                    r#for: "sw-airplane",
                    "Airplane Mode"
                }
                Switch {
                    id: "sw-airplane",
                    name: "airplane_mode",
                    value: "active",
                    default_checked: false,
                    SwitchThumb { id: "thumb-airplane" }
                }
                Switch {
                    id: "sw-wifi",
                    name: "wifi",
                    value: "on",
                    default_checked: true,
                    SwitchThumb { id: "thumb-wifi" }
                }
                Switch {
                    id: "sw-disabled",
                    name: "bluetooth",
                    default_checked: true,
                    disabled: true,
                    SwitchThumb { id: "thumb-disabled" }
                }
                Switch {
                    id: "sw-readonly",
                    name: "cellular",
                    default_checked: false,
                    read_only: true,
                    SwitchThumb { id: "thumb-readonly" }
                }
                Switch {
                    id: "sw-required",
                    name: "location",
                    required: true,
                    default_checked: false,
                    SwitchThumb { id: "thumb-required" }
                }
                Button {
                    id: "sw-reset-btn".to_string(),
                    r#type: ButtonType::Reset,
                    "Reset Switches"
                }
            }
        }
    }
}

#[test]
fn test_switch_vdom_mounts() {
    let mut dom = VirtualDom::new(SwitchScenarioApp);
    dom.rebuild_in_place();
}

#[cfg(feature = "native")]
mod native_tests {
    use super::*;
    use common::native_fixture::*;
    use keyboard_types::{Code, Key, NamedKey};
    use monoxus::foundation::form::collect_form_data;

    #[test]
    fn test_native_switch_attributes_and_permanent_thumb_mount() {
        let mut harness = mount_native_scenario(SwitchScenarioApp);
        advance_frame_and_pump(&mut harness, 0.016);

        let base = harness.base();

        // 1. Unchecked switch: thumb is permanently mounted (SW-5)
        let sw_air_id = harness.node("#sw-airplane");
        let sw_node = base.get_node(sw_air_id).unwrap();
        assert_eq!(sw_node.attr("role".into()), Some("switch"));
        assert_eq!(sw_node.attr("aria-checked".into()), Some("false"));
        assert_eq!(sw_node.attr("data-state".into()), Some("unchecked"));
        assert_eq!(sw_node.attr("data-switch-root".into()), Some(""));
        assert_eq!(sw_node.attr("data-disabled".into()), None);

        let thumb_air_id = harness.node("#thumb-airplane");
        let thumb_node = base.get_node(thumb_air_id).unwrap();
        assert_eq!(thumb_node.attr("data-state".into()), Some("unchecked"));
        assert_eq!(thumb_node.attr("data-switch-thumb".into()), Some(""));

        // 2. Checked switch: thumb is mounted with checked state
        let sw_wifi_id = harness.node("#sw-wifi");
        let wifi_node = base.get_node(sw_wifi_id).unwrap();
        assert_eq!(wifi_node.attr("aria-checked".into()), Some("true"));
        assert_eq!(wifi_node.attr("data-state".into()), Some("checked"));

        let thumb_wifi_id = harness.node("#thumb-wifi");
        let thumb_wifi_node = base.get_node(thumb_wifi_id).unwrap();
        assert_eq!(thumb_wifi_node.attr("data-state".into()), Some("checked"));

        // 3. Disabled switch
        let sw_dis_id = harness.node("#sw-disabled");
        let dis_node = base.get_node(sw_dis_id).unwrap();
        assert!(dis_node.attr("disabled".into()).is_some());
        assert_eq!(dis_node.attr("data-disabled".into()), Some(""));

        let thumb_dis_id = harness.node("#thumb-disabled");
        let thumb_dis_node = base.get_node(thumb_dis_id).unwrap();
        assert_eq!(thumb_dis_node.attr("data-disabled".into()), Some(""));

        // 4. Readonly switch
        let sw_ro_id = harness.node("#sw-readonly");
        let ro_node = base.get_node(sw_ro_id).unwrap();
        assert!(ro_node.attr("disabled".into()).is_none());
        assert_eq!(ro_node.attr("aria-readonly".into()), Some("true"));
        assert_eq!(ro_node.attr("data-readonly".into()), Some(""));

        // 5. Required switch
        let sw_req_id = harness.node("#sw-required");
        let req_node = base.get_node(sw_req_id).unwrap();
        assert_eq!(req_node.attr("aria-required".into()), Some("true"));
        assert_eq!(req_node.attr("data-required".into()), Some(""));
    }

    #[test]
    fn test_native_switch_both_space_and_enter_toggle() {
        let mut harness = mount_native_scenario(SwitchScenarioApp);
        advance_frame_and_pump(&mut harness, 0.016);

        let sw_id = harness.node("#sw-airplane");
        focus_node(&mut harness, "#sw-airplane");

        // 1. Enter key DOES toggle switch (SW-4: Unlike Checkbox, Enter toggles Switch!)
        send_key_and_pump(&mut harness, Key::Named(NamedKey::Enter), Code::Enter);
        assert_eq!(
            harness.base().get_node(sw_id).unwrap().attr("aria-checked".into()),
            Some("true"),
            "Enter key MUST toggle Switch"
        );

        // Enter again toggles back
        send_key_and_pump(&mut harness, Key::Named(NamedKey::Enter), Code::Enter);
        assert_eq!(
            harness.base().get_node(sw_id).unwrap().attr("aria-checked".into()),
            Some("false"),
            "Enter key toggles Switch back to unchecked"
        );

        // 2. Space key DOES toggle switch
        send_key_and_pump(&mut harness, Key::Character(" ".into()), Code::Space);
        assert_eq!(
            harness.base().get_node(sw_id).unwrap().attr("aria-checked".into()),
            Some("true"),
            "Space key MUST toggle Switch"
        );

        // 3. Pointer click DOES toggle switch
        click_and_pump(&mut harness, "#sw-airplane");
        assert_eq!(
            harness.base().get_node(sw_id).unwrap().attr("aria-checked".into()),
            Some("false"),
            "Pointer click MUST toggle Switch"
        );
    }

    #[test]
    fn test_native_switch_disabled_readonly_and_label_forwarding() {
        let mut harness = mount_native_scenario(SwitchScenarioApp);
        advance_frame_and_pump(&mut harness, 0.016);

        // Disabled switch ignores click
        let dis_id = harness.node("#sw-disabled");
        click_and_pump(&mut harness, "#sw-disabled");
        assert_eq!(
            harness.base().get_node(dis_id).unwrap().attr("aria-checked".into()),
            Some("true"),
            "Disabled switch must ignore click"
        );

        // Read-only switch ignores click
        let ro_id = harness.node("#sw-readonly");
        click_and_pump(&mut harness, "#sw-readonly");
        assert_eq!(
            harness.base().get_node(ro_id).unwrap().attr("aria-checked".into()),
            Some("false"),
            "Read-only switch must ignore click"
        );

        // Label click forwarding
        click_and_pump(&mut harness, "#switch-label");
        assert_native_focus_id(&harness, "sw-airplane");
        assert_native_focus_visible(&harness, "sw-airplane", false);
    }

    #[test]
    fn test_native_switch_form_serialization_and_reset() {
        let mut harness = mount_native_scenario(SwitchScenarioApp);
        advance_frame_and_pump(&mut harness, 0.016);

        // 1. Initial form data on #switch-form:
        // - airplane_mode: false -> omitted
        // - wifi: true -> ("wifi", "on")
        // - bluetooth: true but disabled -> omitted
        // - cellular: false -> omitted
        // - location: false -> omitted
        assert_eq!(
            collect_form_data("switch-form").expect("initial switch form data"),
            vec![("wifi".to_string(), "on".to_string())]
        );

        // 2. Toggle airplane mode to on, and wifi to off
        click_and_pump(&mut harness, "#sw-airplane");
        click_and_pump(&mut harness, "#sw-wifi");

        assert_eq!(
            collect_form_data("switch-form").expect("mutated switch form data"),
            vec![("airplane_mode".to_string(), "active".to_string())]
        );

        // 3. Click reset button
        click_and_pump(&mut harness, "#sw-reset-btn");

        let sw_air_id = harness.node("#sw-airplane");
        let sw_wifi_id = harness.node("#sw-wifi");
        assert_eq!(
            harness.base().get_node(sw_air_id).unwrap().attr("aria-checked".into()),
            Some("false"),
            "Airplane switch must reset to false"
        );
        assert_eq!(
            harness.base().get_node(sw_wifi_id).unwrap().attr("aria-checked".into()),
            Some("true"),
            "Wifi switch must reset to true"
        );

        assert_eq!(
            collect_form_data("switch-form").expect("reset switch form data"),
            vec![("wifi".to_string(), "on".to_string())]
        );
    }
}

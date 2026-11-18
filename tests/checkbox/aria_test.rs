use dioxus::prelude::*;
use monoxus::{
    checkbox::{
        Checkbox, CheckboxIndicator, CheckboxPart, CheckedState, CHECKBOX_PARTS,
    },
    foundation::state::DataState,
};

#[cfg(feature = "native")]
#[path = "../common/mod.rs"]
mod common;

#[test]
fn test_checkbox_part_inventory() {
    assert_eq!(CHECKBOX_PARTS.len(), 4);
    assert_eq!(CHECKBOX_PARTS[0], CheckboxPart::Root);
    assert_eq!(CHECKBOX_PARTS[0].as_str(), "root");
    assert_eq!(CHECKBOX_PARTS[1], CheckboxPart::Indicator);
    assert_eq!(CHECKBOX_PARTS[1].as_str(), "indicator");
    assert_eq!(CHECKBOX_PARTS[2], CheckboxPart::Group);
    assert_eq!(CHECKBOX_PARTS[2].as_str(), "group");
    assert_eq!(CHECKBOX_PARTS[3], CheckboxPart::GroupLabel);
    assert_eq!(CHECKBOX_PARTS[3].as_str(), "group-label");
}

#[test]
fn test_checkbox_checked_state_pure_matrix() {
    assert_eq!(CheckedState::Unchecked.aria_checked(), "false");
    assert_eq!(CheckedState::Checked.aria_checked(), "true");
    assert_eq!(CheckedState::Indeterminate.aria_checked(), "mixed");

    assert_eq!(CheckedState::Unchecked.data_state(), DataState::Unchecked);
    assert_eq!(CheckedState::Checked.data_state(), DataState::Checked);
    assert_eq!(CheckedState::Indeterminate.data_state(), DataState::Indeterminate);

    assert_eq!(CheckedState::from(true), CheckedState::Checked);
    assert_eq!(CheckedState::from(false), CheckedState::Unchecked);
}

#[component]
fn CheckboxAriaVdomApp() -> Element {
    rsx! {
        div {
            Checkbox {
                id: "cb-unchecked",
                default_checked: CheckedState::Unchecked,
                CheckboxIndicator { id: "ind-unchecked", "✓" }
            }
            Checkbox {
                id: "cb-checked",
                default_checked: CheckedState::Checked,
                CheckboxIndicator { id: "ind-checked", "✓" }
            }
            Checkbox {
                id: "cb-indeterminate",
                default_checked: CheckedState::Indeterminate,
                CheckboxIndicator { id: "ind-indeterminate", "-" }
            }
            Checkbox {
                id: "cb-force-mount",
                default_checked: CheckedState::Unchecked,
                CheckboxIndicator { id: "ind-force-mount", force_mount: true, "✓" }
            }
            Checkbox {
                id: "cb-disabled",
                default_checked: CheckedState::Checked,
                disabled: true,
                CheckboxIndicator { id: "ind-disabled", "✓" }
            }
            Checkbox {
                id: "cb-readonly",
                default_checked: CheckedState::Checked,
                read_only: true,
                CheckboxIndicator { id: "ind-readonly", "✓" }
            }
            Checkbox {
                id: "cb-required",
                default_checked: CheckedState::Unchecked,
                required: true,
            }
        }
    }
}

#[test]
fn test_checkbox_aria_vdom_mounts() {
    let mut dom = VirtualDom::new(CheckboxAriaVdomApp);
    dom.rebuild_in_place();
}

#[cfg(feature = "native")]
mod native_tests {
    use super::*;
    use common::native_fixture::*;

    #[test]
    fn test_native_checkbox_aria_attributes_and_indicator_presence() {
        let mut harness = mount_native_scenario(CheckboxAriaVdomApp);
        advance_frame_and_pump(&mut harness, 0.016);

        let base = harness.base();

        // 1. Unchecked checkbox: indicator unmounted by default
        let uncheck_btn = harness.node("#cb-unchecked");
        let uncheck_node = base.get_node(uncheck_btn).unwrap();
        assert_eq!(uncheck_node.attr("role".into()), Some("checkbox"));
        assert_eq!(uncheck_node.attr("aria-checked".into()), Some("false"));
        assert_eq!(uncheck_node.attr("data-state".into()), Some("unchecked"));
        assert_eq!(uncheck_node.attr("data-disabled".into()), None);
        assert_eq!(uncheck_node.attr("data-required".into()), None);
        assert_eq!(uncheck_node.attr("data-readonly".into()), None);
        assert_eq!(uncheck_node.attr("data-checkbox-root".into()), Some(""));
        assert!(harness.query("#ind-unchecked").is_none());

        // 2. Checked checkbox: indicator mounted
        let check_btn = harness.node("#cb-checked");
        let check_node = base.get_node(check_btn).unwrap();
        assert_eq!(check_node.attr("aria-checked".into()), Some("true"));
        assert_eq!(check_node.attr("data-state".into()), Some("checked"));
        let ind_check_id = harness.node("#ind-checked");
        let ind_check_node = base.get_node(ind_check_id).unwrap();
        assert_eq!(ind_check_node.attr("data-state".into()), Some("checked"));
        assert_eq!(ind_check_node.attr("data-checkbox-indicator".into()), Some(""));
        assert!(
            ind_check_node
                .attr("style".into())
                .unwrap_or_default()
                .contains("pointer-events: none;")
        );

        // 3. Indeterminate checkbox: aria-checked="mixed", data-state="indeterminate", indicator mounted
        let ind_btn = harness.node("#cb-indeterminate");
        let ind_node = base.get_node(ind_btn).unwrap();
        assert_eq!(ind_node.attr("aria-checked".into()), Some("mixed"));
        assert_eq!(ind_node.attr("data-state".into()), Some("indeterminate"));
        assert!(harness.query("#ind-indeterminate").is_some());

        // 4. Force mounted indicator: mounted even when unchecked
        assert!(harness.query("#ind-force-mount").is_some());
        let fm_ind_id = harness.node("#ind-force-mount");
        let fm_node = base.get_node(fm_ind_id).unwrap();
        assert_eq!(fm_node.attr("data-state".into()), Some("unchecked"));

        // 5. Disabled checkbox: data-disabled="", disabled attribute, aria-readonly & data-readonly omitted
        let dis_btn = harness.node("#cb-disabled");
        let dis_node = base.get_node(dis_btn).unwrap();
        assert!(dis_node.attr("disabled".into()).is_some());
        assert_eq!(dis_node.attr("data-disabled".into()), Some(""));
        assert_eq!(dis_node.attr("aria-readonly".into()), None);
        assert_eq!(dis_node.attr("data-readonly".into()), None);

        // 6. Read-only checkbox: aria-readonly="true", data-readonly="", disabled attribute omitted
        let ro_btn = harness.node("#cb-readonly");
        let ro_node = base.get_node(ro_btn).unwrap();
        assert!(ro_node.attr("disabled".into()).is_none());
        assert_eq!(ro_node.attr("aria-readonly".into()), Some("true"));
        assert_eq!(ro_node.attr("data-readonly".into()), Some(""));
        assert_eq!(ro_node.attr("data-disabled".into()), None);

        // 7. Required checkbox: aria-required="true", data-required=""
        let req_btn = harness.node("#cb-required");
        let req_node = base.get_node(req_btn).unwrap();
        assert_eq!(req_node.attr("aria-required".into()), Some("true"));
        assert_eq!(req_node.attr("data-required".into()), Some(""));
    }
}

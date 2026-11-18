use dioxus::prelude::*;
use monoxus::{
    button::{Button, ButtonType},
    radio_group::{RadioGroupItem, RadioGroupRoot},
};

#[cfg(feature = "native")]
use monoxus::foundation::form::collect_form_data;

#[cfg(feature = "native")]
#[path = "../common/mod.rs"]
mod common;

#[component]
fn RadioGroupFormApp() -> Element {
    rsx! {
        div {
            style: "width: 800px; height: 600px;",
            form {
                id: "subscription-form",
                style: "width: 600px; height: 250px; display: flex; flex-direction: column; gap: 16px;",
                // Unselected initially
                RadioGroupRoot {
                    id: "plan-group",
                    name: "plan",
                    style: "width: 500px; height: 40px; display: flex; flex-direction: row; gap: 8px;",
                    RadioGroupItem { id: "plan-free", value: "free", style: "width: 80px; height: 32px;", "Free" }
                    RadioGroupItem { id: "plan-pro", value: "pro", style: "width: 80px; height: 32px;", "Pro" }
                    RadioGroupItem { id: "plan-enterprise", value: "enterprise", disabled: true, style: "width: 80px; height: 32px;", "Enterprise (Disabled)" }
                }
                Button {
                    id: "sub-reset-btn".to_string(),
                    r#type: ButtonType::Reset,
                    style: "width: 100px; height: 36px;".to_string(),
                    "Reset Form"
                }
            }

            // External radio group associated via form="subscription-form"
            div {
                style: "width: 600px; height: 120px;",
                RadioGroupRoot {
                    id: "ext-group",
                    name: "tier",
                    form: "subscription-form",
                    default_value: "starter",
                    style: "width: 500px; height: 40px; display: flex; flex-direction: row; gap: 8px;",
                    RadioGroupItem { id: "tier-starter", value: "starter", style: "width: 80px; height: 32px;", "Starter" }
                    RadioGroupItem { id: "tier-scale", value: "scale", style: "width: 80px; height: 32px;", "Scale" }
                }
            }
        }
    }
}

#[test]
fn test_radio_group_form_vdom_mounts_cleanly() {
    let mut dom = VirtualDom::new(RadioGroupFormApp);
    dom.rebuild_in_place();
}

#[cfg(feature = "native")]
mod native_tests {
    use super::*;
    use common::native_fixture::*;

    #[test]
    fn test_native_radio_group_form_submission_and_reset() {
        let mut harness = mount_native_scenario(RadioGroupFormApp);
        // Pump frame to allow effects to run and forms to associate
        advance_frame_and_pump(&mut harness, 0.016);

        // 1. Initial State:
        // `plan` is unselected -> zero entries for `plan`! (RG-13: unselected radio group does not submit name="")
        // `tier` is "starter" -> submits ("tier", "starter")
        let initial_entries = collect_form_data("subscription-form").expect("collect_form_data initial");
        assert_eq!(
            initial_entries,
            vec![("tier".to_string(), "starter".to_string())],
            "Unselected radio group must submit zero entries; external group submits initial"
        );

        // 2. Select "free" on `#plan-free`
        click_and_pump(&mut harness, "#plan-free");
        let entries_after_free = collect_form_data("subscription-form").expect("collect after free");
        assert_eq!(
            entries_after_free,
            vec![
                ("plan".to_string(), "free".to_string()),
                ("tier".to_string(), "starter".to_string()),
            ]
        );

        // 3. Select "pro" on `#plan-pro` and "scale" on `#tier-scale`
        click_and_pump(&mut harness, "#plan-pro");
        click_and_pump(&mut harness, "#tier-scale");
        let entries_after_pro_scale = collect_form_data("subscription-form").expect("collect after pro scale");
        assert_eq!(
            entries_after_pro_scale,
            vec![
                ("plan".to_string(), "pro".to_string()),
                ("tier".to_string(), "scale".to_string()),
            ]
        );

        // 4. Test form reset:
        oxidase::dom::Document::current()
            .expect("active native doc")
            .dispatch_form_reset("subscription-form");
        advance_frame_and_pump(&mut harness, 0.016);

        let entries_after_reset = collect_form_data("subscription-form").expect("collect after reset");
        assert_eq!(
            entries_after_reset,
            vec![("tier".to_string(), "starter".to_string())],
            "Form reset must restore initial unselected state on plan and starter on tier"
        );

        // Verify live attributes restored
        let plan_free_id = harness.node("#plan-free");
        let plan_pro_id = harness.node("#plan-pro");
        let tier_starter_id = harness.node("#tier-starter");
        let tier_scale_id = harness.node("#tier-scale");

        let base = harness.base();
        assert_eq!(base.get_node(plan_free_id).unwrap().attr("aria-checked".into()), Some("false"));
        assert_eq!(base.get_node(plan_pro_id).unwrap().attr("aria-checked".into()), Some("false"));
        assert_eq!(base.get_node(tier_starter_id).unwrap().attr("aria-checked".into()), Some("true"));
        assert_eq!(base.get_node(tier_scale_id).unwrap().attr("aria-checked".into()), Some("false"));
    }
}

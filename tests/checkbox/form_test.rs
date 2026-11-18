use dioxus::prelude::*;
use monoxus::button::{Button, ButtonType};
use monoxus::checkbox::{Checkbox, CheckboxIndicator, CheckedState};

#[cfg(feature = "native")]
#[path = "../common/mod.rs"]
mod common;

#[component]
fn CheckboxFormScenarioApp() -> Element {
    rsx! {
        div {
            form {
                id: "test-form",
                label {
                    id: "implicit-label",
                    Checkbox {
                        id: "cb-terms",
                        name: "terms",
                        value: "accepted",
                        default_checked: CheckedState::Checked,
                        CheckboxIndicator { id: "ind-terms", "✓" }
                    }
                    span { "I accept the terms" }
                }
                Checkbox {
                    id: "cb-newsletter",
                    name: "newsletter",
                    value: "weekly",
                    default_checked: CheckedState::Unchecked,
                    CheckboxIndicator { id: "ind-newsletter", "✓" }
                }
                Checkbox {
                    id: "cb-ind",
                    name: "maybe",
                    value: "maybe_val",
                    default_checked: CheckedState::Indeterminate,
                    CheckboxIndicator { id: "ind-maybe", "-" }
                }
                Checkbox {
                    id: "cb-dis",
                    name: "dis_opt",
                    value: "dis_val",
                    default_checked: CheckedState::Checked,
                    disabled: true,
                }
                Button {
                    id: "form-reset-btn".to_string(),
                    r#type: ButtonType::Reset,
                    "Reset Form"
                }
            }

            // External checkbox associated with test-form
            Checkbox {
                id: "cb-ext",
                name: "external_opt",
                value: "ext_val",
                form: "test-form",
                default_checked: CheckedState::Unchecked,
            }
        }
    }
}

#[test]
fn test_checkbox_form_vdom_mounts() {
    let mut dom = VirtualDom::new(CheckboxFormScenarioApp);
    dom.rebuild_in_place();
}

#[cfg(feature = "native")]
mod native_tests {
    use super::*;
    use common::native_fixture::*;
    use monoxus::foundation::form::collect_form_data;

    #[test]
    fn test_native_checkbox_form_serialization_and_reset() {
        let mut harness = mount_native_scenario(CheckboxFormScenarioApp);
        advance_frame_and_pump(&mut harness, 0.016);

        // 1. Sibling order invariant: <button> appears before hidden <input> inside #implicit-label
        let label_id = harness.node("#implicit-label");
        let btn_id = harness.node("#cb-terms");
        {
            let base = harness.base();
            let label_node = base.get_node(label_id).unwrap();
            let children = &label_node.children;
            assert!(
                children.len() >= 2,
                "Label must have button and hidden input as children"
            );
            assert_eq!(
                children[0], btn_id,
                "First child of <label> must be the interactive <button>"
            );
        }

        // 2. Initial form data serialization on #test-form:
        // - terms: checked -> ("terms", "accepted")
        // - newsletter: unchecked -> omitted
        // - maybe: indeterminate -> omitted (FI-8)
        // - dis_opt: disabled -> omitted (E-3)
        // - external_opt: unchecked -> omitted
        assert_eq!(
            collect_form_data("test-form").expect("initial form data"),
            vec![("terms".to_string(), "accepted".to_string())]
        );

        // 3. Mutate controls:
        // - Check newsletter
        // - Check external_opt
        // - Uncheck terms
        click_and_pump(&mut harness, "#cb-newsletter");
        click_and_pump(&mut harness, "#cb-ext");
        click_and_pump(&mut harness, "#cb-terms");

        assert_eq!(
            collect_form_data("test-form").expect("mutated form data"),
            vec![
                ("newsletter".to_string(), "weekly".to_string()),
                ("external_opt".to_string(), "ext_val".to_string()),
            ]
        );

        // 4. Click reset button:
        click_and_pump(&mut harness, "#form-reset-btn");

        assert_eq!(
            harness
                .base()
                .get_node(btn_id)
                .unwrap()
                .attr("aria-checked".into()),
            Some("true"),
            "Terms checkbox must reset to checked"
        );
        let news_id = harness.node("#cb-newsletter");
        assert_eq!(
            harness
                .base()
                .get_node(news_id)
                .unwrap()
                .attr("aria-checked".into()),
            Some("false"),
            "Newsletter checkbox must reset to unchecked"
        );
        let ext_id = harness.node("#cb-ext");
        assert_eq!(
            harness
                .base()
                .get_node(ext_id)
                .unwrap()
                .attr("aria-checked".into()),
            Some("false"),
            "External checkbox must reset to unchecked"
        );

        assert_eq!(
            collect_form_data("test-form").expect("reset form data"),
            vec![("terms".to_string(), "accepted".to_string())]
        );
    }
}

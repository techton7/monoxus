use std::cell::Cell;
use std::rc::Rc;

use dioxus::prelude::*;
#[allow(unused_imports)]
use monoxus::label::{
    LABEL_INTERACTIVE_TARGET_SELECTOR, Label, LabelAttributes, LabelProps, LabelRoot,
    project_label_as_child,
};

#[cfg(feature = "native")]
mod common;

#[component]
fn LabelExplicitForScenario(clicked: Rc<Cell<u32>>) -> Element {
    rsx! {
        div {
            Label {
                id: "explicit-label",
                r#for: "controlled-input".to_string(),
                on_click: move |_| {
                    clicked.set(clicked.get() + 1);
                },
                "Username"
            }
            input {
                id: "controlled-input",
                r#type: "text",
            }
        }
    }
}

#[component]
fn LabelNestedControlScenario() -> Element {
    rsx! {
        Label {
            id: "nested-label",
            "Nested Field: "
            input {
                id: "nested-input",
                r#type: "text",
            }
        }
    }
}

#[test]
fn test_label_vdom_renders_explicit_for() {
    let clicked = Rc::new(Cell::new(0));
    let mut dom = VirtualDom::new_with_props(
        LabelExplicitForScenario,
        LabelExplicitForScenarioProps { clicked },
    );
    dom.rebuild_in_place();
}

#[test]
fn test_label_vdom_renders_nested_control() {
    let mut dom = VirtualDom::new(LabelNestedControlScenario);
    dom.rebuild_in_place();
}

#[test]
fn test_label_interactive_target_selector_matches_spec() {
    assert_eq!(
        LABEL_INTERACTIVE_TARGET_SELECTOR,
        "button, input, select, textarea"
    );
}

#[test]
fn test_label_state_and_projection_contract() {
    let label = Label::new()
        .with_id("user-label")
        .with_html_for("user-field");
    let attrs = label.attributes();
    assert_eq!(attrs.id.as_deref(), Some("user-label"));
    assert_eq!(attrs.r#for.as_deref(), Some("user-field"));
    assert_eq!(attrs.data_label_root, "");

    let (target, projected_attrs, content) =
        project_label_as_child(&label, "custom-label-wrapper", "Child Label Text".into());
    assert_eq!(target, "custom-label-wrapper");
    assert_eq!(projected_attrs.id.as_deref(), Some("user-label"));
    assert_eq!(projected_attrs.r#for.as_deref(), Some("user-field"));
    assert_eq!(projected_attrs.data_label_root, "");
    assert_eq!(content, "Child Label Text");
}

#[cfg(feature = "native")]
mod native_tests {
    use super::*;
    use super::common::native_fixture::*;

    #[component]
    fn NativeLabelExplicitForApp() -> Element {
        let mut clicks = use_signal(|| 0u32);
        rsx! {
            div {
                id: "click-probe",
                "data-clicks": "{clicks}",
                Label {
                    id: "explicit-label",
                    r#for: "controlled-input".to_string(),
                    on_click: move |_| {
                        clicks += 1;
                    },
                    "Username"
                }
                input {
                    id: "controlled-input",
                    r#type: "text",
                }
            }
        }
    }

    #[test]
    fn test_native_label_attributes_and_click_forwarding() {
        let mut harness = mount_native_scenario(NativeLabelExplicitForApp);

        let label_id = harness.query("#explicit-label").expect("Label mounted in DOM");
        {
            let base = harness.base();
            let node = base.get_node(label_id).expect("Node exists");
            assert_eq!(node.attr("data-label-root".into()), Some(""));
            assert_eq!(node.attr("for".into()), Some("controlled-input"));
        }

        // Click label
        click_and_pump(&mut harness, "#explicit-label");
        let probe_id = harness.node("#click-probe");
        assert_eq!(
            harness.base().get_node(probe_id).unwrap().attr("data-clicks".into()),
            Some("1"),
            "Clicking label must invoke click handler"
        );
    }

    #[test]
    fn test_native_nested_label_renders_root_marker() {
        let harness = mount_native_scenario(LabelNestedControlScenario);

        let label_id = harness.query("#nested-label").expect("Nested label mounted");
        {
            let base = harness.base();
            let node = base.get_node(label_id).expect("Node exists");
            assert_eq!(node.attr("data-label-root".into()), Some(""));
            assert_eq!(node.attr("for".into()), None, "Implicit nested label omits for attribute");
        }

        let input_id = harness.query("#nested-input").expect("Nested input mounted");
        {
            let base = harness.base();
            let node = base.get_node(input_id).expect("Input node exists");
            assert_eq!(node.attr("type".into()), Some("text"));
        }
    }
}

use std::rc::Rc;
use std::cell::Cell;

use dioxus::prelude::*;
#[allow(unused_imports)]
use monoxus::button::{
    Button, ButtonAttributes, ButtonProps, ButtonRoot, ButtonState, ButtonTag, ButtonType,
    project_button_as_child,
};

#[cfg(feature = "native")]
use crate::common;

#[component]
fn EnabledButtonScenario(clicked: Rc<Cell<u32>>) -> Element {
    rsx! {
        Button {
            id: "test-btn",
            r#type: ButtonType::Button,
            on_click: move |_| {
                clicked.set(clicked.get() + 1);
            },
            "Click Me"
        }
    }
}

#[component]
fn DisabledButtonScenario(clicked: Rc<Cell<u32>>) -> Element {
    rsx! {
        Button {
            id: "disabled-btn",
            disabled: true,
            on_click: move |_| {
                clicked.set(clicked.get() + 1);
            },
            "Disabled"
        }
    }
}

#[component]
fn EnabledAnchorScenario() -> Element {
    rsx! {
        Button {
            id: "test-anchor",
            href: "https://example.com".to_string(),
            target: "_blank".to_string(),
            rel: "noopener".to_string(),
            download: "file.txt".to_string(),
            "Anchor Link"
        }
    }
}

#[component]
fn DisabledAnchorScenario(clicked: Rc<Cell<u32>>) -> Element {
    rsx! {
        Button {
            id: "disabled-anchor",
            href: "https://example.com".to_string(),
            target: "_blank".to_string(),
            disabled: true,
            on_click: move |_| {
                clicked.set(clicked.get() + 1);
            },
            "Disabled Anchor"
        }
    }
}

#[test]
fn test_button_vdom_renders_button_tag() {
    let clicked = Rc::new(Cell::new(0));
    let mut dom = VirtualDom::new_with_props(EnabledButtonScenario, EnabledButtonScenarioProps { clicked });
    dom.rebuild_in_place();
}

#[test]
fn test_button_vdom_renders_anchor_tag() {
    let mut dom = VirtualDom::new(EnabledAnchorScenario);
    dom.rebuild_in_place();
}

#[test]
fn test_button_vdom_renders_disabled_anchor() {
    let clicked = Rc::new(Cell::new(0));
    let mut dom = VirtualDom::new_with_props(DisabledAnchorScenario, DisabledAnchorScenarioProps { clicked });
    dom.rebuild_in_place();
}

#[test]
fn test_button_state_4_way_matrix_compliance() {
    // 1. Button enabled: type="button", disabled=Some(false), data_disabled=None
    let s1 = ButtonState::new();
    let a1 = s1.attributes();
    assert_eq!(a1.tag, ButtonTag::Button);
    assert_eq!(a1.r#type, Some("button"));
    assert_eq!(a1.href, None);
    assert_eq!(a1.disabled, Some(false));
    assert_eq!(a1.aria_disabled, None);
    assert_eq!(a1.data_disabled, None);
    assert_eq!(a1.data_button_root, "");

    // 2. Button disabled: disabled=Some(true), data_disabled=Some("")
    let s2 = ButtonState::new().with_disabled(true);
    let a2 = s2.attributes();
    assert_eq!(a2.tag, ButtonTag::Button);
    assert_eq!(a2.disabled, Some(true));
    assert_eq!(a2.aria_disabled, None);
    assert_eq!(a2.data_disabled, Some(""));

    // 3. Anchor enabled: href=Some(...), disabled=None, data_disabled=None
    let s3 = ButtonState::new().with_href("/target");
    let a3 = s3.attributes();
    assert_eq!(a3.tag, ButtonTag::Anchor);
    assert_eq!(a3.r#type, None);
    assert_eq!(a3.href.as_deref(), Some("/target"));
    assert_eq!(a3.disabled, None);
    assert_eq!(a3.aria_disabled, None);
    assert_eq!(a3.data_disabled, None);

    // 4. Anchor disabled: href=None, role=Some("link"), aria_disabled=Some("true"), tabindex=Some(-1), data_disabled=Some("")
    let s4 = ButtonState::new().with_href("/target").with_disabled(true);
    let a4 = s4.attributes();
    assert_eq!(a4.tag, ButtonTag::Anchor);
    assert_eq!(a4.r#type, None);
    assert_eq!(a4.href, None);
    assert_eq!(a4.role.as_deref(), Some("link"));
    assert_eq!(a4.aria_disabled, Some("true"));
    assert_eq!(a4.tabindex, Some(-1));
    assert_eq!(a4.data_disabled, Some(""));
}

#[cfg(feature = "native")]
mod native_tests {
    use super::*;
    use super::common::native_fixture::*;

    #[component]
    fn NativeEnabledButtonApp() -> Element {
        let mut clicks = use_signal(|| 0u32);
        rsx! {
            div {
                id: "click-probe",
                "data-clicks": "{clicks}",
                Button {
                    id: "test-btn",
                    r#type: ButtonType::Button,
                    on_click: move |_| {
                        clicks += 1;
                    },
                    "Click Me"
                }
            }
        }
    }

    #[component]
    fn NativeDisabledButtonApp() -> Element {
        let mut clicks = use_signal(|| 0u32);
        rsx! {
            div {
                id: "click-probe",
                "data-clicks": "{clicks}",
                Button {
                    id: "disabled-btn",
                    disabled: true,
                    on_click: move |_| {
                        clicks += 1;
                    },
                    "Disabled"
                }
            }
        }
    }

    #[component]
    fn NativeAnchorsApp() -> Element {
        let mut clicks = use_signal(|| 0u32);
        rsx! {
            div {
                id: "click-probe",
                "data-clicks": "{clicks}",
                EnabledAnchorScenario {}
                Button {
                    id: "disabled-anchor",
                    href: "https://example.com".to_string(),
                    target: "_blank".to_string(),
                    disabled: true,
                    on_click: move |_| {
                        clicks += 1;
                    },
                    "Disabled Anchor"
                }
            }
        }
    }

    #[test]
    fn test_native_button_enabled_attributes_and_click() {
        let _lock = NATIVE_TEST_LOCK.lock().unwrap_or_else(|e| e.into_inner());
        let mut harness = mount_native_scenario(NativeEnabledButtonApp);

        let node_id = harness.query("#test-btn").expect("Button mounted in DOM");
        {
            let base = harness.base();
            let node = base.get_node(node_id).expect("Node exists");
            assert_eq!(node.attr("data-button-root".into()), Some(""));
            assert_eq!(node.attr("data-disabled".into()), None, "data-disabled must be omitted when enabled");
            assert_eq!(node.attr("type".into()), Some("button"));
        }

        click_and_pump(&mut harness, "#test-btn");
        let probe_id = harness.node("#click-probe");
        assert_eq!(
            harness.base().get_node(probe_id).unwrap().attr("data-clicks".into()),
            Some("1"),
            "Click on enabled button must increment counter"
        );
    }

    #[test]
    fn test_native_button_disabled_attributes_and_suppression() {
        let _lock = NATIVE_TEST_LOCK.lock().unwrap_or_else(|e| e.into_inner());
        let mut harness = mount_native_scenario(NativeDisabledButtonApp);

        let node_id = harness.query("#disabled-btn").expect("Disabled button mounted in DOM");
        {
            let base = harness.base();
            let node = base.get_node(node_id).expect("Node exists");
            assert_eq!(node.attr("data-button-root".into()), Some(""));
            assert_eq!(node.attr("data-disabled".into()), Some(""), "data-disabled must be empty string when disabled");
            assert!(node.attr("disabled".into()).is_some(), "Native disabled attribute must be set");
        }

        click_and_pump(&mut harness, "#disabled-btn");
        let probe_id = harness.node("#click-probe");
        assert_eq!(
            harness.base().get_node(probe_id).unwrap().attr("data-clicks".into()),
            Some("0"),
            "Click on disabled button must be suppressed"
        );
    }

    #[test]
    fn test_native_anchor_enabled_vs_disabled_attributes() {
        let _lock = NATIVE_TEST_LOCK.lock().unwrap_or_else(|e| e.into_inner());
        let mut harness = mount_native_scenario(NativeAnchorsApp);

        // Enabled anchor
        let enabled_id = harness.query("#test-anchor").expect("Enabled anchor mounted");
        {
            let base = harness.base();
            let node = base.get_node(enabled_id).expect("Node exists");
            assert_eq!(node.attr("href".into()), Some("https://example.com"));
            assert_eq!(node.attr("target".into()), Some("_blank"));
            assert_eq!(node.attr("rel".into()), Some("noopener"));
            assert_eq!(node.attr("download".into()), Some("file.txt"));
            assert_eq!(node.attr("data-disabled".into()), None, "Enabled anchor omits data-disabled");
            assert_eq!(node.attr("type".into()), None, "Anchor has no type attribute");
        }

        // Disabled anchor
        let disabled_id = harness.query("#disabled-anchor").expect("Disabled anchor mounted");
        {
            let base = harness.base();
            let node = base.get_node(disabled_id).expect("Node exists");
            assert_eq!(node.attr("href".into()), None, "Disabled anchor must omit href");
            assert_eq!(node.attr("target".into()), None, "Disabled anchor must omit target");
            assert_eq!(node.attr("role".into()), Some("link"));
            assert_eq!(node.attr("aria-disabled".into()), Some("true"));
            assert_eq!(node.attr("tabindex".into()), Some("-1"));
            assert_eq!(node.attr("data-disabled".into()), Some(""));
        }

        click_and_pump(&mut harness, "#disabled-anchor");
        let probe_id = harness.node("#click-probe");
        assert_eq!(
            harness.base().get_node(probe_id).unwrap().attr("data-clicks".into()),
            Some("0"),
            "Click on disabled anchor must be suppressed"
        );
    }

    #[component]
    fn NativeButtonMergedAttrsApp() -> Element {
        rsx! {
            Button {
                id: "fm-api-3-btn",
                class: "b",
                style: "margin: 0",
                attributes: vec![
                    Attribute::new("class", "a", None, false),
                    Attribute::new("style", "color: red", None, false),
                    Attribute {
                        name: "style",
                        value: dioxus::core::AttributeValue::None,
                        namespace: None,
                        volatile: false,
                    },
                    Attribute::new("padding", "4px", Some("style"), false),
                ],
                "Merged"
            }
        }
    }

    #[test]
    fn test_native_button_merge_global_attributes() {
        let _lock = NATIVE_TEST_LOCK.lock().unwrap_or_else(|e| e.into_inner());
        let harness = mount_native_scenario(NativeButtonMergedAttrsApp);

        // FM-API-3: merges class and style without ;; and drops AttributeValue::None style.
        assert_eq!(
            harness.attr("#fm-api-3-btn", "class").as_deref(),
            Some("b a")
        );
        let style = harness
            .attr("#fm-api-3-btn", "style")
            .expect("Button style attribute must exist");
        assert!(style.contains("margin: 0"), "style={style}");
        assert!(style.contains("color: red"), "style={style}");
        assert!(!style.contains(";;"), "style={style}");
    }
}

#[test]
fn test_button_merge_global_attributes_vdom() {
    #[component]
    fn ButtonAttrsVdom() -> Element {
        rsx! {
            Button {
                id: "vdom-fm-api-3-btn",
                class: "b",
                style: "margin: 0",
                attributes: vec![
                    Attribute::new("class", "a", None, false),
                    Attribute::new("style", "color: red", None, false),
                    Attribute {
                        name: "style",
                        value: dioxus::core::AttributeValue::None,
                        namespace: None,
                        volatile: false,
                    },
                ],
                "Merged"
            }
        }
    }

    let mut dom = VirtualDom::new(ButtonAttrsVdom);
    dom.rebuild_in_place();
    dom.render_immediate(&mut dioxus::core::NoOpMutations);
}

use dioxus::prelude::*;
#[allow(unused_imports)]
use monoxus::foundation::compose::project_attrs_as_child;
use monoxus::accessible_icon::{
    ACCESSIBLE_ICON_FALLBACK_WRAPPER_STYLE, AccessibleIcon, AccessibleIconAttributes,
    AccessibleIconChildProps, AccessibleIconProps, AccessibleIconRoot,
};
use monoxus::foundation::compose::Slottable;

#[cfg(feature = "native")]
use crate::common;

#[component]
fn AccessibleIconRenderChildScenario() -> Element {
    rsx! {
        button {
            id: "icon-btn-direct",
            r#type: "button",
            AccessibleIcon {
                label: "Favorite",
                child: move |props: Vec<Attribute>| {
                    rsx! {
                        svg {
                            id: "direct-svg",
                            width: "16",
                            height: "16",
                            ..props,
                            path { d: "M0 0h16v16H0z" }
                        }
                    }
                }
            }
        }
    }
}

#[component]
fn AccessibleIconFallbackChildrenScenario() -> Element {
    rsx! {
        button {
            id: "icon-btn-fallback",
            r#type: "button",
            AccessibleIconRoot {
                label: "Close".to_string(),
                svg {
                    id: "fallback-svg",
                    width: "16",
                    height: "16",
                    path { d: "M0 0L16 16" }
                }
            }
        }
    }
}

#[component]
fn AccessibleIconMissingChildScenario() -> Element {
    rsx! {
        div {
            id: "icon-missing-host",
            AccessibleIcon {
                label: "Accessible only".to_string(),
            }
        }
    }
}

#[test]
fn accessible_icon_vdom_renders_direct_fallback_and_missing_child() {
    let mut direct = VirtualDom::new(AccessibleIconRenderChildScenario);
    direct.rebuild_in_place();

    let mut fallback = VirtualDom::new(AccessibleIconFallbackChildrenScenario);
    fallback.rebuild_in_place();

    let mut missing = VirtualDom::new(AccessibleIconMissingChildScenario);
    missing.rebuild_in_place();
}

#[test]
fn accessible_icon_public_contract_and_as_child_projection() {
    let child_props = AccessibleIconChildProps::new();
    assert_eq!(child_props.aria_hidden, "true");
    assert_eq!(child_props.focusable, "false");
    assert_eq!(ACCESSIBLE_ICON_FALLBACK_WRAPPER_STYLE, "display: contents;");

    let (target, attrs, content) =
        project_attrs_as_child(&AccessibleIconAttributes::new("Star"), "svg-target", Slottable::new("icon-path"));
    assert_eq!(target, "svg-target");
    assert_eq!(content, "icon-path");
    assert_eq!(attrs.aria_hidden, "true");
    assert_eq!(attrs.focusable, "false");
    assert_eq!(attrs.label, "Star");

    let _props_type: Option<AccessibleIconProps> = None;
}

#[cfg(feature = "native")]
mod native_tests {
    use super::*;
    use common::native_fixture::*;

    #[component]
    fn NativeAccessibleIconApp() -> Element {
        rsx! {
            div {
                AccessibleIconRenderChildScenario {}
                AccessibleIconFallbackChildrenScenario {}
                AccessibleIconMissingChildScenario {}
            }
        }
    }

    #[test]
    fn native_accessible_icon_projects_aria_hidden_and_visually_hidden_label() {
        let _lock = NATIVE_TEST_LOCK.lock().unwrap_or_else(|e| e.into_inner());
        let harness = mount_native_scenario(NativeAccessibleIconApp);

        // 1. Direct render_child path: zero wrapper around svg, attributes directly on svg
        let direct_svg_id = harness.query("#direct-svg").expect("direct svg mounted");
        {
            let base = harness.base();
            let svg_node = base.get_node(direct_svg_id).expect("svg exists");
            assert_eq!(svg_node.attr("aria-hidden".into()), Some("true"));
            assert_eq!(svg_node.attr("focusable".into()), Some("false"));
        }
        let direct_btn_id = harness
            .query("#icon-btn-direct")
            .expect("direct btn mounted");
        assert!(
            harness
                .query("#icon-btn-direct [data-visually-hidden-root]")
                .is_some(),
            "sibling VisuallyHidden mounted in direct button"
        );
        {
            let base = harness.base();
            assert!(
                base.get_node(direct_btn_id)
                    .unwrap()
                    .children
                    .contains(&direct_svg_id),
                "render_child svg must be a direct child of button without wrapper"
            );
        }
        assert_eq!(
            harness.text_content("#icon-btn-direct [data-visually-hidden-root]"),
            "Favorite"
        );

        // 2. Fallback children path: display: contents wrapper with aria-hidden="true" + VisuallyHidden
        let fallback_wrapper = harness
            .query("#icon-btn-fallback > span[aria-hidden='true']")
            .expect("fallback wrapper mounted");
        assert!(
            harness
                .query("#icon-btn-fallback [data-visually-hidden-root]")
                .is_some(),
            "sibling VisuallyHidden mounted in fallback button"
        );
        {
            let base = harness.base();
            let wrap_node = base.get_node(fallback_wrapper).unwrap();
            assert_eq!(wrap_node.attr("aria-hidden".into()), Some("true"));
            assert!(
                wrap_node
                    .attr("style".into())
                    .is_some_and(|s| s.contains("display: contents"))
            );
        }
        assert_eq!(
            harness.text_content("#icon-btn-fallback [data-visually-hidden-root]"),
            "Close"
        );

        // 3. Missing child gracefully renders VisuallyHidden label without panicking
        assert!(
            harness
                .query("#icon-missing-host [data-visually-hidden-root]")
                .is_some(),
            "missing-child VisuallyHidden mounted"
        );
        assert_eq!(
            harness.text_content("#icon-missing-host [data-visually-hidden-root]"),
            "Accessible only"
        );
    }
}

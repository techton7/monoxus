use std::cell::Cell;
use std::rc::Rc;

use dioxus::prelude::*;
use monoxus::foundation::compose::Slottable;
#[allow(unused_imports)]
use monoxus::visually_hidden::{
    VISUALLY_HIDDEN_STYLE, VisuallyHidden, VisuallyHiddenAttributes, VisuallyHiddenProps,
    VisuallyHiddenRenderProps, VisuallyHiddenRoot, VisuallyHiddenState,
    merge_visually_hidden_style, project_visually_hidden_as_child,
};

#[cfg(feature = "native")]
use crate::common;

#[component]
fn VisuallyHiddenDefaultScenario() -> Element {
    rsx! {
        VisuallyHidden {
            id: "vh-default",
            class: "sr-only-hook",
            style: "outline-color: rgb(1, 2, 3);",
            "Skip to ratings"
        }
    }
}

#[component]
fn VisuallyHiddenAsChildScenario(clicks: Rc<Cell<u32>>) -> Element {
    rsx! {
        VisuallyHiddenRoot {
            id: "vh-as-child",
            class: "custom-vh-class",
            style: "color: rgb(10, 20, 30);",
            as_child: true,
            on_click: move |_| {
                clicks.set(clicks.get() + 1);
            },
            render_child: move |props: VisuallyHiddenRenderProps| {
                rsx! {
                    article {
                        id: props.id,
                        class: props.class,
                        style: props.style,
                        "data-visually-hidden-root": props.data_visually_hidden_root,
                        onclick: move |evt| {
                            if let Some(handler) = props.on_click.as_ref() {
                                handler.call(evt);
                            }
                        },
                        {props.children}
                    }
                }
            },
            "Hidden article text"
        }
    }
}

#[test]
fn visually_hidden_vdom_renders_default_and_as_child() {
    let mut default_dom = VirtualDom::new(VisuallyHiddenDefaultScenario);
    default_dom.rebuild_in_place();

    let clicks = Rc::new(Cell::new(0));
    let mut as_child_dom = VirtualDom::new_with_props(
        VisuallyHiddenAsChildScenario,
        VisuallyHiddenAsChildScenarioProps { clicks },
    );
    as_child_dom.rebuild_in_place();
}

#[test]
fn visually_hidden_public_style_and_projection_contract() {
    let state = VisuallyHiddenState::new();
    let default_attrs = state.attributes();
    assert_eq!(default_attrs.style, VISUALLY_HIDDEN_STYLE);
    assert_eq!(default_attrs.data_visually_hidden_root, "");

    let merged = merge_visually_hidden_style(Some("outline-color: rgb(1, 2, 3);"));
    assert!(merged.starts_with(VISUALLY_HIDDEN_STYLE));
    assert!(merged.ends_with("outline-color: rgb(1, 2, 3);"));

    let (target, projected, content) = project_visually_hidden_as_child(
        &state.attributes_with_style(Some("color: blue;")),
        "article",
        Slottable::new("Screen reader caption"),
    );
    assert_eq!(target, "article");
    assert_eq!(content, "Screen reader caption");
    assert_eq!(projected.data_visually_hidden_root, "");
    assert!(projected.style.contains("clip: rect(0, 0, 0, 0);"));
    assert!(projected.style.ends_with("color: blue;"));

    let _props_type: Option<VisuallyHiddenProps> = None;
}

#[cfg(feature = "native")]
mod native_tests {
    use super::*;
    use common::native_fixture::*;

    #[component]
    fn NativeVisuallyHiddenLayoutApp() -> Element {
        rsx! {
            div {
                id: "vh-host",
                style: "width: 200px;",
                VisuallyHidden {
                    id: "vh-node",
                    class: "sr-class",
                    style: "outline-color: rgb(1, 2, 3);",
                    "Hidden label"
                }
                VisuallyHiddenRoot {
                    id: "vh-article-wrap",
                    as_child: true,
                    render_child: move |props: VisuallyHiddenRenderProps| {
                        rsx! {
                            article {
                                id: props.id,
                                style: props.style,
                                "data-visually-hidden-root": props.data_visually_hidden_root,
                                {props.children}
                            }
                        }
                    },
                    "Slotted hidden text"
                }
            }
        }
    }

    #[test]
    fn native_visually_hidden_dom_attributes_and_zero_layout_shift() {
        let _lock = NATIVE_TEST_LOCK.lock().unwrap_or_else(|e| e.into_inner());
        let harness = mount_native_scenario(NativeVisuallyHiddenLayoutApp);

        let vh_id = harness.query("#vh-node").expect("VisuallyHidden mounted");
        {
            let base = harness.base();
            let node = base.get_node(vh_id).expect("Node exists");
            assert_eq!(node.attr("aria-hidden".into()), None);
            assert_eq!(node.attr("data-visually-hidden-root".into()), Some(""));
            let style_attr = node.attr("style".into()).expect("inline style emitted");
            assert!(style_attr.contains("position: absolute;"));
            assert!(style_attr.contains("clip: rect(0, 0, 0, 0);"));
            assert!(style_attr.contains("outline-color: rgb(1, 2, 3);"));
        }

        let host_rect = harness.layout_rect("#vh-host");
        let vh_rect = harness.layout_rect("#vh-node");
        assert!(
            vh_rect.width <= 1.5 && vh_rect.height <= 1.5,
            "expected <= 1px box, got {}x{}",
            vh_rect.width,
            vh_rect.height
        );
        assert_eq!(
            host_rect.height, 0.0,
            "absolutely positioned VisuallyHidden must not expand container height"
        );

        let article_id = harness
            .query("article#vh-article-wrap")
            .expect("as_child article mounted");
        {
            let base = harness.base();
            let node = base.get_node(article_id).expect("Article exists");
            assert_eq!(node.attr("data-visually-hidden-root".into()), Some(""));
        }
    }
}

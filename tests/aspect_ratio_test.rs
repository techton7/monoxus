use dioxus::prelude::*;
#[allow(unused_imports)]
use monoxus::foundation::compose::project_attrs_as_child;
use monoxus::aspect_ratio::{
    ASPECT_RATIO_ROOT_STYLE, AspectRatio, AspectRatioAttributes, AspectRatioProps, AspectRatioRoot,
    normalize_aspect_ratio, render_aspect_ratio_child,
};
use monoxus::foundation::compose::Slottable;

#[component]
fn AspectRatioContractScenario() -> Element {
    rsx! {
        AspectRatioRoot {
            id: "ratio-root",
            ratio: 16.0 / 9.0,
            class: "consumer-class",
            style: "display: block",
            "aria-label": "contract image",
            div { id: "ratio-content", "content" }
        }
    }
}

#[component]
fn AspectRatioAsChildContractScenario() -> Element {
    rsx! {
        AspectRatio {
            ratio: 2.0,
            child: move |props: Vec<Attribute>| {
                rsx! {
                    figure {
                        id: "custom-ratio-target",
                        ..props,
                        figcaption { "Custom target" }
                    }
                }
            },
        }
    }
}

#[test]
fn test_aspect_ratio_public_contract_renders_both_paths() {
    let mut root = VirtualDom::new(AspectRatioContractScenario);
    root.rebuild_in_place();

    let mut as_child = VirtualDom::new(AspectRatioAsChildContractScenario);
    as_child.rebuild_in_place();
}

#[test]
fn test_aspect_ratio_public_state_projection_contract() {
    let state = AspectRatio::new(4.0);
    let attributes: AspectRatioAttributes = state.attributes();
    assert_eq!(
        attributes.wrapper_style,
        "position: relative; width: 100%; padding-bottom: 25%;"
    );
    assert_eq!(attributes.root_style, ASPECT_RATIO_ROOT_STYLE);
    assert_eq!(attributes.data_radix_aspect_ratio_wrapper, "");
    assert_eq!(attributes.data_aspect_ratio_wrapper, "");
    assert_eq!(attributes.data_aspect_ratio_root, "");

    for invalid in [0.0, -1.0, f64::NAN, f64::INFINITY, f64::NEG_INFINITY] {
        assert_eq!(normalize_aspect_ratio(invalid), 0.0);
    }

    let (target, projected, content) =
        project_attrs_as_child(&state.attributes(), "custom-target", Slottable::new("child"));
    assert_eq!(target, "custom-target");
    assert_eq!(projected.data_aspect_ratio_root, "");
    assert_eq!(content, "child");

    let rendered = render_aspect_ratio_child(
        &state,
        "render-target",
        Slottable::new("render-child"),
        |target, attributes, content| (target, attributes, content),
    );
    assert_eq!(rendered.0, "render-target");
    assert_eq!(rendered.1.data_aspect_ratio_root, "");
    assert_eq!(rendered.2, "render-child");
}

#[test]
fn test_aspect_ratio_component_alias_and_default_props_are_public() {
    let default = AspectRatio::default();
    assert_eq!(default.ratio(), 1.0);
    let _props_type: Option<AspectRatioProps> = None;
}

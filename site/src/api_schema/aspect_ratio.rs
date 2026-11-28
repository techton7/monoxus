use super::{
    ComponentDoc, DataAttrDoc, PROP_ATTRIBUTES, PROP_CHILD, PROP_CLASS, PROP_ID, PROP_ON_MOUNTED,
    PROP_STYLE, PartDoc, PropDoc, PropGroup,
};

pub(super) const ASPECT_RATIO_DOC: ComponentDoc = ComponentDoc {
    slug: "aspect-ratio",
    title: "Aspect Ratio",
    description: "Displays content within a fixed width-to-height ratio using a padding-bottom wrapper and an absolutely inset root container.",
    keywords: "aspect ratio media image video container 16:9 4:3 square layout",
    features: &[
        "Maintains arbitrary width-to-height ratios (e.g. 16.0 / 9.0, 4.0 / 3.0, 1.0)",
        "Renders a relative padding wrapper around an absolutely inset root element",
        "Normalizes non-positive or non-finite ratio inputs safely",
        "Supports child prop composition and forwards GlobalAttributes via merge_props",
    ],
    anatomy: "use monoxus::aspect_ratio::AspectRatio;\n\nrsx! {\n    AspectRatio {\n        ratio: 16.0 / 9.0,\n        div { \"16:9 content\" }\n    }\n}",
    source_path: "src/aspect_ratio.rs",
    aria_url: "https://developer.mozilla.org/en-US/docs/Web/CSS/aspect-ratio",
    aria_label: "MDN CSS aspect-ratio",
    parts: &[PartDoc {
        name: "AspectRatio",
        description: "Outer ratio wrapper and inner absolutely inset container (also aliased as AspectRatioRoot).",
        props: &[
            PropDoc {
                name: "ratio",
                type_simple: "f64",
                type_full: "f64",
                default: Some("1.0"),
                required: false,
                group: PropGroup::Core,
                description: "Desired width-to-height ratio (for example 16.0 / 9.0).",
            },
            PROP_CHILD,
            PROP_ID,
            PROP_CLASS,
            PROP_STYLE,
            PropDoc {
                name: "on_click",
                type_simple: "EventHandler<MouseEvent>",
                type_full: "Option<EventHandler<MouseEvent>>",
                default: None,
                required: false,
                group: PropGroup::Core,
                description: "Invoked when the inner root container is clicked.",
            },
            PROP_ON_MOUNTED,
            PROP_ATTRIBUTES,
            PropDoc {
                name: "children",
                type_simple: "Element",
                type_full: "dioxus::prelude::Element",
                default: None,
                required: false,
                group: PropGroup::Core,
                description: "Content rendered inside the inset aspect-ratio container.",
            },
        ],
        data_attrs: &[
            DataAttrDoc {
                name: "data-aspect-ratio-wrapper",
                values: "\"\"",
                description: "Present on the outer padding wrapper element.",
            },
            DataAttrDoc {
                name: "data-radix-aspect-ratio-wrapper",
                values: "\"\"",
                description: "Radix compatibility marker present on the outer wrapper element.",
            },
            DataAttrDoc {
                name: "data-aspect-ratio-root",
                values: "\"\"",
                description: "Present on the inner inset root element.",
            },
        ],
        css_vars: &[],
    }],
    keyboard: &[],
};

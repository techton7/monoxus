use super::{
    ComponentDoc, DataAttrDoc, PROP_ATTRIBUTES, PROP_CHILD, PROP_CLASS, PROP_ON_MOUNTED,
    PROP_STYLE, PartDoc, PropDoc, PropGroup, prop_id,
};

pub(super) const VISUALLY_HIDDEN_DOC: ComponentDoc = ComponentDoc {
    slug: "visually-hidden",
    title: "Visually Hidden",
    description: "Screen-reader-only wrapper that hides content visually via accessible clipping styles while keeping it exposed to assistive technology.",
    keywords: "visually hidden sr-only screen reader accessibility a11y clip",
    features: &[
        "Applies standard screen-reader-safe clipping styles (1px box, overflow: hidden, clip: rect(0, 0, 0, 0)) without display: none",
        "Preserves semantic text for icon-only buttons, status announcements, and landmark headings",
        "Supports child render composition to attach visually-hidden styles to any custom element",
        "Zero runtime overhead with deterministic pure attribute projection from ui-sdk",
    ],
    anatomy: "use monoxus::visually_hidden::VisuallyHidden;\n\nrsx! {\n    button {\n        \"★\"\n        VisuallyHidden { \"Add to favorites\" }\n    }\n}",
    source_path: "src/visually_hidden.rs",
    aria_url: "https://www.w3.org/WAI/tutorials/forms/labels/#hiding-the-label-element",
    aria_label: "WAI Hiding Label Elements Technique",
    parts: &[
        PartDoc {
            name: "VisuallyHidden",
            description: "Inline <span> container hiding children visually while keeping them readable by screen readers.",
            props: &[
                prop_id("Custom element ID."),
                PROP_CLASS,
                PROP_STYLE,
                PROP_CHILD,
                PropDoc { name: "on_click", type_simple: "EventHandler<MouseEvent>", type_full: "Option<EventHandler<MouseEvent>>", default: None, required: false, group: PropGroup::Core, description: "Click handler forwarded to the element." },
                PROP_ON_MOUNTED,
                PROP_ATTRIBUTES,
                PropDoc { name: "children", type_simple: "Element", type_full: "dioxus::prelude::Element", default: None, required: false, group: PropGroup::Core, description: "Assistive text or elements hidden from visual layout." },
            ],
            data_attrs: &[
                DataAttrDoc { name: "data-visually-hidden-root", values: "\"\"", description: "Identifies the visually hidden element." },
            ],
            css_vars: &[],
        },
    ],
    keyboard: &[],
};

use super::{ComponentDoc, PROP_ATTRIBUTES, PROP_CHILD, PartDoc, PropDoc, PropGroup};

pub(super) const ACCESSIBLE_ICON_DOC: ComponentDoc = ComponentDoc {
    slug: "accessible-icon",
    title: "Accessible Icon",
    description: "Makes decorative icons accessible to assistive technology by hiding the visual icon and rendering a visually hidden label.",
    keywords: "accessible icon svg screen reader visually hidden a11y label",
    features: &[
        "Hides the visual icon from assistive technology with aria-hidden=\"true\"",
        "Renders a sibling VisuallyHidden span with the provided accessible label",
        "Composes directly onto an SVG via the child callback with focusable=\"false\" and aria-hidden=\"true\"",
        "Falls back to a display:contents wrapper span when using regular children",
        "Forwards GlobalAttributes via merge_props",
    ],
    anatomy: "use monoxus::accessible_icon::AccessibleIcon;\n\nrsx! {\n    AccessibleIcon {\n        label: \"Close\",\n        svg { /* icon paths */ }\n    }\n}",
    source_path: "src/accessible_icon.rs",
    aria_url: "https://www.w3.org/WAI/tutorials/images/functional/",
    aria_label: "WAI Functional Images",
    parts: &[PartDoc {
        name: "AccessibleIcon",
        description: "Wrapper pairing a hidden visual icon with a VisuallyHidden accessible label (also aliased as AccessibleIconRoot).",
        props: &[
            PropDoc {
                name: "label",
                type_simple: "String",
                type_full: "String",
                default: None,
                required: true,
                group: PropGroup::Core,
                description: "Accessible label text announced by screen readers via VisuallyHidden.",
            },
            PROP_CHILD,
            PROP_ATTRIBUTES,
            PropDoc {
                name: "children",
                type_simple: "Element",
                type_full: "Option<Element>",
                default: None,
                required: false,
                group: PropGroup::Core,
                description: "Visual icon content wrapped in an aria-hidden span when child is not provided.",
            },
        ],
        data_attrs: &[],
        css_vars: &[],
    }],
    keyboard: &[],
};

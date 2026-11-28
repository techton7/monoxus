use super::{
    ComponentDoc, DataAttrDoc, PROP_ATTRIBUTES, PROP_CHILD, PROP_CLASS, PROP_ID, PROP_ON_MOUNTED,
    PROP_STYLE, PartDoc, PropDoc, PropGroup,
};

pub(super) const SEPARATOR_DOC: ComponentDoc = ComponentDoc {
    slug: "separator",
    title: "Separator",
    description: "Visually or semantically separates content along a horizontal or vertical axis.",
    keywords: "separator divider hr rule horizontal vertical decorative",
    features: &[
        "Supports horizontal and vertical orientations via SeparatorOrientation",
        "Publishes role=\"separator\" with aria-orientation=\"vertical\" for vertical semantic dividers",
        "Sets role=\"none\" and omits aria-orientation when decorative is true",
        "Supports child prop composition and forwards GlobalAttributes via merge_props",
    ],
    anatomy: "use monoxus::separator::{Separator, SeparatorOrientation};\n\nrsx! {\n    Separator { orientation: SeparatorOrientation::Horizontal }\n    Separator { orientation: SeparatorOrientation::Vertical, decorative: true }\n}",
    source_path: "src/separator.rs",
    aria_url: "https://www.w3.org/TR/wai-aria-1.2/#separator",
    aria_label: "WAI-ARIA Separator Role",
    parts: &[PartDoc {
        name: "Separator",
        description: "Semantic or decorative divider element (also aliased as SeparatorRoot).",
        props: &[
            PROP_ID,
            PROP_CLASS,
            PROP_STYLE,
            PropDoc {
                name: "orientation",
                type_simple: "SeparatorOrientation",
                type_full: "monoxus::separator::SeparatorOrientation (Horizontal | Vertical)",
                default: Some("SeparatorOrientation::Horizontal"),
                required: false,
                group: PropGroup::Core,
                description: "Axis of the divider.",
            },
            PropDoc {
                name: "decorative",
                type_simple: "bool",
                type_full: "bool",
                default: Some("false"),
                required: false,
                group: PropGroup::Core,
                description: "When true, marks the divider as purely visual (role=\"none\", no aria-orientation).",
            },
            PROP_CHILD,
            PROP_ON_MOUNTED,
            PROP_ATTRIBUTES,
        ],
        data_attrs: &[
            DataAttrDoc {
                name: "data-separator-root",
                values: "\"\"",
                description: "Present on the separator element.",
            },
            DataAttrDoc {
                name: "data-orientation",
                values: "\"horizontal\" | \"vertical\"",
                description: "Reflects the configured orientation.",
            },
        ],
        css_vars: &[],
    }],
    keyboard: &[],
};

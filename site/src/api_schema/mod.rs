mod button;
mod dialog;
mod select;
mod tabs;
mod tooltip;

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum PropGroup {
    Core,
    Common,
    HtmlAria,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct PropDoc {
    pub name: &'static str,
    pub type_simple: &'static str,
    pub type_full: &'static str,
    pub default: Option<&'static str>,
    pub required: bool,
    pub group: PropGroup,
    pub description: &'static str,
}

pub(super) const PROP_ID: PropDoc = prop_id("Custom DOM element ID.");

pub(super) const fn prop_id(description: &'static str) -> PropDoc {
    PropDoc {
        name: "id",
        type_simple: "String",
        type_full: "Option<String>",
        default: None,
        required: false,
        group: PropGroup::Common,
        description,
    }
}

pub(super) const PROP_CLASS: PropDoc = PropDoc {
    name: "class",
    type_simple: "String",
    type_full: "Option<String>",
    default: None,
    required: false,
    group: PropGroup::Common,
    description: "CSS class names.",
};

pub(super) const PROP_STYLE: PropDoc = PropDoc {
    name: "style",
    type_simple: "String",
    type_full: "Option<String>",
    default: None,
    required: false,
    group: PropGroup::Common,
    description: "Inline CSS styles.",
};

pub(super) const PROP_ATTRIBUTES: PropDoc = PropDoc {
    name: "attributes",
    type_simple: "Vec<Attribute>",
    type_full: "Vec<Attribute> (#[props(extends = GlobalAttributes)])",
    default: None,
    required: false,
    group: PropGroup::Common,
    description: "Additional global HTML/ARIA attributes forwarded to the element.",
};

pub(super) const PROP_ON_MOUNTED: PropDoc = PropDoc {
    name: "on_mounted",
    type_simple: "EventHandler<MountedEvent>",
    type_full: "Option<EventHandler<MountedEvent>>",
    default: None,
    required: false,
    group: PropGroup::Common,
    description: "Invoked when the underlying DOM element mounts.",
};

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct DataAttrDoc {
    pub name: &'static str,
    pub values: &'static str,
    pub description: &'static str,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct CssVarDoc {
    pub name: &'static str,
    pub description: &'static str,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct KeyboardDoc {
    pub key: &'static str,
    pub description: &'static str,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct PartDoc {
    pub name: &'static str,
    pub description: &'static str,
    pub props: &'static [PropDoc],
    pub data_attrs: &'static [DataAttrDoc],
    pub css_vars: &'static [CssVarDoc],
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct ComponentDoc {
    pub slug: &'static str,
    pub title: &'static str,
    pub description: &'static str,
    pub keywords: &'static str,
    pub features: &'static [&'static str],
    pub anatomy: &'static str,
    pub source_path: &'static str,
    pub aria_url: &'static str,
    pub aria_label: &'static str,
    pub parts: &'static [PartDoc],
    pub keyboard: &'static [KeyboardDoc],
}

static ALL_COMPONENT_DOCS: &[ComponentDoc] = &[
    button::BUTTON_DOC,
    dialog::DIALOG_DOC,
    select::SELECT_DOC,
    tabs::TABS_DOC,
    tooltip::TOOLTIP_DOC,
];

pub fn all_component_docs() -> &'static [ComponentDoc] {
    ALL_COMPONENT_DOCS
}

pub fn find_component_doc(slug: &str) -> Option<&'static ComponentDoc> {
    ALL_COMPONENT_DOCS.iter().find(|doc| doc.slug == slug)
}

pub fn component_to_markdown(doc: &ComponentDoc, demo_source: &str) -> String {
    let mut md = String::new();
    md.push_str(&format!("# {}\n\n", doc.title));
    md.push_str(&format!("{}\n\n", doc.description));
    md.push_str(&format!(
        "- Source: `{}`\n- Accessibility: [{}]({})\n\n",
        doc.source_path, doc.aria_label, doc.aria_url
    ));

    if !doc.features.is_empty() {
        md.push_str("## Features\n\n");
        for feat in doc.features {
            md.push_str(&format!("- {feat}\n"));
        }
        md.push('\n');
    }

    if !demo_source.trim().is_empty() {
        md.push_str("## Example\n\n```rust\n");
        md.push_str(demo_source.trim());
        md.push_str("\n```\n\n");
    }

    md.push_str("## Anatomy\n\n```rust\n");
    md.push_str(doc.anatomy.trim());
    md.push_str("\n```\n\n");

    md.push_str("## API Reference\n\n");
    for part in doc.parts {
        md.push_str(&format!("### {}\n\n{}\n\n", part.name, part.description));
        if !part.props.is_empty() {
            md.push_str("| Prop | Type | Default | Description |\n|---|---|---|---|\n");
            for group in [PropGroup::Core, PropGroup::Common, PropGroup::HtmlAria] {
                for prop in part.props.iter().filter(|p| p.group == group) {
                    let def = if prop.required {
                        "Required".to_string()
                    } else {
                        prop.default.unwrap_or("—").to_string()
                    };
                    md.push_str(&format!(
                        "| `{}` | `{}` | `{}` | {} |\n",
                        prop.name, prop.type_simple, def, prop.description
                    ));
                }
            }
            md.push('\n');
        }
        if !part.data_attrs.is_empty() {
            md.push_str("| Data Attribute | Values | Description |\n|---|---|---|\n");
            for attr in part.data_attrs {
                md.push_str(&format!(
                    "| `{}` | `{}` | {} |\n",
                    attr.name, attr.values, attr.description
                ));
            }
            md.push('\n');
        }
        if !part.css_vars.is_empty() {
            md.push_str("| CSS Custom Property | Description |\n|---|---|\n");
            for var in part.css_vars {
                md.push_str(&format!("| `{}` | {} |\n", var.name, var.description));
            }
            md.push('\n');
        }
    }

    if !doc.keyboard.is_empty() {
        md.push_str("## Keyboard Interactions\n\n| Key | Description |\n|---|---|\n");
        for kb in doc.keyboard {
            md.push_str(&format!("| `{}` | {} |\n", kb.key, kb.description));
        }
    }

    md
}

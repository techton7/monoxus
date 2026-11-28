mod shared_menu;

mod accessible_icon;
mod accordion;
mod alert_dialog;
mod aspect_ratio;
mod avatar;
mod button;
mod calendar;
mod checkbox;
mod collapsible;
mod combobox;
mod command;
mod context_menu;
mod date_field;
mod date_picker;
mod date_range_field;
mod date_range_picker;
mod dialog;
mod dropdown_menu;
mod field;
mod hover_card;
mod label;
mod menubar;
mod meter;
mod navigation_menu;
mod pagination;
mod pin_input;
mod popover;
mod progress;
mod radio_group;
mod range_calendar;
mod rating_group;
mod scroll_area;
mod select;
mod separator;
mod slider;
mod switch;
mod tabs;
mod time_field;
mod time_range_field;
mod toast;
mod toggle;
mod toggle_group;
mod toolbar;
mod tooltip;
mod visually_hidden;

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

pub(super) const PROP_CHILD: PropDoc = PropDoc {
    name: "child",
    type_simple: "Callback<Vec<Attribute>, Element>",
    type_full: "Option<Callback<Vec<Attribute>, Element>>",
    default: None,
    required: false,
    group: PropGroup::Common,
    description: "Custom render callback receiving merged primitive + user attributes (annotate closure param as `props: Vec<Attribute>` and spread `..props`).",
};

pub(super) const PROP_FLOATING_CHILD: PropDoc = PropDoc {
    name: "child",
    type_simple: "Callback<FloatingContentChildProps, Element>",
    type_full: "Option<Callback<FloatingContentChildProps, Element>>",
    default: None,
    required: false,
    group: PropGroup::Common,
    description: "Custom render callback receiving `FloatingContentChildProps { wrapper_props, props }` for composing both the floating positioning wrapper and inner content element.",
};

pub(super) const PROP_AGENT: PropDoc = PropDoc {
    name: "agent",
    type_simple: "AgentHints",
    type_full: "Option<AgentHints>",
    default: None,
    required: false,
    group: PropGroup::Common,
    description: "Optional semantic hints overriding label, description, confirmation, secret, hidden, or inert flags when mounted inside `<AgentProvider>`.",
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
    accessible_icon::ACCESSIBLE_ICON_DOC,
    accordion::ACCORDION_DOC,
    alert_dialog::ALERT_DIALOG_DOC,
    aspect_ratio::ASPECT_RATIO_DOC,
    avatar::AVATAR_DOC,
    button::BUTTON_DOC,
    calendar::CALENDAR_DOC,
    checkbox::CHECKBOX_DOC,
    collapsible::COLLAPSIBLE_DOC,
    combobox::COMBOBOX_DOC,
    command::COMMAND_DOC,
    context_menu::CONTEXT_MENU_DOC,
    date_field::DATE_FIELD_DOC,
    date_picker::DATE_PICKER_DOC,
    date_range_field::DATE_RANGE_FIELD_DOC,
    date_range_picker::DATE_RANGE_PICKER_DOC,
    dialog::DIALOG_DOC,
    dropdown_menu::DROPDOWN_MENU_DOC,
    field::FIELD_DOC,
    hover_card::HOVER_CARD_DOC,
    label::LABEL_DOC,
    menubar::MENUBAR_DOC,
    meter::METER_DOC,
    navigation_menu::NAVIGATION_MENU_DOC,
    pagination::PAGINATION_DOC,
    pin_input::PIN_INPUT_DOC,
    popover::POPOVER_DOC,
    progress::PROGRESS_DOC,
    radio_group::RADIO_GROUP_DOC,
    range_calendar::RANGE_CALENDAR_DOC,
    rating_group::RATING_GROUP_DOC,
    scroll_area::SCROLL_AREA_DOC,
    select::SELECT_DOC,
    separator::SEPARATOR_DOC,
    slider::SLIDER_DOC,
    switch::SWITCH_DOC,
    tabs::TABS_DOC,
    time_field::TIME_FIELD_DOC,
    time_range_field::TIME_RANGE_FIELD_DOC,
    toast::TOAST_DOC,
    toggle::TOGGLE_DOC,
    toggle_group::TOGGLE_GROUP_DOC,
    toolbar::TOOLBAR_DOC,
    tooltip::TOOLTIP_DOC,
    visually_hidden::VISUALLY_HIDDEN_DOC,
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

use dioxus::prelude::*;

use crate::foundation::shared::use_stable_id;
use crate::menu::{
    MenuPart, MenuRootContext, MenuSeparatorAttributes, merge_attributes_with_part,
};

#[derive(Props, Clone, PartialEq)]
pub struct MenuSeparatorProps {
    #[props(default)]
    pub id: Option<String>,
    #[props(default)]
    pub class: Option<String>,
    #[props(default)]
    pub style: Option<String>,
    #[props(extends = GlobalAttributes)]
    pub attributes: Vec<Attribute>,
}

/// Non-focusable visual divider (`role="separator"`, `aria-orientation="horizontal"`).
#[component]
pub fn MenuSeparator(props: MenuSeparatorProps) -> Element {
    let root = use_context::<MenuRootContext>();
    let id = use_stable_id(props.id.map(Into::into)).into_owned();
    let attrs = MenuSeparatorAttributes::new(id, root.variant);
    let merged_attrs = merge_attributes_with_part(
        props.attributes,
        MenuPart::Separator.static_data_attr_name(root.variant),
    );

    rsx! {
        div {
            id: "{attrs.id}",
            class: props.class.as_deref().unwrap_or_default(),
            style: props.style.as_deref().unwrap_or_default(),
            role: "{attrs.role}",
            "aria-orientation": "{attrs.aria_orientation}",
            ..merged_attrs,
        }
    }
}

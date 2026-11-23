use dioxus::prelude::*;

use crate::foundation::shared::use_stable_id;
use crate::menu::{
    MenuHeadingAttributes, MenuPart, MenuRootContext, merge_attributes_with_part,
};

#[derive(Props, Clone, PartialEq)]
pub struct MenuLabelProps {
    #[props(default)]
    pub id: Option<String>,
    #[props(default)]
    pub class: Option<String>,
    #[props(default)]
    pub style: Option<String>,
    #[props(extends = GlobalAttributes)]
    pub attributes: Vec<Attribute>,
    pub children: Element,
}

/// Non-interactive section label rendered inside a menu (`3B D-6`).
#[component]
pub fn MenuLabel(props: MenuLabelProps) -> Element {
    let root = use_context::<MenuRootContext>();
    let id = use_stable_id(props.id.map(Into::into)).into_owned();
    let attrs = MenuHeadingAttributes::new(id, root.variant, MenuPart::Label);
    let merged_attrs = merge_attributes_with_part(
        props.attributes,
        MenuPart::Label.static_data_attr_name(root.variant),
    );

    rsx! {
        div {
            id: "{attrs.id}",
            class: props.class.as_deref().unwrap_or_default(),
            style: props.style.as_deref().unwrap_or_default(),
            ..merged_attrs,
            {props.children}
        }
    }
}

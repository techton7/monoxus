use dioxus::prelude::*;

use crate::foundation::{
    compose::{collect_user_attrs, merge_props},
    shared::use_stable_id,
};
use crate::menu::{MenuHeadingAttributes, MenuPart, MenuRootContext};

#[derive(Props, Clone, PartialEq)]
pub struct MenuLabelProps {
    #[props(default, into)]
    pub id: Option<String>,
    #[props(default, into)]
    pub class: Option<String>,
    #[props(default, into)]
    pub style: Option<String>,
    #[props(default)]
    pub child: Option<Callback<Vec<Attribute>, Element>>,
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
    let part_attr = MenuPart::Label.static_data_attr_name(root.variant);

    let base = vec![
        Attribute::new("id", attrs.id, None, false),
        Attribute::new(part_attr, "", None, false),
    ];
    let user = collect_user_attrs(props.class, props.style, props.attributes);
    let merged = merge_props(base, user);

    if let Some(child) = props.child {
        return child.call(merged);
    }

    rsx! {
        div {
            ..merged,
            {props.children}
        }
    }
}

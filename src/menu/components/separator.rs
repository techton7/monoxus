use dioxus::prelude::*;

use crate::foundation::{
    compose::{collect_user_attrs, merge_props},
    shared::use_stable_id,
};
use crate::menu::{MenuPart, MenuRootContext, MenuSeparatorAttributes};

#[derive(Props, Clone, PartialEq)]
pub struct MenuSeparatorProps {
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
}

/// Non-focusable visual divider (`role="separator"`, `aria-orientation="horizontal"`).
#[component]
pub fn MenuSeparator(props: MenuSeparatorProps) -> Element {
    let root = use_context::<MenuRootContext>();
    let id = use_stable_id(props.id.map(Into::into)).into_owned();
    let attrs = MenuSeparatorAttributes::new(id, root.variant);
    let part_attr = MenuPart::Separator.static_data_attr_name(root.variant);

    let base = vec![
        Attribute::new("id", attrs.id, None, false),
        Attribute::new("role", attrs.role, None, false),
        Attribute::new("aria-orientation", attrs.aria_orientation, None, false),
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
        }
    }
}

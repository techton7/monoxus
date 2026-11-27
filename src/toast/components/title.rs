use dioxus::prelude::*;

use crate::foundation::compose::{collect_user_attrs, merge_props};

#[derive(Props, Clone, PartialEq)]
pub struct ToastTitleProps {
    #[props(default, into)]
    pub class: Option<String>,
    #[props(default, into)]
    pub style: Option<String>,
    #[props(default)]
    pub child: Option<Callback<Vec<Attribute>, Element>>,
    #[props(extends = GlobalAttributes)]
    pub attributes: Vec<Attribute>,
    #[props(default)]
    pub children: Element,
}

/// Semantic heading container for primary toast message text.
#[component]
pub fn ToastTitle(props: ToastTitleProps) -> Element {
    let base = vec![Attribute::new("data-title", "", None, false)];
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

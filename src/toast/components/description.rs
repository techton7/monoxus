use dioxus::prelude::*;

use crate::foundation::compose::{collect_user_attrs, merge_props};

#[derive(Props, Clone, PartialEq)]
pub struct ToastDescriptionProps {
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

/// Secondary text container for toast description details.
#[component]
pub fn ToastDescription(props: ToastDescriptionProps) -> Element {
    let base = vec![Attribute::new("data-description", "", None, false)];
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

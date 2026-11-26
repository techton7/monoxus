use crate::foundation::compose::merge_global_attributes;
use dioxus::prelude::*;

#[derive(Props, Clone, PartialEq)]
pub struct ToastTitleProps {
    #[props(default)]
    pub class: Option<String>,
    #[props(extends = GlobalAttributes)]
    pub attributes: Vec<Attribute>,
    #[props(default)]
    pub children: Element,
}

/// Semantic heading container for primary toast message text.
#[component]
pub fn ToastTitle(props: ToastTitleProps) -> Element {
    let merged = merge_global_attributes(props.class.as_deref(), None, props.attributes);

    rsx! {
        div {
            "data-title": "",
            class: merged.class.as_deref().unwrap_or_default(),
            style: merged.style.as_deref(),
            ..merged.rest,
            {props.children}
        }
    }
}

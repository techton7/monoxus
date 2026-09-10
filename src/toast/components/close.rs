use crate::toast::attrs::ToastCloseAttrs;
use crate::toast::runtime::{TOAST_STORE, toast};
use crate::toast::types::ToastId;
use dioxus::prelude::*;

#[derive(Props, Clone, PartialEq)]
pub struct ToastCloseProps {
    #[props(default)]
    pub toast_id: Option<ToastId>,
    #[props(default)]
    pub aria_label: Option<String>,
    #[props(default)]
    pub class: Option<String>,
    #[props(default)]
    pub children: Element,
}

/// Accessible dismiss trigger button excluded from live region speech synthesis.
#[component]
pub fn ToastClose(props: ToastCloseProps) -> Element {
    let store = TOAST_STORE.read();
    let attrs = ToastCloseAttrs::new(&store.config);
    let label = props.aria_label.as_ref().unwrap_or(&attrs.aria_label);

    let handle_click = move |_evt: MouseEvent| {
        if let Some(id) = props.toast_id {
            toast::dismiss(Some(id));
        }
    };

    rsx! {
        button {
            r#type: "{attrs.r#type}",
            aria_label: "{label}",
            "data-radix-toast-announce-exclude": "{attrs.data_radix_toast_announce_exclude}",
            class: props.class.as_deref().unwrap_or_default(),
            onclick: handle_click,
            {props.children}
        }
    }
}

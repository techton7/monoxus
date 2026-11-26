use crate::foundation::compose::merge_global_attributes;
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
    #[props(extends = GlobalAttributes)]
    pub attributes: Vec<Attribute>,
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
            crate::toast::hotkey::handle_post_dismiss_focus(
                Some(id),
                "monoxus-toast-viewport",
            );
        }
    };

    let merged = merge_global_attributes(props.class.as_deref(), None, props.attributes);

    rsx! {
        button {
            r#type: "{attrs.r#type}",
            aria_label: "{label}",
            "data-close-button": "",
            "data-radix-toast-announce-exclude": "{attrs.data_radix_toast_announce_exclude}",
            class: merged.class.as_deref().unwrap_or_default(),
            style: merged.style.as_deref(),
            onclick: handle_click,
            ..merged.rest,
            {props.children}
        }
    }
}

use dioxus::prelude::*;
use crate::toast::attrs::ToastActionAttrs;
use crate::toast::runtime::toast;
use crate::toast::types::{ToastActionEvent, ToastId};

#[derive(Props, Clone, PartialEq)]
pub struct ToastActionProps {
    pub alt_text: String,
    #[props(default)]
    pub toast_id: Option<ToastId>,
    #[props(default)]
    pub on_click: Option<Callback<ToastActionEvent>>,
    #[props(default)]
    pub class: Option<String>,
    #[props(default)]
    pub children: Element,
}

/// Action button trigger invoking an event-aware callback and automatically
/// dismissing the toast unless `event.prevent_default()` was called.
#[component]
pub fn ToastAction(props: ToastActionProps) -> Element {
    let attrs = ToastActionAttrs::new(props.alt_text.clone());

    let handle_click = move |_evt: MouseEvent| {
        let action_evt = ToastActionEvent::new();
        if let Some(cb) = &props.on_click {
            cb.call(action_evt.clone());
        }
        if let Some(id) = props.toast_id.filter(|_| !action_evt.is_default_prevented()) {
            toast::dismiss(Some(id));
        }
    };

    rsx! {
        button {
            r#type: "{attrs.r#type}",
            aria_label: "{attrs.aria_label}",
            class: props.class.as_deref().unwrap_or_default(),
            onclick: handle_click,
            {props.children}
        }
    }
}

use dioxus::core::{AttributeValue, Event, ListenerCallback};
use dioxus::html::PlatformEventData;
use dioxus::prelude::*;

use crate::foundation::compose::{
    collect_user_attrs, merge_props, push_bool_data_attr, push_event_handler,
};
use crate::toast::attrs::ToastActionAttrs;
use crate::toast::runtime::toast;
use crate::toast::types::ToastId;

#[derive(Props, Clone, PartialEq)]
pub struct ToastActionProps {
    #[props(into)]
    pub alt_text: String,
    #[props(default)]
    pub toast_id: Option<ToastId>,
    #[props(default)]
    pub on_click: Option<EventHandler<MouseEvent>>,
    #[props(default = false)]
    pub cancel: bool,
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

/// Action button trigger invoking an optional click handler and automatically
/// dismissing the toast unless `event.prevent_default()` was called.
#[component]
pub fn ToastAction(props: ToastActionProps) -> Element {
    let attrs = ToastActionAttrs::new(props.alt_text);
    let toast_id = props.toast_id;

    let mut base = Vec::with_capacity(5);
    base.push(Attribute::new("type", attrs.r#type, None, false));
    base.push(Attribute::new("aria-label", attrs.aria_label, None, false));
    push_bool_data_attr(&mut base, "data-button", !props.cancel);
    push_bool_data_attr(&mut base, "data-cancel", props.cancel);
    base.push(Attribute::new(
        "onclick",
        AttributeValue::Listener(
            ListenerCallback::<PlatformEventData>::new(move |evt: Event<PlatformEventData>| {
                if evt.default_action_enabled()
                    && let Some(id) = toast_id
                {
                    toast::dismiss(Some(id));
                    crate::toast::hotkey::handle_post_dismiss_focus(
                        Some(id),
                        "monoxus-toast-viewport",
                    );
                }
            })
            .erase(),
        ),
        None,
        false,
    ));

    let mut user = collect_user_attrs(props.class, props.style, props.attributes);
    push_event_handler(&mut user, "onclick", props.on_click);
    let merged = merge_props(base, user);

    if let Some(child) = props.child {
        return child.call(merged);
    }

    rsx! {
        button {
            ..merged,
            {props.children}
        }
    }
}

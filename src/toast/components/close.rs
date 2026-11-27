use dioxus::core::{AttributeValue, Event, ListenerCallback};
use dioxus::html::PlatformEventData;
use dioxus::prelude::*;

use crate::foundation::compose::{collect_user_attrs, merge_props};
use crate::toast::attrs::ToastCloseAttrs;
use crate::toast::runtime::{TOAST_STORE, toast};
use crate::toast::types::ToastId;

#[derive(Props, Clone, PartialEq)]
pub struct ToastCloseProps {
    #[props(default)]
    pub toast_id: Option<ToastId>,
    #[props(default, into)]
    pub aria_label: Option<String>,
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

/// Accessible dismiss trigger button excluded from live region speech synthesis.
#[component]
pub fn ToastClose(props: ToastCloseProps) -> Element {
    let store = TOAST_STORE.read();
    let attrs = ToastCloseAttrs::new(&store.config);
    let label = props.aria_label.unwrap_or(attrs.aria_label);
    let toast_id = props.toast_id;

    let mut base = Vec::with_capacity(5);
    base.push(Attribute::new("type", attrs.r#type, None, false));
    base.push(Attribute::new("aria-label", label, None, false));
    base.push(Attribute::new("data-close-button", "", None, false));
    base.push(Attribute::new(
        "data-radix-toast-announce-exclude",
        attrs.data_radix_toast_announce_exclude,
        None,
        false,
    ));
    base.push(Attribute::new(
        "onclick",
        AttributeValue::Listener(
            ListenerCallback::<PlatformEventData>::new(move |_evt: Event<PlatformEventData>| {
                if let Some(id) = toast_id {
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

    let user = collect_user_attrs(props.class, props.style, props.attributes);
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

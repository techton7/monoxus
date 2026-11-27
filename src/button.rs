use dioxus::prelude::*;

use crate::foundation::{
    compose::{
        MergePropsConfig, collect_user_attrs, merge_props, merge_props_with_config, push_attr_opt,
        push_event_handler,
    },
    keyboard::is_activation_key,
};

pub use ui_sdk::button::*;

/// Properties for the headless `Button` component.
#[derive(Clone, PartialEq, Props)]
pub struct ButtonProps {
    #[props(default, into)]
    pub id: Option<String>,
    #[props(default, into)]
    pub class: Option<String>,
    #[props(default, into)]
    pub style: Option<String>,
    #[props(default)]
    pub r#type: ButtonType,
    #[props(default, into)]
    pub form: Option<String>,
    #[props(default, into)]
    pub href: Option<String>,
    #[props(default, into)]
    pub target: Option<String>,
    #[props(default, into)]
    pub rel: Option<String>,
    #[props(default, into)]
    pub download: Option<String>,
    #[props(default = false)]
    pub disabled: bool,
    #[props(default)]
    pub tabindex: Option<i32>,
    #[props(default, into)]
    pub role: Option<String>,
    #[props(default)]
    pub on_click: Option<EventHandler<MouseEvent>>,
    #[props(default)]
    pub on_key_down: Option<EventHandler<KeyboardEvent>>,
    #[props(default)]
    pub on_mounted: Option<EventHandler<MountedEvent>>,
    #[props(default)]
    pub child: Option<Callback<Vec<Attribute>, Element>>,
    #[props(extends = GlobalAttributes)]
    pub attributes: Vec<Attribute>,
    pub children: Element,
}

/// Headless polymorphic button component rendering `<button>` or `<a>`.
#[component]
pub fn Button(props: ButtonProps) -> Element {
    let state = ButtonState {
        id: props.id.clone(),
        button_type: props.r#type,
        href: props.href.clone(),
        disabled: props.disabled,
        role: props.role.clone(),
        tabindex: props.tabindex,
    };
    let attrs = state.attributes();
    let disabled = props.disabled;

    let mut base = Vec::with_capacity(14);
    push_attr_opt(&mut base, "id", attrs.id);
    if attrs.tag == ButtonTag::Anchor {
        push_attr_opt(&mut base, "href", attrs.href);
        push_attr_opt(&mut base, "role", attrs.role);
        push_attr_opt(&mut base, "tabindex", attrs.tabindex);
        push_attr_opt(&mut base, "aria-disabled", attrs.aria_disabled);
        if !disabled {
            push_attr_opt(&mut base, "target", props.target);
            push_attr_opt(&mut base, "rel", props.rel);
            push_attr_opt(&mut base, "download", props.download);
        }
    } else {
        push_attr_opt(&mut base, "type", attrs.r#type);
        push_attr_opt(&mut base, "form", props.form);
        if disabled {
            base.push(Attribute::new("disabled", true, None, false));
        }
        push_attr_opt(&mut base, "role", attrs.role);
        push_attr_opt(&mut base, "tabindex", attrs.tabindex);
    }
    base.push(Attribute::new(
        "data-button-root",
        attrs.data_button_root,
        None,
        false,
    ));
    push_attr_opt(&mut base, "data-disabled", attrs.data_disabled);

    if disabled {
        push_event_handler(
            &mut base,
            "onclick",
            Some(EventHandler::new(move |evt: MouseEvent| {
                evt.prevent_default();
            })),
        );
        push_event_handler(
            &mut base,
            "onkeydown",
            Some(EventHandler::new(move |evt: KeyboardEvent| {
                if is_activation_key(&evt.key()) {
                    evt.prevent_default();
                }
            })),
        );
    }

    let mut user = collect_user_attrs(props.class, props.style, props.attributes);
    if !disabled {
        push_event_handler(&mut user, "onclick", props.on_click);
        push_event_handler(&mut user, "onkeydown", props.on_key_down);
    } else if let Some(handler) = props.on_key_down {
        push_event_handler(
            &mut user,
            "onkeydown",
            Some(EventHandler::new(move |evt: KeyboardEvent| {
                if is_activation_key(&evt.key()) {
                    evt.prevent_default();
                    return;
                }
                handler.call(evt);
            })),
        );
    }
    push_event_handler(&mut user, "onmounted", props.on_mounted);

    let merged = if disabled {
        merge_props_with_config(
            base,
            user,
            MergePropsConfig {
                invariant_attrs: &[],
                bookkeeping_events: &["onclick", "onkeydown"],
            },
        )
    } else {
        merge_props(base, user)
    };

    if let Some(child) = props.child {
        return child.call(merged);
    }

    if attrs.tag == ButtonTag::Anchor {
        rsx! {
            a {
                ..merged,
                {props.children}
            }
        }
    } else {
        rsx! {
            button {
                ..merged,
                {props.children}
            }
        }
    }
}

pub use Button as ButtonRoot;

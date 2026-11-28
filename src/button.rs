use dioxus::prelude::*;

use crate::foundation::{
    compose::merge_global_attributes,
    keyboard::is_activation_key,
};

pub use ui_sdk::button::*;

/// Properties for the headless `Button` component.
#[derive(Clone, PartialEq, Props)]
pub struct ButtonProps {
    #[props(default)]
    pub id: Option<String>,
    #[props(default)]
    pub class: Option<String>,
    #[props(default)]
    pub style: Option<String>,
    #[props(default)]
    pub r#type: ButtonType,
    #[props(default)]
    pub form: Option<String>,
    #[props(default)]
    pub href: Option<String>,
    #[props(default)]
    pub target: Option<String>,
    #[props(default)]
    pub rel: Option<String>,
    #[props(default)]
    pub download: Option<String>,
    #[props(default = false)]
    pub disabled: bool,
    #[props(default)]
    pub tabindex: Option<i32>,
    #[props(default)]
    pub role: Option<String>,
    #[props(default)]
    pub aria_label: Option<String>,
    #[props(default)]
    pub aria_describedby: Option<String>,
    #[props(default)]
    pub aria_expanded: Option<bool>,
    #[props(default)]
    pub aria_controls: Option<String>,
    #[props(default)]
    pub aria_haspopup: Option<String>,
    #[props(default)]
    pub aria_invalid: Option<String>,
    #[props(default)]
    pub on_click: Option<EventHandler<MouseEvent>>,
    #[props(default)]
    pub on_key_down: Option<EventHandler<KeyboardEvent>>,
    #[props(default)]
    pub on_mounted: Option<EventHandler<MountedEvent>>,
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

    let handle_click = move |evt: MouseEvent| {
        if props.disabled {
            evt.prevent_default();
            return;
        }
        if let Some(handler) = props.on_click.as_ref() {
            handler.call(evt);
        }
    };

    let handle_keydown = move |evt: KeyboardEvent| {
        if props.disabled && is_activation_key(&evt.key()) {
            evt.prevent_default();
            return;
        }
        if let Some(handler) = props.on_key_down.as_ref() {
            handler.call(evt);
        }
    };

    let merged = merge_global_attributes(
        props.class.as_deref(),
        props.style.as_deref(),
        props.attributes,
    );

    if attrs.tag == ButtonTag::Anchor {
        let is_enabled = !props.disabled;
        rsx! {
            a {
                id: attrs.id,
                href: attrs.href,
                role: attrs.role,
                tabindex: attrs.tabindex,
                aria_disabled: attrs.aria_disabled,
                aria_label: props.aria_label,
                aria_describedby: props.aria_describedby,
                aria_expanded: props.aria_expanded,
                aria_controls: props.aria_controls,
                aria_haspopup: props.aria_haspopup,
                aria_invalid: props.aria_invalid,
                target: if is_enabled { props.target.as_deref() } else { None },
                rel: if is_enabled { props.rel.as_deref() } else { None },
                download: if is_enabled { props.download.as_deref() } else { None },
                class: merged.class,
                style: merged.style,
                "data-button-root": attrs.data_button_root,
                "data-disabled": attrs.data_disabled,
                onclick: handle_click,
                onkeydown: handle_keydown,
                onmounted: move |evt| {
                    if let Some(handler) = props.on_mounted.as_ref() {
                        handler.call(evt);
                    }
                },
                ..merged.rest,
                {props.children}
            }
        }
    } else {
        rsx! {
            button {
                id: attrs.id,
                r#type: attrs.r#type,
                form: props.form.as_deref(),
                disabled: props.disabled,
                role: attrs.role,
                tabindex: attrs.tabindex,
                aria_label: props.aria_label,
                aria_describedby: props.aria_describedby,
                aria_expanded: props.aria_expanded,
                aria_controls: props.aria_controls,
                aria_haspopup: props.aria_haspopup,
                aria_invalid: props.aria_invalid,
                class: merged.class,
                style: merged.style,
                "data-button-root": attrs.data_button_root,
                "data-disabled": attrs.data_disabled,
                onclick: handle_click,
                onkeydown: handle_keydown,
                onmounted: move |evt| {
                    if let Some(handler) = props.on_mounted.as_ref() {
                        handler.call(evt);
                    }
                },
                ..merged.rest,
                {props.children}
            }
        }
    }
}

pub use Button as ButtonRoot;

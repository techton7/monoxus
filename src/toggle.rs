use dioxus::prelude::*;

use crate::foundation::state::{ControllableStateProps, use_controllable_state};

pub use monoxus_core::toggle::*;

#[derive(Props, Clone, PartialEq)]
pub struct ToggleProps {
    #[props(default)]
    pub id: Option<String>,
    #[props(default)]
    pub class: Option<String>,
    #[props(default)]
    pub style: Option<String>,
    #[props(default)]
    pub pressed: Option<bool>,
    #[props(default = false)]
    pub default_pressed: bool,
    #[props(default)]
    pub on_pressed_change: Option<EventHandler<bool>>,
    #[props(default = false)]
    pub disabled: bool,
    #[props(default)]
    pub r#type: ButtonType,
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

#[component]
pub fn Toggle(props: ToggleProps) -> Element {
    let on_change = props.on_pressed_change.as_ref().map(|handler| {
        let handler = handler.clone();
        move |next: bool| {
            handler.call(next);
        }
    });

    let pressed_handle = use_controllable_state(ControllableStateProps {
        value: props.pressed,
        default_value: props.default_pressed,
        on_change,
    });

    let is_pressed = *pressed_handle.value();

    let mut toggle_state = Toggle::new()
        .with_pressed(is_pressed)
        .with_disabled(props.disabled)
        .with_button_type(props.r#type);
    if let Some(id) = props.id.as_ref() {
        toggle_state = toggle_state.with_id(id.clone());
    }
    let attrs = toggle_state.attributes();

    let on_click = {
        let on_click = props.on_click.clone();
        let disabled = props.disabled;
        let handle = pressed_handle.clone();
        move |evt: MouseEvent| {
            if let Some(ref handler) = on_click {
                handler.call(evt.clone());
            }
            if evt.default_action_enabled() && !disabled {
                handle.set(!is_pressed);
            }
        }
    };

    let on_key_down = {
        let on_key_down = props.on_key_down.clone();
        move |evt: KeyboardEvent| {
            if let Some(ref handler) = on_key_down {
                handler.call(evt);
            }
        }
    };

    let on_mounted = {
        let on_mounted = props.on_mounted.clone();
        move |evt: MountedEvent| {
            if let Some(ref handler) = on_mounted {
                handler.call(evt);
            }
        }
    };

    rsx! {
        button {
            id: attrs.id.as_deref(),
            class: props.class.as_deref().unwrap_or_default(),
            style: props.style.as_deref().unwrap_or_default(),
            r#type: attrs.r#type,
            disabled: attrs.disabled,
            "aria-pressed": attrs.aria_pressed,
            "data-state": attrs.data_state.as_str(),
            "data-disabled": attrs.data_disabled,
            "data-toggle-root": attrs.data_toggle_root,
            onclick: on_click,
            onkeydown: on_key_down,
            onmounted: on_mounted,
            ..props.attributes,
            {props.children}
        }
    }
}

pub use Toggle as ToggleRoot;

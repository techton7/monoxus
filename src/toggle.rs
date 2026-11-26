use dioxus::prelude::*;

use crate::foundation::{
    compose::merge_global_attributes,
    state::use_controllable_signal,
};

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
    let pressed_ctl =
        use_controllable_signal(props.pressed, props.default_pressed, props.on_pressed_change);
    let is_pressed = pressed_ctl.get();

    let mut toggle_state = Toggle::new()
        .with_pressed(is_pressed)
        .with_disabled(props.disabled)
        .with_button_type(props.r#type);
    if let Some(id) = props.id.as_ref() {
        toggle_state = toggle_state.with_id(id.clone());
    }
    let attrs = toggle_state.attributes();

    let on_click = {
        let on_click = props.on_click;
        let disabled = props.disabled;
        move |evt: MouseEvent| {
            if let Some(ref handler) = on_click {
                handler.call(evt.clone());
            }
            if evt.default_action_enabled() && !disabled {
                let current = pressed_ctl.peek();
                pressed_ctl.set(!current);
            }
        }
    };

    let on_key_down = {
        let on_key_down = props.on_key_down;
        move |evt: KeyboardEvent| {
            if let Some(ref handler) = on_key_down {
                handler.call(evt);
            }
        }
    };

    let on_mounted = {
        let on_mounted = props.on_mounted;
        move |evt: MountedEvent| {
            if let Some(ref handler) = on_mounted {
                handler.call(evt);
            }
        }
    };

    let merged = merge_global_attributes(
        props.class.as_deref(),
        props.style.as_deref(),
        props.attributes,
    );

    rsx! {
        button {
            id: attrs.id.as_deref(),
            class: merged.class.as_deref().unwrap_or_default(),
            style: merged.style.as_deref().unwrap_or_default(),
            r#type: attrs.r#type,
            disabled: attrs.disabled,
            "aria-pressed": attrs.aria_pressed,
            "data-state": attrs.data_state.as_str(),
            "data-disabled": attrs.data_disabled,
            "data-toggle-root": attrs.data_toggle_root,
            onclick: on_click,
            onkeydown: on_key_down,
            onmounted: on_mounted,
            ..merged.rest,
            {props.children}
        }
    }
}

pub use Toggle as ToggleRoot;

use dioxus::prelude::*;

use crate::foundation::{
    compose::{project_as_child, Slottable},
    shared::data_disabled_attr,
    state::{use_controllable_state, ControllableStateProps, DataState},
};

pub use crate::button::ButtonType;

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Toggle {
    id: Option<String>,
    pressed: bool,
    disabled: bool,
    button_type: ButtonType,
}

impl Default for Toggle {
    fn default() -> Self {
        Self::new()
    }
}

impl Toggle {
    pub const fn new() -> Self {
        Self {
            id: None,
            pressed: false,
            disabled: false,
            button_type: ButtonType::Button,
        }
    }

    pub fn with_id(mut self, id: impl Into<String>) -> Self {
        self.id = Some(id.into());
        self
    }

    pub const fn with_pressed(mut self, pressed: bool) -> Self {
        self.pressed = pressed;
        self
    }

    pub const fn with_disabled(mut self, disabled: bool) -> Self {
        self.disabled = disabled;
        self
    }

    pub const fn with_button_type(mut self, button_type: ButtonType) -> Self {
        self.button_type = button_type;
        self
    }

    pub fn id(&self) -> Option<&str> {
        self.id.as_deref()
    }

    pub const fn is_pressed(&self) -> bool {
        self.pressed
    }

    pub const fn is_disabled(&self) -> bool {
        self.disabled
    }

    pub const fn button_type(&self) -> ButtonType {
        self.button_type
    }

    pub fn toggle(&mut self) {
        if !self.disabled {
            self.pressed = !self.pressed;
        }
    }

    pub fn attributes(&self) -> ToggleAttributes {
        ToggleAttributes {
            id: self.id.clone(),
            r#type: self.button_type.as_str(),
            aria_pressed: if self.pressed { "true" } else { "false" },
            data_state: if self.pressed {
                DataState::On
            } else {
                DataState::Off
            },
            data_disabled: data_disabled_attr(self.disabled),
            data_toggle_root: "",
            disabled: self.disabled,
        }
    }
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct ToggleAttributes {
    pub id: Option<String>,
    pub r#type: &'static str,
    pub aria_pressed: &'static str,
    pub data_state: DataState,
    pub data_disabled: Option<&'static str>,
    pub data_toggle_root: &'static str,
    pub disabled: bool,
}

impl ToggleAttributes {
    pub fn id(&self) -> Option<&str> {
        self.id.as_deref()
    }

    pub const fn r#type(&self) -> &'static str {
        self.r#type
    }

    pub const fn aria_pressed(&self) -> &'static str {
        self.aria_pressed
    }

    pub fn data_state(&self) -> DataState {
        self.data_state.clone()
    }

    pub fn data_state_str(&self) -> &str {
        self.data_state.as_str()
    }

    pub const fn data_disabled(&self) -> Option<&'static str> {
        self.data_disabled
    }

    pub const fn data_toggle_root(&self) -> &'static str {
        self.data_toggle_root
    }

    pub const fn is_disabled(&self) -> bool {
        self.disabled
    }
}

pub fn project_toggle_as_child<T, C>(
    toggle: &Toggle,
    target: T,
    content: Slottable<C>,
) -> (T, ToggleAttributes, C) {
    let (target, content) = project_as_child(target, content);
    (target, toggle.attributes(), content)
}

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

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn default_toggle_is_unpressed_and_enabled() {
        let toggle = Toggle::new();
        assert!(!toggle.is_pressed());
        assert!(!toggle.is_disabled());
        assert_eq!(toggle.button_type(), ButtonType::Button);

        let attrs = toggle.attributes();
        assert_eq!(attrs.r#type, "button");
        assert_eq!(attrs.aria_pressed, "false");
        assert_eq!(attrs.data_state, DataState::Off);
        assert_eq!(attrs.data_disabled, None);
        assert_eq!(attrs.data_toggle_root, "");
        assert!(!attrs.disabled);
    }

    #[test]
    fn pressed_toggle_emits_aria_and_data_attributes() {
        let toggle = Toggle::new().with_pressed(true);
        assert!(toggle.is_pressed());

        let attrs = toggle.attributes();
        assert_eq!(attrs.aria_pressed, "true");
        assert_eq!(attrs.data_state, DataState::On);
        assert_eq!(attrs.data_disabled, None);
    }

    #[test]
    fn disabled_toggle_emits_data_disabled_empty_str() {
        let toggle = Toggle::new().with_disabled(true);
        assert!(toggle.is_disabled());

        let attrs = toggle.attributes();
        assert_eq!(attrs.aria_pressed, "false");
        assert_eq!(attrs.data_state, DataState::Off);
        assert_eq!(attrs.data_disabled, Some(""));
        assert!(attrs.disabled);
    }

    #[test]
    fn toggle_transition_flips_pressed_when_enabled() {
        let mut toggle = Toggle::new();
        toggle.toggle();
        assert!(toggle.is_pressed());
        assert_eq!(toggle.attributes().aria_pressed, "true");
        assert_eq!(toggle.attributes().data_state, DataState::On);

        toggle.toggle();
        assert!(!toggle.is_pressed());
        assert_eq!(toggle.attributes().aria_pressed, "false");
        assert_eq!(toggle.attributes().data_state, DataState::Off);
    }

    #[test]
    fn toggle_transition_is_noop_when_disabled() {
        let mut toggle = Toggle::new().with_disabled(true);
        toggle.toggle();
        assert!(!toggle.is_pressed());
        assert_eq!(toggle.attributes().aria_pressed, "false");
    }

    #[test]
    fn project_toggle_as_child_attaches_attributes() {
        let toggle = Toggle::new().with_pressed(true);
        let slottable = Slottable::new("Content text");
        let (target, attrs, content) = project_toggle_as_child(&toggle, 42_u32, slottable);

        assert_eq!(target, 42);
        assert_eq!(content, "Content text");
        assert_eq!(attrs.aria_pressed, "true");
        assert_eq!(attrs.data_state, DataState::On);
    }
}

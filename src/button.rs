use std::fmt;

use dioxus::prelude::keyboard_types::{Key, NamedKey};
use dioxus::prelude::*;

use crate::foundation::compose::{Slottable, project_as_child};

/// Native HTML button type attribute variants.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, Hash)]
pub enum ButtonType {
    #[default]
    Button,
    Submit,
    Reset,
}

impl ButtonType {
    /// Returns the string representation matching standard HTML `type` values.
    pub const fn as_str(&self) -> &'static str {
        match self {
            Self::Button => "button",
            Self::Submit => "submit",
            Self::Reset => "reset",
        }
    }
}

impl fmt::Display for ButtonType {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(self.as_str())
    }
}

/// Element tag rendered by the polymorphic button primitive.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub enum ButtonTag {
    Button,
    Anchor,
}

/// Pure state definition and attribute projector for `Button`.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct ButtonState {
    pub id: Option<String>,
    pub button_type: ButtonType,
    pub href: Option<String>,
    pub disabled: bool,
    pub role: Option<String>,
    pub tabindex: Option<i32>,
}

impl Default for ButtonState {
    fn default() -> Self {
        Self {
            id: None,
            button_type: ButtonType::Button,
            href: None,
            disabled: false,
            role: None,
            tabindex: None,
        }
    }
}

impl ButtonState {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn with_id(mut self, id: impl Into<String>) -> Self {
        self.id = Some(id.into());
        self
    }

    pub fn with_button_type(mut self, button_type: ButtonType) -> Self {
        self.button_type = button_type;
        self
    }

    pub fn with_href(mut self, href: impl Into<String>) -> Self {
        self.href = Some(href.into());
        self
    }

    pub fn with_disabled(mut self, disabled: bool) -> Self {
        self.disabled = disabled;
        self
    }

    pub fn with_role(mut self, role: impl Into<String>) -> Self {
        self.role = Some(role.into());
        self
    }

    pub fn with_tabindex(mut self, tabindex: i32) -> Self {
        self.tabindex = Some(tabindex);
        self
    }

    /// Evaluates the 4-state attribute projection matrix for `<button>` and `<a>`.
    pub fn attributes(&self) -> ButtonAttributes {
        let tag = if self.href.is_some() {
            ButtonTag::Anchor
        } else {
            ButtonTag::Button
        };

        let data_disabled = crate::foundation::shared::data_disabled_attr(self.disabled);

        match tag {
            ButtonTag::Button => ButtonAttributes {
                tag: ButtonTag::Button,
                id: self.id.clone(),
                r#type: Some(self.button_type.as_str()),
                href: None,
                disabled: Some(self.disabled),
                aria_disabled: None,
                role: self.role.clone(),
                tabindex: self.tabindex,
                data_button_root: "",
                data_disabled,
            },
            ButtonTag::Anchor => {
                if self.disabled {
                    ButtonAttributes {
                        tag: ButtonTag::Anchor,
                        id: self.id.clone(),
                        r#type: None,
                        href: None,
                        disabled: None,
                        aria_disabled: Some("true"),
                        role: self.role.clone().or_else(|| Some("link".to_string())),
                        tabindex: Some(-1),
                        data_button_root: "",
                        data_disabled,
                    }
                } else {
                    ButtonAttributes {
                        tag: ButtonTag::Anchor,
                        id: self.id.clone(),
                        r#type: None,
                        href: self.href.clone(),
                        disabled: None,
                        aria_disabled: None,
                        role: self.role.clone(),
                        tabindex: self.tabindex,
                        data_button_root: "",
                        data_disabled,
                    }
                }
            }
        }
    }
}

/// Computed HTML and WAI-ARIA attributes published by `Button`.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct ButtonAttributes {
    pub tag: ButtonTag,
    pub id: Option<String>,
    pub r#type: Option<&'static str>,
    pub href: Option<String>,
    pub disabled: Option<bool>,
    pub aria_disabled: Option<&'static str>,
    pub role: Option<String>,
    pub tabindex: Option<i32>,
    pub data_button_root: &'static str,
    pub data_disabled: Option<&'static str>,
}

/// Projects headless button attributes onto a slottable child without adding wrapper elements.
pub fn project_button_as_child<T, C>(
    state: &ButtonState,
    target: T,
    content: Slottable<C>,
) -> (T, ButtonAttributes, C) {
    let (target, content) = project_as_child(target, content);
    (target, state.attributes(), content)
}

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
        if props.disabled {
            let key = evt.key();
            if key == Key::Named(NamedKey::Enter) || key == Key::Character(" ".to_string()) {
                evt.prevent_default();
                return;
            }
        }
        if let Some(handler) = props.on_key_down.as_ref() {
            handler.call(evt);
        }
    };

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
                class: props.class,
                style: props.style,
                "data-button-root": attrs.data_button_root,
                "data-disabled": attrs.data_disabled,
                onclick: handle_click,
                onkeydown: handle_keydown,
                onmounted: move |evt| {
                    if let Some(handler) = props.on_mounted.as_ref() {
                        handler.call(evt);
                    }
                },
                ..props.attributes,
                {props.children}
            }
        }
    } else {
        rsx! {
            button {
                id: attrs.id,
                r#type: attrs.r#type,
                disabled: props.disabled,
                role: attrs.role,
                tabindex: attrs.tabindex,
                aria_label: props.aria_label,
                aria_describedby: props.aria_describedby,
                aria_expanded: props.aria_expanded,
                aria_controls: props.aria_controls,
                aria_haspopup: props.aria_haspopup,
                aria_invalid: props.aria_invalid,
                class: props.class,
                style: props.style,
                "data-button-root": attrs.data_button_root,
                "data-disabled": attrs.data_disabled,
                onclick: handle_click,
                onkeydown: handle_keydown,
                onmounted: move |evt| {
                    if let Some(handler) = props.on_mounted.as_ref() {
                        handler.call(evt);
                    }
                },
                ..props.attributes,
                {props.children}
            }
        }
    }
}

pub use Button as ButtonRoot;

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn button_state_4_state_matrix() {
        // State 1: <button> enabled
        let s1 = ButtonState::new()
            .with_id("b1")
            .with_button_type(ButtonType::Submit);
        let a1 = s1.attributes();
        assert_eq!(a1.tag, ButtonTag::Button);
        assert_eq!(a1.id.as_deref(), Some("b1"));
        assert_eq!(a1.r#type, Some("submit"));
        assert_eq!(a1.href, None);
        assert_eq!(a1.disabled, Some(false));
        assert_eq!(a1.aria_disabled, None);
        assert_eq!(a1.role, None);
        assert_eq!(a1.tabindex, None);
        assert_eq!(a1.data_button_root, "");
        assert_eq!(a1.data_disabled, None);

        // State 2: <button> disabled
        let s2 = ButtonState::new().with_id("b2").with_disabled(true);
        let a2 = s2.attributes();
        assert_eq!(a2.tag, ButtonTag::Button);
        assert_eq!(a2.id.as_deref(), Some("b2"));
        assert_eq!(a2.r#type, Some("button"));
        assert_eq!(a2.href, None);
        assert_eq!(a2.disabled, Some(true));
        assert_eq!(a2.aria_disabled, None);
        assert_eq!(a2.role, None);
        assert_eq!(a2.tabindex, None);
        assert_eq!(a2.data_button_root, "");
        assert_eq!(a2.data_disabled, Some(""));

        // State 3: <a> enabled
        let s3 = ButtonState::new()
            .with_id("a1")
            .with_href("https://example.com");
        let a3 = s3.attributes();
        assert_eq!(a3.tag, ButtonTag::Anchor);
        assert_eq!(a3.id.as_deref(), Some("a1"));
        assert_eq!(a3.r#type, None);
        assert_eq!(a3.href.as_deref(), Some("https://example.com"));
        assert_eq!(a3.disabled, None);
        assert_eq!(a3.aria_disabled, None);
        assert_eq!(a3.role, None);
        assert_eq!(a3.tabindex, None);
        assert_eq!(a3.data_button_root, "");
        assert_eq!(a3.data_disabled, None);

        // State 4: <a> disabled
        let s4 = ButtonState::new()
            .with_id("a2")
            .with_href("https://example.com")
            .with_disabled(true);
        let a4 = s4.attributes();
        assert_eq!(a4.tag, ButtonTag::Anchor);
        assert_eq!(a4.id.as_deref(), Some("a2"));
        assert_eq!(a4.r#type, None);
        assert_eq!(a4.href, None);
        assert_eq!(a4.disabled, None);
        assert_eq!(a4.aria_disabled, Some("true"));
        assert_eq!(a4.role.as_deref(), Some("link"));
        assert_eq!(a4.tabindex, Some(-1));
        assert_eq!(a4.data_button_root, "");
        assert_eq!(a4.data_disabled, Some(""));
    }

    #[test]
    fn button_state_custom_role_and_tabindex_preserved() {
        let s = ButtonState::new()
            .with_href("/path")
            .with_disabled(true)
            .with_role("menuitem");
        let a = s.attributes();
        assert_eq!(a.role.as_deref(), Some("menuitem"));
        assert_eq!(a.tabindex, Some(-1));

        let s_enabled = ButtonState::new().with_tabindex(4).with_role("switch");
        let a_enabled = s_enabled.attributes();
        assert_eq!(a_enabled.tabindex, Some(4));
        assert_eq!(a_enabled.role.as_deref(), Some("switch"));
    }

    #[test]
    fn button_as_child_slot_projection() {
        let state = ButtonState::new().with_id("slot-btn");
        let (target, attrs, content) =
            project_button_as_child(&state, "target-wrapper", Slottable::new("inner-content"));
        assert_eq!(target, "target-wrapper");
        assert_eq!(attrs.id.as_deref(), Some("slot-btn"));
        assert_eq!(attrs.data_button_root, "");
        assert_eq!(content, "inner-content");
    }
}

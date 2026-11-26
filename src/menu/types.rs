use dioxus::prelude::keyboard_types::{Key, NamedKey};

pub use crate::foundation::compose::{extract_vnode_text, merge_attributes_with_part};
pub use crate::foundation::shared::PreventableEvent;
pub use monoxus_core::menu::types::*;

/// Select event passed to `Item`, `CheckboxItem`, `RadioItem`, and `SubTrigger` `on_select` handlers (`M-10`, `3B D-6`).
pub type MenuSelectEvent = PreventableEvent<()>;

/// Preventable outside-interaction event passed to `on_pointer_down_outside` / `on_focus_outside` / `on_interact_outside`.
pub type MenuOutsideEvent = PreventableEvent<Option<String>>;

/// Preventable event passed to `on_escape_key_down`, `on_open_auto_focus`, and `on_close_auto_focus`.
pub type MenuPreventableEvent = PreventableEvent<()>;

/// Translate Dioxus `keyboard_types::Key` into host-neutral [`MenuKey`].
pub fn translate_menu_key(key: &Key) -> Option<MenuKey> {
    match key {
        Key::Named(NamedKey::ArrowDown) => Some(MenuKey::ArrowDown),
        Key::Named(NamedKey::ArrowUp) => Some(MenuKey::ArrowUp),
        Key::Named(NamedKey::ArrowRight) => Some(MenuKey::ArrowRight),
        Key::Named(NamedKey::ArrowLeft) => Some(MenuKey::ArrowLeft),
        Key::Named(NamedKey::Home) => Some(MenuKey::Home),
        Key::Named(NamedKey::End) => Some(MenuKey::End),
        Key::Named(NamedKey::PageUp) => Some(MenuKey::PageUp),
        Key::Named(NamedKey::PageDown) => Some(MenuKey::PageDown),
        Key::Named(NamedKey::Enter) => Some(MenuKey::Enter),
        Key::Named(NamedKey::Escape) => Some(MenuKey::Escape),
        Key::Named(NamedKey::Tab) => Some(MenuKey::Tab),
        Key::Named(NamedKey::Backspace) => Some(MenuKey::Backspace),
        Key::Character(s) => {
            if s == " " {
                Some(MenuKey::Space)
            } else {
                let mut chars = s.chars();
                match (chars.next(), chars.next()) {
                    (Some(c), None) if !c.is_control() => Some(MenuKey::Char(c)),
                    _ => None,
                }
            }
        }
        _ => None,
    }
}

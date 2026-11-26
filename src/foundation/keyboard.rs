use dioxus::prelude::KeyboardEvent;
use dioxus::prelude::keyboard_types::{Key, NamedKey};

pub use monoxus_core::foundation::keyboard::*;

/// Translates a Dioxus `keyboard_types::Key` into a host-neutral [`NavKey`].
pub fn translate_nav_key(key: &Key) -> Option<NavKey> {
    match key {
        Key::Named(NamedKey::ArrowUp) => Some(NavKey::ArrowUp),
        Key::Named(NamedKey::ArrowDown) => Some(NavKey::ArrowDown),
        Key::Named(NamedKey::ArrowLeft) => Some(NavKey::ArrowLeft),
        Key::Named(NamedKey::ArrowRight) => Some(NavKey::ArrowRight),
        Key::Named(NamedKey::Home) => Some(NavKey::Home),
        Key::Named(NamedKey::End) => Some(NavKey::End),
        Key::Named(NamedKey::PageUp) => Some(NavKey::PageUp),
        Key::Named(NamedKey::PageDown) => Some(NavKey::PageDown),
        Key::Named(NamedKey::Enter) => Some(NavKey::Enter),
        Key::Named(NamedKey::Escape) => Some(NavKey::Escape),
        Key::Named(NamedKey::Tab) => Some(NavKey::Tab),
        Key::Named(NamedKey::Backspace) => Some(NavKey::Backspace),
        Key::Named(NamedKey::Delete) => Some(NavKey::Delete),
        Key::Character(s) => NavKey::from_key_str(s.as_str()),
        _ => None,
    }
}

/// Returns `true` for standard activation keys (`Enter` or `Space`).
#[inline]
pub fn is_activation_key(key: &Key) -> bool {
    translate_nav_key(key).is_some_and(NavKey::is_activation)
}

/// Returns `true` for the `Enter` key.
#[inline]
pub fn is_enter_key(key: &Key) -> bool {
    matches!(translate_nav_key(key), Some(NavKey::Enter))
}

/// Returns `true` for the `Escape` key.
#[inline]
pub fn is_escape_key(key: &Key) -> bool {
    matches!(translate_nav_key(key), Some(NavKey::Escape))
}

/// Returns `true` when `evt` is part of an active IME composition (`is_composing` or `Process` key,
/// covering macOS Safari post-`compositionend` Escape and Blitz Native IME events, `radix#4153`).
#[inline]
pub fn is_ime_composing_key(evt: &KeyboardEvent) -> bool {
    evt.data().is_composing()
        || matches!(evt.key(), Key::Named(NamedKey::Process))
        || matches!(evt.key(), Key::Character(ref s) if s == "Process")
}

#[cfg(test)]
mod tests {
    use dioxus::prelude::keyboard_types::{Key, NamedKey};

    use super::{NavKey, is_activation_key, is_enter_key, is_escape_key, translate_nav_key};

    #[test]
    fn translates_named_and_character_navigation_keys() {
        assert_eq!(
            translate_nav_key(&Key::Named(NamedKey::ArrowUp)),
            Some(NavKey::ArrowUp)
        );
        assert_eq!(
            translate_nav_key(&Key::Named(NamedKey::Escape)),
            Some(NavKey::Escape)
        );
        assert_eq!(
            translate_nav_key(&Key::Character(" ".into())),
            Some(NavKey::Space)
        );
        assert_eq!(
            translate_nav_key(&Key::Character("Tab".into())),
            Some(NavKey::Tab)
        );
        assert_eq!(translate_nav_key(&Key::Character("a".into())), None);

        assert!(is_activation_key(&Key::Named(NamedKey::Enter)));
        assert!(is_activation_key(&Key::Character(" ".into())));
        assert!(!is_activation_key(&Key::Named(NamedKey::Escape)));
        assert!(is_enter_key(&Key::Named(NamedKey::Enter)));
        assert!(!is_enter_key(&Key::Character(" ".into())));
        assert!(is_escape_key(&Key::Named(NamedKey::Escape)));
    }
}

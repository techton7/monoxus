//! Host-neutral keyboard navigation key tokens.

use super::shared::RovingFocusKey;

/// Host-neutral navigation and activation key tokens independent of `keyboard-types` or DOM events.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub enum NavKey {
    ArrowUp,
    ArrowDown,
    ArrowLeft,
    ArrowRight,
    Home,
    End,
    PageUp,
    PageDown,
    Enter,
    Space,
    Escape,
    Tab,
    Backspace,
    Delete,
}

impl NavKey {
    /// Parses a W3C DOM `KeyboardEvent.key` string into a [`NavKey`].
    #[inline]
    pub fn from_key_str(key: &str) -> Option<Self> {
        match key {
            "ArrowUp" => Some(Self::ArrowUp),
            "ArrowDown" => Some(Self::ArrowDown),
            "ArrowLeft" => Some(Self::ArrowLeft),
            "ArrowRight" => Some(Self::ArrowRight),
            "Home" => Some(Self::Home),
            "End" => Some(Self::End),
            "PageUp" => Some(Self::PageUp),
            "PageDown" => Some(Self::PageDown),
            "Enter" => Some(Self::Enter),
            " " | "Spacebar" => Some(Self::Space),
            "Escape" | "Esc" => Some(Self::Escape),
            "Tab" => Some(Self::Tab),
            "Backspace" => Some(Self::Backspace),
            "Delete" | "Del" => Some(Self::Delete),
            _ => None,
        }
    }

    /// Converts directional/boundary navigation keys into a [`RovingFocusKey`].
    #[inline]
    pub const fn to_roving_focus_key(self) -> Option<RovingFocusKey> {
        match self {
            Self::ArrowUp => Some(RovingFocusKey::ArrowUp),
            Self::ArrowDown => Some(RovingFocusKey::ArrowDown),
            Self::ArrowLeft => Some(RovingFocusKey::ArrowLeft),
            Self::ArrowRight => Some(RovingFocusKey::ArrowRight),
            Self::Home => Some(RovingFocusKey::Home),
            Self::End => Some(RovingFocusKey::End),
            _ => None,
        }
    }

    /// Returns `true` for standard button/item activation keys (`Enter` or `Space`).
    #[inline]
    pub const fn is_activation(self) -> bool {
        matches!(self, Self::Enter | Self::Space)
    }

    /// Returns `true` for cardinal arrow keys.
    #[inline]
    pub const fn is_arrow(self) -> bool {
        matches!(
            self,
            Self::ArrowUp | Self::ArrowDown | Self::ArrowLeft | Self::ArrowRight
        )
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn nav_key_parses_w3c_tokens_and_converts_roving_keys() {
        assert_eq!(NavKey::from_key_str("ArrowUp"), Some(NavKey::ArrowUp));
        assert_eq!(NavKey::from_key_str("ArrowDown"), Some(NavKey::ArrowDown));
        assert_eq!(NavKey::from_key_str("ArrowLeft"), Some(NavKey::ArrowLeft));
        assert_eq!(NavKey::from_key_str("ArrowRight"), Some(NavKey::ArrowRight));
        assert_eq!(NavKey::from_key_str("Home"), Some(NavKey::Home));
        assert_eq!(NavKey::from_key_str("End"), Some(NavKey::End));
        assert_eq!(NavKey::from_key_str("PageUp"), Some(NavKey::PageUp));
        assert_eq!(NavKey::from_key_str("PageDown"), Some(NavKey::PageDown));
        assert_eq!(NavKey::from_key_str("Enter"), Some(NavKey::Enter));
        assert_eq!(NavKey::from_key_str(" "), Some(NavKey::Space));
        assert_eq!(NavKey::from_key_str("Escape"), Some(NavKey::Escape));
        assert_eq!(NavKey::from_key_str("Tab"), Some(NavKey::Tab));
        assert_eq!(NavKey::from_key_str("Backspace"), Some(NavKey::Backspace));
        assert_eq!(NavKey::from_key_str("Delete"), Some(NavKey::Delete));
        assert_eq!(NavKey::from_key_str("a"), None);

        assert_eq!(
            NavKey::ArrowLeft.to_roving_focus_key(),
            Some(RovingFocusKey::ArrowLeft)
        );
        assert_eq!(NavKey::Enter.to_roving_focus_key(), None);
        assert!(NavKey::Enter.is_activation());
        assert!(NavKey::Space.is_activation());
        assert!(!NavKey::Escape.is_activation());
        assert!(NavKey::ArrowDown.is_arrow());
        assert!(!NavKey::Home.is_arrow());
    }
}

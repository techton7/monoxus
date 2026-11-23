use std::fmt;

use dioxus::prelude::Callback;

pub use monoxus_core::pin_input::types::*;

/// Character/value validation pattern for [`crate::pin_input::PinInputRoot`].
///
/// Wraps [`PinInputBuiltinPattern`] presets alongside a Dioxus [`Callback`] custom predicate.
#[derive(Clone, Default, PartialEq)]
pub enum PinInputPattern {
    #[default]
    Any,
    Digits,
    Alpha,
    Alphanumeric,
    Custom(Callback<String, bool>),
}

impl fmt::Debug for PinInputPattern {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Any => f.write_str("PinInputPattern::Any"),
            Self::Digits => f.write_str("PinInputPattern::Digits"),
            Self::Alpha => f.write_str("PinInputPattern::Alpha"),
            Self::Alphanumeric => f.write_str("PinInputPattern::Alphanumeric"),
            Self::Custom(_) => f.write_str("PinInputPattern::Custom(<callback>)"),
        }
    }
}

impl From<PinInputBuiltinPattern> for PinInputPattern {
    fn from(value: PinInputBuiltinPattern) -> Self {
        match value {
            PinInputBuiltinPattern::Any => Self::Any,
            PinInputBuiltinPattern::Digits => Self::Digits,
            PinInputBuiltinPattern::Alpha => Self::Alpha,
            PinInputBuiltinPattern::Alphanumeric => Self::Alphanumeric,
        }
    }
}

impl PinInputPattern {
    /// Returns the matching [`PinInputBuiltinPattern`] for built-in presets, or `None` for `Custom`.
    pub const fn as_builtin(&self) -> Option<PinInputBuiltinPattern> {
        match self {
            Self::Any => Some(PinInputBuiltinPattern::Any),
            Self::Digits => Some(PinInputBuiltinPattern::Digits),
            Self::Alpha => Some(PinInputBuiltinPattern::Alpha),
            Self::Alphanumeric => Some(PinInputBuiltinPattern::Alphanumeric),
            Self::Custom(_) => None,
        }
    }

    /// Returns the HTML `pattern` attribute hint when backed by a built-in preset.
    pub const fn html_pattern(&self) -> Option<&'static str> {
        match self.as_builtin() {
            Some(builtin) => builtin.html_pattern(),
            None => None,
        }
    }

    /// Evaluates whether `value` satisfies this pattern.
    pub fn matches_str(&self, value: &str) -> bool {
        match self {
            Self::Any => PinInputBuiltinPattern::Any.matches_str(value),
            Self::Digits => PinInputBuiltinPattern::Digits.matches_str(value),
            Self::Alpha => PinInputBuiltinPattern::Alpha.matches_str(value),
            Self::Alphanumeric => PinInputBuiltinPattern::Alphanumeric.matches_str(value),
            Self::Custom(cb) => cb.call(value.to_string()),
        }
    }
}

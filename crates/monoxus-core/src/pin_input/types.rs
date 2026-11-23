/// All headless parts composing a `PinInput` primitive.
pub const PIN_INPUT_PARTS: [PinInputPart; 3] = [
    PinInputPart::Root,
    PinInputPart::Input,
    PinInputPart::Cell,
];

/// Structural role of a part inside a `PinInput` compound component.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub enum PinInputPart {
    Root,
    Input,
    Cell,
}

impl PinInputPart {
    pub const fn as_str(&self) -> &'static str {
        match self {
            Self::Root => "root",
            Self::Input => "input",
            Self::Cell => "cell",
        }
    }
}

/// Built-in character-validation presets for `PinInput` (zero regex crate dependency).
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, Hash)]
pub enum PinInputBuiltinPattern {
    #[default]
    Any,
    Digits,
    Alpha,
    Alphanumeric,
}

impl PinInputBuiltinPattern {
    /// Returns `true` when `ch` satisfies this built-in pattern.
    pub const fn matches_char(&self, ch: char) -> bool {
        match self {
            Self::Any => true,
            Self::Digits => ch.is_ascii_digit(),
            Self::Alpha => ch.is_ascii_alphabetic(),
            Self::Alphanumeric => ch.is_ascii_alphanumeric(),
        }
    }

    /// Returns `true` when every character in `value` satisfies this pattern (empty strings are valid).
    pub fn matches_str(&self, value: &str) -> bool {
        value.chars().all(|ch| self.matches_char(ch))
    }

    /// Returns the optional HTML `pattern` attribute string for Web input semantics.
    pub const fn html_pattern(&self) -> Option<&'static str> {
        match self {
            Self::Any => None,
            Self::Digits => Some("[0-9]*"),
            Self::Alpha => Some("[a-zA-Z]*"),
            Self::Alphanumeric => Some("[a-zA-Z0-9]*"),
        }
    }
}

/// Horizontal text alignment applied to the overlaid `<input>`.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, Hash)]
pub enum PinInputTextAlign {
    #[default]
    Left,
    Center,
    Right,
}

impl PinInputTextAlign {
    pub const fn as_str(&self) -> &'static str {
        match self {
            Self::Left => "left",
            Self::Center => "center",
            Self::Right => "right",
        }
    }
}

/// Virtual keyboard hint emitted as HTML `inputmode` on the overlaid `<input>`.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, Hash)]
pub enum PinInputMode {
    #[default]
    Numeric,
    Text,
    Decimal,
    Tel,
}

impl PinInputMode {
    pub const fn as_str(&self) -> &'static str {
        match self {
            Self::Numeric => "numeric",
            Self::Text => "text",
            Self::Decimal => "decimal",
            Self::Tel => "tel",
        }
    }
}

/// HTML `autocomplete` token emitted on the overlaid `<input>`.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, Hash)]
pub enum PinInputAutoComplete {
    #[default]
    OneTimeCode,
    Off,
}

impl PinInputAutoComplete {
    pub const fn as_str(&self) -> &'static str {
        match self {
            Self::OneTimeCode => "one-time-code",
            Self::Off => "off",
        }
    }
}

/// Direction of an active text selection inside the overlaid `<input>`.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, Hash)]
pub enum PinInputSelectionDirection {
    #[default]
    None,
    Forward,
    Backward,
}

impl PinInputSelectionDirection {
    pub const fn as_str(&self) -> &'static str {
        match self {
            Self::None => "none",
            Self::Forward => "forward",
            Self::Backward => "backward",
        }
    }
}

/// Character-indexed selection state mirrored from the overlaid `<input>`.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, Hash)]
pub struct PinInputSelection {
    pub start: Option<usize>,
    pub end: Option<usize>,
    pub direction: PinInputSelectionDirection,
}

impl PinInputSelection {
    pub const fn new(
        start: Option<usize>,
        end: Option<usize>,
        direction: PinInputSelectionDirection,
    ) -> Self {
        Self {
            start,
            end,
            direction,
        }
    }

    pub const fn collapsed(pos: usize) -> Self {
        Self {
            start: Some(pos),
            end: Some(pos),
            direction: PinInputSelectionDirection::None,
        }
    }

    pub const fn range(start: usize, end: usize, direction: PinInputSelectionDirection) -> Self {
        Self {
            start: Some(start),
            end: Some(end),
            direction,
        }
    }

    pub const fn empty() -> Self {
        Self {
            start: None,
            end: None,
            direction: PinInputSelectionDirection::None,
        }
    }
}

/// Projected per-cell state exposed to `PinInputCell` and `PinInputSnapshot`.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub struct PinInputCellData {
    pub index: usize,
    pub char: Option<char>,
    pub is_active: bool,
    pub has_fake_caret: bool,
}

impl PinInputCellData {
    /// Returns the character to display for this cell, applying `mask_char` when `mask` is enabled.
    pub const fn display_char(&self, mask: bool, mask_char: char) -> Option<char> {
        match self.char {
            Some(_) if mask => Some(mask_char),
            Some(ch) => Some(ch),
            None => None,
        }
    }
}

/// Read-only snapshot of the current `PinInput` state exposed via context and `render_children`.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct PinInputSnapshot {
    pub value: String,
    pub cells: Vec<PinInputCellData>,
    pub is_focused: bool,
    pub is_hovering: bool,
    pub disabled: bool,
    pub mask: bool,
}

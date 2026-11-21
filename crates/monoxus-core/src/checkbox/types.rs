use std::rc::Rc;

use crate::foundation::state::DataState;

pub const CHECKBOX_PARTS: [CheckboxPart; 4] = [
    CheckboxPart::Root,
    CheckboxPart::Indicator,
    CheckboxPart::Group,
    CheckboxPart::GroupLabel,
];

/// Structural parts comprising the headless checkbox primitive family.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub enum CheckboxPart {
    Root,
    Indicator,
    Group,
    GroupLabel,
}

impl CheckboxPart {
    pub const fn as_str(&self) -> &'static str {
        match self {
            Self::Root => "root",
            Self::Indicator => "indicator",
            Self::Group => "group",
            Self::GroupLabel => "group-label",
        }
    }
}

/// Tri-state representation of checkbox checkedness.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, Hash)]
pub enum CheckedState {
    Checked,
    #[default]
    Unchecked,
    Indeterminate,
}

impl CheckedState {
    pub const fn is_checked(self) -> bool {
        matches!(self, Self::Checked)
    }

    pub const fn is_unchecked(self) -> bool {
        matches!(self, Self::Unchecked)
    }

    pub const fn is_indeterminate(self) -> bool {
        matches!(self, Self::Indeterminate)
    }

    /// User activation state transition:
    /// `Indeterminate -> Checked`, `Unchecked -> Checked`, `Checked -> Unchecked`.
    pub const fn toggle(self) -> Self {
        match self {
            Self::Checked => Self::Unchecked,
            Self::Unchecked | Self::Indeterminate => Self::Checked,
        }
    }

    pub const fn aria_checked(self) -> &'static str {
        match self {
            Self::Checked => "true",
            Self::Unchecked => "false",
            Self::Indeterminate => "mixed",
        }
    }

    pub const fn data_state(self) -> DataState {
        match self {
            Self::Checked => DataState::Checked,
            Self::Unchecked => DataState::Unchecked,
            Self::Indeterminate => DataState::Indeterminate,
        }
    }

    pub const fn data_state_str(self) -> &'static str {
        match self {
            Self::Checked => "checked",
            Self::Unchecked => "unchecked",
            Self::Indeterminate => "indeterminate",
        }
    }
}

impl From<bool> for CheckedState {
    fn from(value: bool) -> Self {
        if value {
            Self::Checked
        } else {
            Self::Unchecked
        }
    }
}

pub type CheckboxCheckedChangeHandler = Rc<dyn Fn(CheckedState)>;
pub type CheckboxGroupValuesChangeHandler = Rc<dyn Fn(Vec<String>)>;

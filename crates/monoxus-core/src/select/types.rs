// [STAGE1-DROP]
#[cfg(feature = "serde")]
use serde::{Deserialize, Serialize};

pub use ui_sdk::select::types::{
    DismissBehavior, PlacementAlign, PlacementSide, PortalHost, PressedSelectItem, SELECT_PARTS,
    SelectItemData, SelectPart,
};

// [STAGE1-DROP] Legacy `SelectMode` preserving `Single { allow_deselect }` payload until Stage 1 migrates `monoxus::select`.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
#[cfg_attr(feature = "serde", derive(Serialize, Deserialize))]
pub enum SelectMode {
    Single { allow_deselect: bool },
    Multiple,
}

impl Default for SelectMode {
    fn default() -> Self {
        Self::Single {
            allow_deselect: false,
        }
    }
}

impl SelectMode {
    pub const fn single() -> Self {
        Self::Single {
            allow_deselect: false,
        }
    }

    pub const fn single_with_deselect(allow_deselect: bool) -> Self {
        Self::Single { allow_deselect }
    }

    pub const fn multiple() -> Self {
        Self::Multiple
    }

    pub const fn is_single(&self) -> bool {
        matches!(self, Self::Single { .. })
    }

    pub const fn is_multiple(&self) -> bool {
        matches!(self, Self::Multiple)
    }

    pub const fn allow_deselect(&self) -> bool {
        match self {
            Self::Single { allow_deselect } => *allow_deselect,
            Self::Multiple => true,
        }
    }
}

impl From<SelectMode> for ui_sdk::select::SelectMode {
    fn from(mode: SelectMode) -> Self {
        match mode {
            SelectMode::Single { .. } => Self::Single,
            SelectMode::Multiple => Self::Multiple,
        }
    }
}

impl From<ui_sdk::select::SelectMode> for SelectMode {
    fn from(mode: ui_sdk::select::SelectMode) -> Self {
        match mode {
            ui_sdk::select::SelectMode::Single => Self::Single {
                allow_deselect: false,
            },
            ui_sdk::select::SelectMode::Multiple => Self::Multiple,
        }
    }
}

impl From<SelectMode> for ui_sdk::kernel::SelectionMode {
    fn from(mode: SelectMode) -> Self {
        ui_sdk::select::SelectMode::from(mode).into()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn legacy_select_mode_defaults_helpers_and_ui_sdk_conversions() {
        assert_eq!(
            SelectMode::default(),
            SelectMode::Single {
                allow_deselect: false
            }
        );
        assert!(SelectMode::single().is_single());
        assert!(!SelectMode::single().is_multiple());
        assert!(!SelectMode::single().allow_deselect());

        let deselect = SelectMode::single_with_deselect(true);
        assert!(deselect.is_single());
        assert!(deselect.allow_deselect());

        assert!(SelectMode::multiple().is_multiple());
        assert!(SelectMode::multiple().allow_deselect());

        assert_eq!(
            ui_sdk::select::SelectMode::from(deselect),
            ui_sdk::select::SelectMode::Single
        );
        assert_eq!(
            ui_sdk::select::SelectMode::from(SelectMode::Multiple),
            ui_sdk::select::SelectMode::Multiple
        );
        assert_eq!(
            SelectMode::from(ui_sdk::select::SelectMode::Single),
            SelectMode::Single {
                allow_deselect: false
            }
        );
        assert_eq!(
            SelectMode::from(ui_sdk::select::SelectMode::Multiple),
            SelectMode::Multiple
        );
    }
}

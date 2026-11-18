use std::rc::Rc;

use crate::foundation::shared::Orientation;

pub const TOGGLE_GROUP_PARTS: [ToggleGroupPart; 2] = [
    ToggleGroupPart::Root,
    ToggleGroupPart::Item,
];

#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub enum ToggleGroupPart {
    Root,
    Item,
}

impl ToggleGroupPart {
    pub const fn as_str(&self) -> &'static str {
        match self {
            Self::Root => "root",
            Self::Item => "item",
        }
    }
}

#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, Hash)]
pub enum ToggleGroupMode {
    #[default]
    Single,
    Multiple,
}

impl ToggleGroupMode {
    pub const fn as_str(&self) -> &'static str {
        match self {
            Self::Single => "single",
            Self::Multiple => "multiple",
        }
    }

    pub const fn is_single(&self) -> bool {
        matches!(self, Self::Single)
    }

    pub const fn is_multiple(&self) -> bool {
        matches!(self, Self::Multiple)
    }
}

#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, Hash)]
pub enum ToggleGroupOrientation {
    #[default]
    Horizontal,
    Vertical,
}

impl ToggleGroupOrientation {
    pub const fn as_str(&self) -> &'static str {
        match self {
            Self::Horizontal => "horizontal",
            Self::Vertical => "vertical",
        }
    }

    pub const fn is_horizontal(&self) -> bool {
        matches!(self, Self::Horizontal)
    }

    pub const fn is_vertical(&self) -> bool {
        matches!(self, Self::Vertical)
    }
}

impl From<ToggleGroupOrientation> for Orientation {
    fn from(orientation: ToggleGroupOrientation) -> Self {
        match orientation {
            ToggleGroupOrientation::Horizontal => Orientation::Horizontal,
            ToggleGroupOrientation::Vertical => Orientation::Vertical,
        }
    }
}

impl From<Orientation> for ToggleGroupOrientation {
    fn from(orientation: Orientation) -> Self {
        match orientation {
            Orientation::Vertical => ToggleGroupOrientation::Vertical,
            _ => ToggleGroupOrientation::Horizontal,
        }
    }
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct ToggleGroupItemRegistration {
    pub value: String,
    pub id: String,
    pub disabled: bool,
}

pub type ToggleGroupSingleChangeHandler = Rc<dyn Fn(String)>;
pub type ToggleGroupMultipleChangeHandler = Rc<dyn Fn(Vec<String>)>;

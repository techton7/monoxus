use std::rc::Rc;

use crate::foundation::shared::Orientation;

pub const RADIO_GROUP_PARTS: [RadioGroupPart; 3] = [
    RadioGroupPart::Root,
    RadioGroupPart::Item,
    RadioGroupPart::Indicator,
];

#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub enum RadioGroupPart {
    Root,
    Item,
    Indicator,
}

impl RadioGroupPart {
    pub const fn as_str(&self) -> &'static str {
        match self {
            Self::Root => "root",
            Self::Item => "item",
            Self::Indicator => "indicator",
        }
    }
}

#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, Hash)]
pub enum RadioGroupOrientation {
    #[default]
    Horizontal,
    Vertical,
}

impl RadioGroupOrientation {
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

impl From<RadioGroupOrientation> for Orientation {
    fn from(orientation: RadioGroupOrientation) -> Self {
        match orientation {
            RadioGroupOrientation::Horizontal => Orientation::Horizontal,
            RadioGroupOrientation::Vertical => Orientation::Vertical,
        }
    }
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct RadioGroupItemRegistration {
    pub value: String,
    pub id: String,
    pub disabled: bool,
}

pub type RadioGroupValueChangeHandler = Rc<dyn Fn(String)>;

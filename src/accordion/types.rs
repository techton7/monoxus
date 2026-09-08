use std::rc::Rc;

use crate::foundation::shared::{Direction, Orientation};

pub const ACCORDION_PARTS: [AccordionPart; 5] = [
    AccordionPart::Root,
    AccordionPart::Item,
    AccordionPart::Header,
    AccordionPart::Trigger,
    AccordionPart::Content,
];

#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub enum AccordionPart {
    Root,
    Item,
    Header,
    Trigger,
    Content,
}

impl AccordionPart {
    pub const fn as_str(&self) -> &'static str {
        match self {
            Self::Root => "root",
            Self::Item => "item",
            Self::Header => "header",
            Self::Trigger => "trigger",
            Self::Content => "content",
        }
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash, Default)]
pub enum AccordionOrientation {
    #[default]
    Vertical,
    Horizontal,
}

impl AccordionOrientation {
    pub const fn as_str(&self) -> &'static str {
        match self {
            Self::Vertical => "vertical",
            Self::Horizontal => "horizontal",
        }
    }

    pub const fn is_vertical(&self) -> bool {
        matches!(self, Self::Vertical)
    }

    pub const fn is_horizontal(&self) -> bool {
        matches!(self, Self::Horizontal)
    }
}

impl From<AccordionOrientation> for Orientation {
    fn from(orientation: AccordionOrientation) -> Self {
        match orientation {
            AccordionOrientation::Vertical => Orientation::Vertical,
            AccordionOrientation::Horizontal => Orientation::Horizontal,
        }
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash, Default)]
pub enum AccordionDirection {
    #[default]
    Ltr,
    Rtl,
}

impl AccordionDirection {
    pub const fn as_str(&self) -> &'static str {
        match self {
            Self::Ltr => "ltr",
            Self::Rtl => "rtl",
        }
    }

    pub const fn is_rtl(&self) -> bool {
        matches!(self, Self::Rtl)
    }
}

impl From<AccordionDirection> for Direction {
    fn from(direction: AccordionDirection) -> Self {
        match direction {
            AccordionDirection::Ltr => Direction::Ltr,
            AccordionDirection::Rtl => Direction::Rtl,
        }
    }
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub enum AccordionMode {
    Single { collapsible: bool },
    Multiple,
}

impl Default for AccordionMode {
    fn default() -> Self {
        Self::Single { collapsible: false }
    }
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct AccordionItemRegistration {
    pub value: String,
    pub disabled: bool,
}

pub type AccordionValueChangeHandler = Rc<dyn Fn(Vec<String>)>;

use std::rc::Rc;

use crate::foundation::shared::{Direction, Orientation};

pub const TABS_PARTS: [TabsPart; 4] = [
    TabsPart::Root,
    TabsPart::List,
    TabsPart::Trigger,
    TabsPart::Content,
];

#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub enum TabsPart {
    Root,
    List,
    Trigger,
    Content,
}

impl TabsPart {
    pub const fn as_str(&self) -> &'static str {
        match self {
            Self::Root => "root",
            Self::List => "list",
            Self::Trigger => "trigger",
            Self::Content => "content",
        }
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash, Default)]
pub enum TabsOrientation {
    #[default]
    Horizontal,
    Vertical,
}

impl TabsOrientation {
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

impl From<TabsOrientation> for Orientation {
    fn from(orientation: TabsOrientation) -> Self {
        match orientation {
            TabsOrientation::Horizontal => Orientation::Horizontal,
            TabsOrientation::Vertical => Orientation::Vertical,
        }
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash, Default)]
pub enum TabsDirection {
    #[default]
    Ltr,
    Rtl,
}

impl TabsDirection {
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

impl From<TabsDirection> for Direction {
    fn from(direction: TabsDirection) -> Self {
        match direction {
            TabsDirection::Ltr => Direction::Ltr,
            TabsDirection::Rtl => Direction::Rtl,
        }
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash, Default)]
pub enum TabsActivationMode {
    #[default]
    Automatic,
    Manual,
}

impl TabsActivationMode {
    pub const fn is_automatic(&self) -> bool {
        matches!(self, Self::Automatic)
    }

    pub const fn is_manual(&self) -> bool {
        matches!(self, Self::Manual)
    }
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct TriggerRegistration {
    pub value: String,
    pub disabled: bool,
}

pub type TabsValueChangeHandler = Rc<dyn Fn(String)>;

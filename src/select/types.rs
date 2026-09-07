pub use crate::foundation::overlay::{PlacementAlign, PlacementSide, PortalHost};
use super::runtime::SelectRuntime;

pub const SELECT_PARTS: [SelectPart; 16] = [
    SelectPart::Root,
    SelectPart::Trigger,
    SelectPart::Value,
    SelectPart::Icon,
    SelectPart::Portal,
    SelectPart::Content,
    SelectPart::ContentStatic,
    SelectPart::Viewport,
    SelectPart::Group,
    SelectPart::Label,
    SelectPart::Item,
    SelectPart::ItemText,
    SelectPart::ItemIndicator,
    SelectPart::Separator,
    SelectPart::Arrow,
    SelectPart::HiddenInput,
];

#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub enum SelectPart {
    Root,
    Trigger,
    Value,
    Icon,
    Portal,
    Content,
    ContentStatic,
    Viewport,
    Group,
    Label,
    Item,
    ItemText,
    ItemIndicator,
    Separator,
    Arrow,
    HiddenInput,
}

impl SelectPart {
    pub const fn as_str(&self) -> &'static str {
        match self {
            Self::Root => "root",
            Self::Trigger => "trigger",
            Self::Value => "value",
            Self::Icon => "icon",
            Self::Portal => "portal",
            Self::Content => "content",
            Self::ContentStatic => "content-static",
            Self::Viewport => "viewport",
            Self::Group => "group",
            Self::Label => "label",
            Self::Item => "item",
            Self::ItemText => "item-text",
            Self::ItemIndicator => "item-indicator",
            Self::Separator => "separator",
            Self::Arrow => "arrow",
            Self::HiddenInput => "hidden-input",
        }
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
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

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct SelectItemData {
    pub value: String,
    pub text: String,
    pub disabled: bool,
}

impl SelectItemData {
    pub fn new(value: impl Into<String>, text: impl Into<String>, disabled: bool) -> Self {
        Self {
            value: value.into(),
            text: text.into(),
            disabled,
        }
    }
}

#[derive(Clone)]
pub struct SelectContext {
    pub runtime: SelectRuntime,
    pub name: Option<String>,
}

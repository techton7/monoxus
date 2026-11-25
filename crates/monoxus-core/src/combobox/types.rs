pub use crate::foundation::overlay::{PlacementAlign, PlacementSide, PortalHost};

pub const COMBOBOX_PARTS: [ComboboxPart; 14] = [
    ComboboxPart::Root,
    ComboboxPart::Input,
    ComboboxPart::Trigger,
    ComboboxPart::Content,
    ComboboxPart::ContentStatic,
    ComboboxPart::Viewport,
    ComboboxPart::Item,
    ComboboxPart::Group,
    ComboboxPart::GroupHeading,
    ComboboxPart::ScrollUpButton,
    ComboboxPart::ScrollDownButton,
    ComboboxPart::Arrow,
    ComboboxPart::Portal,
    ComboboxPart::HiddenInput,
];

#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub enum ComboboxPart {
    Root,
    Input,
    Trigger,
    Content,
    ContentStatic,
    Viewport,
    Item,
    Group,
    GroupHeading,
    ScrollUpButton,
    ScrollDownButton,
    Arrow,
    Portal,
    HiddenInput,
    Separator,
}

impl ComboboxPart {
    pub const fn as_str(&self) -> &'static str {
        match self {
            Self::Root => "root",
            Self::Input => "input",
            Self::Trigger => "trigger",
            Self::Content => "content",
            Self::ContentStatic => "content-static",
            Self::Viewport => "viewport",
            Self::Item => "item",
            Self::Group => "group",
            Self::GroupHeading => "group-heading",
            Self::ScrollUpButton => "scroll-up-button",
            Self::ScrollDownButton => "scroll-down-button",
            Self::Arrow => "arrow",
            Self::Portal => "portal",
            Self::HiddenInput => "hidden-input",
            Self::Separator => "separator",
        }
    }
}

#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, Hash)]
pub enum ComboboxMode {
    #[default]
    Single,
    Multiple,
}

impl ComboboxMode {
    pub const fn is_single(&self) -> bool {
        matches!(self, Self::Single)
    }

    pub const fn is_multiple(&self) -> bool {
        matches!(self, Self::Multiple)
    }
}

#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, Hash)]
pub enum ComboboxScrollAlignment {
    #[default]
    Nearest,
    Center,
}

impl ComboboxScrollAlignment {
    pub const fn as_str(&self) -> &'static str {
        match self {
            Self::Nearest => "nearest",
            Self::Center => "center",
        }
    }
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct ComboboxItemData {
    pub value: String,
    pub label: String,
    pub disabled: bool,
}

impl ComboboxItemData {
    pub fn new(value: impl Into<String>, label: impl Into<String>, disabled: bool) -> Self {
        Self {
            value: value.into(),
            label: label.into(),
            disabled,
        }
    }

    pub fn from_value(value: impl Into<String>) -> Self {
        let value = value.into();
        Self {
            label: value.clone(),
            value,
            disabled: false,
        }
    }
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct ComboboxRegisteredItem {
    pub value: String,
    pub label: String,
    pub dom_id: String,
    pub disabled: bool,
    pub order: usize,
}

impl ComboboxRegisteredItem {
    pub fn new(
        value: impl Into<String>,
        label: impl Into<String>,
        dom_id: impl Into<String>,
        disabled: bool,
        order: usize,
    ) -> Self {
        Self {
            value: value.into(),
            label: label.into(),
            dom_id: dom_id.into(),
            disabled,
            order,
        }
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub enum ComboboxNavKey {
    ArrowDown,
    ArrowUp,
    PageDown,
    PageUp,
    AltArrowUp,
    Enter,
    Tab,
    Escape,
    Home,
    End,
    Backspace,
    Printable,
}

#[derive(Clone, Debug, PartialEq)]
pub struct PressedComboboxItem {
    pub value: String,
    pub client_x: f64,
    pub client_y: f64,
}

#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub struct ComboboxItemRenderState {
    pub selected: bool,
    pub highlighted: bool,
}

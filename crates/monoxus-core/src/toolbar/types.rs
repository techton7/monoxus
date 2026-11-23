use crate::foundation::shared::Orientation;
pub use crate::toggle_group::ToggleGroupMode as ToolbarToggleGroupMode;

pub const TOOLBAR_PARTS: [ToolbarPart; 6] = [
    ToolbarPart::Root,
    ToolbarPart::Button,
    ToolbarPart::Link,
    ToolbarPart::Separator,
    ToolbarPart::ToggleGroup,
    ToolbarPart::ToggleItem,
];

#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub enum ToolbarPart {
    Root,
    Button,
    Link,
    Separator,
    ToggleGroup,
    ToggleItem,
}

impl ToolbarPart {
    pub const fn as_str(&self) -> &'static str {
        match self {
            Self::Root => "root",
            Self::Button => "button",
            Self::Link => "link",
            Self::Separator => "separator",
            Self::ToggleGroup => "toggle-group",
            Self::ToggleItem => "toggle-item",
        }
    }
}

pub const fn toolbar_orientation_as_str(orientation: Orientation) -> &'static str {
    match orientation {
        Orientation::Vertical => "vertical",
        Orientation::Horizontal | Orientation::Both => "horizontal",
    }
}

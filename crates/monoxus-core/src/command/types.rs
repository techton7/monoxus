use std::collections::{BTreeMap, BTreeSet};

#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub enum CommandPart {
    Root,
    InputLabel,
    Input,
    List,
    Viewport,
    Empty,
    Loading,
    Group,
    GroupHeading,
    GroupItems,
    Item,
    Separator,
}

pub const COMMAND_PARTS: [CommandPart; 12] = [
    CommandPart::Root,
    CommandPart::InputLabel,
    CommandPart::Input,
    CommandPart::List,
    CommandPart::Viewport,
    CommandPart::Empty,
    CommandPart::Loading,
    CommandPart::Group,
    CommandPart::GroupHeading,
    CommandPart::GroupItems,
    CommandPart::Item,
    CommandPart::Separator,
];

impl CommandPart {
    pub const fn as_str(&self) -> &'static str {
        match self {
            Self::Root => "root",
            Self::InputLabel => "input-label",
            Self::Input => "input",
            Self::List => "list",
            Self::Viewport => "viewport",
            Self::Empty => "empty",
            Self::Loading => "loading",
            Self::Group => "group",
            Self::GroupHeading => "group-heading",
            Self::GroupItems => "group-items",
            Self::Item => "item",
            Self::Separator => "separator",
        }
    }

    pub const fn data_attr(&self) -> &'static str {
        match self {
            Self::Root => "data-command-root",
            Self::InputLabel => "data-command-input-label",
            Self::Input => "data-command-input",
            Self::List => "data-command-list",
            Self::Viewport => "data-command-viewport",
            Self::Empty => "data-command-empty",
            Self::Loading => "data-command-loading",
            Self::Group => "data-command-group",
            Self::GroupHeading => "data-command-group-heading",
            Self::GroupItems => "data-command-group-items",
            Self::Item => "data-command-item",
            Self::Separator => "data-command-separator",
        }
    }
}

#[derive(Clone, Debug, PartialEq, Eq)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub struct CommandItemEntry {
    pub value: String,
    pub dom_id: String,
    pub group_value: Option<String>,
    pub disabled: bool,
    pub force_mount: bool,
    pub keywords: Vec<String>,
    pub order: usize,
}

#[derive(Clone, Debug, PartialEq, Eq)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub struct CommandGroupEntry {
    pub value: String,
    pub dom_id: String,
    pub heading_id: Option<String>,
    pub force_mount: bool,
    pub order: usize,
}

#[derive(Clone, Debug, Default, PartialEq)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub struct CommandFilteredState {
    pub count: usize,
    pub item_scores: BTreeMap<String, f64>,
    pub visible_groups: BTreeSet<String>,
}

#[derive(Clone, Debug, Default, PartialEq)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub struct CommandStateSnapshot {
    pub search: String,
    pub value: String,
    pub filtered: CommandFilteredState,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub enum CommandNavKey {
    ArrowDown,
    ArrowUp,
    ArrowLeft,
    ArrowRight,
    Home,
    End,
    VimNext,
    VimPrev,
    VimLeft,
    VimRight,
}

#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, Hash)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub struct CommandNavModifiers {
    pub meta: bool,
    pub alt: bool,
    pub ctrl: bool,
}

impl CommandNavModifiers {
    pub const fn new(meta: bool, alt: bool, ctrl: bool) -> Self {
        Self { meta, alt, ctrl }
    }
}

use crate::{
    foundation::{
        compose::{Slottable, project_as_child},
        state::DataState,
    },
    separator::SeparatorAttributes,
};

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct ToolbarRootAttributes {
    pub id: String,
    pub role: &'static str,
    pub aria_orientation: &'static str,
    pub dir: &'static str,
    pub data_orientation: &'static str,
    pub data_toolbar_root: &'static str,
}

impl ToolbarRootAttributes {
    pub fn id(&self) -> &str {
        &self.id
    }

    pub const fn role(&self) -> &'static str {
        self.role
    }

    pub const fn aria_orientation(&self) -> &'static str {
        self.aria_orientation
    }

    pub const fn dir(&self) -> &'static str {
        self.dir
    }

    pub const fn data_orientation(&self) -> &'static str {
        self.data_orientation
    }

    pub const fn data_toolbar_root(&self) -> &'static str {
        self.data_toolbar_root
    }
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct ToolbarButtonAttributes {
    pub id: String,
    pub r#type: &'static str,
    pub tabindex: i32,
    pub disabled: bool,
    pub data_disabled: Option<&'static str>,
    pub data_orientation: &'static str,
    pub data_toolbar_button: &'static str,
    pub data_toolbar_item: &'static str,
}

impl ToolbarButtonAttributes {
    pub fn id(&self) -> &str {
        &self.id
    }

    pub const fn button_type(&self) -> &'static str {
        self.r#type
    }

    pub const fn tabindex(&self) -> i32 {
        self.tabindex
    }

    pub const fn is_disabled(&self) -> bool {
        self.disabled
    }

    pub const fn data_disabled(&self) -> Option<&'static str> {
        self.data_disabled
    }

    pub const fn data_orientation(&self) -> &'static str {
        self.data_orientation
    }

    pub const fn data_toolbar_button(&self) -> &'static str {
        self.data_toolbar_button
    }

    pub const fn data_toolbar_item(&self) -> &'static str {
        self.data_toolbar_item
    }
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct ToolbarLinkAttributes {
    pub id: String,
    pub href: Option<String>,
    pub tabindex: i32,
    pub data_orientation: &'static str,
    pub data_toolbar_link: &'static str,
    pub data_toolbar_item: &'static str,
}

impl ToolbarLinkAttributes {
    pub fn id(&self) -> &str {
        &self.id
    }

    pub fn href(&self) -> Option<&str> {
        self.href.as_deref()
    }

    pub const fn tabindex(&self) -> i32 {
        self.tabindex
    }

    pub const fn data_orientation(&self) -> &'static str {
        self.data_orientation
    }

    pub const fn data_toolbar_link(&self) -> &'static str {
        self.data_toolbar_link
    }

    pub const fn data_toolbar_item(&self) -> &'static str {
        self.data_toolbar_item
    }
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct ToolbarSeparatorAttributes {
    pub id: Option<String>,
    pub separator: SeparatorAttributes,
    pub data_toolbar_separator: &'static str,
}

impl ToolbarSeparatorAttributes {
    pub fn id(&self) -> Option<&str> {
        self.id.as_deref()
    }

    pub const fn role(&self) -> &'static str {
        self.separator.role
    }

    pub const fn aria_orientation(&self) -> Option<&'static str> {
        self.separator.aria_orientation
    }

    pub const fn data_orientation(&self) -> &'static str {
        self.separator.data_orientation
    }

    pub const fn data_separator_root(&self) -> &'static str {
        self.separator.data_separator_root
    }

    pub const fn data_toolbar_separator(&self) -> &'static str {
        self.data_toolbar_separator
    }
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct ToolbarToggleGroupAttributes {
    pub id: String,
    pub role: &'static str,
    pub dir: &'static str,
    pub data_orientation: &'static str,
    pub data_disabled: Option<&'static str>,
    pub data_toolbar_group: &'static str,
}

impl ToolbarToggleGroupAttributes {
    pub fn id(&self) -> &str {
        &self.id
    }

    pub const fn role(&self) -> &'static str {
        self.role
    }

    pub const fn dir(&self) -> &'static str {
        self.dir
    }

    pub const fn data_orientation(&self) -> &'static str {
        self.data_orientation
    }

    pub const fn data_disabled(&self) -> Option<&'static str> {
        self.data_disabled
    }

    pub const fn data_toolbar_group(&self) -> &'static str {
        self.data_toolbar_group
    }
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct ToolbarToggleItemAttributes {
    pub id: String,
    pub r#type: &'static str,
    pub role: Option<&'static str>,
    pub aria_checked: Option<&'static str>,
    pub aria_pressed: Option<&'static str>,
    pub tabindex: i32,
    pub disabled: bool,
    pub data_disabled: Option<&'static str>,
    pub data_state: DataState,
    pub data_value: String,
    pub data_orientation: &'static str,
    pub data_toolbar_group_item: &'static str,
    pub data_toolbar_item: &'static str,
}

impl ToolbarToggleItemAttributes {
    pub fn id(&self) -> &str {
        &self.id
    }

    pub const fn button_type(&self) -> &'static str {
        self.r#type
    }

    pub const fn role(&self) -> Option<&'static str> {
        self.role
    }

    pub const fn aria_checked(&self) -> Option<&'static str> {
        self.aria_checked
    }

    pub const fn aria_pressed(&self) -> Option<&'static str> {
        self.aria_pressed
    }

    pub const fn tabindex(&self) -> i32 {
        self.tabindex
    }

    pub const fn is_disabled(&self) -> bool {
        self.disabled
    }

    pub const fn data_disabled(&self) -> Option<&'static str> {
        self.data_disabled
    }

    pub fn data_state(&self) -> &DataState {
        &self.data_state
    }

    pub fn data_value(&self) -> &str {
        &self.data_value
    }

    pub const fn data_orientation(&self) -> &'static str {
        self.data_orientation
    }

    pub const fn data_toolbar_group_item(&self) -> &'static str {
        self.data_toolbar_group_item
    }

    pub const fn data_toolbar_item(&self) -> &'static str {
        self.data_toolbar_item
    }
}

pub fn project_toolbar_root_as_child<T, C>(
    attrs: &ToolbarRootAttributes,
    target: T,
    content: Slottable<C>,
) -> (T, ToolbarRootAttributes, C) {
    let (target, content) = project_as_child(target, content);
    (target, attrs.clone(), content)
}

pub fn project_toolbar_button_as_child<T, C>(
    attrs: &ToolbarButtonAttributes,
    target: T,
    content: Slottable<C>,
) -> (T, ToolbarButtonAttributes, C) {
    let (target, content) = project_as_child(target, content);
    (target, attrs.clone(), content)
}

pub fn project_toolbar_link_as_child<T, C>(
    attrs: &ToolbarLinkAttributes,
    target: T,
    content: Slottable<C>,
) -> (T, ToolbarLinkAttributes, C) {
    let (target, content) = project_as_child(target, content);
    (target, attrs.clone(), content)
}

pub fn project_toolbar_separator_as_child<T, C>(
    attrs: &ToolbarSeparatorAttributes,
    target: T,
    content: Slottable<C>,
) -> (T, ToolbarSeparatorAttributes, C) {
    let (target, content) = project_as_child(target, content);
    (target, attrs.clone(), content)
}

pub fn project_toolbar_toggle_group_as_child<T, C>(
    attrs: &ToolbarToggleGroupAttributes,
    target: T,
    content: Slottable<C>,
) -> (T, ToolbarToggleGroupAttributes, C) {
    let (target, content) = project_as_child(target, content);
    (target, attrs.clone(), content)
}

pub fn project_toolbar_toggle_item_as_child<T, C>(
    attrs: &ToolbarToggleItemAttributes,
    target: T,
    content: Slottable<C>,
) -> (T, ToolbarToggleItemAttributes, C) {
    let (target, content) = project_as_child(target, content);
    (target, attrs.clone(), content)
}

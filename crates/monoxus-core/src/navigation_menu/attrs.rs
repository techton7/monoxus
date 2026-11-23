use crate::foundation::{
    compose::{Slottable, project_as_child},
    state::DataState,
};

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct NavigationMenuRootAttributes {
    pub id: String,
    pub aria_label: Option<String>,
    pub dir: &'static str,
    pub data_orientation: &'static str,
    pub data_monoxus_navigation_menu_root: &'static str,
}

impl NavigationMenuRootAttributes {
    pub fn id(&self) -> &str {
        &self.id
    }

    pub fn aria_label(&self) -> Option<&str> {
        self.aria_label.as_deref()
    }

    pub const fn dir(&self) -> &'static str {
        self.dir
    }

    pub const fn data_orientation(&self) -> &'static str {
        self.data_orientation
    }

    pub const fn data_monoxus_navigation_menu_root(&self) -> &'static str {
        self.data_monoxus_navigation_menu_root
    }
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct NavigationMenuSubAttributes {
    pub id: String,
    pub data_orientation: &'static str,
    pub data_monoxus_navigation_menu_sub: &'static str,
}

impl NavigationMenuSubAttributes {
    pub fn id(&self) -> &str {
        &self.id
    }

    pub const fn data_orientation(&self) -> &'static str {
        self.data_orientation
    }

    pub const fn data_monoxus_navigation_menu_sub(&self) -> &'static str {
        self.data_monoxus_navigation_menu_sub
    }
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct NavigationMenuListTrackAttributes {
    pub id: String,
    pub style: String,
    pub data_monoxus_navigation_menu_track: &'static str,
}

impl NavigationMenuListTrackAttributes {
    pub fn id(&self) -> &str {
        &self.id
    }

    pub fn style(&self) -> &str {
        &self.style
    }

    pub const fn data_monoxus_navigation_menu_track(&self) -> &'static str {
        self.data_monoxus_navigation_menu_track
    }
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct NavigationMenuListAttributes {
    pub id: String,
    pub data_orientation: &'static str,
    pub data_monoxus_navigation_menu_list: &'static str,
}

impl NavigationMenuListAttributes {
    pub fn id(&self) -> &str {
        &self.id
    }

    pub const fn data_orientation(&self) -> &'static str {
        self.data_orientation
    }

    pub const fn data_monoxus_navigation_menu_list(&self) -> &'static str {
        self.data_monoxus_navigation_menu_list
    }
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct NavigationMenuItemAttributes {
    pub id: String,
    pub data_monoxus_navigation_menu_item: &'static str,
}

impl NavigationMenuItemAttributes {
    pub fn id(&self) -> &str {
        &self.id
    }

    pub const fn data_monoxus_navigation_menu_item(&self) -> &'static str {
        self.data_monoxus_navigation_menu_item
    }
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct NavigationMenuTriggerAttributes {
    pub id: String,
    pub r#type: &'static str,
    pub disabled: bool,
    pub aria_expanded: &'static str,
    pub aria_controls: Option<String>,
    pub data_state: DataState,
    pub data_disabled: Option<&'static str>,
    pub data_value: String,
    pub data_monoxus_navigation_menu_trigger: &'static str,
}

impl NavigationMenuTriggerAttributes {
    pub fn id(&self) -> &str {
        &self.id
    }

    pub const fn button_type(&self) -> &'static str {
        self.r#type
    }

    pub const fn is_disabled(&self) -> bool {
        self.disabled
    }

    pub const fn aria_expanded(&self) -> &'static str {
        self.aria_expanded
    }

    pub fn aria_controls(&self) -> Option<&str> {
        self.aria_controls.as_deref()
    }

    pub const fn data_state(&self) -> &DataState {
        &self.data_state
    }

    pub const fn data_disabled(&self) -> Option<&'static str> {
        self.data_disabled
    }

    pub fn data_value(&self) -> &str {
        &self.data_value
    }

    pub const fn data_monoxus_navigation_menu_trigger(&self) -> &'static str {
        self.data_monoxus_navigation_menu_trigger
    }
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct NavigationMenuFocusProxyAttributes {
    pub id: String,
    pub aria_hidden: &'static str,
    pub tabindex: i32,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct NavigationMenuAriaOwnsAttributes {
    pub id: String,
    pub aria_owns: String,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct NavigationMenuContentAttributes {
    pub id: String,
    pub aria_labelledby: String,
    pub data_state: DataState,
    pub data_orientation: &'static str,
    pub data_motion: Option<&'static str>,
    pub style: String,
    pub data_monoxus_navigation_menu_content: &'static str,
}

impl NavigationMenuContentAttributes {
    pub fn id(&self) -> &str {
        &self.id
    }

    pub fn aria_labelledby(&self) -> &str {
        &self.aria_labelledby
    }

    pub const fn data_state(&self) -> &DataState {
        &self.data_state
    }

    pub const fn data_orientation(&self) -> &'static str {
        self.data_orientation
    }

    pub const fn data_motion(&self) -> Option<&'static str> {
        self.data_motion
    }

    pub fn style(&self) -> &str {
        &self.style
    }

    pub const fn data_monoxus_navigation_menu_content(&self) -> &'static str {
        self.data_monoxus_navigation_menu_content
    }
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct NavigationMenuLinkAttributes {
    pub id: String,
    pub href: Option<String>,
    pub target: Option<String>,
    pub rel: Option<String>,
    pub tabindex: i32,
    pub aria_current: Option<&'static str>,
    pub data_active: Option<&'static str>,
    pub data_focused: Option<&'static str>,
    pub data_monoxus_navigation_menu_link: &'static str,
}

impl NavigationMenuLinkAttributes {
    pub fn id(&self) -> &str {
        &self.id
    }

    pub fn href(&self) -> Option<&str> {
        self.href.as_deref()
    }

    pub fn target(&self) -> Option<&str> {
        self.target.as_deref()
    }

    pub fn rel(&self) -> Option<&str> {
        self.rel.as_deref()
    }

    pub const fn tabindex(&self) -> i32 {
        self.tabindex
    }

    pub const fn aria_current(&self) -> Option<&'static str> {
        self.aria_current
    }

    pub const fn data_active(&self) -> Option<&'static str> {
        self.data_active
    }

    pub const fn data_focused(&self) -> Option<&'static str> {
        self.data_focused
    }

    pub const fn data_monoxus_navigation_menu_link(&self) -> &'static str {
        self.data_monoxus_navigation_menu_link
    }
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct NavigationMenuIndicatorAttributes {
    pub id: String,
    pub aria_hidden: &'static str,
    pub data_state: &'static str,
    pub data_orientation: &'static str,
    pub style: String,
    pub data_monoxus_navigation_menu_indicator: &'static str,
}

impl NavigationMenuIndicatorAttributes {
    pub fn id(&self) -> &str {
        &self.id
    }

    pub const fn aria_hidden(&self) -> &'static str {
        self.aria_hidden
    }

    pub const fn data_state(&self) -> &'static str {
        self.data_state
    }

    pub const fn data_orientation(&self) -> &'static str {
        self.data_orientation
    }

    pub fn style(&self) -> &str {
        &self.style
    }

    pub const fn data_monoxus_navigation_menu_indicator(&self) -> &'static str {
        self.data_monoxus_navigation_menu_indicator
    }
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct NavigationMenuViewportAttributes {
    pub id: String,
    pub data_state: DataState,
    pub data_orientation: &'static str,
    pub style: String,
    pub data_monoxus_navigation_menu_viewport: &'static str,
}

impl NavigationMenuViewportAttributes {
    pub fn id(&self) -> &str {
        &self.id
    }

    pub const fn data_state(&self) -> &DataState {
        &self.data_state
    }

    pub const fn data_orientation(&self) -> &'static str {
        self.data_orientation
    }

    pub fn style(&self) -> &str {
        &self.style
    }

    pub const fn data_monoxus_navigation_menu_viewport(&self) -> &'static str {
        self.data_monoxus_navigation_menu_viewport
    }
}

pub fn project_navigation_menu_root_as_child<T, C>(
    attrs: &NavigationMenuRootAttributes,
    target: T,
    content: Slottable<C>,
) -> (T, NavigationMenuRootAttributes, C) {
    let (target, content) = project_as_child(target, content);
    (target, attrs.clone(), content)
}

pub fn project_navigation_menu_sub_as_child<T, C>(
    attrs: &NavigationMenuSubAttributes,
    target: T,
    content: Slottable<C>,
) -> (T, NavigationMenuSubAttributes, C) {
    let (target, content) = project_as_child(target, content);
    (target, attrs.clone(), content)
}

pub fn project_navigation_menu_list_as_child<T, C>(
    attrs: &NavigationMenuListAttributes,
    target: T,
    content: Slottable<C>,
) -> (T, NavigationMenuListAttributes, C) {
    let (target, content) = project_as_child(target, content);
    (target, attrs.clone(), content)
}

pub fn project_navigation_menu_item_as_child<T, C>(
    attrs: &NavigationMenuItemAttributes,
    target: T,
    content: Slottable<C>,
) -> (T, NavigationMenuItemAttributes, C) {
    let (target, content) = project_as_child(target, content);
    (target, attrs.clone(), content)
}

pub fn project_navigation_menu_trigger_as_child<T, C>(
    attrs: &NavigationMenuTriggerAttributes,
    target: T,
    content: Slottable<C>,
) -> (T, NavigationMenuTriggerAttributes, C) {
    let (target, content) = project_as_child(target, content);
    (target, attrs.clone(), content)
}

pub fn project_navigation_menu_content_as_child<T, C>(
    attrs: &NavigationMenuContentAttributes,
    target: T,
    content: Slottable<C>,
) -> (T, NavigationMenuContentAttributes, C) {
    let (target, content) = project_as_child(target, content);
    (target, attrs.clone(), content)
}

pub fn project_navigation_menu_link_as_child<T, C>(
    attrs: &NavigationMenuLinkAttributes,
    target: T,
    content: Slottable<C>,
) -> (T, NavigationMenuLinkAttributes, C) {
    let (target, content) = project_as_child(target, content);
    (target, attrs.clone(), content)
}

pub fn project_navigation_menu_indicator_as_child<T, C>(
    attrs: &NavigationMenuIndicatorAttributes,
    target: T,
    content: Slottable<C>,
) -> (T, NavigationMenuIndicatorAttributes, C) {
    let (target, content) = project_as_child(target, content);
    (target, attrs.clone(), content)
}

pub fn project_navigation_menu_viewport_as_child<T, C>(
    attrs: &NavigationMenuViewportAttributes,
    target: T,
    content: Slottable<C>,
) -> (T, NavigationMenuViewportAttributes, C) {
    let (target, content) = project_as_child(target, content);
    (target, attrs.clone(), content)
}

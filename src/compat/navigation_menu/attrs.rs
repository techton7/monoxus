use ui_sdk::foundation::state::DataState;

pub use ui_sdk::navigation_menu::attrs::{
    NavigationMenuAriaOwnsAttributes, NavigationMenuFocusProxyAttributes,
};

// [KEEP-MONOXUS-BRAND] Compat attribute structs preserving all 10 `data_monoxus_navigation_menu_*` markers.

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

impl From<ui_sdk::navigation_menu::attrs::NavigationMenuRootAttributes>
    for NavigationMenuRootAttributes
{
    fn from(attrs: ui_sdk::navigation_menu::attrs::NavigationMenuRootAttributes) -> Self {
        Self {
            id: attrs.id,
            aria_label: attrs.aria_label,
            dir: attrs.dir,
            data_orientation: attrs.data_orientation,
            data_monoxus_navigation_menu_root: "",
        }
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

impl From<ui_sdk::navigation_menu::attrs::NavigationMenuSubAttributes>
    for NavigationMenuSubAttributes
{
    fn from(attrs: ui_sdk::navigation_menu::attrs::NavigationMenuSubAttributes) -> Self {
        Self {
            id: attrs.id,
            data_orientation: attrs.data_orientation,
            data_monoxus_navigation_menu_sub: "",
        }
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

impl From<ui_sdk::navigation_menu::attrs::NavigationMenuListTrackAttributes>
    for NavigationMenuListTrackAttributes
{
    fn from(attrs: ui_sdk::navigation_menu::attrs::NavigationMenuListTrackAttributes) -> Self {
        Self {
            id: attrs.id,
            style: attrs.style,
            data_monoxus_navigation_menu_track: "",
        }
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

impl From<ui_sdk::navigation_menu::attrs::NavigationMenuListAttributes>
    for NavigationMenuListAttributes
{
    fn from(attrs: ui_sdk::navigation_menu::attrs::NavigationMenuListAttributes) -> Self {
        Self {
            id: attrs.id,
            data_orientation: attrs.data_orientation,
            data_monoxus_navigation_menu_list: "",
        }
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

impl From<ui_sdk::navigation_menu::attrs::NavigationMenuItemAttributes>
    for NavigationMenuItemAttributes
{
    fn from(attrs: ui_sdk::navigation_menu::attrs::NavigationMenuItemAttributes) -> Self {
        Self {
            id: attrs.id,
            data_monoxus_navigation_menu_item: "",
        }
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

impl From<ui_sdk::navigation_menu::attrs::NavigationMenuTriggerAttributes>
    for NavigationMenuTriggerAttributes
{
    fn from(attrs: ui_sdk::navigation_menu::attrs::NavigationMenuTriggerAttributes) -> Self {
        Self {
            id: attrs.id,
            r#type: attrs.r#type,
            disabled: attrs.disabled,
            aria_expanded: attrs.aria_expanded,
            aria_controls: attrs.aria_controls,
            data_state: attrs.data_state,
            data_disabled: attrs.data_disabled,
            data_value: attrs.data_value,
            data_monoxus_navigation_menu_trigger: "",
        }
    }
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

impl From<ui_sdk::navigation_menu::attrs::NavigationMenuContentAttributes>
    for NavigationMenuContentAttributes
{
    fn from(attrs: ui_sdk::navigation_menu::attrs::NavigationMenuContentAttributes) -> Self {
        Self {
            id: attrs.id,
            aria_labelledby: attrs.aria_labelledby,
            data_state: attrs.data_state,
            data_orientation: attrs.data_orientation,
            data_motion: attrs.data_motion,
            style: attrs.style,
            data_monoxus_navigation_menu_content: "",
        }
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

impl From<ui_sdk::navigation_menu::attrs::NavigationMenuLinkAttributes>
    for NavigationMenuLinkAttributes
{
    fn from(attrs: ui_sdk::navigation_menu::attrs::NavigationMenuLinkAttributes) -> Self {
        Self {
            id: attrs.id,
            href: attrs.href,
            target: attrs.target,
            rel: attrs.rel,
            tabindex: attrs.tabindex,
            aria_current: attrs.aria_current,
            data_active: attrs.data_active,
            data_focused: attrs.data_focused,
            data_monoxus_navigation_menu_link: "",
        }
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

impl From<ui_sdk::navigation_menu::attrs::NavigationMenuIndicatorAttributes>
    for NavigationMenuIndicatorAttributes
{
    fn from(attrs: ui_sdk::navigation_menu::attrs::NavigationMenuIndicatorAttributes) -> Self {
        Self {
            id: attrs.id,
            aria_hidden: attrs.aria_hidden,
            data_state: attrs.data_state,
            data_orientation: attrs.data_orientation,
            style: attrs.style,
            data_monoxus_navigation_menu_indicator: "",
        }
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

impl From<ui_sdk::navigation_menu::attrs::NavigationMenuViewportAttributes>
    for NavigationMenuViewportAttributes
{
    fn from(attrs: ui_sdk::navigation_menu::attrs::NavigationMenuViewportAttributes) -> Self {
        Self {
            id: attrs.id,
            data_state: attrs.data_state,
            data_orientation: attrs.data_orientation,
            style: attrs.style,
            data_monoxus_navigation_menu_viewport: "",
        }
    }
}

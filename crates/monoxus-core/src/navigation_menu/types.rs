use std::{cell::Cell, rc::Rc};

use crate::foundation::shared::Orientation;

pub const NAVIGATION_MENU_DEFAULT_DELAY_MS: u64 = 200;
pub const NAVIGATION_MENU_DEFAULT_SKIP_DELAY_MS: u64 = 300;
pub const NAVIGATION_MENU_CLOSE_GRACE_MS: u64 = 150;
pub const NAVIGATION_MENU_POINTER_CLICK_GUARD_MS: u64 = 300;
pub const NAVIGATION_MENU_GEOMETRY_NAMESPACE: &str = "navigation-menu";
pub const NAVIGATION_MENU_RADIX_PREFIX: &str = "radix-navigation-menu";

pub const NAVIGATION_MENU_PARTS: [NavigationMenuPart; 9] = [
    NavigationMenuPart::Root,
    NavigationMenuPart::Sub,
    NavigationMenuPart::List,
    NavigationMenuPart::Item,
    NavigationMenuPart::Trigger,
    NavigationMenuPart::Content,
    NavigationMenuPart::Link,
    NavigationMenuPart::Indicator,
    NavigationMenuPart::Viewport,
];

#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub enum NavigationMenuPart {
    Root,
    Sub,
    List,
    Item,
    Trigger,
    Content,
    Link,
    Indicator,
    Viewport,
}

impl NavigationMenuPart {
    pub const fn as_str(&self) -> &'static str {
        match self {
            Self::Root => "root",
            Self::Sub => "sub",
            Self::List => "list",
            Self::Item => "item",
            Self::Trigger => "trigger",
            Self::Content => "content",
            Self::Link => "link",
            Self::Indicator => "indicator",
            Self::Viewport => "viewport",
        }
    }
}

#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, Hash)]
pub enum NavigationMenuActivationMode {
    #[default]
    Automatic,
    Manual,
}

impl NavigationMenuActivationMode {
    pub const fn is_automatic(self) -> bool {
        matches!(self, Self::Automatic)
    }

    pub const fn is_manual(self) -> bool {
        matches!(self, Self::Manual)
    }

    pub const fn as_str(&self) -> &'static str {
        match self {
            Self::Automatic => "automatic",
            Self::Manual => "manual",
        }
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub enum NavigationMenuMotion {
    FromStart,
    FromEnd,
    ToStart,
    ToEnd,
}

impl NavigationMenuMotion {
    pub const fn as_str(&self) -> &'static str {
        match self {
            Self::FromStart => "from-start",
            Self::FromEnd => "from-end",
            Self::ToStart => "to-start",
            Self::ToEnd => "to-end",
        }
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub enum NavigationMenuFocusProxySide {
    Start,
    End,
}

impl NavigationMenuFocusProxySide {
    pub const fn as_str(&self) -> &'static str {
        match self {
            Self::Start => "start",
            Self::End => "end",
        }
    }
}

#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, Hash)]
pub enum NavigationMenuOutsideBehavior {
    #[default]
    Close,
    Ignore,
}

impl NavigationMenuOutsideBehavior {
    pub const fn should_close(self) -> bool {
        matches!(self, Self::Close)
    }

    pub const fn as_str(&self) -> &'static str {
        match self {
            Self::Close => "close",
            Self::Ignore => "ignore",
        }
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub enum NavigationMenuIndicatorState {
    Visible,
    Hidden,
}

impl NavigationMenuIndicatorState {
    pub const fn as_str(self) -> &'static str {
        match self {
            Self::Visible => "visible",
            Self::Hidden => "hidden",
        }
    }
}

#[derive(Clone, Copy, Debug, Default, PartialEq)]
pub struct NavigationMenuViewportSize {
    pub width: f64,
    pub height: f64,
}

#[derive(Clone, Copy, Debug, Default, PartialEq)]
pub struct NavigationMenuIndicatorPosition {
    pub size: f64,
    pub offset: f64,
}

pub type NavigationMenuTriggerPosition = NavigationMenuIndicatorPosition;

#[derive(Clone, Copy, Debug, Default, PartialEq)]
pub struct NavigationMenuGeometryVars {
    pub viewport_size: Option<NavigationMenuViewportSize>,
    pub indicator_position: Option<NavigationMenuIndicatorPosition>,
    pub orientation: Orientation,
}

impl NavigationMenuGeometryVars {
    pub const fn new(orientation: Orientation) -> Self {
        Self {
            viewport_size: None,
            indicator_position: None,
            orientation,
        }
    }

    pub const fn with_viewport_size(mut self, size: Option<NavigationMenuViewportSize>) -> Self {
        self.viewport_size = size;
        self
    }

    pub const fn with_indicator_position(
        mut self,
        position: Option<NavigationMenuIndicatorPosition>,
    ) -> Self {
        self.indicator_position = position;
        self
    }

    pub fn viewport_css_variables(&self) -> Vec<(String, String)> {
        let Some(size) = self.viewport_size else {
            return Vec::new();
        };
        let w = format!("{:.0}px", size.width);
        let h = format!("{:.0}px", size.height);
        vec![
            (
                format!("--monoxus-{NAVIGATION_MENU_GEOMETRY_NAMESPACE}-viewport-width"),
                w.clone(),
            ),
            (
                format!("--monoxus-{NAVIGATION_MENU_GEOMETRY_NAMESPACE}-viewport-height"),
                h.clone(),
            ),
            (format!("--{NAVIGATION_MENU_RADIX_PREFIX}-viewport-width"), w),
            (format!("--{NAVIGATION_MENU_RADIX_PREFIX}-viewport-height"), h),
        ]
    }

    pub fn indicator_css_variables(&self) -> Vec<(String, String)> {
        let Some(pos) = self.indicator_position else {
            return Vec::new();
        };
        let axis = match self.orientation {
            Orientation::Vertical => "y",
            Orientation::Horizontal | Orientation::Both => "x",
        };
        let offset = format!("{}px", pos.offset);
        vec![
            (
                format!("--monoxus-{NAVIGATION_MENU_GEOMETRY_NAMESPACE}-indicator-translate-{axis}"),
                offset.clone(),
            ),
            (
                format!("--{NAVIGATION_MENU_RADIX_PREFIX}-indicator-translate-{axis}"),
                offset,
            ),
        ]
    }
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct NavigationMenuConfig {
    pub delay_duration_ms: u64,
    pub skip_delay_duration_ms: u64,
    pub close_grace_ms: u64,
    pub orientation: Orientation,
    pub activation_mode: NavigationMenuActivationMode,
    pub aria_label: Option<String>,
}

impl Default for NavigationMenuConfig {
    fn default() -> Self {
        Self {
            delay_duration_ms: NAVIGATION_MENU_DEFAULT_DELAY_MS,
            skip_delay_duration_ms: NAVIGATION_MENU_DEFAULT_SKIP_DELAY_MS,
            close_grace_ms: NAVIGATION_MENU_CLOSE_GRACE_MS,
            orientation: Orientation::Horizontal,
            activation_mode: NavigationMenuActivationMode::Automatic,
            aria_label: Some("Main".to_string()),
        }
    }
}

/// Cancelable selection event emitted by `NavigationMenuLink`.
#[derive(Clone, Debug)]
pub struct NavigationMenuLinkSelectEvent {
    default_prevented: Rc<Cell<bool>>,
    meta_key: bool,
    href: Option<String>,
}

impl PartialEq for NavigationMenuLinkSelectEvent {
    fn eq(&self, other: &Self) -> bool {
        self.default_prevented.get() == other.default_prevented.get()
            && self.meta_key == other.meta_key
            && self.href == other.href
    }
}

impl Eq for NavigationMenuLinkSelectEvent {}

impl NavigationMenuLinkSelectEvent {
    pub fn new(href: Option<String>, meta_key: bool) -> Self {
        Self {
            default_prevented: Rc::new(Cell::new(false)),
            meta_key,
            href,
        }
    }

    pub fn prevent_default(&self) {
        self.default_prevented.set(true);
    }

    pub fn is_default_prevented(&self) -> bool {
        self.default_prevented.get()
    }

    pub const fn meta_key(&self) -> bool {
        self.meta_key
    }

    pub fn href(&self) -> Option<&str> {
        self.href.as_deref()
    }

    pub fn should_dismiss_root(&self) -> bool {
        !self.default_prevented.get() && !self.meta_key
    }
}

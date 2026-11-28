use serde::{Deserialize, Serialize};

use ui_sdk::foundation::shared::Orientation;

pub use ui_sdk::navigation_menu::types::{
    NAVIGATION_MENU_CLOSE_GRACE_MS, NAVIGATION_MENU_DEFAULT_DELAY_MS,
    NAVIGATION_MENU_DEFAULT_SKIP_DELAY_MS, NAVIGATION_MENU_GEOMETRY_NAMESPACE,
    NAVIGATION_MENU_PARTS, NAVIGATION_MENU_POINTER_CLICK_GUARD_MS, NavigationMenuActivationMode,
    NavigationMenuConfig, NavigationMenuFocusProxySide, NavigationMenuIndicatorPosition,
    NavigationMenuIndicatorState, NavigationMenuLinkSelectDetail, NavigationMenuLinkSelectEvent,
    NavigationMenuMotion, NavigationMenuOutsideBehavior, NavigationMenuPart,
    NavigationMenuTriggerPosition, NavigationMenuViewportSize,
};

// [KEEP-MONOXUS-BRAND] Legacy Radix compatibility prefix and dual `--monoxus-*`/`--radix-*` geometry vars.
pub const NAVIGATION_MENU_RADIX_PREFIX: &str = "radix-navigation-menu";

const MONOXUS_CSS_PREFIX: &str = "monoxus";

#[derive(Clone, Copy, Debug, Default, PartialEq)]
#[derive(Serialize, Deserialize)]
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

    fn to_ui_sdk(self) -> ui_sdk::navigation_menu::types::NavigationMenuGeometryVars {
        ui_sdk::navigation_menu::types::NavigationMenuGeometryVars::new(self.orientation)
            .with_viewport_size(self.viewport_size)
            .with_indicator_position(self.indicator_position)
    }

    pub fn viewport_css_variables(&self) -> Vec<(String, String)> {
        let Some(size) = self.viewport_size else {
            return Vec::new();
        };
        let inner = self.to_ui_sdk();
        let mut vars = inner.viewport_css_variables_with_prefix(MONOXUS_CSS_PREFIX);
        let w = format!("{:.0}px", size.width);
        let h = format!("{:.0}px", size.height);
        vars.push((format!("--{NAVIGATION_MENU_RADIX_PREFIX}-viewport-width"), w));
        vars.push((format!("--{NAVIGATION_MENU_RADIX_PREFIX}-viewport-height"), h));
        vars
    }

    pub fn viewport_css_variables_with_prefix(&self, prefix: &str) -> Vec<(String, String)> {
        self.to_ui_sdk().viewport_css_variables_with_prefix(prefix)
    }

    pub fn indicator_css_variables(&self) -> Vec<(String, String)> {
        let Some(pos) = self.indicator_position else {
            return Vec::new();
        };
        let inner = self.to_ui_sdk();
        let mut vars = inner.indicator_css_variables_with_prefix(MONOXUS_CSS_PREFIX);
        let axis = match self.orientation {
            Orientation::Vertical => "y",
            Orientation::Horizontal | Orientation::Both => "x",
        };
        vars.push((
            format!("--{NAVIGATION_MENU_RADIX_PREFIX}-indicator-translate-{axis}"),
            format!("{}px", pos.offset),
        ));
        vars
    }

    pub fn indicator_css_variables_with_prefix(&self, prefix: &str) -> Vec<(String, String)> {
        self.to_ui_sdk().indicator_css_variables_with_prefix(prefix)
    }
}

impl From<NavigationMenuGeometryVars>
    for ui_sdk::navigation_menu::types::NavigationMenuGeometryVars
{
    fn from(vars: NavigationMenuGeometryVars) -> Self {
        vars.to_ui_sdk()
    }
}

impl From<ui_sdk::navigation_menu::types::NavigationMenuGeometryVars>
    for NavigationMenuGeometryVars
{
    fn from(vars: ui_sdk::navigation_menu::types::NavigationMenuGeometryVars) -> Self {
        Self {
            viewport_size: vars.viewport_size,
            indicator_position: vars.indicator_position,
            orientation: vars.orientation,
        }
    }
}

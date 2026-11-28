pub use crate::foundation::shared::{
    EscapeKeyDownEvent, FocusOutsideEvent, InteractOutsideEvent, PointerDownOutsideEvent,
    PreventableEvent,
};
pub use crate::compat::navigation_menu::types::{
    NAVIGATION_MENU_CLOSE_GRACE_MS, NAVIGATION_MENU_DEFAULT_DELAY_MS,
    NAVIGATION_MENU_DEFAULT_SKIP_DELAY_MS, NAVIGATION_MENU_GEOMETRY_NAMESPACE,
    NAVIGATION_MENU_PARTS, NAVIGATION_MENU_POINTER_CLICK_GUARD_MS, NAVIGATION_MENU_RADIX_PREFIX,
    NavigationMenuActivationMode, NavigationMenuConfig, NavigationMenuFocusProxySide,
    NavigationMenuGeometryVars, NavigationMenuIndicatorPosition, NavigationMenuIndicatorState,
    NavigationMenuMotion, NavigationMenuOutsideBehavior, NavigationMenuPart,
    NavigationMenuTriggerPosition, NavigationMenuViewportSize,
};

/// Payload carried by [`NavigationMenuLinkSelectEvent`].
#[derive(Clone, Debug, Default, PartialEq, Eq)]
pub struct NavigationMenuLinkSelectDetail {
    pub href: Option<String>,
    pub meta_key: bool,
}

/// Cancelable selection event emitted by `NavigationMenuLink`, backed by [`PreventableEvent`].
pub type NavigationMenuLinkSelectEvent = PreventableEvent<NavigationMenuLinkSelectDetail>;

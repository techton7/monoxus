// [KEEP-MONOXUS-BRAND] Thin `DropdownMenuState` wrapper over `ui_sdk` preserving Monoxus/Radix CSS vars and `data-monoxus-*` attrs.
use crate::foundation::{
    overlay::{FloatingLayer, GeometryVars, PortalHost},
    shared::Direction,
    state::DataState,
};
use crate::menu::{MenuContentState, MenuKey, MenuOpenFocusTarget, MenuVariant, menu_css_vars};

use super::{
    attrs::{
        DropdownMenuContentAttributes, DropdownMenuRootAttributes, DropdownMenuTriggerAttributes,
    },
    relationships::DropdownMenuRelationships,
    types::DropdownMenuTriggerAction,
};

/// Pure headless state container for a `DropdownMenu` root + trigger + content pair.
#[derive(Clone, Debug, PartialEq)]
pub struct DropdownMenuState(ui_sdk::dropdown_menu::state::DropdownMenuState);

impl From<ui_sdk::dropdown_menu::state::DropdownMenuState> for DropdownMenuState {
    #[inline]
    fn from(inner: ui_sdk::dropdown_menu::state::DropdownMenuState) -> Self {
        Self(inner)
    }
}

impl From<DropdownMenuState> for ui_sdk::dropdown_menu::state::DropdownMenuState {
    #[inline]
    fn from(state: DropdownMenuState) -> Self {
        state.0
    }
}

impl DropdownMenuState {
    pub fn new(relationships: DropdownMenuRelationships, open: bool) -> Self {
        Self(ui_sdk::dropdown_menu::state::DropdownMenuState::new(
            relationships,
            open,
        ))
    }

    pub fn inner(&self) -> &ui_sdk::dropdown_menu::state::DropdownMenuState {
        &self.0
    }

    pub fn as_inner(&self) -> &ui_sdk::dropdown_menu::state::DropdownMenuState {
        &self.0
    }

    pub fn into_inner(self) -> ui_sdk::dropdown_menu::state::DropdownMenuState {
        self.0
    }

    pub fn with_direction(self, direction: Direction) -> Self {
        Self(self.0.with_direction(direction))
    }

    pub fn with_modal(self, modal: bool) -> Self {
        Self(self.0.with_modal(modal))
    }

    pub fn with_loop_focus(self, loop_focus: bool) -> Self {
        Self(self.0.with_loop_focus(loop_focus))
    }

    pub fn with_disabled(self, disabled: bool) -> Self {
        Self(self.0.with_disabled(disabled))
    }

    pub fn with_portal_host(self, portal_host: PortalHost) -> Self {
        Self(self.0.with_portal_host(portal_host))
    }

    pub fn with_floating(self, floating: FloatingLayer) -> Self {
        Self(self.0.with_floating(floating))
    }

    pub fn with_open_on_hover(self, open_on_hover: bool) -> Self {
        Self(self.0.with_open_on_hover(open_on_hover))
    }

    pub fn with_hover_open_delay_ms(self, delay_ms: u64) -> Self {
        Self(self.0.with_hover_open_delay_ms(delay_ms))
    }

    pub fn with_hover_close_delay_ms(self, delay_ms: u64) -> Self {
        Self(self.0.with_hover_close_delay_ms(delay_ms))
    }

    pub fn relationships(&self) -> &DropdownMenuRelationships {
        self.0.relationships()
    }

    pub fn content(&self) -> &MenuContentState {
        self.0.content()
    }

    pub fn content_mut(&mut self) -> &mut MenuContentState {
        self.0.content_mut()
    }

    pub fn is_open(&self) -> bool {
        self.0.is_open()
    }

    pub fn is_disabled(&self) -> bool {
        self.0.is_disabled()
    }

    pub fn is_modal(&self) -> bool {
        self.0.is_modal()
    }

    pub fn open_on_hover(&self) -> bool {
        self.0.open_on_hover()
    }

    pub fn hover_open_delay_ms(&self) -> u64 {
        self.0.hover_open_delay_ms()
    }

    pub fn hover_close_delay_ms(&self) -> u64 {
        self.0.hover_close_delay_ms()
    }

    pub fn data_state(&self) -> DataState {
        self.0.data_state()
    }

    pub fn open_focus_target(&self) -> MenuOpenFocusTarget {
        self.0.open_focus_target()
    }

    pub fn set_open_focus_target(&mut self, target: MenuOpenFocusTarget) {
        self.0.set_open_focus_target(target);
    }

    pub fn resolve_trigger_pointer_down(&self, trigger_disabled: bool) -> DropdownMenuTriggerAction {
        self.0.resolve_trigger_pointer_down(trigger_disabled)
    }

    pub fn resolve_trigger_key(
        &self,
        key: MenuKey,
        trigger_disabled: bool,
    ) -> DropdownMenuTriggerAction {
        self.0.resolve_trigger_key(key, trigger_disabled)
    }

    pub fn css_vars(&self, geometry: &GeometryVars) -> Vec<(String, String)> {
        menu_css_vars(geometry, MenuVariant::DropdownMenu)
    }

    pub fn css_vars_with_prefix(
        &self,
        geometry: &GeometryVars,
        prefix: &str,
    ) -> Vec<(String, String)> {
        self.0.css_vars_with_prefix(geometry, prefix)
    }

    pub fn root_attrs(&self) -> DropdownMenuRootAttributes {
        self.0.root_attrs().into()
    }

    pub fn trigger_attrs(&self, trigger_disabled: bool) -> DropdownMenuTriggerAttributes {
        self.0.trigger_attrs(trigger_disabled).into()
    }

    pub fn content_attrs(&self) -> DropdownMenuContentAttributes {
        self.0.content_attrs().into()
    }
}

#[cfg(test)]
mod tests {
    use crate::foundation::{
        overlay::{FloatingLayer, PlacementSide, Rect, Size},
        shared::ScopeHandle,
    };

    use super::*;
    use crate::dropdown_menu::types::{
        DROPDOWN_MENU_PARTS, DROPDOWN_MENU_RADIX_ANCHOR_LABEL,
        DROPDOWN_MENU_RADIX_COMPATIBILITY_PREFIX,
    };

    #[test]
    fn dropdown_menu_compat_emits_monoxus_attrs_and_radix_css_vars() {
        let rel = DropdownMenuRelationships::new(ScopeHandle::root("dropdown-menu").child("dm-1"));
        let state = DropdownMenuState::new(rel, true);

        assert_eq!(state.root_attrs().part_attr, "data-monoxus-dropdown-menu-root");
        assert_eq!(
            state.trigger_attrs(false).part_attr,
            "data-monoxus-dropdown-menu-trigger"
        );
        assert_eq!(
            state.content_attrs().part_attr,
            "data-monoxus-dropdown-menu-content"
        );
        assert_eq!(DROPDOWN_MENU_PARTS.len(), 20);
        assert_eq!(
            DROPDOWN_MENU_RADIX_COMPATIBILITY_PREFIX,
            "radix-dropdown-menu"
        );
        assert_eq!(DROPDOWN_MENU_RADIX_ANCHOR_LABEL, "trigger");

        let layer = FloatingLayer::new(PlacementSide::Bottom).with_namespace("dropdown-menu");
        let geometry =
            layer.geometry_vars(Rect::new(0.0, 0.0, 120.0, 32.0), Size::new(180.0, 120.0));
        let vars = state.css_vars(&geometry);
        assert!(vars.iter().any(|(k, _)| k == "--monoxus-dropdown-menu-anchor-width"));
        assert!(vars.iter().any(|(k, _)| k == "--radix-dropdown-menu-trigger-width"));
    }
}

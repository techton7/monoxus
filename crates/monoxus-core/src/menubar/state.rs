// [KEEP-MONOXUS-BRAND] Thin `MenubarState` wrapper over `ui_sdk` preserving Monoxus/Radix CSS vars and `data-monoxus-*` attrs.
use crate::foundation::{
    overlay::GeometryVars,
    shared::{Direction, Orientation},
};
use crate::menu::{MenuKey, MenuVariant, menu_css_vars};

use super::{
    attrs::{MenubarRootAttributes, MenubarTriggerAttributes},
    relationships::MenubarRelationships,
    types::{MenubarCrossMenuResult, MenubarMenuEntry, MenubarNavigationStep},
};

/// Pure headless state machine coordinating top-level `Menubar` focus, hover swap, and cross-menu arrows.
#[derive(Clone, Debug, PartialEq)]
pub struct MenubarState(ui_sdk::menubar::state::MenubarState);

impl From<ui_sdk::menubar::state::MenubarState> for MenubarState {
    #[inline]
    fn from(inner: ui_sdk::menubar::state::MenubarState) -> Self {
        Self(inner)
    }
}

impl From<MenubarState> for ui_sdk::menubar::state::MenubarState {
    #[inline]
    fn from(state: MenubarState) -> Self {
        state.0
    }
}

impl MenubarState {
    pub fn new(relationships: MenubarRelationships) -> Self {
        Self(ui_sdk::menubar::state::MenubarState::new(relationships))
    }

    pub fn inner(&self) -> &ui_sdk::menubar::state::MenubarState {
        &self.0
    }

    pub fn as_inner(&self) -> &ui_sdk::menubar::state::MenubarState {
        &self.0
    }

    pub fn into_inner(self) -> ui_sdk::menubar::state::MenubarState {
        self.0
    }

    pub fn with_active_value(self, active_value: Option<String>) -> Self {
        Self(self.0.with_active_value(active_value))
    }

    pub fn with_direction(self, direction: Direction) -> Self {
        Self(self.0.with_direction(direction))
    }

    pub fn with_orientation(self, orientation: Orientation) -> Self {
        Self(self.0.with_orientation(orientation))
    }

    pub fn with_loop_focus(self, loop_focus: bool) -> Self {
        Self(self.0.with_loop_focus(loop_focus))
    }

    pub fn with_modal(self, modal: bool) -> Self {
        Self(self.0.with_modal(modal))
    }

    pub fn with_disabled(self, disabled: bool) -> Self {
        Self(self.0.with_disabled(disabled))
    }

    pub fn relationships(&self) -> &MenubarRelationships {
        self.0.relationships()
    }

    pub fn active_value(&self) -> Option<&str> {
        self.0.active_value()
    }

    pub fn set_active_value(&mut self, value: Option<String>) {
        self.0.set_active_value(value);
    }

    pub fn current_tabstop_value(&self) -> Option<&str> {
        self.0.current_tabstop_value()
    }

    pub fn set_current_tabstop_value(&mut self, value: Option<String>) {
        self.0.set_current_tabstop_value(value);
    }

    pub fn highlighted_value(&self) -> Option<&str> {
        self.0.highlighted_value()
    }

    pub fn set_highlighted_value(&mut self, value: Option<String>) {
        self.0.set_highlighted_value(value);
    }

    pub fn direction(&self) -> Direction {
        self.0.direction()
    }

    pub fn orientation(&self) -> Orientation {
        self.0.orientation()
    }

    pub fn loop_focus(&self) -> bool {
        self.0.loop_focus()
    }

    pub fn is_modal(&self) -> bool {
        self.0.is_modal()
    }

    pub fn is_disabled(&self) -> bool {
        self.0.is_disabled()
    }

    pub fn has_submenu_open(&self) -> bool {
        self.0.has_submenu_open()
    }

    pub fn data_has_submenu_open(&self) -> Option<&'static str> {
        self.0.data_has_submenu_open()
    }

    pub fn data_modal(&self) -> Option<&'static str> {
        self.0.data_modal()
    }

    pub fn data_disabled(&self) -> Option<&'static str> {
        self.0.data_disabled()
    }

    pub fn data_orientation(&self) -> &'static str {
        self.0.data_orientation()
    }

    pub fn menus(&self) -> &[MenubarMenuEntry] {
        self.0.menus()
    }

    pub fn register_menu(&mut self, entry: MenubarMenuEntry) {
        self.0.register_menu(entry);
    }

    pub fn unregister_menu(&mut self, value: &str) {
        self.0.unregister_menu(value);
    }

    pub fn sync_dom_order(&mut self, ordered_trigger_ids: &[String]) {
        self.0.sync_dom_order(ordered_trigger_ids);
    }

    pub fn is_tabstop(&self, value: &str) -> bool {
        self.0.is_tabstop(value)
    }

    pub fn open_menu(&mut self, value: &str) -> (Option<String>, bool) {
        self.0.open_menu(value)
    }

    pub fn close_menu(&mut self, value: &str) -> bool {
        self.0.close_menu(value)
    }

    pub fn close_all(&mut self) {
        self.0.close_all();
    }

    pub fn is_trigger_element_id(&self, id: &str) -> bool {
        self.0.is_trigger_element_id(id)
    }

    pub fn first_enabled_menu(&self) -> Option<&MenubarMenuEntry> {
        self.0.first_enabled_menu()
    }

    pub fn last_enabled_menu(&self) -> Option<&MenubarMenuEntry> {
        self.0.last_enabled_menu()
    }

    pub fn adjacent_enabled_menu(
        &self,
        current_value: &str,
        forward: bool,
    ) -> Option<&MenubarMenuEntry> {
        self.0.adjacent_enabled_menu(current_value, forward)
    }

    pub fn on_trigger_pointer_enter(&self, value: &str) -> Option<&MenubarMenuEntry> {
        self.0.on_trigger_pointer_enter(value)
    }

    pub fn on_trigger_key(&self, current_value: &str, key: MenuKey) -> MenubarNavigationStep {
        self.0.on_trigger_key(current_value, key)
    }

    pub fn resolve_cross_menu_arrow(
        &self,
        current_value: &str,
        key: MenuKey,
        on_sub_trigger: bool,
        in_submenu: bool,
    ) -> MenubarCrossMenuResult {
        self.0
            .resolve_cross_menu_arrow(current_value, key, on_sub_trigger, in_submenu)
    }

    pub fn css_vars(geometry: &GeometryVars) -> Vec<(String, String)> {
        menu_css_vars(geometry, MenuVariant::Menubar)
    }

    pub fn css_vars_with_prefix(geometry: &GeometryVars, prefix: &str) -> Vec<(String, String)> {
        ui_sdk::menubar::state::MenubarState::css_vars_with_prefix(geometry, prefix)
    }

    pub fn root_attrs(&self) -> MenubarRootAttributes {
        self.0.root_attrs().into()
    }

    pub fn trigger_attrs(
        &self,
        value: &str,
        trigger_id: String,
        content_id: &str,
        disabled: bool,
    ) -> MenubarTriggerAttributes {
        self.0
            .trigger_attrs(value, trigger_id, content_id, disabled)
            .into()
    }
}

#[cfg(test)]
mod tests {
    use crate::foundation::{
        overlay::{FloatingLayer, PlacementSide, Rect, Size},
        shared::ScopeHandle,
    };

    use super::*;
    use crate::menubar::types::{
        MENUBAR_PARTS, MENUBAR_RADIX_ANCHOR_LABEL, MENUBAR_RADIX_COMPATIBILITY_PREFIX,
    };

    #[test]
    fn menubar_compat_emits_monoxus_attrs_and_radix_css_vars() {
        let rel = MenubarRelationships::new(ScopeHandle::root("menubar").child("mb-1"));
        let mut state = MenubarState::new(rel);
        state.register_menu(MenubarMenuEntry::new("file", "t-file", "c-file", false, 0));

        let root = state.root_attrs();
        assert_eq!(root.role, "menubar");
        assert_eq!(root.aria_orientation, "horizontal");
        assert_eq!(root.part_attr, "data-monoxus-menubar-root");

        let trigger = state.trigger_attrs("file", "t-file".into(), "c-file", false);
        assert_eq!(trigger.part_attr, "data-monoxus-menubar-trigger");
        assert_eq!(MENUBAR_PARTS.len(), 20);
        assert_eq!(MENUBAR_RADIX_COMPATIBILITY_PREFIX, "radix-menubar");
        assert_eq!(MENUBAR_RADIX_ANCHOR_LABEL, "trigger");

        let layer = FloatingLayer::new(PlacementSide::Bottom).with_namespace("menubar");
        let geometry = layer.geometry_vars(Rect::new(0.0, 0.0, 60.0, 28.0), Size::new(160.0, 96.0));
        let vars = MenubarState::css_vars(&geometry);
        assert!(vars.iter().any(|(k, _)| k == "--monoxus-menubar-anchor-width"));
        assert!(vars.iter().any(|(k, _)| k == "--radix-menubar-trigger-width"));
    }
}

// [KEEP-MONOXUS-BRAND] Thin `ContextMenuState` wrapper over `ui_sdk` preserving Monoxus/Radix CSS vars and `data-monoxus-*` attrs.
use crate::foundation::{
    overlay::{FloatingLayer, GeometryVars, PortalHost},
    shared::Direction,
    state::DataState,
};
use crate::menu::{MenuContentState, MenuOpenFocusTarget, MenuVariant, menu_css_vars};

use super::{
    attrs::{ContextMenuContentAttributes, ContextMenuRootAttributes, ContextMenuTriggerAttributes},
    relationships::ContextMenuRelationships,
    types::{ContextMenuTriggerAction, VirtualAnchorPoint},
};

/// Pure headless state container for a `ContextMenu` root, trigger region, and virtual-anchored content.
#[derive(Clone, Debug, PartialEq)]
pub struct ContextMenuState(ui_sdk::context_menu::state::ContextMenuState);

impl From<ui_sdk::context_menu::state::ContextMenuState> for ContextMenuState {
    #[inline]
    fn from(inner: ui_sdk::context_menu::state::ContextMenuState) -> Self {
        Self(inner)
    }
}

impl From<ContextMenuState> for ui_sdk::context_menu::state::ContextMenuState {
    #[inline]
    fn from(state: ContextMenuState) -> Self {
        state.0
    }
}

impl ContextMenuState {
    pub fn new(relationships: ContextMenuRelationships, open: bool) -> Self {
        Self(ui_sdk::context_menu::state::ContextMenuState::new(
            relationships,
            open,
        ))
    }

    pub fn inner(&self) -> &ui_sdk::context_menu::state::ContextMenuState {
        &self.0
    }

    pub fn as_inner(&self) -> &ui_sdk::context_menu::state::ContextMenuState {
        &self.0
    }

    pub fn into_inner(self) -> ui_sdk::context_menu::state::ContextMenuState {
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

    pub fn relationships(&self) -> &ContextMenuRelationships {
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

    pub fn data_state(&self) -> DataState {
        self.0.data_state()
    }

    pub fn anchor_point(&self) -> Option<VirtualAnchorPoint> {
        self.0.anchor_point()
    }

    pub fn set_anchor_point(&mut self, point: Option<VirtualAnchorPoint>) {
        self.0.set_anchor_point(point);
    }

    pub fn open_focus_target(&self) -> MenuOpenFocusTarget {
        self.0.open_focus_target()
    }

    /// Innermost nested `ContextMenuTrigger` claim check.
    pub const fn can_claim_trigger_event(already_claimed_by_descendant: bool) -> bool {
        ui_sdk::context_menu::state::ContextMenuState::can_claim_trigger_event(
            already_claimed_by_descendant,
        )
    }

    /// Non-mouse long press eligibility check.
    pub fn should_start_long_press(&self, pointer_type: &str, trigger_disabled: bool) -> bool {
        self.0
            .should_start_long_press(pointer_type, trigger_disabled)
    }

    /// Evaluate a `contextmenu` or completed long-press gesture at `(x, y)`.
    pub fn open_at_point(
        &mut self,
        point: VirtualAnchorPoint,
        trigger_disabled: bool,
        already_claimed_by_descendant: bool,
    ) -> ContextMenuTriggerAction {
        self.0
            .open_at_point(point, trigger_disabled, already_claimed_by_descendant)
    }

    pub fn css_vars(&self, geometry: &GeometryVars) -> Vec<(String, String)> {
        menu_css_vars(geometry, MenuVariant::ContextMenu)
    }

    pub fn css_vars_with_prefix(
        &self,
        geometry: &GeometryVars,
        prefix: &str,
    ) -> Vec<(String, String)> {
        self.0.css_vars_with_prefix(geometry, prefix)
    }

    pub fn root_attrs(&self) -> ContextMenuRootAttributes {
        self.0.root_attrs().into()
    }

    pub fn trigger_attrs(&self, trigger_disabled: bool) -> ContextMenuTriggerAttributes {
        self.0.trigger_attrs(trigger_disabled).into()
    }

    pub fn content_attrs(&self) -> ContextMenuContentAttributes {
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
    use crate::context_menu::types::{
        CONTEXT_MENU_PARTS, CONTEXT_MENU_RADIX_ANCHOR_LABEL,
        CONTEXT_MENU_RADIX_COMPATIBILITY_PREFIX,
    };

    #[test]
    fn context_menu_compat_emits_monoxus_attrs_and_radix_css_vars() {
        let rel = ContextMenuRelationships::new(ScopeHandle::root("context-menu").child("cm-1"));
        let state = ContextMenuState::new(rel, true);

        assert_eq!(state.root_attrs().part_attr, "data-monoxus-context-menu-root");
        let trigger = state.trigger_attrs(false);
        assert_eq!(trigger.part_attr, "data-monoxus-context-menu-trigger");
        assert_eq!(trigger.style, "-webkit-touch-callout: none;");
        assert_eq!(
            state.content_attrs().part_attr,
            "data-monoxus-context-menu-content"
        );
        assert_eq!(CONTEXT_MENU_PARTS.len(), 20);
        assert_eq!(CONTEXT_MENU_RADIX_COMPATIBILITY_PREFIX, "radix-context-menu");
        assert_eq!(CONTEXT_MENU_RADIX_ANCHOR_LABEL, "trigger");

        let layer = FloatingLayer::new(PlacementSide::Right).with_namespace("context-menu");
        let geometry = layer.geometry_vars(Rect::new(10.0, 20.0, 0.0, 0.0), Size::new(140.0, 96.0));
        let vars = state.css_vars(&geometry);
        assert!(vars.iter().any(|(k, _)| k == "--monoxus-context-menu-content-transform-origin"));
        assert!(vars.iter().any(|(k, _)| k == "--radix-context-menu-trigger-width"));
    }
}

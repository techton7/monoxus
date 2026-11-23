use crate::foundation::{
    overlay::{FloatingLayer, GeometryVars, PlacementAlign, PlacementSide, PortalHost, StickyStrategy},
    shared::Direction,
    state::DataState,
};
use crate::menu::{
    MENU_CONTEXT_DEFAULT_SIDE_OFFSET, MenuContentState, MenuOpenFocusTarget, MenuPart, MenuVariant,
    menu_css_vars,
};

use super::{
    attrs::{ContextMenuContentAttributes, ContextMenuRootAttributes, ContextMenuTriggerAttributes},
    relationships::ContextMenuRelationships,
    types::{ContextMenuTriggerAction, VirtualAnchorPoint},
};

/// Pure headless state container for a `ContextMenu` root, trigger region, and virtual-anchored content.
#[derive(Clone, Debug, PartialEq)]
pub struct ContextMenuState {
    relationships: ContextMenuRelationships,
    content: MenuContentState,
    anchor_point: Option<VirtualAnchorPoint>,
    disabled: bool,
    open_focus_target: MenuOpenFocusTarget,
}

impl ContextMenuState {
    pub fn new(relationships: ContextMenuRelationships, open: bool) -> Self {
        let content_id = relationships.content_id().to_owned();
        let floating = FloatingLayer::new(PlacementSide::Right)
            .with_align(PlacementAlign::Start)
            .with_side_offset(MENU_CONTEXT_DEFAULT_SIDE_OFFSET)
            .with_sticky(StickyStrategy::Partial)
            .with_namespace(MenuVariant::ContextMenu.geometry_namespace());

        Self {
            relationships,
            content: MenuContentState::new(MenuVariant::ContextMenu, content_id, open)
                .with_floating(floating),
            anchor_point: None,
            disabled: false,
            open_focus_target: MenuOpenFocusTarget::ContentContainer,
        }
    }

    pub fn with_direction(mut self, direction: Direction) -> Self {
        self.content = self.content.with_direction(direction);
        self
    }

    pub fn with_modal(mut self, modal: bool) -> Self {
        self.content = self.content.with_modal(modal);
        self
    }

    pub fn with_loop_focus(mut self, loop_focus: bool) -> Self {
        self.content = self.content.with_loop_focus(loop_focus);
        self
    }

    pub fn with_disabled(mut self, disabled: bool) -> Self {
        self.disabled = disabled;
        self
    }

    pub fn with_portal_host(mut self, portal_host: PortalHost) -> Self {
        self.content = self.content.with_portal_host(portal_host);
        self
    }

    pub fn with_floating(mut self, floating: FloatingLayer) -> Self {
        self.content = self.content.with_floating(floating);
        self
    }

    pub fn relationships(&self) -> &ContextMenuRelationships {
        &self.relationships
    }

    pub fn content(&self) -> &MenuContentState {
        &self.content
    }

    pub fn content_mut(&mut self) -> &mut MenuContentState {
        &mut self.content
    }

    pub fn is_open(&self) -> bool {
        self.content.is_open()
    }

    pub fn is_disabled(&self) -> bool {
        self.disabled
    }

    pub fn is_modal(&self) -> bool {
        self.content.is_modal()
    }

    pub fn data_state(&self) -> DataState {
        self.content.data_state()
    }

    pub fn anchor_point(&self) -> Option<VirtualAnchorPoint> {
        self.anchor_point
    }

    pub fn set_anchor_point(&mut self, point: Option<VirtualAnchorPoint>) {
        self.anchor_point = point;
    }

    pub fn open_focus_target(&self) -> MenuOpenFocusTarget {
        self.open_focus_target
    }

    /// Innermost nested `ContextMenuTrigger` claim check (`CM-7`).
    pub const fn can_claim_trigger_event(already_claimed_by_descendant: bool) -> bool {
        !already_claimed_by_descendant
    }

    /// Non-mouse long press eligibility check (`CM-5`).
    pub fn should_start_long_press(&self, pointer_type: &str, trigger_disabled: bool) -> bool {
        !(self.disabled || trigger_disabled) && !pointer_type.eq_ignore_ascii_case("mouse")
    }

    /// Evaluate a `contextmenu` or completed long-press gesture at `(x, y)` (`CM-3`, `CM-6`).
    pub fn open_at_point(
        &mut self,
        point: VirtualAnchorPoint,
        trigger_disabled: bool,
        already_claimed_by_descendant: bool,
    ) -> ContextMenuTriggerAction {
        if self.disabled || trigger_disabled || !Self::can_claim_trigger_event(already_claimed_by_descendant) {
            return ContextMenuTriggerAction::Ignore;
        }
        self.anchor_point = Some(point);
        self.content.presence_mut().sync(true);
        ContextMenuTriggerAction::OpenAtPoint { point }
    }

    pub fn css_vars(&self, geometry: &GeometryVars) -> Vec<(String, String)> {
        menu_css_vars(geometry, MenuVariant::ContextMenu)
    }

    pub fn root_attrs(&self) -> ContextMenuRootAttributes {
        ContextMenuRootAttributes::new(
            self.relationships.root_id().to_owned(),
            self.is_open(),
            MenuVariant::ContextMenu,
        )
    }

    pub fn trigger_attrs(&self, trigger_disabled: bool) -> ContextMenuTriggerAttributes {
        ContextMenuTriggerAttributes::from_parts(
            self.relationships.trigger_id().to_owned(),
            self.is_open(),
            self.disabled || trigger_disabled,
        )
    }

    pub fn content_attrs(&self) -> ContextMenuContentAttributes {
        ContextMenuContentAttributes::new(
            self.relationships.content_id().to_owned(),
            self.relationships.wrapper_id().to_owned(),
            None,
            self.content.direction(),
            self.is_open(),
            self.content.floating().data_side(),
            self.content.floating().data_align(),
            MenuVariant::ContextMenu,
            MenuPart::Content,
        )
    }
}

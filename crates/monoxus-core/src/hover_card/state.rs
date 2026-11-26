use crate::foundation::{
    overlay::{
        DismissLayer, FloatingLayer, GeometryVars, PlacementAlign, PlacementSide, Point2D,
        PortalHost, Presence, Rect, Rect2D, Size, StickyStrategy, create_grace_area,
        is_point_in_polygon,
    },
    shared::{Direction, ScopeHandle},
    state::DataState,
};

use super::{
    attrs::{
        HoverCardArrowAttributes, HoverCardContentAttributes, HoverCardContentStaticAttributes,
        HoverCardPortalAttributes, HoverCardRootAttributes, HoverCardTriggerAttributes,
    },
    relationships::HoverCardRelationships,
    types::{
        HOVER_CARD_DEFAULT_CLOSE_DELAY_MS, HOVER_CARD_DEFAULT_OPEN_DELAY_MS,
        HOVER_CARD_DEFAULT_SIDE_OFFSET, HOVER_CARD_GEOMETRY_NAMESPACE, HOVER_CARD_PARTS,
        HOVER_CARD_RADIX_ANCHOR_LABEL, HOVER_CARD_RADIX_COMPATIBILITY_PREFIX, HoverCardPart,
    },
};

#[derive(Clone, Debug, PartialEq)]
pub struct HoverCardLifecycle {
    portal_host: PortalHost,
    presence: Presence,
    dismiss_layer: DismissLayer<String>,
    floating: FloatingLayer,
}

impl HoverCardLifecycle {
    pub fn new(relationships: &HoverCardRelationships, open: bool) -> Self {
        Self {
            portal_host: PortalHost::default_host(),
            presence: Presence::new(open).with_retained_mount(true),
            dismiss_layer: DismissLayer::new(relationships.content_id().to_owned())
                .with_modal(false)
                .with_focus_outside_dismiss(false),
            floating: FloatingLayer::new(PlacementSide::Bottom)
                .with_align(PlacementAlign::Center)
                .with_side_offset(HOVER_CARD_DEFAULT_SIDE_OFFSET)
                .with_sticky(StickyStrategy::Partial)
                .with_namespace(HOVER_CARD_GEOMETRY_NAMESPACE),
        }
    }

    pub fn with_portal_host(mut self, portal_host: PortalHost) -> Self {
        self.portal_host = portal_host;
        self
    }

    pub fn with_floating(mut self, floating: FloatingLayer) -> Self {
        self.floating = floating.with_namespace(HOVER_CARD_GEOMETRY_NAMESPACE);
        self
    }

    pub fn portal_host(&self) -> &PortalHost {
        &self.portal_host
    }

    pub fn presence(&self) -> &Presence {
        &self.presence
    }

    pub fn presence_mut(&mut self) -> &mut Presence {
        &mut self.presence
    }

    pub fn dismiss_layer(&self) -> &DismissLayer<String> {
        &self.dismiss_layer
    }

    pub fn dismiss_layer_mut(&mut self) -> &mut DismissLayer<String> {
        &mut self.dismiss_layer
    }

    pub fn register_branch(&mut self, branch: impl Into<String>) -> bool {
        self.dismiss_layer.register_branch(branch.into())
    }

    pub fn floating(&self) -> &FloatingLayer {
        &self.floating
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum HoverCardGraceStatus {
    InTransit,
    ReachedTarget,
    ExitedCorridor,
}

#[derive(Clone, Debug, PartialEq)]
pub struct HoverCardGraceTransit {
    pub exit_point: Point2D,
    pub from_rect: Rect2D,
    pub to_rect: Rect2D,
    pub corridor: Vec<Point2D>,
}

impl HoverCardGraceTransit {
    pub fn new(
        exit_point: Point2D,
        from_rect: Rect2D,
        to_rect: Rect2D,
        padding: Option<f64>,
    ) -> Self {
        let clamped_exit = Point2D::new(
            exit_point.x.clamp(from_rect.left(), from_rect.right()),
            exit_point.y.clamp(from_rect.top(), from_rect.bottom()),
        );
        let corridor = create_grace_area(clamped_exit, &from_rect, &to_rect, padding);
        Self {
            exit_point: clamped_exit,
            from_rect,
            to_rect,
            corridor,
        }
    }

    pub fn inspect_point(&self, point: Point2D) -> HoverCardGraceStatus {
        if self.to_rect.contains_point(point) {
            HoverCardGraceStatus::ReachedTarget
        } else if self.from_rect.contains_point(point) || is_point_in_polygon(point, &self.corridor) {
            HoverCardGraceStatus::InTransit
        } else {
            HoverCardGraceStatus::ExitedCorridor
        }
    }
}

/// Pure interaction guard state for `HoverCard` open/close suppression rules.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub struct HoverCardInteractionState {
    pub trigger_hovered: bool,
    pub content_hovered: bool,
    pub opened_via_focus: bool,
    pub is_pointer_down_on_content: bool,
    pub has_selection: bool,
    pub contain_selection: bool,
    pub open_request_token: u64,
    pub close_request_token: u64,
}

impl HoverCardInteractionState {
    pub const fn is_touch_pointer(pointer_type: &str) -> bool {
        matches!(pointer_type.as_bytes(), b"touch")
    }

    pub const fn should_suppress_close(&self) -> bool {
        self.has_selection || self.is_pointer_down_on_content
    }

    pub const fn should_ignore_pointer_leave(&self) -> bool {
        self.opened_via_focus || self.should_suppress_close()
    }

    pub const fn can_commit_scheduled_close(&self) -> bool {
        !self.trigger_hovered
            && !self.content_hovered
            && !self.opened_via_focus
            && !self.should_suppress_close()
    }

    pub fn schedule_open_token(&mut self) -> u64 {
        self.close_request_token = self.close_request_token.saturating_add(1);
        self.open_request_token = self.open_request_token.saturating_add(1);
        self.open_request_token
    }

    pub fn cancel_pending_open(&mut self) {
        self.open_request_token = self.open_request_token.saturating_add(1);
    }

    pub fn schedule_close_token(&mut self) -> u64 {
        self.open_request_token = self.open_request_token.saturating_add(1);
        self.close_request_token = self.close_request_token.saturating_add(1);
        self.close_request_token
    }

    pub fn cancel_pending_close(&mut self) {
        self.close_request_token = self.close_request_token.saturating_add(1);
    }

    pub fn begin_content_selection(&mut self) {
        self.contain_selection = true;
        self.has_selection = false;
        self.is_pointer_down_on_content = true;
    }

    pub fn end_content_pointer_down(&mut self, has_text_selection: bool) {
        self.contain_selection = false;
        self.is_pointer_down_on_content = false;
        self.has_selection = has_text_selection;
    }

    pub fn reset_selection_and_hover(&mut self) {
        self.trigger_hovered = false;
        self.content_hovered = false;
        self.opened_via_focus = false;
        self.is_pointer_down_on_content = false;
        self.has_selection = false;
        self.contain_selection = false;
    }
}

#[derive(Clone, Debug, PartialEq)]
pub struct HoverCard {
    relationships: HoverCardRelationships,
    lifecycle: HoverCardLifecycle,
    open: bool,
    open_delay_ms: u64,
    close_delay_ms: u64,
    disabled: bool,
    ignore_non_keyboard_focus: bool,
}

impl HoverCard {
    pub fn new(scope: ScopeHandle, open: bool) -> Self {
        let relationships = HoverCardRelationships::new(scope);
        let lifecycle = HoverCardLifecycle::new(&relationships, open);

        Self {
            relationships,
            lifecycle,
            open,
            open_delay_ms: HOVER_CARD_DEFAULT_OPEN_DELAY_MS,
            close_delay_ms: HOVER_CARD_DEFAULT_CLOSE_DELAY_MS,
            disabled: false,
            ignore_non_keyboard_focus: true,
        }
    }

    pub const fn parts() -> &'static [HoverCardPart] {
        &HOVER_CARD_PARTS
    }

    pub const fn is_open(&self) -> bool {
        self.open
    }

    pub fn data_state(&self) -> DataState {
        if self.open {
            DataState::Open
        } else {
            DataState::Closed
        }
    }

    pub fn relationships(&self) -> &HoverCardRelationships {
        &self.relationships
    }

    pub fn lifecycle(&self) -> &HoverCardLifecycle {
        &self.lifecycle
    }

    pub fn lifecycle_mut(&mut self) -> &mut HoverCardLifecycle {
        &mut self.lifecycle
    }

    pub const fn open_delay_ms(&self) -> u64 {
        self.open_delay_ms
    }

    pub const fn close_delay_ms(&self) -> u64 {
        self.close_delay_ms
    }

    pub const fn is_disabled(&self) -> bool {
        self.disabled
    }

    pub const fn ignore_non_keyboard_focus(&self) -> bool {
        self.ignore_non_keyboard_focus
    }

    pub fn with_trigger_id(mut self, trigger_id: impl Into<String>) -> Self {
        self.relationships = self.relationships.with_trigger_id(trigger_id);
        self
    }

    pub fn with_content_id(mut self, content_id: impl Into<String>) -> Self {
        self.relationships = self.relationships.with_content_id(content_id);
        self
    }

    pub fn with_arrow_id(mut self, arrow_id: impl Into<String>) -> Self {
        self.relationships = self.relationships.with_arrow_id(arrow_id);
        self
    }

    pub fn with_portal_host(mut self, portal_host: PortalHost) -> Self {
        self.lifecycle = self.lifecycle.with_portal_host(portal_host);
        self
    }

    pub fn with_floating(mut self, floating: FloatingLayer) -> Self {
        self.lifecycle = self.lifecycle.with_floating(floating);
        self
    }

    pub fn with_direction(mut self, direction: Direction) -> Self {
        let updated = self.lifecycle.floating().clone().with_direction(direction);
        self.lifecycle = self.lifecycle.with_floating(updated);
        self
    }

    pub const fn with_open_delay_ms(mut self, open_delay_ms: u64) -> Self {
        self.open_delay_ms = open_delay_ms;
        self
    }

    pub const fn with_close_delay_ms(mut self, close_delay_ms: u64) -> Self {
        self.close_delay_ms = close_delay_ms;
        self
    }

    pub const fn with_disabled(mut self, disabled: bool) -> Self {
        self.disabled = disabled;
        self
    }

    pub const fn with_ignore_non_keyboard_focus(
        mut self,
        ignore_non_keyboard_focus: bool,
    ) -> Self {
        self.ignore_non_keyboard_focus = ignore_non_keyboard_focus;
        self
    }

    pub fn geometry_vars(&self, anchor: Rect, content: Size) -> GeometryVars {
        self.lifecycle.floating().geometry_vars(anchor, content)
    }

    pub fn content_css_variables(geometry: &GeometryVars) -> Vec<(String, String)> {
        geometry
            .css_iter()
            .chain(geometry.compatibility_alias_iter(
                HOVER_CARD_RADIX_COMPATIBILITY_PREFIX,
                HOVER_CARD_RADIX_ANCHOR_LABEL,
            ))
            .collect()
    }

    pub fn root(&self) -> HoverCardRootAttributes {
        HoverCardRootAttributes {
            id: self.relationships.root_id().to_owned(),
            data_state: self.data_state(),
        }
    }

    pub fn trigger(
        &self,
        href: Option<String>,
        target: Option<String>,
        rel: Option<String>,
        trigger_disabled: bool,
    ) -> HoverCardTriggerAttributes {
        let effective_disabled = self.disabled || trigger_disabled;
        HoverCardTriggerAttributes::from_parts(
            self.relationships.trigger_id().to_owned(),
            href,
            target,
            rel,
            effective_disabled,
            self.data_state(),
        )
    }

    pub fn portal(&self) -> HoverCardPortalAttributes {
        HoverCardPortalAttributes {
            host: self.lifecycle.portal_host().clone(),
        }
    }

    pub fn content(&self) -> HoverCardContentAttributes {
        HoverCardContentAttributes {
            id: self.relationships.content_id().to_owned(),
            wrapper_id: self.relationships.wrapper_id().to_owned(),
            tabindex: -1,
            data_state: self.data_state(),
            data_side: self.lifecycle.floating().data_side(),
            data_align: self.lifecycle.floating().data_align(),
            data_monoxus_hover_card_content: "",
        }
    }

    pub fn content_static(&self) -> HoverCardContentStaticAttributes {
        HoverCardContentStaticAttributes {
            id: self.relationships.content_id().to_owned(),
            tabindex: -1,
            data_state: self.data_state(),
            data_monoxus_hover_card_content: "",
        }
    }

    pub fn arrow(&self) -> HoverCardArrowAttributes {
        HoverCardArrowAttributes {
            id: self.relationships.arrow_id().to_owned(),
            data_state: self.data_state(),
            data_side: self.lifecycle.floating().data_side(),
            data_align: self.lifecycle.floating().data_align(),
            data_monoxus_hover_card_arrow: "",
        }
    }
}

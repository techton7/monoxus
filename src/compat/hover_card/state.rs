use ui_sdk::foundation::{
    overlay::{FloatingLayer, GeometryVars, PortalHost, Rect, Size},
    shared::{Direction, ScopeHandle},
    state::DataState,
};

pub use ui_sdk::hover_card::state::{
    HoverCardGraceStatus, HoverCardGraceTransit, HoverCardInteractionState, HoverCardLifecycle,
};

use super::{
    attrs::{
        HoverCardArrowAttributes, HoverCardContentAttributes, HoverCardContentStaticAttributes,
        HoverCardPortalAttributes, HoverCardRootAttributes, HoverCardTriggerAttributes,
    },
    relationships::HoverCardRelationships,
    types::{
        HOVER_CARD_RADIX_ANCHOR_LABEL, HOVER_CARD_RADIX_COMPATIBILITY_PREFIX, HoverCardPart,
    },
};

const MONOXUS_CSS_PREFIX: &str = "monoxus";

// [KEEP-MONOXUS-BRAND] Compat wrapper preserving `--monoxus-hover-card-*` + `--radix-hover-card-*`
// CSS variable emission and `data_monoxus_hover_card_*` attribute projections.
#[derive(Clone, Debug, PartialEq)]
pub struct HoverCard(ui_sdk::hover_card::state::HoverCard);

impl From<ui_sdk::hover_card::state::HoverCard> for HoverCard {
    fn from(inner: ui_sdk::hover_card::state::HoverCard) -> Self {
        Self(inner)
    }
}

impl HoverCard {
    pub fn new(scope: ScopeHandle, open: bool) -> Self {
        Self(ui_sdk::hover_card::state::HoverCard::new(scope, open))
    }

    pub fn as_inner(&self) -> &ui_sdk::hover_card::state::HoverCard {
        &self.0
    }

    pub fn into_inner(self) -> ui_sdk::hover_card::state::HoverCard {
        self.0
    }

    pub const fn parts() -> &'static [HoverCardPart] {
        ui_sdk::hover_card::state::HoverCard::parts()
    }

    pub const fn is_open(&self) -> bool {
        self.0.is_open()
    }

    pub fn data_state(&self) -> DataState {
        self.0.data_state()
    }

    pub fn relationships(&self) -> &HoverCardRelationships {
        self.0.relationships()
    }

    pub fn lifecycle(&self) -> &HoverCardLifecycle {
        self.0.lifecycle()
    }

    pub fn lifecycle_mut(&mut self) -> &mut HoverCardLifecycle {
        self.0.lifecycle_mut()
    }

    pub const fn open_delay_ms(&self) -> u64 {
        self.0.open_delay_ms()
    }

    pub const fn close_delay_ms(&self) -> u64 {
        self.0.close_delay_ms()
    }

    pub const fn is_disabled(&self) -> bool {
        self.0.is_disabled()
    }

    pub const fn ignore_non_keyboard_focus(&self) -> bool {
        self.0.ignore_non_keyboard_focus()
    }

    pub fn with_trigger_id(self, trigger_id: impl Into<String>) -> Self {
        Self(self.0.with_trigger_id(trigger_id))
    }

    pub fn set_trigger_id(&mut self, trigger_id: impl Into<String>) {
        self.0.set_trigger_id(trigger_id);
    }

    pub fn with_content_id(self, content_id: impl Into<String>) -> Self {
        Self(self.0.with_content_id(content_id))
    }

    pub fn set_content_id(&mut self, content_id: impl Into<String>) {
        self.0.set_content_id(content_id);
    }

    pub fn with_arrow_id(self, arrow_id: impl Into<String>) -> Self {
        Self(self.0.with_arrow_id(arrow_id))
    }

    pub fn with_portal_host(self, portal_host: PortalHost) -> Self {
        Self(self.0.with_portal_host(portal_host))
    }

    pub fn set_portal_host(&mut self, portal_host: PortalHost) {
        self.0.set_portal_host(portal_host);
    }

    pub fn with_floating(self, floating: FloatingLayer) -> Self {
        Self(self.0.with_floating(floating))
    }

    pub fn set_floating(&mut self, floating: FloatingLayer) {
        self.0.set_floating(floating);
    }

    pub fn with_direction(self, direction: Direction) -> Self {
        Self(self.0.with_direction(direction))
    }

    pub fn with_open_delay_ms(self, open_delay_ms: u64) -> Self {
        Self(self.0.with_open_delay_ms(open_delay_ms))
    }

    pub fn with_close_delay_ms(self, close_delay_ms: u64) -> Self {
        Self(self.0.with_close_delay_ms(close_delay_ms))
    }

    pub fn with_disabled(self, disabled: bool) -> Self {
        Self(self.0.with_disabled(disabled))
    }

    pub fn with_ignore_non_keyboard_focus(self, ignore_non_keyboard_focus: bool) -> Self {
        Self(self.0.with_ignore_non_keyboard_focus(ignore_non_keyboard_focus))
    }

    pub fn geometry_vars(&self, anchor: Rect, content: Size) -> GeometryVars {
        self.0
            .geometry_vars(anchor, content)
            .with_css_prefix(MONOXUS_CSS_PREFIX)
    }

    pub fn content_css_variables(geometry: &GeometryVars) -> Vec<(String, String)> {
        let branded = geometry.clone().with_css_prefix(MONOXUS_CSS_PREFIX);
        branded
            .css_iter()
            .chain(branded.compatibility_alias_iter(
                HOVER_CARD_RADIX_COMPATIBILITY_PREFIX,
                HOVER_CARD_RADIX_ANCHOR_LABEL,
            ))
            .collect()
    }

    pub fn root(&self) -> HoverCardRootAttributes {
        self.0.root()
    }

    pub fn trigger(
        &self,
        href: Option<String>,
        target: Option<String>,
        rel: Option<String>,
        trigger_disabled: bool,
    ) -> HoverCardTriggerAttributes {
        self.0
            .trigger(href, target, rel, trigger_disabled)
            .into()
    }

    pub fn portal(&self) -> HoverCardPortalAttributes {
        self.0.portal()
    }

    pub fn content(&self) -> HoverCardContentAttributes {
        self.0.content().into()
    }

    pub fn content_static(&self) -> HoverCardContentStaticAttributes {
        self.0.content_static().into()
    }

    pub fn arrow(&self) -> HoverCardArrowAttributes {
        self.0.arrow().into()
    }
}

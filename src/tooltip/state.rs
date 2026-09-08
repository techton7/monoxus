use crate::foundation::{
    overlay::{DismissLayer, FloatingLayer, GeometryVars, PlacementSide, PortalHost, Presence, Rect, Size},
    shared::ScopeHandle,
    state::DataState,
};

use super::{
    attrs::{
        TooltipArrowAttributes, TooltipContentAttributes, TooltipPortalAttributes,
        TooltipRootAttributes, TooltipTriggerAttributes,
    },
    relationships::TooltipRelationships,
    types::{TOOLTIP_GEOMETRY_NAMESPACE, TOOLTIP_PARTS, TooltipPart, TooltipStateRequest},
};

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct TooltipProvider {
    id: String,
    delay_duration_ms: u64,
    skip_delay_duration_ms: u64,
    disable_hoverable_content: bool,
    close_on_trigger_click: bool,
    ignore_non_keyboard_focus: bool,
}

impl TooltipProvider {
    pub const DEFAULT_DELAY_DURATION_MS: u64 = 700;
    pub const DEFAULT_SKIP_DELAY_DURATION_MS: u64 = 300;

    pub fn new(scope: ScopeHandle) -> Self {
        Self {
            id: scope.token(),
            delay_duration_ms: Self::DEFAULT_DELAY_DURATION_MS,
            skip_delay_duration_ms: Self::DEFAULT_SKIP_DELAY_DURATION_MS,
            disable_hoverable_content: false,
            close_on_trigger_click: true,
            ignore_non_keyboard_focus: false,
        }
    }

    pub fn with_delay_duration_ms(mut self, delay_duration_ms: u64) -> Self {
        self.delay_duration_ms = delay_duration_ms;
        self
    }

    pub fn with_skip_delay_duration_ms(mut self, skip_delay_duration_ms: u64) -> Self {
        self.skip_delay_duration_ms = skip_delay_duration_ms;
        self
    }

    pub fn with_disable_hoverable_content(mut self, disable_hoverable_content: bool) -> Self {
        self.disable_hoverable_content = disable_hoverable_content;
        self
    }

    pub fn with_close_on_trigger_click(mut self, close_on_trigger_click: bool) -> Self {
        self.close_on_trigger_click = close_on_trigger_click;
        self
    }

    pub fn with_ignore_non_keyboard_focus(mut self, ignore_non_keyboard_focus: bool) -> Self {
        self.ignore_non_keyboard_focus = ignore_non_keyboard_focus;
        self
    }

    pub fn id(&self) -> &str {
        &self.id
    }

    pub const fn delay_duration_ms(&self) -> u64 {
        self.delay_duration_ms
    }

    pub const fn skip_delay_duration_ms(&self) -> u64 {
        self.skip_delay_duration_ms
    }

    pub const fn disable_hoverable_content(&self) -> bool {
        self.disable_hoverable_content
    }

    pub const fn close_on_trigger_click(&self) -> bool {
        self.close_on_trigger_click
    }

    pub const fn ignore_non_keyboard_focus(&self) -> bool {
        self.ignore_non_keyboard_focus
    }
}

#[derive(Clone, Debug, PartialEq)]
pub struct TooltipLifecycle {
    portal_host: PortalHost,
    presence: Presence,
    dismiss_layer: DismissLayer<String>,
    floating: FloatingLayer,
    provider: Option<TooltipProvider>,
    content_autofocus_suppressed: bool,
}

impl TooltipLifecycle {
    pub fn new(relationships: &TooltipRelationships, open: bool) -> Self {
        Self {
            portal_host: PortalHost::default_host(),
            presence: Presence::new(open).with_retained_mount(true),
            dismiss_layer: DismissLayer::new(relationships.content_id().to_owned()),
            floating: FloatingLayer::new(PlacementSide::Top)
                .with_side_offset(8.0)
                .with_namespace(TOOLTIP_GEOMETRY_NAMESPACE),
            provider: None,
            content_autofocus_suppressed: true,
        }
    }

    pub fn with_portal_host(mut self, portal_host: PortalHost) -> Self {
        self.portal_host = portal_host;
        self
    }

    pub fn with_floating(mut self, floating: FloatingLayer) -> Self {
        self.floating = floating.with_namespace(TOOLTIP_GEOMETRY_NAMESPACE);
        self
    }

    pub fn with_provider(mut self, provider: TooltipProvider) -> Self {
        self.provider = Some(provider);
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

    pub fn provider(&self) -> Option<&TooltipProvider> {
        self.provider.as_ref()
    }

    pub const fn content_autofocus_suppressed(&self) -> bool {
        self.content_autofocus_suppressed
    }
}

#[derive(Clone, Debug, PartialEq)]
pub struct Tooltip {
    relationships: TooltipRelationships,
    lifecycle: TooltipLifecycle,
    open: bool,
}

impl Tooltip {
    pub fn new(scope: ScopeHandle, open: bool) -> Self {
        let relationships = TooltipRelationships::new(scope);
        let lifecycle = TooltipLifecycle::new(&relationships, open);

        Self {
            relationships,
            lifecycle,
            open,
        }
    }

    pub const fn parts() -> &'static [TooltipPart] {
        &TOOLTIP_PARTS
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

    pub fn relationships(&self) -> &TooltipRelationships {
        &self.relationships
    }

    pub fn lifecycle(&self) -> &TooltipLifecycle {
        &self.lifecycle
    }

    pub fn lifecycle_mut(&mut self) -> &mut TooltipLifecycle {
        &mut self.lifecycle
    }

    pub fn provider(&self) -> Option<&TooltipProvider> {
        self.lifecycle.provider()
    }

    pub fn with_portal_host(mut self, portal_host: PortalHost) -> Self {
        self.lifecycle = self.lifecycle.with_portal_host(portal_host);
        self
    }

    pub fn with_floating(mut self, floating: FloatingLayer) -> Self {
        self.lifecycle = self.lifecycle.with_floating(floating);
        self
    }

    pub fn with_provider(mut self, provider: TooltipProvider) -> Self {
        self.lifecycle = self.lifecycle.with_provider(provider);
        self
    }

    pub fn geometry_vars(&self, anchor: Rect, content: Size) -> GeometryVars {
        self.lifecycle.floating().geometry_vars(anchor, content)
    }

    pub fn root(&self) -> TooltipRootAttributes {
        TooltipRootAttributes {
            id: self.relationships.root_id().to_owned(),
            data_state: self.data_state(),
        }
    }

    pub fn trigger(&self) -> TooltipTriggerAttributes {
        TooltipTriggerAttributes {
            id: self.relationships.trigger_id().to_owned(),
            aria_describedby: if self.is_open() {
                Some(self.relationships.content_id().to_owned())
            } else {
                None
            },
            provider_id: self.provider().map(|provider| provider.id().to_owned()),
            data_state: self.data_state(),
            open_request: TooltipStateRequest::Open,
            close_request: TooltipStateRequest::Close,
        }
    }

    pub fn portal(&self) -> TooltipPortalAttributes {
        TooltipPortalAttributes {
            host: self.lifecycle.portal_host().clone(),
        }
    }

    pub fn content(&self) -> TooltipContentAttributes {
        TooltipContentAttributes {
            id: self.relationships.content_id().to_owned(),
            role: "tooltip",
            data_state: self.data_state(),
            data_side: self.lifecycle.floating().data_side(),
            data_align: self.lifecycle.floating().data_align(),
            autofocus_suppressed: self.lifecycle.content_autofocus_suppressed(),
        }
    }

    pub fn arrow(&self) -> TooltipArrowAttributes {
        TooltipArrowAttributes {
            id: self.relationships.arrow_id().to_owned(),
            data_state: self.data_state(),
            data_side: self.lifecycle.floating().data_side(),
            data_align: self.lifecycle.floating().data_align(),
        }
    }
}

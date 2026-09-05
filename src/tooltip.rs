use std::{rc::Rc, time::Duration};

use dioxus::{document, document::Eval, prelude::*};
use futures_timer::Delay;

pub use crate::foundation::compose::{
    compose_part_event_handlers, compose_part_refs, project_as_child,
};

use crate::foundation::{
    browser::{
        FloatingAutoUpdateEvent, recv_floating_auto_update_event,
        start_floating_auto_update_monitor, stop_floating_auto_update_monitor,
    },
    overlay::{
        DismissLayer, FloatingLayer, FloatingPlacement, GeometryVars, PlacementSide, PortalHost,
        Presence, Rect, Size,
    },
    shared::ScopeHandle,
    state::DataState,
};

pub const TOOLTIP_GEOMETRY_NAMESPACE: &str = "tooltip";
const TOOLTIP_HOVER_TRANSFER_GRACE_MS: u64 = 40;

pub const TOOLTIP_PARTS: [TooltipPart; 6] = [
    TooltipPart::Root,
    TooltipPart::Trigger,
    TooltipPart::Portal,
    TooltipPart::Content,
    TooltipPart::Arrow,
    TooltipPart::Provider,
];

#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub enum TooltipPart {
    Root,
    Trigger,
    Portal,
    Content,
    Arrow,
    Provider,
}

impl TooltipPart {
    pub const fn as_str(&self) -> &'static str {
        match self {
            Self::Root => "root",
            Self::Trigger => "trigger",
            Self::Portal => "portal",
            Self::Content => "content",
            Self::Arrow => "arrow",
            Self::Provider => "provider",
        }
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub enum TooltipStateRequest {
    Open,
    Close,
}

impl TooltipStateRequest {
    pub const fn next_open(self) -> bool {
        matches!(self, Self::Open)
    }

    pub fn data_state(self) -> DataState {
        if self.next_open() {
            DataState::Open
        } else {
            DataState::Closed
        }
    }
}

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

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct TooltipRelationships {
    scope: ScopeHandle,
    root_id: String,
    trigger_id: String,
    content_id: String,
    arrow_id: String,
}

impl TooltipRelationships {
    pub fn new(scope: ScopeHandle) -> Self {
        Self {
            root_id: scope.token(),
            trigger_id: scope.qualify("trigger"),
            content_id: scope.qualify("content"),
            arrow_id: scope.qualify("arrow"),
            scope,
        }
    }

    pub fn scope(&self) -> &ScopeHandle {
        &self.scope
    }

    pub fn root_id(&self) -> &str {
        &self.root_id
    }

    pub fn trigger_id(&self) -> &str {
        &self.trigger_id
    }

    pub fn content_id(&self) -> &str {
        &self.content_id
    }

    pub fn arrow_id(&self) -> &str {
        &self.arrow_id
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

type TooltipOpenChangeHandler = Rc<dyn Fn(bool)>;

#[derive(Clone, Copy)]
struct TooltipProviderRuntimeState {
    active_tooltip_id: Signal<Option<String>>,
    pending_open_tooltip_id: Signal<Option<String>>,
    open_request_token: Signal<u64>,
    skip_delay_token: Signal<u64>,
    instant_phase: Signal<bool>,
}

#[derive(Clone)]
pub struct TooltipProviderRuntime {
    provider: TooltipProvider,
    state: TooltipProviderRuntimeState,
}

pub fn use_tooltip_provider_runtime(provider: TooltipProvider) -> TooltipProviderRuntime {
    let state = TooltipProviderRuntimeState {
        active_tooltip_id: use_signal(|| None),
        pending_open_tooltip_id: use_signal(|| None),
        open_request_token: use_signal(|| 0),
        skip_delay_token: use_signal(|| 0),
        instant_phase: use_signal(|| false),
    };
    let cleanup_state = state;

    dioxus::core::use_drop(move || {
        advance_tooltip_token(cleanup_state.open_request_token);
        advance_tooltip_token(cleanup_state.skip_delay_token);
    });

    TooltipProviderRuntime { provider, state }
}

impl TooltipProviderRuntime {
    pub fn provider(&self) -> &TooltipProvider {
        &self.provider
    }

    pub fn active_tooltip_id(&self) -> Option<String> {
        self.state.active_tooltip_id.cloned()
    }

    pub fn pending_open_tooltip_id(&self) -> Option<String> {
        self.state.pending_open_tooltip_id.cloned()
    }

    pub fn opens_instantly(&self) -> bool {
        *self.state.instant_phase.peek()
    }

    pub fn request_open(&self, tooltip_id: impl Into<String>, immediate: bool) {
        let tooltip_id = tooltip_id.into();
        let should_open_immediately = immediate
            || self.provider.delay_duration_ms() == 0
            || self.opens_instantly()
            || self.active_tooltip_id().is_some();

        if should_open_immediately {
            self.commit_open(tooltip_id);
            return;
        }

        let request_token = advance_tooltip_token(self.state.open_request_token);
        let mut pending_open_tooltip_id = self.state.pending_open_tooltip_id;
        pending_open_tooltip_id.set(Some(tooltip_id.clone()));
        let provider_runtime = self.clone();

        spawn(async move {
            Delay::new(Duration::from_millis(
                provider_runtime.provider.delay_duration_ms(),
            ))
            .await;

            if *provider_runtime.state.open_request_token.peek() != request_token {
                return;
            }

            if provider_runtime
                .state
                .pending_open_tooltip_id
                .with_peek(|pending| pending.as_deref() == Some(tooltip_id.as_str()))
            {
                provider_runtime.commit_open(tooltip_id);
            }
        });
    }

    pub fn request_close(&self, tooltip_id: &str) {
        let mut active_tooltip_id = self.state.active_tooltip_id;
        let was_active =
            active_tooltip_id.with_peek(|active| active.as_deref() == Some(tooltip_id));
        let was_pending = self
            .state
            .pending_open_tooltip_id
            .with_peek(|pending| pending.as_deref() == Some(tooltip_id));

        if was_pending {
            self.clear_pending_open();
            advance_tooltip_token(self.state.open_request_token);
        }

        if was_active {
            active_tooltip_id.set(None);
            self.start_skip_delay_window();
        }
    }

    fn commit_open(&self, tooltip_id: String) {
        self.clear_pending_open();
        advance_tooltip_token(self.state.open_request_token);
        self.invalidate_skip_delay();
        let mut active_tooltip_id = self.state.active_tooltip_id;
        active_tooltip_id.set(Some(tooltip_id));
        let mut instant_phase = self.state.instant_phase;
        instant_phase.set(true);
    }

    fn clear_pending_open(&self) {
        let mut pending_open_tooltip_id = self.state.pending_open_tooltip_id;
        pending_open_tooltip_id.set(None);
    }

    fn invalidate_skip_delay(&self) {
        advance_tooltip_token(self.state.skip_delay_token);
    }

    fn start_skip_delay_window(&self) {
        self.invalidate_skip_delay();

        if self.provider.skip_delay_duration_ms() == 0 {
            let mut instant_phase = self.state.instant_phase;
            instant_phase.set(false);
            return;
        }

        let request_token = *self.state.skip_delay_token.peek();
        let provider_runtime = self.clone();

        spawn(async move {
            Delay::new(Duration::from_millis(
                provider_runtime.provider.skip_delay_duration_ms(),
            ))
            .await;

            if *provider_runtime.state.skip_delay_token.peek() != request_token {
                return;
            }

            if provider_runtime.active_tooltip_id().is_none() {
                let mut instant_phase = provider_runtime.state.instant_phase;
                instant_phase.set(false);
            }
        });
    }
}

#[derive(Clone, Copy)]
struct TooltipRuntimeState {
    trigger_handle: Signal<Option<Rc<MountedData>>>,
    content_handle: Signal<Option<Rc<MountedData>>>,
    placement: Signal<Option<FloatingPlacement>>,
    pointer_down_inside: Signal<bool>,
    trigger_hovered: Signal<bool>,
    content_hovered: Signal<bool>,
    hover_transfer_token: Signal<u64>,
    open_request_token: Signal<u64>,
    position_loop_token: Signal<u64>,
    position_monitor: Signal<Option<Eval>>,
}

#[derive(Clone)]
pub struct TooltipRuntime {
    tooltip: Tooltip,
    provider_runtime: Option<TooltipProviderRuntime>,
    on_open_change: TooltipOpenChangeHandler,
    state: TooltipRuntimeState,
}

pub fn use_tooltip_runtime<F>(
    tooltip: Tooltip,
    provider_runtime: Option<TooltipProviderRuntime>,
    on_open_change: F,
) -> TooltipRuntime
where
    F: Fn(bool) + 'static,
{
    let state = TooltipRuntimeState {
        trigger_handle: use_signal(|| None),
        content_handle: use_signal(|| None),
        placement: use_signal(|| None),
        pointer_down_inside: use_signal(|| false),
        trigger_hovered: use_signal(|| false),
        content_hovered: use_signal(|| false),
        hover_transfer_token: use_signal(|| 0),
        open_request_token: use_signal(|| 0),
        position_loop_token: use_signal(|| 0),
        position_monitor: use_signal(|| Option::<Eval>::None),
    };
    let cleanup_state = state;
    let synced_provider_runtime = provider_runtime.clone();
    let synced_open_change: TooltipOpenChangeHandler = Rc::new(on_open_change);
    let effect_open_change = Rc::clone(&synced_open_change);
    let reset_state = state;
    let position_state = state;
    let effect_tooltip = tooltip.clone();
    let is_open = tooltip.is_open();
    let tooltip_root_id = tooltip.relationships().root_id().to_owned();
    let provider_active_tooltip_id = synced_provider_runtime
        .as_ref()
        .map(|runtime| runtime.active_tooltip_id())
        .flatten();

    use_effect(use_reactive!(|is_open, provider_active_tooltip_id| {
        if synced_provider_runtime.is_some() {
            let next_open = provider_active_tooltip_id.as_deref() == Some(tooltip_root_id.as_str());
            if next_open != is_open {
                effect_open_change(next_open);
            }
        }

        if !is_open {
            if *reset_state.content_hovered.peek() {
                let mut content_hovered = reset_state.content_hovered;
                content_hovered.set(false);
            }
            if *reset_state.pointer_down_inside.peek() {
                let mut pointer_down_inside = reset_state.pointer_down_inside;
                pointer_down_inside.set(false);
            }
        }

        sync_tooltip_positioning(
            &effect_tooltip,
            synced_provider_runtime.clone(),
            Rc::clone(&effect_open_change),
            position_state,
        );
    }));

    dioxus::core::use_drop(move || {
        advance_tooltip_token(cleanup_state.hover_transfer_token);
        advance_tooltip_token(cleanup_state.open_request_token);
        advance_tooltip_token(cleanup_state.position_loop_token);
        stop_tooltip_position_monitor(cleanup_state);
    });

    TooltipRuntime {
        tooltip,
        provider_runtime,
        on_open_change: synced_open_change,
        state,
    }
}

impl TooltipRuntime {
    pub fn tooltip(&self) -> &Tooltip {
        &self.tooltip
    }

    pub const fn is_open(&self) -> bool {
        self.tooltip.is_open()
    }

    pub fn data_state(&self) -> DataState {
        self.tooltip.data_state()
    }

    pub fn relationships(&self) -> &TooltipRelationships {
        self.tooltip.relationships()
    }

    pub fn lifecycle(&self) -> &TooltipLifecycle {
        self.tooltip.lifecycle()
    }

    pub fn provider(&self) -> Option<&TooltipProvider> {
        self.tooltip.provider()
    }

    pub fn provider_runtime(&self) -> Option<&TooltipProviderRuntime> {
        self.provider_runtime.as_ref()
    }

    pub fn root(&self) -> TooltipRootAttributes {
        self.tooltip.root()
    }

    pub fn trigger(&self) -> TooltipTriggerAttributes {
        self.tooltip.trigger()
    }

    pub fn portal(&self) -> TooltipPortalAttributes {
        self.tooltip.portal()
    }

    pub fn content(&self) -> TooltipContentAttributes {
        let mut content = self.tooltip.content();
        if let Some(placement) = self.placement() {
            content.data_side = placement.side().as_str();
            content.data_align = placement.align().as_str();
        }
        content
    }

    pub fn arrow(&self) -> TooltipArrowAttributes {
        let mut arrow = self.tooltip.arrow();
        if let Some(placement) = self.placement() {
            arrow.data_side = placement.side().as_str();
            arrow.data_align = placement.align().as_str();
        }
        arrow
    }

    pub fn geometry_vars(&self, anchor: Rect, content: Size) -> GeometryVars {
        self.tooltip.geometry_vars(anchor, content)
    }

    pub fn placement(&self) -> Option<FloatingPlacement> {
        self.state.placement.cloned()
    }

    pub fn mount_trigger(&self) -> impl FnMut(MountedEvent) + 'static {
        let runtime = self.clone();
        move |event| {
            let mut trigger_handle = runtime.state.trigger_handle;
            trigger_handle.set(Some(event.data()));
            runtime.refresh_live_placement();
        }
    }

    pub fn mount_content(&self) -> impl FnMut(MountedEvent) + 'static {
        let runtime = self.clone();
        move |event| {
            let mut content_handle = runtime.state.content_handle;
            content_handle.set(Some(event.data()));
            runtime.refresh_live_placement();
        }
    }

    pub fn escape_keydown(&self) -> impl FnMut(Event<KeyboardData>) + 'static {
        let runtime = self.clone();
        move |event| {
            if event.key().to_string() == "Escape" && runtime.dismiss_escape() {
                event.prevent_default();
                event.stop_propagation();
            }
        }
    }

    pub fn dismiss_escape(&self) -> bool {
        let stack = self.dismiss_stack();
        if self
            .lifecycle()
            .dismiss_layer()
            .should_dismiss_escape(&stack)
        {
            self.request_close();
            return true;
        }

        false
    }

    pub fn dismiss_outside_pointer(&self, target: Option<&str>) -> bool {
        let target = target.map(|target| target.to_owned());
        let stack = self.dismiss_stack();
        if self
            .lifecycle()
            .dismiss_layer()
            .should_dismiss_outside_pointer(target.as_ref(), &stack)
        {
            self.request_close();
            return true;
        }

        false
    }

    pub fn dismiss_outside_focus(&self, target: Option<&str>) -> bool {
        let target = target.map(|target| target.to_owned());
        let stack = self.dismiss_stack();
        if self
            .lifecycle()
            .dismiss_layer()
            .should_dismiss_outside_focus(target.as_ref(), &stack)
        {
            self.request_close();
            return true;
        }

        false
    }

    pub fn trigger_pointer_down(&self) -> impl FnMut(Event<PointerData>) + 'static {
        let runtime = self.clone();
        move |event| {
            event.stop_propagation();
            let mut pointer_down_inside = runtime.state.pointer_down_inside;
            pointer_down_inside.set(true);
        }
    }

    pub fn trigger_pointer_enter(&self) -> impl FnMut(Event<MouseData>) + 'static {
        let runtime = self.clone();
        move |_| {
            let mut trigger_hovered = runtime.state.trigger_hovered;
            trigger_hovered.set(true);
            runtime.cancel_hover_transfer();
            runtime.request_open(false);
        }
    }

    pub fn trigger_pointer_leave(&self) -> impl FnMut(Event<MouseData>) + 'static {
        let runtime = self.clone();
        move |_| {
            let mut trigger_hovered = runtime.state.trigger_hovered;
            trigger_hovered.set(false);

            if runtime.disable_hoverable_content() {
                runtime.request_close();
            } else {
                runtime.schedule_hover_transfer_close();
            }
        }
    }

    pub fn trigger_focus(&self) -> impl FnMut(Event<FocusData>) + 'static {
        let runtime = self.clone();
        move |_| {
            let opened_from_pointer = *runtime.state.pointer_down_inside.peek();
            let mut pointer_down_inside = runtime.state.pointer_down_inside;
            pointer_down_inside.set(false);

            if runtime.ignore_non_keyboard_focus() && opened_from_pointer {
                return;
            }

            runtime.request_open(true);
        }
    }

    pub fn trigger_blur(&self) -> impl FnMut(Event<FocusData>) + 'static {
        let runtime = self.clone();
        move |_| {
            let mut pointer_down_inside = runtime.state.pointer_down_inside;
            pointer_down_inside.set(false);

            if !*runtime.state.trigger_hovered.peek() && !*runtime.state.content_hovered.peek() {
                runtime.request_close();
            }
        }
    }

    pub fn trigger_click(&self) -> impl FnMut(Event<MouseData>) + 'static {
        let runtime = self.clone();
        move |_| {
            if runtime.close_on_trigger_click() && runtime.is_open() {
                runtime.request_close();
            }
        }
    }

    pub fn content_pointer_enter(&self) -> impl FnMut(Event<MouseData>) + 'static {
        let runtime = self.clone();
        move |_| {
            if runtime.disable_hoverable_content() {
                return;
            }

            let mut content_hovered = runtime.state.content_hovered;
            content_hovered.set(true);
            runtime.cancel_hover_transfer();
            runtime.request_open(true);
        }
    }

    pub fn content_pointer_leave(&self) -> impl FnMut(Event<MouseData>) + 'static {
        let runtime = self.clone();
        move |_| {
            let mut content_hovered = runtime.state.content_hovered;
            content_hovered.set(false);

            if !*runtime.state.trigger_hovered.peek() {
                runtime.request_close();
            }
        }
    }

    fn request_open(&self, immediate: bool) {
        if let Some(provider_runtime) = &self.provider_runtime {
            provider_runtime.request_open(self.relationships().root_id().to_owned(), immediate);
            return;
        }

        let request_token = advance_tooltip_token(self.state.open_request_token);
        if immediate || self.delay_duration_ms() == 0 {
            (self.on_open_change)(true);
            return;
        }

        let runtime = self.clone();
        spawn(async move {
            Delay::new(Duration::from_millis(runtime.delay_duration_ms())).await;
            if *runtime.state.open_request_token.peek() == request_token {
                (runtime.on_open_change)(true);
            }
        });
    }

    fn request_close(&self) {
        self.cancel_hover_transfer();
        if let Some(provider_runtime) = &self.provider_runtime {
            provider_runtime.request_close(self.relationships().root_id());
            return;
        }

        advance_tooltip_token(self.state.open_request_token);
        if self.is_open() {
            (self.on_open_change)(false);
        }
    }

    fn schedule_hover_transfer_close(&self) {
        let request_token = advance_tooltip_token(self.state.hover_transfer_token);
        let runtime = self.clone();

        spawn(async move {
            Delay::new(Duration::from_millis(TOOLTIP_HOVER_TRANSFER_GRACE_MS)).await;
            if *runtime.state.hover_transfer_token.peek() != request_token {
                return;
            }

            if !*runtime.state.trigger_hovered.peek() && !*runtime.state.content_hovered.peek() {
                runtime.request_close();
            }
        });
    }

    fn cancel_hover_transfer(&self) {
        advance_tooltip_token(self.state.hover_transfer_token);
    }

    fn delay_duration_ms(&self) -> u64 {
        self.provider()
            .map(TooltipProvider::delay_duration_ms)
            .unwrap_or(TooltipProvider::DEFAULT_DELAY_DURATION_MS)
    }

    fn disable_hoverable_content(&self) -> bool {
        self.provider()
            .map(TooltipProvider::disable_hoverable_content)
            .unwrap_or(false)
    }

    fn close_on_trigger_click(&self) -> bool {
        self.provider()
            .map(TooltipProvider::close_on_trigger_click)
            .unwrap_or(true)
    }

    fn ignore_non_keyboard_focus(&self) -> bool {
        self.provider()
            .map(TooltipProvider::ignore_non_keyboard_focus)
            .unwrap_or(false)
    }

    fn dismiss_stack(&self) -> Vec<String> {
        vec![self.relationships().content_id().to_owned()]
    }

    fn refresh_live_placement(&self) {
        if !self.is_open() {
            return;
        }

        let runtime = self.clone();
        spawn(async move {
            if let Err(error) = measure_tooltip_placement(runtime.tooltip(), runtime.state).await {
                eprintln!(
                    "monoxus tooltip runtime could not refresh placement for {}: {error}",
                    runtime.relationships().root_id(),
                );
            }
        });
    }
}

fn sync_tooltip_positioning(
    tooltip: &Tooltip,
    provider_runtime: Option<TooltipProviderRuntime>,
    on_open_change: TooltipOpenChangeHandler,
    state: TooltipRuntimeState,
) {
    stop_tooltip_position_monitor(state);

    if !tooltip.is_open() {
        advance_tooltip_token(state.position_loop_token);
        clear_tooltip_content_handle(state);
        clear_tooltip_placement(state);
        return;
    }

    let position_loop_token = advance_tooltip_token(state.position_loop_token);
    let tooltip = tooltip.clone();
    let provider_runtime = provider_runtime.clone();
    let on_open_change = Rc::clone(&on_open_change);
    let monitor = start_floating_auto_update_monitor(
        &[tooltip.relationships().trigger_id()],
        tooltip.relationships().content_id(),
    );
    let mut position_monitor = state.position_monitor;
    position_monitor.set(Some(monitor));

    spawn(async move {
        let mut monitor = monitor;

        if let Err(error) = measure_tooltip_placement(&tooltip, state).await {
            eprintln!(
                "monoxus tooltip runtime could not measure placement for {}: {error}",
                tooltip.relationships().root_id(),
            );
        }

        loop {
            if *state.position_loop_token.peek() != position_loop_token {
                break;
            }

            match recv_floating_auto_update_event(&mut monitor).await {
                Ok(FloatingAutoUpdateEvent::Scroll) => {
                    close_tooltip_from_scroll(&tooltip, provider_runtime.as_ref(), &on_open_change);
                    break;
                }
                Ok(FloatingAutoUpdateEvent::Update) => {}
                Ok(FloatingAutoUpdateEvent::Stopped) => break,
                Err(error) => {
                    if *state.position_loop_token.peek() == position_loop_token {
                        eprintln!(
                            "monoxus tooltip runtime auto-update monitor failed for {}: {error}",
                            tooltip.relationships().root_id(),
                        );
                    }
                    break;
                }
            }

            if *state.position_loop_token.peek() != position_loop_token {
                break;
            }

            if let Err(error) = measure_tooltip_placement(&tooltip, state).await {
                eprintln!(
                    "monoxus tooltip runtime could not measure placement for {}: {error}",
                    tooltip.relationships().root_id(),
                );
            }
        }
    });
}

fn close_tooltip_from_scroll(
    tooltip: &Tooltip,
    provider_runtime: Option<&TooltipProviderRuntime>,
    on_open_change: &TooltipOpenChangeHandler,
) {
    if let Some(provider_runtime) = provider_runtime {
        provider_runtime.request_close(tooltip.relationships().root_id());
        return;
    }

    on_open_change(false);
}

fn stop_tooltip_position_monitor(state: TooltipRuntimeState) {
    let Some(monitor) = state.position_monitor.with_peek(|monitor| *monitor) else {
        return;
    };

    let mut position_monitor = state.position_monitor;
    position_monitor.set(None);

    if let Err(error) = stop_floating_auto_update_monitor(monitor) {
        eprintln!("monoxus tooltip runtime could not stop auto-update monitor: {error}");
    }
}

async fn measure_tooltip_placement(
    tooltip: &Tooltip,
    state: TooltipRuntimeState,
) -> Result<(), String> {
    let Some(trigger_handle) = state.trigger_handle.with_peek(|handle| handle.clone()) else {
        return Ok(());
    };
    let Some(content_handle) = state.content_handle.with_peek(|handle| handle.clone()) else {
        return Ok(());
    };

    let anchor_rect = read_client_rect(trigger_handle, "trigger").await?;
    let content_rect = read_client_rect(content_handle, "content").await?;
    let content_size = Size::new(content_rect.width(), content_rect.height());
    let viewport_size = read_viewport_size().await?;
    let placement = tooltip.lifecycle().floating().position_with_available_size(
        anchor_rect,
        content_size,
        viewport_size,
    );

    let should_update = state
        .placement
        .with_peek(|current| current.as_ref() != Some(&placement));
    if should_update {
        let mut current = state.placement;
        current.set(Some(placement));
    }

    Ok(())
}

async fn read_client_rect(mounted: Rc<MountedData>, label: &str) -> Result<Rect, String> {
    let rect = mounted
        .get_client_rect()
        .await
        .map_err(|error| format!("{label} get_client_rect failed: {error}"))?;

    Ok(Rect::new(
        rect.origin.x as f32,
        rect.origin.y as f32,
        rect.width() as f32,
        rect.height() as f32,
    ))
}

async fn read_viewport_size() -> Result<Size, String> {
    let viewport: [f64; 2] = document::eval("return [window.innerWidth, window.innerHeight];")
        .join()
        .await
        .map_err(|error| format!("viewport query failed: {error}"))?;

    Ok(Size::new(viewport[0] as f32, viewport[1] as f32))
}

fn clear_tooltip_content_handle(state: TooltipRuntimeState) {
    if state.content_handle.with_peek(|handle| handle.is_some()) {
        let mut content_handle = state.content_handle;
        content_handle.set(None);
    }
}

fn clear_tooltip_placement(state: TooltipRuntimeState) {
    if state.placement.with_peek(|placement| placement.is_some()) {
        let mut placement = state.placement;
        placement.set(None);
    }
}

fn advance_tooltip_token(signal: Signal<u64>) -> u64 {
    let next = signal.with_peek(|value| value.saturating_add(1));
    let mut signal = signal;
    signal.set(next);
    next
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct TooltipRootAttributes {
    id: String,
    data_state: DataState,
}

impl TooltipRootAttributes {
    pub fn id(&self) -> &str {
        &self.id
    }

    pub fn data_state(&self) -> &DataState {
        &self.data_state
    }
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct TooltipTriggerAttributes {
    id: String,
    aria_describedby: Option<String>,
    provider_id: Option<String>,
    data_state: DataState,
    open_request: TooltipStateRequest,
    close_request: TooltipStateRequest,
}

impl TooltipTriggerAttributes {
    pub fn id(&self) -> &str {
        &self.id
    }

    pub fn aria_describedby(&self) -> Option<&str> {
        self.aria_describedby.as_deref()
    }

    pub fn provider_id(&self) -> Option<&str> {
        self.provider_id.as_deref()
    }

    pub fn data_state(&self) -> &DataState {
        &self.data_state
    }

    pub const fn open_request(&self) -> TooltipStateRequest {
        self.open_request
    }

    pub const fn close_request(&self) -> TooltipStateRequest {
        self.close_request
    }
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct TooltipPortalAttributes {
    host: PortalHost,
}

impl TooltipPortalAttributes {
    pub fn host(&self) -> &PortalHost {
        &self.host
    }
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct TooltipContentAttributes {
    id: String,
    role: &'static str,
    data_state: DataState,
    data_side: &'static str,
    data_align: &'static str,
    autofocus_suppressed: bool,
}

impl TooltipContentAttributes {
    pub fn id(&self) -> &str {
        &self.id
    }

    pub const fn role(&self) -> &'static str {
        self.role
    }

    pub fn data_state(&self) -> &DataState {
        &self.data_state
    }

    pub const fn data_side(&self) -> &'static str {
        self.data_side
    }

    pub const fn data_align(&self) -> &'static str {
        self.data_align
    }

    pub const fn autofocus_suppressed(&self) -> bool {
        self.autofocus_suppressed
    }
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct TooltipArrowAttributes {
    id: String,
    data_state: DataState,
    data_side: &'static str,
    data_align: &'static str,
}

impl TooltipArrowAttributes {
    pub fn id(&self) -> &str {
        &self.id
    }

    pub fn data_state(&self) -> &DataState {
        &self.data_state
    }

    pub const fn data_side(&self) -> &'static str {
        self.data_side
    }

    pub const fn data_align(&self) -> &'static str {
        self.data_align
    }
}

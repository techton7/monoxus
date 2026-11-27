use std::rc::Rc;

use dioxus::prelude::*;

pub use crate::foundation::compose::{
    compose_part_event_handlers, compose_part_refs, project_as_child,
};

use crate::foundation::{
    browser::{
        get_element_client_rect, is_element_focus_visible_by_id, is_element_focused_by_id,
        start_document_pointer_up_observer,
    },
    keyboard::{is_escape_key, is_ime_composing_key},
    overlay::{
        CancellableDelay, DismissCallbacks, DismissConfig, DismissEvent, DismissLayerHandle,
        FloatingHandle, FloatingMonitorSpec, FloatingPlacement, FloatingReadiness,
        FloatingScrollPolicy, GeometryVars, GraceCorridorHandle, GraceCorridorOutcome,
        GraceTransit, GraceTransitStatus, Point2D, PresenceHandle, Rect, Rect2D, Size, WatcherSlot,
        active_overlay_layers_snapshot, is_overlay_escape_consumed, is_topmost_overlay_layer,
        mark_overlay_escape_consumed, use_cancellable_delay, use_dismissable_layer,
        use_floating_layer, use_grace_corridor, use_presence, use_watcher_slot,
    },
    state::{DataState, sync_signal},
};

use super::{
    attrs::{
        TooltipArrowAttributes, TooltipContentAttributes, TooltipPortalAttributes,
        TooltipRootAttributes, TooltipTriggerAttributes,
    },
    relationships::TooltipRelationships,
    state::{Tooltip, TooltipLifecycle, TooltipProvider},
    types::{TOOLTIP_GEOMETRY_NAMESPACE, TOOLTIP_HOVER_TRANSFER_GRACE_MS},
};

const TOOLTIP_RADIX_COMPATIBILITY_PREFIX: &str = "radix-tooltip";
const TOOLTIP_RADIX_ANCHOR_LABEL: &str = "trigger";
const TOOLTIP_TRANSIT_INTENT_TIMEOUT_MS: u64 = 180;

type TooltipOpenChangeHandler = Rc<dyn Fn(bool)>;

#[derive(Clone, Copy)]
struct TooltipProviderRuntimeState {
    active_tooltip_id: Signal<Option<String>>,
    pending_open_tooltip_id: Signal<Option<String>>,
    open_delay: CancellableDelay,
    skip_delay: CancellableDelay,
    instant_phase: Signal<bool>,
    alive: CopyValue<bool>,
}

#[derive(Clone)]
pub struct TooltipProviderRuntime {
    provider: TooltipProvider,
    state: TooltipProviderRuntimeState,
}

pub fn use_tooltip_provider_runtime(provider: TooltipProvider) -> TooltipProviderRuntime {
    let active_tooltip_id = use_signal(|| None::<String>);
    let pending_open_tooltip_id = use_signal(|| None::<String>);
    let open_delay = use_cancellable_delay();
    let skip_delay = use_cancellable_delay();
    let instant_phase = use_signal(|| false);
    let mut alive = use_hook(|| CopyValue::new(true));

    use_drop(move || {
        if let Ok(mut flag) = alive.try_write() {
            *flag = false;
        }
        open_delay.cancel();
        skip_delay.cancel();
    });

    TooltipProviderRuntime {
        provider,
        state: TooltipProviderRuntimeState {
            active_tooltip_id,
            pending_open_tooltip_id,
            open_delay,
            skip_delay,
            instant_phase,
            alive,
        },
    }
}

impl TooltipProviderRuntime {
    pub fn provider(&self) -> &TooltipProvider {
        &self.provider
    }

    pub fn is_alive(&self) -> bool {
        self.state.alive.try_read().is_ok_and(|a| *a)
    }

    pub fn active_tooltip_id(&self) -> Option<String> {
        if self.is_alive() {
            self.state.active_tooltip_id.cloned()
        } else {
            None
        }
    }

    pub fn pending_open_tooltip_id(&self) -> Option<String> {
        if self.is_alive() {
            self.state.pending_open_tooltip_id.cloned()
        } else {
            None
        }
    }

    pub fn opens_instantly(&self) -> bool {
        if self.is_alive() {
            *self.state.instant_phase.read()
        } else {
            false
        }
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

        let mut pending = self.state.pending_open_tooltip_id;
        let _ = pending.try_write().map(|mut p| *p = Some(tooltip_id.clone()));

        let provider_runtime = self.clone();
        self.state
            .open_delay
            .schedule(self.provider.delay_duration_ms(), move || {
                if !provider_runtime.is_alive() {
                    return;
                }
                if provider_runtime
                    .state
                    .pending_open_tooltip_id
                    .try_peek()
                    .is_ok_and(|p| p.as_deref() == Some(tooltip_id.as_str()))
                {
                    provider_runtime.commit_open(tooltip_id);
                }
            });
    }

    pub fn request_close(&self, tooltip_id: &str) {
        if !self.is_alive() {
            return;
        }
        let mut active_tooltip_id = self.state.active_tooltip_id;
        let was_active = active_tooltip_id
            .try_peek()
            .is_ok_and(|active| active.as_deref() == Some(tooltip_id));
        let was_pending = self
            .state
            .pending_open_tooltip_id
            .try_peek()
            .is_ok_and(|pending| pending.as_deref() == Some(tooltip_id));

        if was_pending {
            self.clear_pending_open();
            self.state.open_delay.cancel();
        }

        if was_active {
            let _ = active_tooltip_id.try_write().map(|mut active| *active = None);
            self.start_skip_delay_window();
        }
    }

    fn commit_open(&self, tooltip_id: String) {
        if !self.is_alive() {
            return;
        }
        self.clear_pending_open();
        self.state.open_delay.cancel();
        self.state.skip_delay.cancel();
        let mut active_tooltip_id = self.state.active_tooltip_id;
        let _ = active_tooltip_id
            .try_write()
            .map(|mut active| *active = Some(tooltip_id));
        let mut instant_phase = self.state.instant_phase;
        let _ = instant_phase.try_write().map(|mut instant| *instant = true);
    }

    fn clear_pending_open(&self) {
        let mut pending = self.state.pending_open_tooltip_id;
        let _ = pending.try_write().map(|mut p| *p = None);
    }

    fn start_skip_delay_window(&self) {
        self.state.skip_delay.cancel();
        let skip_ms = self.provider.skip_delay_duration_ms();
        if skip_ms == 0 {
            let mut instant = self.state.instant_phase;
            let _ = instant.try_write().map(|mut i| *i = false);
            return;
        }

        let provider_runtime = self.clone();
        self.state.skip_delay.schedule(skip_ms, move || {
            if provider_runtime.is_alive() && provider_runtime.active_tooltip_id().is_none() {
                let mut instant = provider_runtime.state.instant_phase;
                let _ = instant.try_write().map(|mut i| *i = false);
            }
        });
    }
}

#[derive(Clone, Debug, PartialEq)]
pub struct TooltipGraceTransit {
    inner: GraceTransit,
    pub exit_point: Point2D,
    pub trigger_rect: Rect2D,
    pub content_rect: Rect2D,
    pub corridor: Vec<Point2D>,
}

impl TooltipGraceTransit {
    pub fn new(
        exit_point: Point2D,
        trigger_rect: Rect2D,
        content_rect: Rect2D,
        padding: Option<f64>,
    ) -> Self {
        let inner = GraceTransit::new(exit_point, trigger_rect, content_rect, padding, false);
        Self {
            corridor: inner.corridor.clone(),
            inner,
            exit_point,
            trigger_rect,
            content_rect,
        }
    }

    pub fn inspect_point(&self, point: Point2D) -> GraceTransitStatus {
        self.inner.inspect_point(point)
    }
}

#[derive(Clone, Copy)]
struct TooltipRuntimeState {
    live_tooltip: Signal<Tooltip>,
    trigger_handle: Signal<Option<Rc<MountedData>>>,
    content_handle: Signal<Option<Rc<MountedData>>>,
    presence: PresenceHandle,
    floating: FloatingHandle,
    #[allow(dead_code)]
    dismiss: DismissLayerHandle,
    grace: GraceCorridorHandle,
    hover_transfer_delay: CancellableDelay,
    standalone_open_delay: CancellableDelay,
    pointer_up_watcher: WatcherSlot,
    pointer_down_inside: CopyValue<bool>,
    trigger_hovered: CopyValue<bool>,
    content_hovered: CopyValue<bool>,
    pending_standalone_open: CopyValue<bool>,
    opened_via_focus: CopyValue<bool>,
    last_pointer_type: CopyValue<Option<String>>,
    registered_trigger_id: Signal<Option<String>>,
    content_id: Signal<Option<String>>,
    measurement_id: Signal<Option<String>>,
    collision_boundary: Signal<Option<String>>,
    collision_padding: Signal<Option<f32>>,
    alive: CopyValue<bool>,
}

#[derive(Clone)]
pub struct TooltipRuntime {
    tooltip: Tooltip,
    provider_runtime: Option<TooltipProviderRuntime>,
    on_open_change: CopyValue<TooltipOpenChangeHandler>,
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
    let synced_open_change: TooltipOpenChangeHandler = Rc::new(on_open_change);
    let mut on_open_change_slot = use_hook(|| CopyValue::new(Rc::clone(&synced_open_change)));
    on_open_change_slot.set(synced_open_change);

    let mut live_tooltip = use_signal(|| tooltip.clone());
    sync_signal(&mut live_tooltip, tooltip.clone());

    let trigger_handle = use_signal(|| None::<Rc<MountedData>>);
    let content_handle = use_signal(|| None::<Rc<MountedData>>);
    let pointer_down_inside = use_hook(|| CopyValue::new(false));
    let trigger_hovered = use_hook(|| CopyValue::new(false));
    let content_hovered = use_hook(|| CopyValue::new(false));
    let pending_standalone_open = use_hook(|| CopyValue::new(false));
    let opened_via_focus = use_hook(|| CopyValue::new(false));
    let last_pointer_type = use_hook(|| CopyValue::new(Option::<String>::None));
    let registered_trigger_id = use_signal(|| None::<String>);
    let content_id = use_signal(|| None::<String>);
    let measurement_id = use_signal(|| None::<String>);
    let collision_boundary = use_signal(|| None::<String>);
    let collision_padding = use_signal(|| None::<f32>);
    let mut alive = use_hook(|| CopyValue::new(true));

    let effective_trigger_id = registered_trigger_id
        .read()
        .clone()
        .unwrap_or_else(|| tooltip.relationships().trigger_id().to_owned());
    let effective_content_id = content_id
        .read()
        .clone()
        .unwrap_or_else(|| tooltip.relationships().content_id().to_owned());
    let effective_measurement_id = measurement_id.read().clone();
    let presence_target_id = effective_measurement_id
        .as_deref()
        .filter(|id| !id.is_empty())
        .unwrap_or(effective_content_id.as_str());

    let presence = use_presence(
        tooltip.is_open(),
        presence_target_id,
        tooltip.lifecycle().presence().retain_mount(),
        None,
    );

    let mut provider_slot = use_hook(|| CopyValue::new(provider_runtime.clone()));
    provider_slot.set(provider_runtime.clone());
    let root_id_for_close = tooltip.relationships().root_id().to_owned();
    let mut root_id_slot = use_hook(|| CopyValue::new(root_id_for_close.clone()));
    if root_id_slot
        .try_read()
        .is_ok_and(|cur| cur.as_str() != root_id_for_close.as_str())
    {
        root_id_slot.set(root_id_for_close.clone());
    }

    let on_scroll_close = use_hook(move || {
        EventHandler::new(move |()| {
            if let Ok(Some(prov)) = provider_slot.try_read().map(|g| g.clone()) {
                let rid = root_id_slot
                    .try_read()
                    .map(|g| g.clone())
                    .unwrap_or_default();
                prov.request_close(&rid);
                return;
            }
            if let Ok(cb) = on_open_change_slot.try_read().map(|g| Rc::clone(&*g)) {
                cb(false);
            }
        })
    });

    let floating_cfg = tooltip.lifecycle().floating();
    let mut floating_spec =
        FloatingMonitorSpec::new(effective_trigger_id.clone(), effective_content_id.clone());
    floating_spec.measurement_id = effective_measurement_id;
    floating_spec.arrow_id = Some(tooltip.relationships().arrow_id().to_owned());
    floating_spec.boundary_id = collision_boundary.read().clone();
    floating_spec.side = floating_cfg.side();
    floating_spec.align = floating_cfg.align();
    floating_spec.direction = floating_cfg.direction();
    floating_spec.side_offset = floating_cfg.side_offset();
    floating_spec.align_offset = floating_cfg.align_offset();
    floating_spec.avoid_collisions = true;
    floating_spec.collision_padding = (*collision_padding.read()).unwrap_or(0.0);
    floating_spec.arrow_padding = 4.0;
    floating_spec.hide_when_detached = floating_cfg.hide_when_detached();
    floating_spec.sticky = false;
    floating_spec.namespace = Some(TOOLTIP_GEOMETRY_NAMESPACE.to_owned());
    floating_spec.scroll_policy = FloatingScrollPolicy::CloseOnScrollUnlessGuarded;

    let floating = use_floating_layer(
        tooltip.is_open() && presence.should_render(),
        floating_spec,
        Some(on_scroll_close),
    );

    let grace = use_grace_corridor();
    let hover_transfer_delay = use_cancellable_delay();
    let standalone_open_delay = use_cancellable_delay();
    let pointer_up_watcher = use_watcher_slot();

    let on_dismiss_escape = use_hook(move || {
        EventHandler::new(move |_event: DismissEvent| {
            if !alive.try_read().is_ok_and(|a| *a) {
                return;
            }
            let mut ovf = opened_via_focus;
            let _ = ovf.try_write().map(|mut w| *w = false);
            hover_transfer_delay.cancel();
            grace.clear();
            if let Ok(Some(prov)) = provider_slot.try_read().map(|g| g.clone()) {
                let rid = root_id_slot
                    .try_read()
                    .map(|g| g.clone())
                    .unwrap_or_default();
                prov.request_close(&rid);
                return;
            }
            let mut pending = pending_standalone_open;
            let _ = pending.try_write().map(|mut w| *w = false);
            standalone_open_delay.cancel();
            if let Ok(cb) = on_open_change_slot.try_read().map(|g| Rc::clone(&*g)) {
                cb(false);
            }
        })
    });

    let dismiss_config = DismissConfig::new(effective_content_id)
        .with_boundaries(vec![effective_trigger_id])
        .with_modal(false)
        .with_listen_escape(true)
        .with_listen_pointer_down_outside(false)
        .with_listen_focus_outside(false);

    let dismiss = use_dismissable_layer(
        tooltip.is_open(),
        dismiss_config,
        DismissCallbacks {
            on_escape_key_down: None,
            on_pointer_down_outside: None,
            on_focus_outside: None,
            on_interact_outside: None,
            on_dismiss: Some(on_dismiss_escape),
        },
    );

    let state = TooltipRuntimeState {
        live_tooltip,
        trigger_handle,
        content_handle,
        presence,
        floating,
        dismiss,
        grace,
        hover_transfer_delay,
        standalone_open_delay,
        pointer_up_watcher,
        pointer_down_inside,
        trigger_hovered,
        content_hovered,
        pending_standalone_open,
        opened_via_focus,
        last_pointer_type,
        registered_trigger_id,
        content_id,
        measurement_id,
        collision_boundary,
        collision_padding,
        alive,
    };

    let provider_active_tooltip_id = provider_runtime
        .as_ref()
        .and_then(TooltipProviderRuntime::active_tooltip_id);
    let has_provider = provider_runtime.is_some();
    let is_open_now = tooltip.is_open();
    let should_render_now = presence.should_render();
    let disable_hoverable = tooltip
        .provider()
        .map(TooltipProvider::disable_hoverable_content)
        .unwrap_or(false);

    use_effect(use_reactive(
        (
            &is_open_now,
            &should_render_now,
            &provider_active_tooltip_id,
            &disable_hoverable,
        ),
        move |(open_now, render_now, active_id, disable_hover)| {
            if has_provider {
                let next_open = active_id.as_deref() == Some(root_id_for_close.as_str());
                if next_open != open_now
                    && let Ok(cb) = on_open_change_slot.try_read().map(|g| Rc::clone(&*g))
                {
                    cb(next_open);
                }
            }

            if !open_now || disable_hover {
                state.grace.clear();
            }

            if !open_now {
                let mut ch = state.content_hovered;
                let _ = ch.try_write().map(|mut w| *w = false);
                if !render_now {
                    let mut content_h = state.content_handle;
                    if content_h.try_peek().is_ok_and(|h| h.is_some())
                        && let Ok(mut w) = content_h.try_write()
                    {
                        *w = None;
                    }
                    state.floating.clear();
                }
            }
        },
    ));

    use_drop(move || {
        if let Ok(mut flag) = alive.try_write() {
            *flag = false;
        }
        hover_transfer_delay.cancel();
        standalone_open_delay.cancel();
        pointer_up_watcher.clear();
        grace.clear();
    });

    TooltipRuntime {
        tooltip,
        provider_runtime,
        on_open_change: on_open_change_slot,
        state,
    }
}

impl TooltipRuntime {
    pub fn is_alive(&self) -> bool {
        self.state.alive.try_read().is_ok_and(|a| *a)
    }

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
        let mut trigger = self.tooltip.trigger();
        trigger.aria_describedby = if self.is_open() {
            let content_id = self
                .state
                .content_id
                .read()
                .clone()
                .unwrap_or_else(|| self.relationships().content_id().to_string());
            Some(content_id)
        } else {
            None
        };
        trigger
    }

    pub fn portal(&self) -> TooltipPortalAttributes {
        self.tooltip.portal()
    }

    pub fn should_render_portal(&self) -> bool {
        self.state.presence.should_render()
    }

    pub fn should_render_content(&self) -> bool {
        self.state.presence.should_render()
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

    pub fn content_readiness(&self) -> FloatingReadiness {
        self.state.floating.readiness()
    }

    pub fn content_positioning_state(&self) -> &'static str {
        self.content_readiness().positioning_state()
    }

    pub fn content_css_variables(&self) -> Vec<(String, String)> {
        self.placement()
            .map(|placement| tooltip_content_css_variables(placement.geometry()))
            .unwrap_or_default()
    }

    pub fn content_css_custom_properties(&self) -> String {
        self.state.floating.css_custom_properties(Some((
            TOOLTIP_RADIX_COMPATIBILITY_PREFIX,
            TOOLTIP_RADIX_ANCHOR_LABEL,
        )))
    }

    pub fn wrapper_style(&self) -> String {
        self.state.floating.wrapper_style()
    }

    pub fn clear_content_z_index(&self) {
        self.state.floating.clear_z_index();
    }

    pub fn placement(&self) -> Option<FloatingPlacement> {
        self.state.floating.placement()
    }

    pub fn register_content_id(&self, id: impl Into<String>) {
        let id = id.into();
        let mut sig = self.state.content_id;
        sync_signal(&mut sig, Some(id.clone()));
        self.state.dismiss.rename_layer_id(&id);
    }

    pub fn unregister_content_id(&self, id: &str) {
        let mut sig = self.state.content_id;
        if sig.try_peek().is_ok_and(|c| c.as_deref() == Some(id))
            && let Ok(mut w) = sig.try_write()
        {
            *w = None;
        }
    }

    pub fn register_trigger_id(&self, id: impl Into<String>) {
        let mut sig = self.state.registered_trigger_id;
        sync_signal(&mut sig, Some(id.into()));
    }

    pub fn unregister_trigger_id(&self, id: &str) {
        let mut sig = self.state.registered_trigger_id;
        if sig.try_peek().is_ok_and(|c| c.as_deref() == Some(id))
            && let Ok(mut w) = sig.try_write()
        {
            *w = None;
        }
    }

    pub fn register_arrow_id(&self, id: impl Into<String>) {
        self.state.floating.register_arrow_id(id);
    }

    pub fn unregister_arrow_id(&self, id: &str) {
        self.state.floating.unregister_arrow_id(id);
    }

    pub fn effective_trigger_id(&self) -> String {
        self.state
            .registered_trigger_id
            .try_peek()
            .ok()
            .and_then(|id| id.clone())
            .unwrap_or_else(|| self.tooltip.relationships().trigger_id().to_owned())
    }

    pub(crate) fn resync_positioning_if_open(&self) {
        if self.is_live_open() {
            self.state.floating.refresh();
        }
    }

    pub fn set_collision_boundary(&self, boundary: Option<String>) {
        let mut sig = self.state.collision_boundary;
        sync_signal(&mut sig, boundary);
    }

    pub fn set_collision_padding(&self, padding: Option<f32>) {
        let mut sig = self.state.collision_padding;
        sync_signal(&mut sig, padding);
    }

    pub fn set_measurement_id(&self, id: Option<String>) {
        let mut sig = self.state.measurement_id;
        sync_signal(&mut sig, id);
    }

    pub fn trigger_has_focus(&self) -> bool {
        let has_focus = self
            .state
            .trigger_handle
            .try_peek()
            .ok()
            .and_then(|handle| handle.clone())
            .map(oxidase::Element::from_mounted)
            .filter(|el| el.tag_name().is_some())
            .map(|el| el.has_focus());

        if let Some(focus) = has_focus {
            return focus;
        }

        let trigger_id = self.effective_trigger_id();
        is_element_focused_by_id(&trigger_id)
    }

    pub fn has_active_grace_monitor(&self) -> bool {
        self.state.grace.is_active()
    }

    pub fn stop_grace_monitor(&self) {
        self.state.grace.clear();
    }

    pub(crate) fn start_hover_grace_corridor(
        &self,
        exit_point: Point2D,
        from_trigger: bool,
    ) -> bool {
        if self.disable_hoverable_content() || !self.is_live_open() {
            return false;
        }

        let trigger_id = self.effective_trigger_id();
        let target_content_id = self
            .state
            .measurement_id
            .try_peek()
            .ok()
            .and_then(|id| id.clone())
            .filter(|id| !id.is_empty())
            .unwrap_or_else(|| self.effective_content_id());

        let (Some(trigger_rect), Some(content_rect)) = (
            get_element_client_rect(&trigger_id),
            get_element_client_rect(&target_content_id),
        ) else {
            return false;
        };

        let (from_rect, to_rect) = if from_trigger {
            (trigger_rect, content_rect)
        } else {
            (content_rect, trigger_rect)
        };

        let transit = GraceTransit::new(exit_point, from_rect, to_rect, Some(5.0), false);
        let runtime = self.clone();
        self.state.grace.start(
            transit,
            Some(TOOLTIP_TRANSIT_INTENT_TIMEOUT_MS),
            move |outcome| {
                if !runtime.is_alive() {
                    return;
                }
                match outcome {
                    GraceCorridorOutcome::InTransit(_) => {
                        runtime.cancel_hover_transfer();
                    }
                    GraceCorridorOutcome::ReachedTarget(_) => {
                        runtime.cancel_hover_transfer();
                    }
                    GraceCorridorOutcome::ExitedCorridor(_) | GraceCorridorOutcome::Stalled => {
                        runtime.request_close();
                    }
                }
            },
        );
        self.schedule_hover_transfer_close();
        true
    }

    pub fn mount_trigger(&self) -> impl FnMut(MountedEvent) + 'static {
        let runtime = self.clone();
        move |event| {
            let mut trigger_handle = runtime.state.trigger_handle;
            let _ = trigger_handle.try_write().map(|mut w| *w = Some(event.data()));
            runtime.state.floating.capture_anchor(event.data());
        }
    }

    pub fn mount_content(&self) -> impl FnMut(MountedEvent) + 'static {
        let runtime = self.clone();
        move |event| {
            let mut content_handle = runtime.state.content_handle;
            let _ = content_handle.try_write().map(|mut w| *w = Some(event.data()));
            runtime.state.floating.capture_content(event.data());
        }
    }

    pub fn escape_keydown(&self) -> impl FnMut(Event<KeyboardData>) + 'static {
        let runtime = self.clone();
        move |event| {
            if is_ime_composing_key(&event) {
                return;
            }
            if is_escape_key(&event.key()) && runtime.dismiss_escape() {
                event.prevent_default();
                event.stop_propagation();
            }
        }
    }

    pub fn dismiss_escape(&self) -> bool {
        if is_overlay_escape_consumed() {
            return false;
        }
        let active_cid = self.effective_content_id();
        if !is_topmost_overlay_layer(&active_cid) {
            return false;
        }
        let stack = self.dismiss_stack();
        let should_dismiss = self.state.live_tooltip.with_peek(|t| {
            let mut layer = t.lifecycle().dismiss_layer().clone();
            layer.set_id(active_cid.clone());
            layer.should_dismiss_escape(&stack)
        }) || {
            let mut layer = self.lifecycle().dismiss_layer().clone();
            layer.set_id(active_cid);
            layer.should_dismiss_escape(&stack)
        };
        if should_dismiss {
            mark_overlay_escape_consumed();
            self.request_close();
            return true;
        }
        false
    }

    pub fn dismiss_outside_pointer(&self, target: Option<&str>) -> bool {
        let target_owned = target.map(str::to_owned);
        let stack = self.dismiss_stack();
        let active_cid = self.effective_content_id();
        let should_dismiss = self.state.live_tooltip.with_peek(|t| {
            let mut layer = t.lifecycle().dismiss_layer().clone();
            layer.set_id(active_cid.clone());
            layer.should_dismiss_outside_pointer(target_owned.as_ref(), &stack)
        }) || {
            let mut layer = self.lifecycle().dismiss_layer().clone();
            layer.set_id(active_cid);
            layer.should_dismiss_outside_pointer(target_owned.as_ref(), &stack)
        };
        if should_dismiss {
            self.request_close();
            return true;
        }
        false
    }

    pub fn dismiss_outside_focus(&self, target: Option<&str>) -> bool {
        let target_owned = target.map(str::to_owned);
        let stack = self.dismiss_stack();
        let active_cid = self.effective_content_id();
        let should_dismiss = self.state.live_tooltip.with_peek(|t| {
            let mut layer = t.lifecycle().dismiss_layer().clone();
            layer.set_id(active_cid.clone());
            layer.should_dismiss_outside_focus(target_owned.as_ref(), &stack)
        }) || {
            let mut layer = self.lifecycle().dismiss_layer().clone();
            layer.set_id(active_cid);
            layer.should_dismiss_outside_focus(target_owned.as_ref(), &stack)
        };
        if should_dismiss {
            self.request_close();
            return true;
        }
        false
    }

    pub fn trigger_pointer_down(&self) -> impl FnMut(Event<PointerData>) + 'static {
        let runtime = self.clone();
        move |event| {
            if runtime.close_on_trigger_click() {
                runtime.request_close();
            }
            let mut pdi = runtime.state.pointer_down_inside;
            let _ = pdi.try_write().map(|mut w| *w = true);
            let mut lpt = runtime.state.last_pointer_type;
            let _ = lpt
                .try_write()
                .map(|mut w| *w = Some(event.data().pointer_type().to_string()));
            runtime.arm_pointer_up_reset();
        }
    }

    pub fn trigger_pointer_cancel(&self) -> impl FnMut(Event<PointerData>) + 'static {
        let runtime = self.clone();
        move |_| {
            runtime.clear_pointer_down_inside();
            runtime.state.pointer_up_watcher.clear();
        }
    }

    pub fn trigger_pointer_enter(&self) -> impl FnMut(Event<MouseData>) + 'static {
        let runtime = self.clone();
        move |_| {
            let mut th = runtime.state.trigger_hovered;
            let _ = th.try_write().map(|mut w| *w = true);
            let mut lpt = runtime.state.last_pointer_type;
            if lpt.try_read().ok().and_then(|p| p.clone()).as_deref() != Some("touch") {
                let _ = lpt.try_write().map(|mut w| *w = None);
            }
            runtime.cancel_hover_transfer();
            runtime.stop_grace_monitor();
            runtime.request_open(false);
        }
    }

    pub fn trigger_pointer_leave(&self) -> impl FnMut(Event<MouseData>) + 'static {
        let runtime = self.clone();
        move |event| {
            let mut th = runtime.state.trigger_hovered;
            let _ = th.try_write().map(|mut w| *w = false);

            if runtime.state.opened_via_focus.try_read().is_ok_and(|f| *f) {
                return;
            }

            if runtime.disable_hoverable_content() {
                runtime.request_close();
            } else {
                let coords = event.client_coordinates();
                let exit_point = Point2D::new(coords.x, coords.y);
                if !runtime.start_hover_grace_corridor(exit_point, true)
                    && !runtime.has_active_grace_monitor()
                {
                    runtime.schedule_hover_transfer_close();
                }
            }
        }
    }

    pub fn trigger_focus(&self) -> impl FnMut(Event<FocusData>) + 'static {
        let runtime = self.clone();
        move |_| {
            if runtime.state.pointer_down_inside.try_read().is_ok_and(|d| *d) {
                return;
            }

            let is_touch = runtime
                .state
                .last_pointer_type
                .try_read()
                .ok()
                .and_then(|p| p.clone())
                .as_deref()
                == Some("touch");
            if is_touch {
                return;
            }

            if runtime.ignore_non_keyboard_focus() {
                let focus_visible = runtime
                    .state
                    .trigger_handle
                    .try_peek()
                    .ok()
                    .and_then(|h| h.clone())
                    .map(oxidase::Element::from_mounted)
                    .filter(|el| el.tag_name().is_some())
                    .map(|el| el.matches_focus_visible())
                    .or_else(|| {
                        let trigger_id = runtime.effective_trigger_id();
                        is_element_focus_visible_by_id(&trigger_id)
                    });
                if focus_visible == Some(false) {
                    return;
                }
            }

            let mut ovf = runtime.state.opened_via_focus;
            let _ = ovf.try_write().map(|mut w| *w = true);
            runtime.cancel_hover_transfer();
            runtime.stop_grace_monitor();
            runtime.state.floating.arm_focus_scroll_guard();
            runtime.request_open(true);
        }
    }

    pub fn trigger_blur(&self) -> impl FnMut(Event<FocusData>) + 'static {
        let runtime = self.clone();
        move |_| {
            runtime.clear_pointer_down_inside();
            runtime.state.pointer_up_watcher.clear();
            let mut lpt = runtime.state.last_pointer_type;
            let _ = lpt.try_write().map(|mut w| *w = None);
            let mut ovf = runtime.state.opened_via_focus;
            let _ = ovf.try_write().map(|mut w| *w = false);

            let trigger_hovered = runtime.state.trigger_hovered.try_read().is_ok_and(|h| *h);
            let content_hovered = runtime.state.content_hovered.try_read().is_ok_and(|h| *h);
            if !trigger_hovered && !content_hovered {
                runtime.request_close();
            }
        }
    }

    pub fn trigger_click(&self) -> impl FnMut(Event<MouseData>) + 'static {
        let runtime = self.clone();
        move |_| {
            runtime.clear_pointer_down_inside();
            if runtime.close_on_trigger_click() {
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
            let mut ch = runtime.state.content_hovered;
            let _ = ch.try_write().map(|mut w| *w = true);
            runtime.cancel_hover_transfer();
            runtime.stop_grace_monitor();
            runtime.request_open(true);
        }
    }

    pub fn content_pointer_leave(&self) -> impl FnMut(Event<MouseData>) + 'static {
        let runtime = self.clone();
        move |event| {
            let mut ch = runtime.state.content_hovered;
            let _ = ch.try_write().map(|mut w| *w = false);

            if runtime.state.opened_via_focus.try_read().is_ok_and(|f| *f) {
                return;
            }

            if runtime.disable_hoverable_content() {
                runtime.request_close();
            } else {
                let coords = event.client_coordinates();
                let exit_point = Point2D::new(coords.x, coords.y);
                if !runtime.start_hover_grace_corridor(exit_point, false) {
                    let trigger_hovered =
                        runtime.state.trigger_hovered.try_read().is_ok_and(|h| *h);
                    if !runtime.has_active_grace_monitor() && !trigger_hovered {
                        runtime.schedule_hover_transfer_close();
                    }
                }
            }
        }
    }

    fn request_open(&self, immediate: bool) {
        if let Some(provider_runtime) = &self.provider_runtime {
            provider_runtime.request_open(self.relationships().root_id().to_owned(), immediate);
            return;
        }

        let mut pending = self.state.pending_standalone_open;
        let _ = pending.try_write().map(|mut w| *w = true);

        let delay_ms = self.delay_duration_ms();
        if immediate || delay_ms == 0 {
            self.state.standalone_open_delay.cancel();
            if let Ok(cb) = self.on_open_change.try_read().map(|g| Rc::clone(&*g)) {
                cb(true);
            }
            return;
        }

        let runtime = self.clone();
        self.state.standalone_open_delay.schedule(delay_ms, move || {
            if runtime.is_alive()
                && let Ok(cb) = runtime.on_open_change.try_read().map(|g| Rc::clone(&*g))
            {
                cb(true);
            }
        });
    }

    fn request_close(&self) {
        if !self.is_alive() {
            return;
        }
        let mut ovf = self.state.opened_via_focus;
        let _ = ovf.try_write().map(|mut w| *w = false);
        self.cancel_hover_transfer();
        self.stop_grace_monitor();
        if let Some(provider_runtime) = &self.provider_runtime {
            provider_runtime.request_close(self.relationships().root_id());
            return;
        }

        let was_pending = self
            .state
            .pending_standalone_open
            .try_read()
            .is_ok_and(|p| *p);
        let mut pending = self.state.pending_standalone_open;
        let _ = pending.try_write().map(|mut w| *w = false);
        self.state.standalone_open_delay.cancel();
        if (self.is_live_open() || was_pending)
            && let Ok(cb) = self.on_open_change.try_read().map(|g| Rc::clone(&*g))
        {
            cb(false);
        }
    }

    fn is_live_open(&self) -> bool {
        if let Some(provider_runtime) = &self.provider_runtime
            && provider_runtime.active_tooltip_id().as_deref()
                == Some(self.relationships().root_id())
        {
            return true;
        }
        self.state
            .live_tooltip
            .try_peek()
            .map(|t| t.is_open())
            .unwrap_or_else(|_| self.is_open())
    }

    fn schedule_hover_transfer_close(&self) {
        let runtime = self.clone();
        self.state
            .hover_transfer_delay
            .schedule(TOOLTIP_HOVER_TRANSFER_GRACE_MS, move || {
                if !runtime.is_alive() {
                    return;
                }
                let trigger_hovered = runtime.state.trigger_hovered.try_read().is_ok_and(|h| *h);
                let content_hovered = runtime.state.content_hovered.try_read().is_ok_and(|h| *h);
                let opened_via_focus =
                    runtime.state.opened_via_focus.try_read().is_ok_and(|o| *o);
                if !trigger_hovered && !content_hovered && !opened_via_focus {
                    runtime.request_close();
                }
            });
    }

    fn cancel_hover_transfer(&self) {
        self.state.hover_transfer_delay.cancel();
    }

    fn clear_pointer_down_inside(&self) {
        if !self.is_alive() {
            return;
        }
        let mut pdi = self.state.pointer_down_inside;
        let _ = pdi.try_write().map(|mut w| *w = false);
    }

    fn arm_pointer_up_reset(&self) {
        let runtime = self.clone();
        self.state.pointer_up_watcher.arm_with(move |expected_gen| {
            let rt_cb = runtime.clone();
            start_document_pointer_up_observer(move || {
                if !rt_cb.is_alive()
                    || rt_cb.state.pointer_up_watcher.generation() != expected_gen
                {
                    return;
                }
                rt_cb.clear_pointer_down_inside();
                rt_cb.state.pointer_up_watcher.clear_if_generation(expected_gen);
            })
        });
    }

    fn delay_duration_ms(&self) -> u64 {
        self.state
            .live_tooltip
            .try_peek()
            .map(|t| {
                t.provider()
                    .map(TooltipProvider::delay_duration_ms)
                    .unwrap_or(TooltipProvider::DEFAULT_DELAY_DURATION_MS)
            })
            .unwrap_or(TooltipProvider::DEFAULT_DELAY_DURATION_MS)
    }

    fn disable_hoverable_content(&self) -> bool {
        self.state.live_tooltip.with_peek(|t| {
            t.provider()
                .map(TooltipProvider::disable_hoverable_content)
                .unwrap_or(false)
        })
    }

    fn close_on_trigger_click(&self) -> bool {
        self.state.live_tooltip.with_peek(|t| {
            t.provider()
                .map(TooltipProvider::close_on_trigger_click)
                .unwrap_or(true)
        })
    }

    fn ignore_non_keyboard_focus(&self) -> bool {
        self.state.live_tooltip.with_peek(|t| {
            t.provider()
                .map(TooltipProvider::ignore_non_keyboard_focus)
                .unwrap_or(true)
        })
    }

    fn effective_content_id(&self) -> String {
        self.state
            .content_id
            .try_peek()
            .ok()
            .and_then(|id| id.clone())
            .unwrap_or_else(|| self.relationships().content_id().to_owned())
    }

    fn dismiss_stack(&self) -> Vec<String> {
        let active = active_overlay_layers_snapshot();
        if active.is_empty() {
            vec![self.effective_content_id()]
        } else {
            active
        }
    }
}

fn tooltip_content_css_variables(geometry: &GeometryVars) -> Vec<(String, String)> {
    geometry
        .css_iter()
        .chain(geometry.compatibility_alias_iter(
            TOOLTIP_RADIX_COMPATIBILITY_PREFIX,
            TOOLTIP_RADIX_ANCHOR_LABEL,
        ))
        .collect()
}

#[cfg(test)]
mod tests {
    use super::tooltip_content_css_variables;
    use crate::foundation::overlay::{
        FloatingReadiness, GeometryVars, Presence, PresenceController, PresenceState,
        serialize_css_custom_properties,
    };

    #[test]
    fn phase_3_7_step_4_tooltip_presence_lane_retains_content_until_close_completion() {
        let presence = Presence::new(true).with_retained_mount(true);
        let mut controller = PresenceController::new(presence.desired_present())
            .with_retained_mount(presence.retain_mount());

        let close = controller.sync(false);
        let cycle_id = close.started_close_cycle().unwrap();

        assert_eq!(close.state(), PresenceState::Suspended);
        assert!(controller.should_render());

        assert!(controller.complete_close_cycle(cycle_id));
        assert!(!controller.should_render());
    }

    #[test]
    fn tooltip_content_css_variables_include_monoxus_and_radix_compatibility_names() {
        let default_geometry = GeometryVars::new(
            "tooltip", 24.0, 44.0, 26.0, 0.0, 120.0, 80.0, 40.0, 16.0, 30.0, 12.0,
        );
        assert_eq!(
            default_geometry.css_get("--ui-tooltip-floating-x"),
            Some("24px".to_string())
        );

        let geometry = default_geometry.with_css_prefix("monoxus");
        let variables = tooltip_content_css_variables(&geometry);
        let serialized = serialize_css_custom_properties(&variables);

        assert_eq!(
            variables,
            vec![
                (
                    "--monoxus-tooltip-floating-x".to_string(),
                    "24px".to_string()
                ),
                (
                    "--monoxus-tooltip-floating-y".to_string(),
                    "44px".to_string()
                ),
                (
                    "--monoxus-tooltip-transform-origin-x".to_string(),
                    "26px".to_string(),
                ),
                (
                    "--monoxus-tooltip-transform-origin-y".to_string(),
                    "0px".to_string(),
                ),
                (
                    "--monoxus-tooltip-available-width".to_string(),
                    "120px".to_string(),
                ),
                (
                    "--monoxus-tooltip-available-height".to_string(),
                    "80px".to_string(),
                ),
                (
                    "--monoxus-tooltip-anchor-width".to_string(),
                    "40px".to_string(),
                ),
                (
                    "--monoxus-tooltip-anchor-height".to_string(),
                    "16px".to_string(),
                ),
                (
                    "--monoxus-tooltip-content-width".to_string(),
                    "30px".to_string(),
                ),
                (
                    "--monoxus-tooltip-content-height".to_string(),
                    "12px".to_string(),
                ),
                (
                    "--radix-tooltip-content-transform-origin".to_string(),
                    "26px 0px".to_string(),
                ),
                (
                    "--radix-tooltip-content-available-width".to_string(),
                    "120px".to_string(),
                ),
                (
                    "--radix-tooltip-content-available-height".to_string(),
                    "80px".to_string(),
                ),
                (
                    "--radix-tooltip-trigger-width".to_string(),
                    "40px".to_string()
                ),
                (
                    "--radix-tooltip-trigger-height".to_string(),
                    "16px".to_string(),
                ),
            ],
        );
        assert!(serialized.contains("--monoxus-tooltip-floating-x: 24px;"));
        assert!(serialized.contains("--radix-tooltip-content-transform-origin: 26px 0px;"));
        assert!(serialized.contains("--radix-tooltip-trigger-height: 16px;"));
    }

    #[test]
    fn tooltip_readiness_and_positioning_state_contract() {
        let initial = FloatingReadiness::default();
        assert_eq!(initial, FloatingReadiness::Measuring);
        assert_eq!(initial.positioning_state(), "unpositioned");

        let ready = FloatingReadiness::Ready;
        assert_eq!(ready.positioning_state(), "positioned");
    }

    #[test]
    fn test_tooltip_grace_transit_vertical_corridor() {
        use super::{GraceTransitStatus, TooltipGraceTransit};
        use crate::foundation::overlay::{Point2D, Rect2D};

        let trigger = Rect2D::new(100.0, 50.0, 50.0, 20.0);
        let content = Rect2D::new(80.0, 100.0, 90.0, 40.0);
        let exit_point = Point2D::new(125.0, 70.0);

        let transit = TooltipGraceTransit::new(exit_point, trigger, content, Some(5.0));
        assert!(transit.corridor.len() >= 3);

        assert_eq!(
            transit.inspect_point(Point2D::new(125.0, 85.0)),
            GraceTransitStatus::InTransit
        );
        assert_eq!(
            transit.inspect_point(Point2D::new(125.0, 115.0)),
            GraceTransitStatus::ReachedTarget
        );
        assert_eq!(
            transit.inspect_point(Point2D::new(125.0, 60.0)),
            GraceTransitStatus::InTransit
        );
        assert_eq!(
            transit.inspect_point(Point2D::new(125.0, 70.0)),
            GraceTransitStatus::InTransit
        );
        assert_eq!(
            transit.inspect_point(Point2D::new(0.0, 85.0)),
            GraceTransitStatus::ExitedCorridor
        );
        assert_eq!(
            transit.inspect_point(Point2D::new(300.0, 85.0)),
            GraceTransitStatus::ExitedCorridor
        );
        assert_eq!(
            transit.inspect_point(Point2D::new(125.0, 10.0)),
            GraceTransitStatus::ExitedCorridor
        );
    }

    #[test]
    fn test_tooltip_grace_transit_reverse_and_lateral() {
        use super::{GraceTransitStatus, TooltipGraceTransit};
        use crate::foundation::overlay::{Point2D, Rect2D};

        let trigger = Rect2D::new(100.0, 50.0, 50.0, 20.0);
        let content = Rect2D::new(80.0, 100.0, 90.0, 40.0);
        let exit_point = Point2D::new(125.0, 100.0);

        let transit = TooltipGraceTransit::new(exit_point, content, trigger, Some(5.0));
        assert_eq!(
            transit.inspect_point(Point2D::new(125.0, 85.0)),
            GraceTransitStatus::InTransit
        );
        assert_eq!(
            transit.inspect_point(Point2D::new(125.0, 100.0)),
            GraceTransitStatus::InTransit
        );
        assert_eq!(
            transit.inspect_point(Point2D::new(125.0, 60.0)),
            GraceTransitStatus::ReachedTarget
        );
        assert_eq!(
            transit.inspect_point(Point2D::new(10.0, 85.0)),
            GraceTransitStatus::ExitedCorridor
        );

        let trigger_lat = Rect2D::new(50.0, 100.0, 30.0, 30.0);
        let content_lat = Rect2D::new(120.0, 90.0, 60.0, 50.0);
        let exit_lat = Point2D::new(80.0, 115.0);

        let transit_lat = TooltipGraceTransit::new(exit_lat, trigger_lat, content_lat, Some(5.0));
        assert_eq!(
            transit_lat.inspect_point(Point2D::new(100.0, 115.0)),
            GraceTransitStatus::InTransit
        );
        assert_eq!(
            transit_lat.inspect_point(Point2D::new(140.0, 115.0)),
            GraceTransitStatus::ReachedTarget
        );
        assert_eq!(
            transit_lat.inspect_point(Point2D::new(100.0, 0.0)),
            GraceTransitStatus::ExitedCorridor
        );
    }

    #[test]
    fn test_tooltip_grace_guard_lifecycle() {
        use crate::foundation::browser::DocumentPointerStreamGuard;

        let mut guard = DocumentPointerStreamGuard::noop();
        guard.stop();
        drop(guard);
    }
}

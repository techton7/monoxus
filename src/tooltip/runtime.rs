use std::{rc::Rc, time::Duration};

use dioxus::{document::Eval, prelude::*};
use futures_timer::Delay;

pub use crate::foundation::compose::{
    compose_part_event_handlers, compose_part_refs, project_as_child,
};

use crate::foundation::{
    browser::{
        FloatingAutoUpdateEvent, PresenceMonitorEvent, recv_floating_auto_update_event,
        recv_presence_monitor_event, start_floating_auto_update_monitor, start_presence_monitor,
        stop_floating_auto_update_monitor, stop_presence_monitor,
    },
    overlay::{
        FloatingPlacement, GeometryVars, Presence, PresenceCloseCycleId, PresenceController,
        PresenceControllerUpdate, Rect, Size,
    },
    state::DataState,
};

use super::{
    attrs::{
        TooltipArrowAttributes, TooltipContentAttributes, TooltipPortalAttributes,
        TooltipRootAttributes, TooltipTriggerAttributes,
    },
    relationships::TooltipRelationships,
    state::{Tooltip, TooltipLifecycle, TooltipProvider},
    types::TOOLTIP_HOVER_TRANSFER_GRACE_MS,
};

type TooltipOpenChangeHandler = Rc<dyn Fn(bool)>;

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
struct TooltipContentPresenceLane {
    content: PresenceController,
}

impl TooltipContentPresenceLane {
    fn new(presence: &Presence) -> Self {
        Self {
            content: PresenceController::new(presence.desired_present())
                .with_retained_mount(presence.retain_mount()),
        }
    }

    fn sync(&mut self, desired_present: bool) -> PresenceControllerUpdate {
        self.content.sync(desired_present)
    }

    const fn should_render_portal(&self) -> bool {
        self.should_render_content()
    }

    const fn should_render_content(&self) -> bool {
        self.content.should_render()
    }

    const fn should_track_live_placement(&self, desired_present: bool) -> bool {
        desired_present && self.should_render_content()
    }

    const fn should_clear_positioning(&self, desired_present: bool) -> bool {
        !desired_present && !self.should_render_content()
    }

    fn complete_close_cycle(&mut self, cycle_id: PresenceCloseCycleId) -> bool {
        self.content.complete_close_cycle(cycle_id)
    }
}

#[derive(Clone, Copy)]
struct RetainedRootPresenceMonitorState {
    monitor: Signal<Option<Eval>>,
    cycle_id: Signal<Option<PresenceCloseCycleId>>,
}

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
    presence_lane: Signal<TooltipContentPresenceLane>,
    presence_monitor: RetainedRootPresenceMonitorState,
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
        presence_lane: use_signal(|| {
            TooltipContentPresenceLane::new(tooltip.lifecycle().presence())
        }),
        presence_monitor: RetainedRootPresenceMonitorState {
            monitor: use_signal(|| Option::<Eval>::None),
            cycle_id: use_signal(|| None),
        },
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
    let cleanup_tooltip = tooltip.clone();
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
        stop_tooltip_presence_monitor(cleanup_state, cleanup_tooltip.relationships().content_id());
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

    pub fn should_render_portal(&self) -> bool {
        self.state
            .presence_lane
            .with_peek(|lane| lane.should_render_portal())
    }

    pub fn should_render_content(&self) -> bool {
        self.state
            .presence_lane
            .with_peek(|lane| lane.should_render_content())
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
    sync_tooltip_presence(tooltip, state);
    stop_tooltip_position_monitor(state);

    let should_track_live_placement = state
        .presence_lane
        .with_peek(|lane| lane.should_track_live_placement(tooltip.is_open()));
    let should_clear_positioning = state
        .presence_lane
        .with_peek(|lane| lane.should_clear_positioning(tooltip.is_open()));

    if should_clear_positioning {
        advance_tooltip_token(state.position_loop_token);
        clear_tooltip_retained_content_state(state);
        return;
    }

    if !should_track_live_placement {
        advance_tooltip_token(state.position_loop_token);
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
    let viewport = crate::foundation::browser::get_viewport_size().await?;
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

fn clear_tooltip_retained_content_state(state: TooltipRuntimeState) {
    clear_tooltip_content_handle(state);
    clear_tooltip_placement(state);
}

fn advance_tooltip_token(signal: Signal<u64>) -> u64 {
    let next = signal.with_peek(|value| value.saturating_add(1));
    let mut signal = signal;
    signal.set(next);
    next
}

fn sync_tooltip_presence(tooltip: &Tooltip, mut state: TooltipRuntimeState) {
    let update = state
        .presence_lane
        .with_mut(|lane| lane.sync(tooltip.is_open()));

    if update.invalidated_close_cycle().is_some() || !update.should_render() {
        stop_tooltip_presence_monitor(state, tooltip.relationships().content_id());
    }

    if let Some(cycle_id) = update.started_close_cycle() {
        start_tooltip_presence_monitor(tooltip, state, cycle_id);
    }
}

fn start_tooltip_presence_monitor(
    tooltip: &Tooltip,
    state: TooltipRuntimeState,
    cycle_id: PresenceCloseCycleId,
) {
    stop_tooltip_presence_monitor(state, tooltip.relationships().content_id());

    let content_id = tooltip.relationships().content_id().to_owned();
    let monitor = start_presence_monitor(content_id.as_str(), cycle_id);
    let mut active_monitor = state.presence_monitor.monitor;
    active_monitor.set(Some(monitor));
    let mut active_cycle_id = state.presence_monitor.cycle_id;
    active_cycle_id.set(Some(cycle_id));

    spawn(async move {
        let mut monitor = monitor;

        loop {
            if state
                .presence_monitor
                .cycle_id
                .with_peek(|current| *current != Some(cycle_id))
            {
                break;
            }

            match recv_presence_monitor_event(&mut monitor).await {
                Ok(
                    PresenceMonitorEvent::Fallback {
                        cycle_id: event_cycle,
                        ..
                    }
                    | PresenceMonitorEvent::AnimationEnd {
                        cycle_id: event_cycle,
                        ..
                    }
                    | PresenceMonitorEvent::AnimationCancel {
                        cycle_id: event_cycle,
                        ..
                    },
                ) => {
                    complete_tooltip_presence_close_cycle(state, event_cycle);
                    break;
                }
                Ok(PresenceMonitorEvent::Stopped {
                    cycle_id: event_cycle,
                }) => {
                    if state
                        .presence_monitor
                        .cycle_id
                        .with_peek(|current| *current == Some(event_cycle))
                    {
                        clear_tooltip_presence_monitor_state(state);
                    }
                    break;
                }
                Err(error) => {
                    if state
                        .presence_monitor
                        .cycle_id
                        .with_peek(|current| *current == Some(cycle_id))
                    {
                        eprintln!(
                            "monoxus tooltip runtime could not observe content presence for {content_id}: {error}",
                        );
                        clear_tooltip_presence_monitor_state(state);
                    }
                    break;
                }
            }
        }
    });
}

fn complete_tooltip_presence_close_cycle(
    mut state: TooltipRuntimeState,
    cycle_id: PresenceCloseCycleId,
) {
    clear_tooltip_presence_monitor_state(state);
    let completed = state
        .presence_lane
        .with_mut(|lane| lane.complete_close_cycle(cycle_id));
    if completed {
        clear_tooltip_retained_content_state(state);
    }
}

fn stop_tooltip_presence_monitor(state: TooltipRuntimeState, content_id: &str) {
    let Some(monitor) = state.presence_monitor.monitor.with_peek(|monitor| *monitor) else {
        return;
    };

    clear_tooltip_presence_monitor_state(state);

    if let Err(error) = stop_presence_monitor(monitor) {
        eprintln!(
            "monoxus tooltip runtime could not stop content presence monitor for {content_id}: {error}",
        );
    }
}

fn clear_tooltip_presence_monitor_state(state: TooltipRuntimeState) {
    let mut active_monitor = state.presence_monitor.monitor;
    active_monitor.set(None);
    let mut active_cycle_id = state.presence_monitor.cycle_id;
    active_cycle_id.set(None);
}

#[cfg(test)]
mod tests {
    use super::TooltipContentPresenceLane;
    use crate::foundation::overlay::{Presence, PresenceState};

    #[test]
    fn phase_3_7_step_4_tooltip_presence_lane_retains_content_until_close_completion() {
        let presence = Presence::new(true).with_retained_mount(true);
        let mut lane = TooltipContentPresenceLane::new(&presence);

        let close = lane.sync(false);
        let cycle_id = close.started_close_cycle().unwrap();

        assert_eq!(close.state(), PresenceState::Suspended);
        assert!(lane.should_render_portal());
        assert!(lane.should_render_content());
        assert!(!lane.should_track_live_placement(false));
        assert!(!lane.should_clear_positioning(false));

        assert!(lane.complete_close_cycle(cycle_id));
        assert!(!lane.should_render_portal());
        assert!(!lane.should_render_content());
        assert!(lane.should_clear_positioning(false));
    }
}

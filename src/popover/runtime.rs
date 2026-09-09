use std::{collections::HashMap, rc::Rc, time::Duration};

use dioxus::{document::Eval, prelude::*};
use futures_timer::Delay;

pub use crate::foundation::compose::{
    MountedHandle as PopoverMountedHandle, compose_part_event_handlers, compose_part_refs,
    project_as_child,
};

use crate::foundation::{
    browser::{
        DocumentDismissEvent, FloatingAutoUpdateEvent, PresenceMonitorEvent, acquire_scroll_lock,
        focus_element_by_id, focus_first_focusable, focus_mounted_handle,
        recv_document_dismiss_event, recv_floating_auto_update_event, recv_presence_monitor_event,
        release_scroll_lock, restore_focus_element_by_id, start_document_dismiss_monitor,
        start_floating_auto_update_monitor, start_presence_monitor, stop_document_dismiss_monitor,
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
        PopoverAnchorAttributes, PopoverArrowAttributes, PopoverCloseAttributes,
        PopoverContentAttributes, PopoverPortalAttributes, PopoverRootAttributes,
        PopoverTriggerAttributes,
    },
    relationships::PopoverRelationships,
    state::{Popover, PopoverLifecycle},
    types::{PopoverCloseFocusPolicy, PopoverOpenFocusPolicy, PopoverStateRequest},
};

type PopoverOpenChangeHandler = Rc<dyn Fn(bool)>;

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
struct PopoverContentPresenceLane {
    content: PresenceController,
}

impl PopoverContentPresenceLane {
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
struct PopoverRuntimeState {
    trigger_handle: Signal<Option<PopoverMountedHandle>>,
    anchor_handle: Signal<Option<PopoverMountedHandle>>,
    content_handle: Signal<Option<PopoverMountedHandle>>,
    placement: Signal<Option<FloatingPlacement>>,
    focus_targets: Signal<HashMap<String, PopoverMountedHandle>>,
    presence_lane: Signal<PopoverContentPresenceLane>,
    presence_monitor: RetainedRootPresenceMonitorState,
    position_loop_token: Signal<u64>,
    position_monitor: Signal<Option<Eval>>,
    dismiss_loop_token: Signal<u64>,
    dismiss_monitor: Signal<Option<Eval>>,
    last_open: Signal<bool>,
    pending_open_focus: Signal<bool>,
    scroll_lock_held: Signal<bool>,
}

#[derive(Clone)]
pub struct PopoverRuntime {
    popover: Popover,
    on_open_change: PopoverOpenChangeHandler,
    state: PopoverRuntimeState,
}

pub fn use_popover_runtime<F>(popover: Popover, on_open_change: F) -> PopoverRuntime
where
    F: Fn(bool) + 'static,
{
    let synced_open_change: PopoverOpenChangeHandler = Rc::new(on_open_change);
    let state = PopoverRuntimeState {
        trigger_handle: use_signal(|| None),
        anchor_handle: use_signal(|| None),
        content_handle: use_signal(|| None),
        placement: use_signal(|| None),
        focus_targets: use_signal(HashMap::new),
        presence_lane: use_signal(|| {
            PopoverContentPresenceLane::new(popover.lifecycle().presence())
        }),
        presence_monitor: RetainedRootPresenceMonitorState {
            monitor: use_signal(|| Option::<Eval>::None),
            cycle_id: use_signal(|| None),
        },
        position_loop_token: use_signal(|| 0),
        position_monitor: use_signal(|| Option::<Eval>::None),
        dismiss_loop_token: use_signal(|| 0),
        dismiss_monitor: use_signal(|| Option::<Eval>::None),
        last_open: use_signal(|| popover.is_open()),
        pending_open_focus: use_signal(|| popover.is_open()),
        scroll_lock_held: use_signal(|| false),
    };
    let effect_state = state;
    let cleanup_state = state;
    let cleanup_key = popover.relationships().root_id().to_owned();
    let cleanup_content_id = popover.relationships().content_id().to_owned();
    let effect_open_change = Rc::clone(&synced_open_change);

    use_effect(use_reactive((&popover,), move |(popover,)| {
        sync_popover_runtime(&popover, effect_state, Rc::clone(&effect_open_change));
    }));

    dioxus::core::use_drop(move || {
        stop_popover_presence_monitor(cleanup_state, cleanup_content_id.as_str());
        advance_popover_token(cleanup_state.position_loop_token);
        stop_popover_position_monitor(cleanup_state);
        advance_popover_token(cleanup_state.dismiss_loop_token);
        stop_popover_dismiss_monitor(cleanup_state);
        if *cleanup_state.scroll_lock_held.peek() {
            release_scroll_lock(&cleanup_key, None);
        }
    });

    PopoverRuntime {
        popover,
        on_open_change: synced_open_change,
        state,
    }
}

impl PopoverRuntime {
    pub fn popover(&self) -> &Popover {
        &self.popover
    }

    pub const fn is_open(&self) -> bool {
        self.popover.is_open()
    }

    pub fn data_state(&self) -> DataState {
        self.popover.data_state()
    }

    pub fn relationships(&self) -> &PopoverRelationships {
        self.popover.relationships()
    }

    pub fn lifecycle(&self) -> &PopoverLifecycle {
        self.popover.lifecycle()
    }

    pub fn root(&self) -> PopoverRootAttributes {
        self.popover.root()
    }

    pub fn trigger(&self) -> PopoverTriggerAttributes {
        self.popover.trigger()
    }

    pub fn anchor(&self) -> PopoverAnchorAttributes {
        self.popover.anchor()
    }

    pub fn portal(&self) -> PopoverPortalAttributes {
        self.popover.portal()
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

    pub fn content(&self) -> PopoverContentAttributes {
        let mut content = self.popover.content();
        if let Some(placement) = self.placement() {
            content.data_side = placement.side().as_str();
            content.data_align = placement.align().as_str();
        }
        content
    }

    pub fn arrow(&self) -> PopoverArrowAttributes {
        let mut arrow = self.popover.arrow();
        if let Some(placement) = self.placement() {
            arrow.data_side = placement.side().as_str();
            arrow.data_align = placement.align().as_str();
        }
        arrow
    }

    pub fn close(&self) -> PopoverCloseAttributes {
        self.popover.close()
    }

    pub fn geometry_vars(&self, anchor: Rect, content: Size) -> GeometryVars {
        self.popover.geometry_vars(anchor, content)
    }

    pub fn trigger_handle(&self) -> Option<PopoverMountedHandle> {
        self.state.trigger_handle.cloned()
    }

    pub fn anchor_handle(&self) -> Option<PopoverMountedHandle> {
        self.state.anchor_handle.cloned()
    }

    pub fn content_handle(&self) -> Option<PopoverMountedHandle> {
        self.state.content_handle.cloned()
    }

    pub fn placement(&self) -> Option<FloatingPlacement> {
        self.state.placement.cloned()
    }

    pub fn mounted_focus_target(&self, id: &str) -> Option<PopoverMountedHandle> {
        self.state
            .focus_targets
            .with_peek(|targets| targets.get(id).cloned())
    }

    pub fn request_state(&self, request: PopoverStateRequest) -> bool {
        let next_open = request.next_open(self.is_open());
        if next_open != self.is_open() {
            (self.on_open_change)(next_open);
        }
        next_open
    }

    pub fn open(&self) {
        self.request_state(PopoverStateRequest::Open);
    }

    pub fn close_now(&self) {
        self.request_state(PopoverStateRequest::Close);
    }

    pub fn toggle(&self) {
        self.request_state(PopoverStateRequest::Toggle);
    }

    pub fn capture_trigger(&self, mounted: PopoverMountedHandle) {
        let mut trigger_handle = self.state.trigger_handle;
        trigger_handle.set(Some(mounted.clone()));
        self.capture_focus_target(self.relationships().trigger_id().to_owned(), mounted);
        self.refresh_live_placement();
    }

    pub fn capture_anchor(&self, mounted: PopoverMountedHandle) {
        let mut anchor_handle = self.state.anchor_handle;
        anchor_handle.set(Some(mounted));
        self.refresh_live_placement();
    }

    pub fn capture_content(&self, mounted: PopoverMountedHandle) {
        let mut content_handle = self.state.content_handle;
        content_handle.set(Some(mounted.clone()));
        self.capture_focus_target(self.relationships().content_id().to_owned(), mounted);
        self.refresh_live_placement();
        self.try_apply_pending_open_focus();
    }

    pub fn capture_close(&self, mounted: PopoverMountedHandle) {
        self.capture_focus_target(self.relationships().close_id().to_owned(), mounted);
    }

    pub fn capture_focus_target(&self, id: impl Into<String>, mounted: PopoverMountedHandle) {
        let id = id.into();
        let mut focus_targets = self.state.focus_targets;
        focus_targets.with_mut(|targets| {
            targets.insert(id, mounted);
        });
        self.try_apply_pending_open_focus();
    }

    pub fn mount_trigger(&self) -> impl FnMut(MountedEvent) + 'static {
        let runtime = self.clone();
        move |event| runtime.capture_trigger(event.data())
    }

    pub fn mount_anchor(&self) -> impl FnMut(MountedEvent) + 'static {
        let runtime = self.clone();
        move |event| runtime.capture_anchor(event.data())
    }

    pub fn mount_content(&self) -> impl FnMut(MountedEvent) + 'static {
        let runtime = self.clone();
        move |event| runtime.capture_content(event.data())
    }

    pub fn mount_close(&self) -> impl FnMut(MountedEvent) + 'static {
        let runtime = self.clone();
        move |event| runtime.capture_close(event.data())
    }

    pub fn mount_focus_target(&self, id: impl Into<String>) -> impl FnMut(MountedEvent) + 'static {
        let runtime = self.clone();
        let id = id.into();
        move |event| runtime.capture_focus_target(id.clone(), event.data())
    }

    pub fn inside_pointer_down(&self) -> impl FnMut(Event<PointerData>) + 'static {
        move |event| event.stop_propagation()
    }

    pub fn inside_focus_in(&self) -> impl FnMut(Event<FocusData>) + 'static {
        move |event| event.stop_propagation()
    }

    pub fn outside_pointer_down(&self) -> impl FnMut(Event<PointerData>) + 'static {
        let runtime = self.clone();
        move |event| {
            if runtime.dismiss_outside_pointer(None) {
                event.stop_propagation();
            }
        }
    }

    pub fn outside_focus_in(&self) -> impl FnMut(Event<FocusData>) + 'static {
        let runtime = self.clone();
        move |event| {
            if runtime.dismiss_outside_focus(None) {
                event.stop_propagation();
            }
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
            self.close_now();
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
            self.close_now();
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
            self.close_now();
            return true;
        }

        false
    }

    pub fn trigger_click(&self) -> impl FnMut(Event<MouseData>) + 'static {
        let runtime = self.clone();
        move |_| runtime.toggle()
    }

    pub fn close_click(&self) -> impl FnMut(Event<MouseData>) + 'static {
        let runtime = self.clone();
        move |_| runtime.close_now()
    }

    fn refresh_live_placement(&self) {
        if !self.is_open() {
            return;
        }

        let runtime = self.clone();
        spawn(async move {
            if let Err(error) = measure_popover_placement(runtime.popover(), runtime.state).await {
                eprintln!(
                    "monoxus popover runtime could not refresh placement for {}: {error}",
                    runtime.relationships().root_id(),
                );
            }
        });
    }

    fn dismiss_stack(&self) -> Vec<String> {
        vec![self.relationships().content_id().to_owned()]
    }

    fn try_apply_pending_open_focus(&self) {
        schedule_pending_open_focus(self.popover.clone(), self.state);
    }
}

fn schedule_pending_open_focus(popover: Popover, state: PopoverRuntimeState) {
    if !popover.is_open() || !*state.pending_open_focus.peek() {
        return;
    }

    spawn(async move {
        loop {
            if !popover.is_open() || !*state.pending_open_focus.peek() {
                return;
            }

            if apply_popover_open_focus(&popover, state).await {
                let mut pending_open_focus = state.pending_open_focus;
                pending_open_focus.set(false);
                return;
            }

            Delay::new(Duration::from_millis(16)).await;
        }
    });
}

fn sync_popover_runtime(
    popover: &Popover,
    state: PopoverRuntimeState,
    on_open_change: PopoverOpenChangeHandler,
) {
    sync_popover_presence(popover, state);

    let is_open = popover.is_open();
    let was_open = *state.last_open.peek();
    let should_hold_scroll_lock = is_open
        && popover.lifecycle().is_modal()
        && popover.lifecycle().scroll_lock_policy().is_enabled();
    let is_holding_scroll_lock = *state.scroll_lock_held.peek();

    if is_open && !was_open && !*state.pending_open_focus.peek() {
        let mut pending_open_focus = state.pending_open_focus;
        pending_open_focus.set(true);
        schedule_pending_open_focus(popover.clone(), state);
    }

    sync_popover_document_dismissal(popover, state, on_open_change);
    sync_popover_positioning(popover, state);

    if should_hold_scroll_lock && !is_holding_scroll_lock {
        acquire_scroll_lock(popover.relationships().root_id());
        let mut scroll_lock_held = state.scroll_lock_held;
        scroll_lock_held.set(true);
    } else if !should_hold_scroll_lock && is_holding_scroll_lock {
        let restore_delay = if was_open && !is_open {
            popover.lifecycle().scroll_lock_policy().restore_delay()
        } else {
            None
        };
        release_scroll_lock(popover.relationships().root_id(), restore_delay);
        let mut scroll_lock_held = state.scroll_lock_held;
        scroll_lock_held.set(false);
    }

    if !is_open {
        if was_open {
            restore_popover_close_focus(popover, state);
        }

        if *state.pending_open_focus.peek() {
            let mut pending_open_focus = state.pending_open_focus;
            pending_open_focus.set(false);
        }
    }

    if was_open != is_open {
        let mut last_open = state.last_open;
        last_open.set(is_open);
    }
}

fn sync_popover_document_dismissal(
    popover: &Popover,
    state: PopoverRuntimeState,
    on_open_change: PopoverOpenChangeHandler,
) {
    stop_popover_dismiss_monitor(state);

    if !popover.is_open() {
        advance_popover_token(state.dismiss_loop_token);
        return;
    }

    let dismiss_loop_token = advance_popover_token(state.dismiss_loop_token);
    let popover = popover.clone();
    let monitor = start_document_dismiss_monitor();
    let mut dismiss_monitor = state.dismiss_monitor;
    dismiss_monitor.set(Some(monitor));

    spawn(async move {
        let mut monitor = monitor;

        loop {
            if *state.dismiss_loop_token.peek() != dismiss_loop_token {
                break;
            }

            match recv_document_dismiss_event(&mut monitor).await {
                Ok(DocumentDismissEvent::Stopped) => break,
                Ok(event) => {
                    if should_dismiss_popover_from_document_event(&popover, &event) {
                        (on_open_change)(false);
                        break;
                    }
                }
                Err(error) => {
                    if *state.dismiss_loop_token.peek() == dismiss_loop_token {
                        eprintln!(
                            "monoxus popover runtime could not read dismissal events for {}: {error}",
                            popover.relationships().root_id(),
                        );
                    }
                    break;
                }
            }
        }
    });
}

fn should_dismiss_popover_from_document_event(
    popover: &Popover,
    event: &DocumentDismissEvent,
) -> bool {
    let dismiss_stack = popover_dismiss_stack(popover);
    match event {
        DocumentDismissEvent::PointerDown { path_ids } => {
            popover_document_path_is_outside(popover, path_ids)
                && popover
                    .lifecycle()
                    .dismiss_layer()
                    .should_dismiss_outside_pointer(None, &dismiss_stack)
        }
        DocumentDismissEvent::FocusIn { path_ids } => {
            popover_document_path_is_outside(popover, path_ids)
                && popover
                    .lifecycle()
                    .dismiss_layer()
                    .should_dismiss_outside_focus(None, &dismiss_stack)
        }
        DocumentDismissEvent::Escape => popover
            .lifecycle()
            .dismiss_layer()
            .should_dismiss_escape(&dismiss_stack),
        DocumentDismissEvent::Stopped => false,
    }
}

fn popover_dismiss_stack(popover: &Popover) -> Vec<String> {
    vec![popover.relationships().content_id().to_owned()]
}

pub(crate) fn popover_document_path_is_outside(popover: &Popover, path_ids: &[String]) -> bool {
    !popover_document_path_is_inside(popover, path_ids)
}

pub(crate) fn popover_document_path_is_inside(popover: &Popover, path_ids: &[String]) -> bool {
    let relationships = popover.relationships();
    let focus_guards = popover.lifecycle().focus_guards();
    let dismiss_layer = popover.lifecycle().dismiss_layer();

    path_ids.iter().any(|id| {
        id == relationships.content_id()
            || id == relationships.trigger_id()
            || id == relationships.anchor_id()
            || id == focus_guards.before()
            || id == focus_guards.after()
            || dismiss_layer.branches().iter().any(|branch| branch == id)
    })
}

async fn apply_popover_open_focus(popover: &Popover, state: PopoverRuntimeState) -> bool {
    if !popover.lifecycle().focus_scope().autofocus_enabled() {
        return true;
    }

    match popover.lifecycle().open_focus_policy() {
        PopoverOpenFocusPolicy::FirstFocusable => {
            if state.content_handle.with_peek(|handle| handle.is_none()) {
                return false;
            }

            focus_first_focusable(popover.relationships().content_id(), None);
            true
        }
        PopoverOpenFocusPolicy::Target(target) => {
            focus_registered_target(state, target);
            focus_element_by_id(target);
            active_element_matches_id(target).await
        }
        PopoverOpenFocusPolicy::Suppress => true,
    }
}

fn restore_popover_close_focus(popover: &Popover, _state: PopoverRuntimeState) {
    match popover.lifecycle().close_focus_policy() {
        PopoverCloseFocusPolicy::Trigger => {
            restore_focus_element_by_id(popover.relationships().trigger_id());
        }
        PopoverCloseFocusPolicy::Target(target) => {
            restore_focus_element_by_id(target);
        }
        PopoverCloseFocusPolicy::None => {}
    }
}

fn focus_registered_target(state: PopoverRuntimeState, target: &str) -> bool {
    focus_mounted_handle(
        state
            .focus_targets
            .with_peek(|targets| targets.get(target).cloned()),
    )
}

async fn active_element_matches_id(target_id: &str) -> bool {
    crate::foundation::browser::active_element_matches_id(target_id).await
}

fn sync_popover_positioning(popover: &Popover, state: PopoverRuntimeState) {
    stop_popover_position_monitor(state);

    let should_track_live_placement = state
        .presence_lane
        .with_peek(|lane| lane.should_track_live_placement(popover.is_open()));
    let should_clear_positioning = state
        .presence_lane
        .with_peek(|lane| lane.should_clear_positioning(popover.is_open()));

    if should_clear_positioning {
        advance_popover_token(state.position_loop_token);
        clear_popover_retained_content_state(state);
        return;
    }

    if !should_track_live_placement {
        advance_popover_token(state.position_loop_token);
        return;
    }

    let position_loop_token = advance_popover_token(state.position_loop_token);
    let popover = popover.clone();
    let monitor = start_floating_auto_update_monitor(
        &[
            popover.relationships().anchor_id(),
            popover.relationships().trigger_id(),
        ],
        popover.relationships().content_id(),
    );
    let mut position_monitor = state.position_monitor;
    position_monitor.set(Some(monitor));

    spawn(async move {
        let mut monitor = monitor;

        if let Err(error) = measure_popover_placement(&popover, state).await {
            eprintln!(
                "monoxus popover runtime could not measure placement for {}: {error}",
                popover.relationships().root_id(),
            );
        }

        loop {
            if *state.position_loop_token.peek() != position_loop_token {
                break;
            }

            match recv_floating_auto_update_event(&mut monitor).await {
                Ok(FloatingAutoUpdateEvent::Scroll) => {
                    match sync_hidden_popover_placement(&popover, state).await {
                        Ok(true) => continue,
                        Ok(false) => {}
                        Err(error) => {
                            eprintln!(
                                "monoxus popover runtime could not evaluate detached reference state for {}: {error}",
                                popover.relationships().root_id(),
                            );
                        }
                    }
                }
                Ok(FloatingAutoUpdateEvent::Update) => {}
                Ok(FloatingAutoUpdateEvent::Stopped) => break,
                Err(error) => {
                    if *state.position_loop_token.peek() == position_loop_token {
                        eprintln!(
                            "monoxus popover runtime auto-update monitor failed for {}: {error}",
                            popover.relationships().root_id(),
                        );
                    }
                    break;
                }
            }

            if *state.position_loop_token.peek() != position_loop_token {
                break;
            }

            if let Err(error) = measure_popover_placement(&popover, state).await {
                eprintln!(
                    "monoxus popover runtime could not measure placement for {}: {error}",
                    popover.relationships().root_id(),
                );
            }
        }
    });
}

fn stop_popover_position_monitor(state: PopoverRuntimeState) {
    let Some(monitor) = state.position_monitor.with_peek(|monitor| *monitor) else {
        return;
    };

    let mut position_monitor = state.position_monitor;
    position_monitor.set(None);

    if let Err(error) = stop_floating_auto_update_monitor(monitor) {
        eprintln!("monoxus popover runtime could not stop auto-update monitor: {error}");
    }
}

fn stop_popover_dismiss_monitor(state: PopoverRuntimeState) {
    let Some(monitor) = state.dismiss_monitor.with_peek(|monitor| *monitor) else {
        return;
    };

    let mut dismiss_monitor = state.dismiss_monitor;
    dismiss_monitor.set(None);

    if let Err(error) = stop_document_dismiss_monitor(monitor) {
        eprintln!("monoxus popover runtime could not stop dismiss monitor: {error}");
    }
}

async fn sync_hidden_popover_placement(
    popover: &Popover,
    state: PopoverRuntimeState,
) -> Result<bool, String> {
    if !popover.lifecycle().floating().hide_when_detached() {
        return Ok(false);
    }

    if !popover_reference_is_hidden(popover).await? {
        return Ok(false);
    }

    let Some(current_placement) = state.placement.with_peek(|current| current.clone()) else {
        return Ok(false);
    };
    if current_placement.reference_hidden() {
        return Ok(true);
    }

    let mut placement = state.placement;
    placement.set(Some(current_placement.hide_reference()));
    Ok(true)
}

async fn popover_reference_is_hidden(popover: &Popover) -> Result<bool, String> {
    let anchor_ids = [
        popover.relationships().anchor_id(),
        popover.relationships().trigger_id(),
    ];
    crate::foundation::browser::is_reference_hidden(&anchor_ids).await
}

async fn measure_popover_placement(
    popover: &Popover,
    state: PopoverRuntimeState,
) -> Result<(), String> {
    let Some(anchor_handle) = state
        .anchor_handle
        .with_peek(|handle| handle.clone())
        .or_else(|| state.trigger_handle.with_peek(|handle| handle.clone()))
    else {
        return Ok(());
    };
    let Some(content_handle) = state.content_handle.with_peek(|handle| handle.clone()) else {
        return Ok(());
    };

    let anchor_rect = read_client_rect(anchor_handle, "anchor").await?;
    let content_rect = read_client_rect(content_handle, "content").await?;
    let content_size = Size::new(content_rect.width(), content_rect.height());
    let viewport_size = read_viewport_size().await?;
    let placement = popover.lifecycle().floating().position_with_available_size(
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

async fn read_client_rect(mounted: PopoverMountedHandle, label: &str) -> Result<Rect, String> {
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

fn clear_popover_content_handle(state: PopoverRuntimeState) {
    if state.content_handle.with_peek(|handle| handle.is_some()) {
        let mut content_handle = state.content_handle;
        content_handle.set(None);
    }
}

fn clear_popover_placement(state: PopoverRuntimeState) {
    if state.placement.with_peek(|placement| placement.is_some()) {
        let mut placement = state.placement;
        placement.set(None);
    }
}

fn clear_popover_retained_content_state(state: PopoverRuntimeState) {
    clear_popover_content_handle(state);
    clear_popover_placement(state);
}

fn advance_popover_token(signal: Signal<u64>) -> u64 {
    let next = signal.with_peek(|value| value.saturating_add(1));
    let mut signal = signal;
    signal.set(next);
    next
}

fn sync_popover_presence(popover: &Popover, mut state: PopoverRuntimeState) {
    let update = state
        .presence_lane
        .with_mut(|lane| lane.sync(popover.is_open()));

    if update.invalidated_close_cycle().is_some() || !update.should_render() {
        stop_popover_presence_monitor(state, popover.relationships().content_id());
    }

    if let Some(cycle_id) = update.started_close_cycle() {
        start_popover_presence_monitor(popover, state, cycle_id);
    }
}

fn start_popover_presence_monitor(
    popover: &Popover,
    state: PopoverRuntimeState,
    cycle_id: PresenceCloseCycleId,
) {
    stop_popover_presence_monitor(state, popover.relationships().content_id());

    let content_id = popover.relationships().content_id().to_owned();
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
                    complete_popover_presence_close_cycle(state, event_cycle);
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
                        clear_popover_presence_monitor_state(state);
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
                            "monoxus popover runtime could not observe content presence for {content_id}: {error}",
                        );
                        clear_popover_presence_monitor_state(state);
                    }
                    break;
                }
            }
        }
    });
}

fn complete_popover_presence_close_cycle(
    mut state: PopoverRuntimeState,
    cycle_id: PresenceCloseCycleId,
) {
    clear_popover_presence_monitor_state(state);
    let completed = state
        .presence_lane
        .with_mut(|lane| lane.complete_close_cycle(cycle_id));
    if completed {
        clear_popover_retained_content_state(state);
    }
}

fn stop_popover_presence_monitor(state: PopoverRuntimeState, content_id: &str) {
    let Some(monitor) = state.presence_monitor.monitor.with_peek(|monitor| *monitor) else {
        return;
    };

    clear_popover_presence_monitor_state(state);

    if let Err(error) = stop_presence_monitor(monitor) {
        eprintln!(
            "monoxus popover runtime could not stop content presence monitor for {content_id}: {error}",
        );
    }
}

fn clear_popover_presence_monitor_state(state: PopoverRuntimeState) {
    let mut active_monitor = state.presence_monitor.monitor;
    active_monitor.set(None);
    let mut active_cycle_id = state.presence_monitor.cycle_id;
    active_cycle_id.set(None);
}

#[cfg(test)]
mod tests {
    use super::PopoverContentPresenceLane;
    use crate::foundation::overlay::{Presence, PresenceState};

    #[test]
    fn phase_3_7_step_4_popover_presence_lane_retains_content_until_close_completion() {
        let presence = Presence::new(true).with_retained_mount(true);
        let mut lane = PopoverContentPresenceLane::new(&presence);

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

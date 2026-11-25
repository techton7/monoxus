use std::{cell::Cell, rc::Rc, time::Duration};

use dioxus::prelude::keyboard_types::{Key, NamedKey};
use dioxus::prelude::*;
use futures_timer::Delay;

pub use crate::foundation::compose::{
    compose_part_event_handlers, compose_part_refs, project_as_child,
};

use crate::foundation::{
    browser::{
        DEFAULT_FOCUSABLE_SELECTOR, DocumentDismissEvent, DocumentPointerStreamGuard,
        FloatingAutoUpdateEvent, PresenceMonitorEvent, WatcherGuard, document_has_text_selection,
        get_element_client_rect, get_element_computed_z_index, is_element_focus_visible_by_id,
        is_element_focused_by_id, query_selector_all_by_id, restore_body_user_select,
        set_body_user_select_none, spawn_root_task, start_document_dismiss_monitor_with_boundaries,
        start_document_pointer_stream, start_document_pointer_up_observer,
        start_floating_auto_update_monitor, start_presence_monitor,
    },
    overlay::{
        FloatingLayer, FloatingPlacement, FloatingReadiness, GeometryVars, OverlayPositionConfig,
        PlacementAlign, PlacementSide, Point2D, Presence, PresenceCloseCycleId, PresenceController,
        PresenceControllerUpdate, Rect, Size, compute_overlay_position, normalize_hoisted_z_index,
    },
    shared::Direction,
    state::DataState,
};

use super::{
    attrs::{
        HoverCardArrowAttributes, HoverCardContentAttributes, HoverCardContentStaticAttributes,
        HoverCardPortalAttributes, HoverCardRootAttributes, HoverCardTriggerAttributes,
    },
    relationships::HoverCardRelationships,
    state::{HoverCard, HoverCardGraceStatus, HoverCardGraceTransit, HoverCardLifecycle},
    types::{HOVER_CARD_GEOMETRY_NAMESPACE, HoverCardOutsideBehavior},
};

type HoverCardOpenChangeHandler = Rc<dyn Fn(bool)>;

#[derive(Clone, Debug, Default, PartialEq)]
pub struct HoverCardFloatingOverrides {
    pub side: Option<PlacementSide>,
    pub side_offset: Option<f32>,
    pub align: Option<PlacementAlign>,
    pub align_offset: Option<f32>,
    pub avoid_collisions: Option<bool>,
    pub collision_boundary: Option<String>,
    pub collision_padding: Option<f32>,
    pub arrow_padding: Option<f32>,
    pub hide_when_detached: Option<bool>,
    pub dir: Option<Direction>,
}

#[derive(Clone, Default)]
pub struct HoverCardOutsideConfig {
    pub escape_keydown_behavior: HoverCardOutsideBehavior,
    pub interact_outside_behavior: HoverCardOutsideBehavior,
    pub on_escape_key_down: Option<EventHandler<()>>,
    pub on_pointer_down_outside: Option<EventHandler<()>>,
    pub on_interact_outside: Option<EventHandler<()>>,
}

impl PartialEq for HoverCardOutsideConfig {
    fn eq(&self, other: &Self) -> bool {
        self.escape_keydown_behavior == other.escape_keydown_behavior
            && self.interact_outside_behavior == other.interact_outside_behavior
            && self.on_escape_key_down == other.on_escape_key_down
            && self.on_pointer_down_outside == other.on_pointer_down_outside
            && self.on_interact_outside == other.on_interact_outside
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
struct HoverCardContentPresenceLane {
    content: PresenceController,
}

impl HoverCardContentPresenceLane {
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
    monitor: Signal<Option<WatcherGuard>>,
    cycle_id: Signal<Option<PresenceCloseCycleId>>,
}

#[derive(Clone, Copy)]
struct HoverCardRuntimeState {
    live_hover_card: Signal<HoverCard>,
    desired_open: Signal<bool>,
    last_open: Signal<bool>,
    trigger_handle: Signal<Option<Rc<MountedData>>>,
    content_handle: Signal<Option<Rc<MountedData>>>,
    placement: Signal<Option<FloatingPlacement>>,
    content_readiness: Signal<FloatingReadiness>,
    content_z_index: Signal<Option<String>>,
    presence_lane: Signal<HoverCardContentPresenceLane>,
    presence_monitor: RetainedRootPresenceMonitorState,
    pointer_down_inside_trigger: Signal<bool>,
    trigger_hovered: Signal<bool>,
    content_hovered: Signal<bool>,
    opened_via_focus: Signal<bool>,
    last_pointer_type: Signal<Option<String>>,
    is_pointer_down_on_content: Signal<bool>,
    has_selection: Signal<bool>,
    contain_selection: Signal<bool>,
    open_delay_token: Signal<u64>,
    close_delay_token: Signal<u64>,
    pending_open: Signal<bool>,
    position_loop_token: Signal<u64>,
    position_monitor: Signal<Option<WatcherGuard>>,
    dismiss_monitor: Signal<Option<WatcherGuard>>,
    pointer_up_monitor: Signal<Option<WatcherGuard>>,
    grace_monitor: Signal<Option<DocumentPointerStreamGuard>>,
    registered_trigger_id: Signal<Option<String>>,
    content_id: Signal<Option<String>>,
    branches: Signal<Vec<String>>,
    floating_overrides: Signal<HoverCardFloatingOverrides>,
    outside_config: Signal<HoverCardOutsideConfig>,
}

#[derive(Clone)]
pub struct HoverCardRuntime {
    hover_card: HoverCard,
    on_open_change: HoverCardOpenChangeHandler,
    on_open_change_complete: Option<HoverCardOpenChangeHandler>,
    state: HoverCardRuntimeState,
    alive: Rc<Cell<bool>>,
}

pub fn use_hover_card_runtime<F>(
    hover_card: HoverCard,
    on_open_change: F,
    on_open_change_complete: Option<HoverCardOpenChangeHandler>,
) -> HoverCardRuntime
where
    F: Fn(bool) + 'static,
{
    let mut live_hover_card = use_signal(|| hover_card.clone());
    let mut desired_open = use_signal(|| hover_card.is_open());
    if live_hover_card.with_peek(|current| current != &hover_card) {
        live_hover_card.set(hover_card.clone());
        desired_open.set(hover_card.is_open());
    }

    let state = HoverCardRuntimeState {
        live_hover_card,
        desired_open,
        last_open: use_signal(|| hover_card.is_open()),
        trigger_handle: use_signal(|| None),
        content_handle: use_signal(|| None),
        placement: use_signal(|| None),
        content_readiness: use_signal(FloatingReadiness::default),
        content_z_index: use_signal(|| None),
        presence_lane: use_signal(|| {
            HoverCardContentPresenceLane::new(hover_card.lifecycle().presence())
        }),
        presence_monitor: RetainedRootPresenceMonitorState {
            monitor: use_signal(|| None),
            cycle_id: use_signal(|| None),
        },
        pointer_down_inside_trigger: use_signal(|| false),
        trigger_hovered: use_signal(|| false),
        content_hovered: use_signal(|| false),
        opened_via_focus: use_signal(|| false),
        last_pointer_type: use_signal(|| None),
        is_pointer_down_on_content: use_signal(|| false),
        has_selection: use_signal(|| false),
        contain_selection: use_signal(|| false),
        open_delay_token: use_signal(|| 0),
        close_delay_token: use_signal(|| 0),
        pending_open: use_signal(|| false),
        position_loop_token: use_signal(|| 0),
        position_monitor: use_signal(|| None),
        dismiss_monitor: use_signal(|| None),
        pointer_up_monitor: use_signal(|| None),
        grace_monitor: use_signal(|| None),
        registered_trigger_id: use_signal(|| None),
        content_id: use_signal(|| None),
        branches: use_signal(Vec::new),
        floating_overrides: use_signal(HoverCardFloatingOverrides::default),
        outside_config: use_signal(HoverCardOutsideConfig::default),
    };

    let cleanup_state = state;
    let synced_open_change: HoverCardOpenChangeHandler = Rc::new(on_open_change);
    let effect_open_change = Rc::clone(&synced_open_change);
    let effect_open_complete = on_open_change_complete.clone();
    let effect_state = state;
    let cleanup_card = hover_card.clone();
    let alive = Rc::new(Cell::new(true));
    let cleanup_alive = alive.clone();

    let runtime_for_effects = HoverCardRuntime {
        hover_card: hover_card.clone(),
        on_open_change: Rc::clone(&synced_open_change),
        on_open_change_complete: on_open_change_complete.clone(),
        state,
        alive: Rc::clone(&alive),
    };

    use_effect(use_reactive((&hover_card,), move |(card,)| {
        let was_open = *effect_state.last_open.peek();
        let is_open = card.is_open();

        if !is_open {
            reset_hover_and_selection_state(effect_state);
            stop_hover_card_grace_monitor(effect_state);
            stop_hover_card_pointer_up_monitor(effect_state);
            stop_hover_card_dismiss_monitor(effect_state);
        } else {
            if !was_open {
                if let Some(ref on_complete) = effect_open_complete {
                    on_complete(true);
                }
                let content_id = effective_content_id_from_state(&card, effect_state);
                suppress_tabbable_descendants(&content_id);
                let cid_next = content_id.clone();
                spawn_root_task(async move {
                    let _ = oxidase::frame::next_frame().await;
                    suppress_tabbable_descendants(&cid_next);
                });
            }
            sync_hover_card_document_dismissal(&runtime_for_effects, &card);
            sync_hover_card_pointer_up_observer(&runtime_for_effects);
        }

        if was_open != is_open {
            let mut last_open = effect_state.last_open;
            if let Ok(mut w) = last_open.try_write() {
                *w = is_open;
            }
        }

        sync_hover_card_positioning(
            &card,
            effect_state,
            Rc::clone(&effect_open_change),
            effect_open_complete.clone(),
        );
    }));

    dioxus::core::use_drop(move || {
        cleanup_alive.set(false);
        if cleanup_state
            .contain_selection
            .try_peek()
            .is_ok_and(|c| *c)
        {
            restore_body_user_select();
        }
        stop_hover_card_presence_monitor(cleanup_state, cleanup_card.relationships().content_id());
        advance_token(cleanup_state.open_delay_token);
        advance_token(cleanup_state.close_delay_token);
        advance_token(cleanup_state.position_loop_token);
        stop_hover_card_position_monitor(cleanup_state);
        stop_hover_card_dismiss_monitor(cleanup_state);
        stop_hover_card_pointer_up_monitor(cleanup_state);
        stop_hover_card_grace_monitor(cleanup_state);
    });

    HoverCardRuntime {
        hover_card,
        on_open_change: synced_open_change,
        on_open_change_complete,
        state,
        alive,
    }
}

impl HoverCardRuntime {
    pub fn is_alive(&self) -> bool {
        self.alive.get() && self.state.open_delay_token.try_peek().is_ok()
    }

    pub fn hover_card(&self) -> &HoverCard {
        &self.hover_card
    }

    pub const fn is_open(&self) -> bool {
        self.hover_card.is_open()
    }

    pub fn is_live_open(&self) -> bool {
        self.state
            .desired_open
            .try_peek()
            .map(|o| *o)
            .unwrap_or_else(|_| self.hover_card.is_open())
    }

    pub fn is_disabled(&self) -> bool {
        self.state
            .live_hover_card
            .try_peek()
            .map(|c| c.is_disabled())
            .unwrap_or_else(|_| self.hover_card.is_disabled())
    }

    pub fn data_state(&self) -> DataState {
        self.hover_card.data_state()
    }

    pub fn relationships(&self) -> &HoverCardRelationships {
        self.hover_card.relationships()
    }

    pub fn lifecycle(&self) -> &HoverCardLifecycle {
        self.hover_card.lifecycle()
    }

    pub fn root(&self) -> HoverCardRootAttributes {
        self.hover_card.root()
    }

    pub fn trigger(
        &self,
        href: Option<String>,
        target: Option<String>,
        rel: Option<String>,
        disabled: bool,
    ) -> HoverCardTriggerAttributes {
        self.hover_card.trigger(href, target, rel, disabled)
    }

    pub fn portal(&self) -> HoverCardPortalAttributes {
        self.hover_card.portal()
    }

    pub fn should_render_portal(&self) -> bool {
        self.state.presence_lane.read().should_render_portal()
    }

    pub fn should_render_content(&self) -> bool {
        self.state.presence_lane.read().should_render_content()
    }

    pub fn content(&self) -> HoverCardContentAttributes {
        let mut content = self.hover_card.content();
        if let Some(placement) = self.placement() {
            content.data_side = placement.side().as_str();
            content.data_align = placement.align().as_str();
        } else {
            let effective = effective_floating_layer(&self.hover_card, self.state);
            content.data_side = effective.data_side();
            content.data_align = effective.data_align();
        }
        content
    }

    pub fn content_static(&self) -> HoverCardContentStaticAttributes {
        self.hover_card.content_static()
    }

    pub fn arrow(&self) -> HoverCardArrowAttributes {
        let mut arrow = self.hover_card.arrow();
        if let Some(placement) = self.placement() {
            arrow.data_side = placement.side().as_str();
            arrow.data_align = placement.align().as_str();
        } else {
            let effective = effective_floating_layer(&self.hover_card, self.state);
            arrow.data_side = effective.data_side();
            arrow.data_align = effective.data_align();
        }
        arrow
    }

    pub fn geometry_vars(&self, anchor: Rect, content: Size) -> GeometryVars {
        effective_floating_layer(&self.hover_card, self.state).geometry_vars(anchor, content)
    }

    pub fn content_readiness(&self) -> FloatingReadiness {
        *self.state.content_readiness.read()
    }

    pub fn content_positioning_state(&self) -> &'static str {
        self.content_readiness().positioning_state()
    }

    pub fn content_css_variables(&self) -> Vec<(String, String)> {
        self.placement()
            .map(|placement| HoverCard::content_css_variables(placement.geometry()))
            .unwrap_or_default()
    }

    pub fn content_css_custom_properties(&self) -> String {
        serialize_css_custom_properties(&self.content_css_variables())
    }

    pub fn wrapper_style(&self) -> String {
        let z_index = self.state.content_z_index.read();
        match self.placement() {
            Some(ref placement) => {
                placement.wrapper_style(self.content_readiness(), z_index.as_deref())
            }
            None => FloatingReadiness::wrapper_measuring_style(z_index.as_deref()),
        }
    }

    pub fn clear_content_z_index(&self) {
        clear_hover_card_content_z_index(self.state);
    }

    pub fn placement(&self) -> Option<FloatingPlacement> {
        self.state.placement.cloned()
    }

    pub fn is_contain_selection(&self) -> bool {
        *self.state.contain_selection.read()
    }

    pub fn has_selection(&self) -> bool {
        self.state
            .has_selection
            .try_peek()
            .map(|s| *s)
            .unwrap_or(false)
    }

    pub fn is_pointer_down_on_content(&self) -> bool {
        self.state
            .is_pointer_down_on_content
            .try_peek()
            .map(|p| *p)
            .unwrap_or(false)
    }

    pub fn register_content_id(&self, id: impl Into<String>) {
        register_id_signal(self.state.content_id, id.into());
    }

    pub fn unregister_content_id(&self, id: &str) {
        unregister_id_signal(self.state.content_id, id);
    }

    pub fn register_trigger_id(&self, id: impl Into<String>) {
        register_id_signal(self.state.registered_trigger_id, id.into());
    }

    pub fn unregister_trigger_id(&self, id: &str) {
        unregister_id_signal(self.state.registered_trigger_id, id);
    }

    pub fn register_branch(&self, branch: String) {
        let mut branches = self.state.branches;
        let already = branches
            .try_peek()
            .is_ok_and(|list| list.iter().any(|b| b == &branch));
        if !already {
            if let Ok(mut w) = branches.try_write() {
                w.push(branch);
            }
        }
    }

    pub fn unregister_branch(&self, branch: &str) {
        let mut branches = self.state.branches;
        if let Ok(mut w) = branches.try_write() {
            if let Some(pos) = w.iter().position(|b| b == branch) {
                w.remove(pos);
            }
        }
    }

    pub fn branches_signal(&self) -> Signal<Vec<String>> {
        self.state.branches
    }

    pub fn effective_trigger_id(&self) -> String {
        effective_trigger_id_from_state(&self.hover_card, self.state)
    }

    pub fn effective_content_id(&self) -> String {
        effective_content_id_from_state(&self.hover_card, self.state)
    }

    pub fn set_floating_overrides(&self, overrides: HoverCardFloatingOverrides) {
        let mut sig = self.state.floating_overrides;
        if sig.try_peek().is_ok_and(|current| *current != overrides) {
            if let Ok(mut w) = sig.try_write() {
                *w = overrides;
            }
        }
    }

    pub fn set_outside_config(&self, config: HoverCardOutsideConfig) {
        let mut sig = self.state.outside_config;
        if sig.try_peek().is_ok_and(|current| *current != config) {
            if let Ok(mut w) = sig.try_write() {
                *w = config;
            }
        }
    }

    pub fn resync_positioning_if_open(&self) {
        if self.is_live_open() {
            let live_card = self
                .state
                .live_hover_card
                .try_peek()
                .map(|c| c.clone())
                .unwrap_or_else(|_| self.hover_card.clone());
            sync_hover_card_positioning(
                &live_card,
                self.state,
                Rc::clone(&self.on_open_change),
                self.on_open_change_complete.clone(),
            );
        }
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
        self.state
            .grace_monitor
            .with_peek(|monitor| monitor.is_some())
    }

    pub fn stop_grace_monitor(&self) {
        stop_hover_card_grace_monitor(self.state);
    }

    pub fn start_hover_grace_corridor(&self, exit_point: Point2D, from_trigger: bool) -> bool {
        if !self.is_live_open() {
            return false;
        }

        let trigger_id = self.effective_trigger_id();
        let content_id = self.effective_content_id();

        let (Some(trigger_rect), Some(content_rect)) = (
            get_element_client_rect(&trigger_id),
            get_element_client_rect(&content_id),
        ) else {
            return false;
        };

        let (from_rect, to_rect) = if from_trigger {
            (trigger_rect, content_rect)
        } else {
            (content_rect, trigger_rect)
        };

        let transit = HoverCardGraceTransit::new(exit_point, from_rect, to_rect, Some(5.0));
        match transit.inspect_point(exit_point) {
            HoverCardGraceStatus::ExitedCorridor => return false,
            HoverCardGraceStatus::ReachedTarget if to_rect.contains_point(exit_point) => {
                return true;
            }
            HoverCardGraceStatus::InTransit | HoverCardGraceStatus::ReachedTarget => {}
        }
        let state = self.state;
        let runtime = self.clone();

        let guard = start_document_pointer_stream(move |point| {
            match transit.inspect_point(point) {
                HoverCardGraceStatus::InTransit => {}
                HoverCardGraceStatus::ReachedTarget => {
                    stop_hover_card_grace_monitor(state);
                    advance_token(state.close_delay_token);
                }
                HoverCardGraceStatus::ExitedCorridor => {
                    stop_hover_card_grace_monitor(state);
                    runtime.schedule_close();
                }
            }
        });

        let mut grace_monitor = self.state.grace_monitor;
        if let Ok(mut w) = grace_monitor.try_write() {
            *w = Some(guard);
        }
        true
    }

    pub fn mount_trigger(&self) -> impl FnMut(MountedEvent) + 'static {
        let runtime = self.clone();
        move |event| {
            let mut trigger_handle = runtime.state.trigger_handle;
            if let Ok(mut w) = trigger_handle.try_write() {
                *w = Some(event.data());
            }
        }
    }

    pub fn mount_content(&self) -> impl FnMut(MountedEvent) + 'static {
        let runtime = self.clone();
        move |event| {
            let mut content_handle = runtime.state.content_handle;
            if let Ok(mut w) = content_handle.try_write() {
                *w = Some(event.data());
            }
            let content_id = runtime.effective_content_id();
            suppress_tabbable_descendants(&content_id);

            let live_card = runtime
                .state
                .live_hover_card
                .try_peek()
                .map(|c| c.clone())
                .unwrap_or_else(|_| runtime.hover_card.clone());
            sync_hover_card_content_z_index(&live_card, runtime.state);
            sync_hover_card_positioning(
                &live_card,
                runtime.state,
                Rc::clone(&runtime.on_open_change),
                runtime.on_open_change_complete.clone(),
            );
            if let Some(placement) = compute_hover_card_position_with_state(&live_card, runtime.state)
            {
                if placement.geometry().content_height() > 0.0 {
                    sync_hover_card_content_z_index(&live_card, runtime.state);
                    update_placement_if_changed(runtime.state, placement);
                    set_hover_card_content_readiness(runtime.state, FloatingReadiness::Ready);
                    return;
                }
            }

            let runtime_clone = runtime.clone();
            spawn_root_task(async move {
                let _ = oxidase::frame::next_frame().await;
                if !runtime_clone.is_alive() {
                    return;
                }
                let cid = runtime_clone.effective_content_id();
                suppress_tabbable_descendants(&cid);
                let live_card = runtime_clone
                    .state
                    .live_hover_card
                    .try_peek()
                    .map(|c| c.clone())
                    .unwrap_or_else(|_| runtime_clone.hover_card.clone());
                sync_hover_card_content_z_index(&live_card, runtime_clone.state);
                if let Some(placement) =
                    compute_hover_card_position_with_state(&live_card, runtime_clone.state)
                {
                    update_placement_if_changed(runtime_clone.state, placement);
                    set_hover_card_content_readiness(
                        runtime_clone.state,
                        FloatingReadiness::Ready,
                    );
                }
            });
        }
    }

    pub fn mount_content_static(&self) -> impl FnMut(MountedEvent) + 'static {
        let runtime = self.clone();
        move |event| {
            let mut content_handle = runtime.state.content_handle;
            if let Ok(mut w) = content_handle.try_write() {
                *w = Some(event.data());
            }
            let content_id = runtime.effective_content_id();
            suppress_tabbable_descendants(&content_id);
            let runtime_clone = runtime.clone();
            spawn_root_task(async move {
                let _ = oxidase::frame::next_frame().await;
                if runtime_clone.is_alive() {
                    suppress_tabbable_descendants(&runtime_clone.effective_content_id());
                }
            });
        }
    }

    pub fn trigger_touch_start(&self) -> impl FnMut(Event<TouchData>) + 'static {
        move |event| {
            event.prevent_default();
        }
    }

    pub fn trigger_pointer_down(&self) -> impl FnMut(Event<PointerData>) + 'static {
        let runtime = self.clone();
        move |event| {
            if runtime.is_disabled() {
                return;
            }
            let mut pointer_down_inside = runtime.state.pointer_down_inside_trigger;
            if let Ok(mut w) = pointer_down_inside.try_write() {
                *w = true;
            }
            let mut last_pointer_type = runtime.state.last_pointer_type;
            if let Ok(mut w) = last_pointer_type.try_write() {
                *w = Some(event.pointer_type());
            }
        }
    }

    pub fn trigger_pointer_enter(&self, item_disabled: bool) -> impl FnMut(Event<PointerData>) + 'static {
        let runtime = self.clone();
        move |event| {
            if item_disabled || runtime.is_disabled() || event.pointer_type() == "touch" {
                return;
            }
            let mut trigger_hovered = runtime.state.trigger_hovered;
            if let Ok(mut w) = trigger_hovered.try_write() {
                *w = true;
            }
            let mut last_pointer_type = runtime.state.last_pointer_type;
            if last_pointer_type
                .try_peek()
                .ok()
                .and_then(|p| p.clone())
                .as_deref()
                != Some("touch")
            {
                if let Ok(mut w) = last_pointer_type.try_write() {
                    *w = None;
                }
            }
            runtime.cancel_close();
            runtime.stop_grace_monitor();
            runtime.schedule_open(false);
        }
    }

    pub fn trigger_pointer_leave(&self, item_disabled: bool) -> impl FnMut(Event<PointerData>) + 'static {
        let runtime = self.clone();
        move |event| {
            if item_disabled || event.pointer_type() == "touch" {
                return;
            }
            let mut trigger_hovered = runtime.state.trigger_hovered;
            if let Ok(mut w) = trigger_hovered.try_write() {
                *w = false;
            }

            // Cancel any pending delayed open if pointer leaves before open_delay elapses.
            runtime.cancel_pending_open();

            if !runtime.is_live_open() {
                return;
            }

            if runtime
                .state
                .opened_via_focus
                .try_peek()
                .is_ok_and(|f| *f)
                || runtime.trigger_has_focus()
                || runtime.has_selection()
                || runtime.is_pointer_down_on_content()
            {
                return;
            }

            let coords = event.client_coordinates();
            let exit_point = Point2D::new(coords.x, coords.y);
            if !runtime.start_hover_grace_corridor(exit_point, true)
                && !runtime.has_active_grace_monitor()
            {
                runtime.schedule_close();
            }
        }
    }

    pub fn trigger_focus(&self, item_disabled: bool) -> impl FnMut(Event<FocusData>) + 'static {
        let runtime = self.clone();
        move |_| {
            if item_disabled || runtime.is_disabled() {
                return;
            }
            let opened_from_pointer = runtime
                .state
                .pointer_down_inside_trigger
                .try_peek()
                .is_ok_and(|p| *p);
            let mut pointer_down_inside = runtime.state.pointer_down_inside_trigger;
            if let Ok(mut w) = pointer_down_inside.try_write() {
                *w = false;
            }

            let is_touch = runtime
                .state
                .last_pointer_type
                .try_peek()
                .ok()
                .and_then(|p| p.clone())
                .as_deref()
                == Some("touch");
            if is_touch {
                return;
            }

            if runtime.ignore_non_keyboard_focus() {
                if opened_from_pointer {
                    return;
                }
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

            let mut opened_via_focus = runtime.state.opened_via_focus;
            if let Ok(mut w) = opened_via_focus.try_write() {
                *w = true;
            }
            runtime.cancel_close();
            runtime.stop_grace_monitor();
            runtime.schedule_open(true);
        }
    }

    pub fn trigger_blur(&self) -> impl FnMut(Event<FocusData>) + 'static {
        let runtime = self.clone();
        move |_| {
            let mut pointer_down_inside = runtime.state.pointer_down_inside_trigger;
            if let Ok(mut w) = pointer_down_inside.try_write() {
                *w = false;
            }
            let mut last_pointer_type = runtime.state.last_pointer_type;
            if let Ok(mut w) = last_pointer_type.try_write() {
                *w = None;
            }
            let mut opened_via_focus = runtime.state.opened_via_focus;
            if let Ok(mut w) = opened_via_focus.try_write() {
                *w = false;
            }

            let trigger_hovered = runtime
                .state
                .trigger_hovered
                .try_peek()
                .is_ok_and(|h| *h);
            let content_hovered = runtime
                .state
                .content_hovered
                .try_peek()
                .is_ok_and(|h| *h);
            if !trigger_hovered
                && !content_hovered
                && !runtime.has_selection()
                && !runtime.is_pointer_down_on_content()
            {
                runtime.schedule_close();
            }
        }
    }

    pub fn content_pointer_down(&self) -> impl FnMut(Event<PointerData>) + 'static {
        let runtime = self.clone();
        move |event| {
            event.stop_propagation();
            let mut contain = runtime.state.contain_selection;
            if let Ok(mut w) = contain.try_write() {
                *w = true;
            }
            let mut has_sel = runtime.state.has_selection;
            if let Ok(mut w) = has_sel.try_write() {
                *w = false;
            }
            let mut down = runtime.state.is_pointer_down_on_content;
            if let Ok(mut w) = down.try_write() {
                *w = true;
            }
            set_body_user_select_none();
        }
    }

    pub fn content_pointer_enter(&self) -> impl FnMut(Event<PointerData>) + 'static {
        let runtime = self.clone();
        move |event| {
            if runtime.is_disabled() || event.pointer_type() == "touch" {
                return;
            }
            let mut content_hovered = runtime.state.content_hovered;
            if let Ok(mut w) = content_hovered.try_write() {
                *w = true;
            }
            runtime.cancel_close();
            runtime.stop_grace_monitor();
        }
    }

    pub fn content_pointer_leave(&self) -> impl FnMut(Event<PointerData>) + 'static {
        let runtime = self.clone();
        move |event| {
            if event.pointer_type() == "touch" {
                return;
            }
            let mut content_hovered = runtime.state.content_hovered;
            if let Ok(mut w) = content_hovered.try_write() {
                *w = false;
            }

            if runtime
                .state
                .opened_via_focus
                .try_peek()
                .is_ok_and(|f| *f)
                || runtime.trigger_has_focus()
                || runtime.has_selection()
                || runtime.is_pointer_down_on_content()
            {
                return;
            }

            let coords = event.client_coordinates();
            let exit_point = Point2D::new(coords.x, coords.y);
            if !runtime.start_hover_grace_corridor(exit_point, false)
                && !runtime.has_active_grace_monitor()
                && !runtime
                    .state
                    .trigger_hovered
                    .try_peek()
                    .is_ok_and(|h| *h)
            {
                runtime.schedule_close();
            }
        }
    }

    pub fn escape_keydown(&self) -> impl FnMut(Event<KeyboardData>) + 'static {
        let runtime = self.clone();
        move |event| {
            if event.key() == Key::Named(NamedKey::Escape) && runtime.dismiss_escape() {
                event.prevent_default();
                event.stop_propagation();
            }
        }
    }

    pub fn dismiss_escape(&self) -> bool {
        if !self.is_live_open() || crate::foundation::overlay::is_overlay_escape_consumed() {
            return false;
        }
        let effective_content_id = self.effective_content_id();
        if !crate::foundation::overlay::is_topmost_overlay_layer(&effective_content_id) {
            return false;
        }
        let stack = self.dismiss_stack();
        let should_dismiss_layer = self.state.live_hover_card.with_peek(|c| {
            let mut layer = c.lifecycle().dismiss_layer().clone();
            layer.set_id(effective_content_id.clone());
            layer.should_dismiss_escape(&stack)
        });
        if !should_dismiss_layer {
            return false;
        }

        let cfg = self
            .state
            .outside_config
            .try_peek()
            .map(|c| c.clone())
            .unwrap_or_default();
        if let Some(ref handler) = cfg.on_escape_key_down {
            handler.call(());
        }
        if cfg.escape_keydown_behavior.should_close() {
            crate::foundation::overlay::mark_overlay_escape_consumed();
            self.force_close();
            return true;
        }
        false
    }

    pub fn dismiss_outside_pointer(&self, target: Option<&str>) -> bool {
        if !self.is_live_open() {
            return false;
        }
        let target = target.map(|t| t.to_owned());
        let stack = self.dismiss_stack();
        let effective_content_id = self.effective_content_id();
        let should_dismiss_layer = self.state.live_hover_card.with_peek(|c| {
            let mut layer = c.lifecycle().dismiss_layer().clone();
            layer.set_id(effective_content_id.clone());
            if let Ok(branches) = self.state.branches.try_peek() {
                for branch in branches.iter() {
                    layer.register_branch(branch.clone());
                }
            }
            layer.should_dismiss_outside_pointer(target.as_ref(), &stack)
        });
        if !should_dismiss_layer {
            return false;
        }

        let cfg = self
            .state
            .outside_config
            .try_peek()
            .map(|c| c.clone())
            .unwrap_or_default();
        if let Some(ref handler) = cfg.on_pointer_down_outside {
            handler.call(());
        }
        if let Some(ref handler) = cfg.on_interact_outside {
            handler.call(());
        }
        if cfg.interact_outside_behavior.should_close() {
            self.force_close();
            return true;
        }
        false
    }

    pub fn schedule_open(&self, immediate: bool) {
        if !self.is_alive() || self.is_disabled() {
            return;
        }

        self.cancel_close();
        if self.is_live_open() {
            return;
        }

        let open_delay = self.open_delay_ms();
        if immediate || open_delay == 0 {
            let mut pending = self.state.pending_open;
            if let Ok(mut w) = pending.try_write() {
                *w = false;
            }
            advance_token(self.state.open_delay_token);
            self.commit_open(true);
            return;
        }

        let request_token = advance_token(self.state.open_delay_token);
        let mut pending = self.state.pending_open;
        if let Ok(mut w) = pending.try_write() {
            *w = true;
        }
        let runtime = self.clone();

        spawn_root_task(async move {
            Delay::new(Duration::from_millis(open_delay)).await;
            if !runtime.is_alive() || runtime.is_disabled() {
                return;
            }
            let token_matches = runtime
                .state
                .open_delay_token
                .try_peek()
                .is_ok_and(|t| *t == request_token);
            let still_pending = runtime
                .state
                .pending_open
                .try_peek()
                .is_ok_and(|p| *p);
            if token_matches && still_pending {
                let mut pending = runtime.state.pending_open;
                if let Ok(mut w) = pending.try_write() {
                    *w = false;
                }
                runtime.commit_open(true);
            }
        });
    }

    pub fn cancel_pending_open(&self) {
        advance_token(self.state.open_delay_token);
        let mut pending = self.state.pending_open;
        if let Ok(mut w) = pending.try_write() {
            *w = false;
        }
    }

    pub fn cancel_close(&self) {
        advance_token(self.state.close_delay_token);
    }

    pub fn schedule_close(&self) {
        if !self.is_alive() {
            return;
        }
        self.cancel_pending_open();

        if self.has_selection() || self.is_pointer_down_on_content() {
            return;
        }

        let close_delay = self.close_delay_ms();
        if close_delay == 0 {
            self.try_commit_close();
            return;
        }

        let request_token = advance_token(self.state.close_delay_token);
        let runtime = self.clone();

        spawn_root_task(async move {
            Delay::new(Duration::from_millis(close_delay)).await;
            if !runtime.is_alive() {
                return;
            }
            let token_matches = runtime
                .state
                .close_delay_token
                .try_peek()
                .is_ok_and(|t| *t == request_token);
            if token_matches {
                runtime.try_commit_close();
            }
        });
    }

    pub fn force_close(&self) {
        if !self.is_alive() {
            return;
        }
        if self
            .state
            .contain_selection
            .try_peek()
            .is_ok_and(|c| *c)
        {
            restore_body_user_select();
        }
        reset_hover_and_selection_state(self.state);
        self.cancel_pending_open();
        self.cancel_close();
        self.stop_grace_monitor();
        if self.is_live_open() {
            self.commit_open(false);
        }
    }

    fn try_commit_close(&self) {
        let trigger_hovered = self
            .state
            .trigger_hovered
            .try_peek()
            .is_ok_and(|h| *h);
        let content_hovered = self
            .state
            .content_hovered
            .try_peek()
            .is_ok_and(|h| *h);
        let opened_via_focus = self
            .state
            .opened_via_focus
            .try_peek()
            .is_ok_and(|f| *f);

        if !trigger_hovered
            && !content_hovered
            && !opened_via_focus
            && !self.trigger_has_focus()
            && !self.has_selection()
            && !self.is_pointer_down_on_content()
            && self.is_live_open()
        {
            self.commit_open(false);
        }
    }

    fn commit_open(&self, next_open: bool) {
        let mut desired = self.state.desired_open;
        if desired.try_peek().is_ok_and(|d| *d == next_open) {
            return;
        }
        if let Ok(mut w) = desired.try_write() {
            *w = next_open;
        }
        (self.on_open_change)(next_open);
    }

    fn open_delay_ms(&self) -> u64 {
        self.state
            .live_hover_card
            .try_peek()
            .map(|c| c.open_delay_ms())
            .unwrap_or_else(|_| self.hover_card.open_delay_ms())
    }

    fn close_delay_ms(&self) -> u64 {
        self.state
            .live_hover_card
            .try_peek()
            .map(|c| c.close_delay_ms())
            .unwrap_or_else(|_| self.hover_card.close_delay_ms())
    }

    fn ignore_non_keyboard_focus(&self) -> bool {
        self.state
            .live_hover_card
            .try_peek()
            .map(|c| c.ignore_non_keyboard_focus())
            .unwrap_or_else(|_| self.hover_card.ignore_non_keyboard_focus())
    }

    fn dismiss_stack(&self) -> Vec<String> {
        vec![self.effective_content_id()]
    }
}

pub(crate) fn suppress_tabbable_descendants(content_id: &str) {
    let candidates = query_selector_all_by_id(content_id, DEFAULT_FOCUSABLE_SELECTOR);
    for el in candidates {
        el.set_attribute("tabindex", "-1");
    }
}

fn sync_hover_card_document_dismissal(runtime: &HoverCardRuntime, card: &HoverCard) {
    stop_hover_card_dismiss_monitor(runtime.state);

    if !card.is_open() {
        return;
    }

    let trigger_id = effective_trigger_id_from_state(card, runtime.state);
    let content_id = effective_content_id_from_state(card, runtime.state);
    let mut boundary_ids = vec![trigger_id, content_id];
    if let Ok(branches) = runtime.state.branches.try_peek() {
        for b in branches.iter() {
            if !boundary_ids.contains(b) {
                boundary_ids.push(b.clone());
            }
        }
    }
    let boundary_refs: Vec<&str> = boundary_ids.iter().map(String::as_str).collect();
    let rt = runtime.clone();
    let card_clone = card.clone();

    let watcher = start_document_dismiss_monitor_with_boundaries(&boundary_refs, move |event| {
        if !rt.is_alive() || !rt.is_live_open() {
            return;
        }
        match event {
            DocumentDismissEvent::Escape => {
                rt.dismiss_escape();
            }
            DocumentDismissEvent::PointerDown { path_ids }
            | DocumentDismissEvent::Click { path_ids } => {
                if hover_card_document_path_is_outside(&card_clone, rt.state, &path_ids) {
                    rt.dismiss_outside_pointer(None);
                }
            }
            DocumentDismissEvent::FocusIn { .. } => {}
        }
    });

    let mut dismiss_monitor = runtime.state.dismiss_monitor;
    if let Ok(mut w) = dismiss_monitor.try_write() {
        *w = Some(watcher);
    }
}

fn hover_card_document_path_is_outside(
    card: &HoverCard,
    state: HoverCardRuntimeState,
    path_ids: &[String],
) -> bool {
    let trigger_id = effective_trigger_id_from_state(card, state);
    let content_id = effective_content_id_from_state(card, state);
    let branches = state
        .branches
        .try_peek()
        .map(|b| b.clone())
        .unwrap_or_default();

    !path_ids.iter().any(|id| {
        id == &trigger_id
            || id == &content_id
            || card.lifecycle().dismiss_layer().branches().contains(id)
            || branches.contains(id)
    })
}

fn sync_hover_card_pointer_up_observer(runtime: &HoverCardRuntime) {
    if runtime
        .state
        .pointer_up_monitor
        .try_peek()
        .is_ok_and(|m| m.is_some())
    {
        return;
    }

    let rt = runtime.clone();
    let watcher = start_document_pointer_up_observer(move || {
        if !rt.is_alive() {
            return;
        }
        let was_containing = rt
            .state
            .contain_selection
            .try_peek()
            .is_ok_and(|c| *c);
        let was_pointer_down = rt
            .state
            .is_pointer_down_on_content
            .try_peek()
            .is_ok_and(|p| *p);

        if was_containing {
            restore_body_user_select();
            let mut contain = rt.state.contain_selection;
            if let Ok(mut w) = contain.try_write() {
                *w = false;
            }
        }
        if was_pointer_down {
            let mut down = rt.state.is_pointer_down_on_content;
            if let Ok(mut w) = down.try_write() {
                *w = false;
            }
        }

        if was_containing || was_pointer_down {
            let rt_frame = rt.clone();
            spawn_root_task(async move {
                let _ = oxidase::frame::next_frame().await;
                if !rt_frame.is_alive() {
                    return;
                }
                let has_sel = document_has_text_selection();
                let mut has_sel_sig = rt_frame.state.has_selection;
                if let Ok(mut w) = has_sel_sig.try_write() {
                    *w = has_sel;
                }
                if !has_sel {
                    let trigger_hovered = rt_frame
                        .state
                        .trigger_hovered
                        .try_peek()
                        .is_ok_and(|h| *h);
                    let content_hovered = rt_frame
                        .state
                        .content_hovered
                        .try_peek()
                        .is_ok_and(|h| *h);
                    let opened_via_focus = rt_frame
                        .state
                        .opened_via_focus
                        .try_peek()
                        .is_ok_and(|f| *f);
                    if !trigger_hovered
                        && !content_hovered
                        && !opened_via_focus
                        && !rt_frame.trigger_has_focus()
                    {
                        rt_frame.schedule_close();
                    }
                }
            });
        }
    });

    let mut pointer_up_monitor = runtime.state.pointer_up_monitor;
    if let Ok(mut w) = pointer_up_monitor.try_write() {
        *w = Some(watcher);
    }
}

fn effective_floating_layer(card: &HoverCard, state: HoverCardRuntimeState) -> FloatingLayer {
    let base = card.lifecycle().floating().clone();
    let Ok(overrides) = state.floating_overrides.try_peek() else {
        return base;
    };
    let mut layer = if let Some(side) = overrides.side {
        FloatingLayer::new(side)
            .with_align(base.align())
            .with_direction(base.direction())
            .with_side_offset(base.side_offset())
            .with_align_offset(base.align_offset())
            .with_avoid_collisions(base.avoid_collisions())
            .with_collision_padding(base.collision_padding())
            .with_arrow_padding(base.arrow_padding())
            .with_sticky(base.sticky())
            .with_hide_when_detached(base.hide_when_detached())
            .with_namespace(HOVER_CARD_GEOMETRY_NAMESPACE)
    } else {
        base
    };

    if let Some(align) = overrides.align {
        layer = layer.with_align(align);
    }
    if let Some(side_offset) = overrides.side_offset {
        layer = layer.with_side_offset(side_offset);
    }
    if let Some(align_offset) = overrides.align_offset {
        layer = layer.with_align_offset(align_offset);
    }
    if let Some(avoid_collisions) = overrides.avoid_collisions {
        layer = layer.with_avoid_collisions(avoid_collisions);
    }
    if let Some(collision_padding) = overrides.collision_padding {
        layer = layer.with_collision_padding(collision_padding);
    }
    if let Some(arrow_padding) = overrides.arrow_padding {
        layer = layer.with_arrow_padding(arrow_padding);
    }
    if let Some(hide_when_detached) = overrides.hide_when_detached {
        layer = layer.with_hide_when_detached(hide_when_detached);
    }
    if let Some(dir) = overrides.dir {
        layer = layer.with_direction(dir);
    }
    layer
}

fn compute_hover_card_position_with_state(
    card: &HoverCard,
    state: HoverCardRuntimeState,
) -> Option<FloatingPlacement> {
    let trigger_id = effective_trigger_id_from_state(card, state);
    let content_id = effective_content_id_from_state(card, state);
    let boundary_id = state
        .floating_overrides
        .try_peek()
        .ok()
        .and_then(|o| o.collision_boundary.clone());
    let floating_layer = effective_floating_layer(card, state);
    let config = OverlayPositionConfig {
        anchor_id: &trigger_id,
        content_id: &content_id,
        arrow_id: Some(card.relationships().arrow_id()),
        boundary_id: boundary_id.as_deref(),
        side: floating_layer.side(),
        align: floating_layer.align(),
        side_offset: floating_layer.side_offset(),
        align_offset: floating_layer.align_offset(),
        avoid_collisions: floating_layer.avoid_collisions(),
        collision_padding: floating_layer.collision_padding(),
        arrow_padding: floating_layer.arrow_padding(),
        hide_when_detached: floating_layer.hide_when_detached(),
        sticky: true,
        namespace: Some(HOVER_CARD_GEOMETRY_NAMESPACE),
    };
    compute_overlay_position(&config)
}

fn update_placement_if_changed(state: HoverCardRuntimeState, placement: FloatingPlacement) {
    let should_update = state
        .placement
        .try_peek()
        .is_ok_and(|current| current.as_ref() != Some(&placement));
    if should_update {
        let mut current = state.placement;
        if let Ok(mut w) = current.try_write() {
            *w = Some(placement);
        }
    }
}

fn sync_hover_card_positioning(
    card: &HoverCard,
    state: HoverCardRuntimeState,
    _on_open_change: HoverCardOpenChangeHandler,
    on_open_change_complete: Option<HoverCardOpenChangeHandler>,
) {
    sync_hover_card_presence(card, state, on_open_change_complete);
    stop_hover_card_position_monitor(state);

    let should_track_live_placement = state
        .presence_lane
        .with_peek(|lane| lane.should_track_live_placement(card.is_open()));
    let should_clear_positioning = state
        .presence_lane
        .with_peek(|lane| lane.should_clear_positioning(card.is_open()));

    if should_clear_positioning {
        advance_token(state.position_loop_token);
        clear_hover_card_retained_content_state(state);
        return;
    }

    if !should_track_live_placement {
        advance_token(state.position_loop_token);
        return;
    }

    let _position_loop_token = advance_token(state.position_loop_token);

    sync_hover_card_content_z_index(card, state);
    if let Some(placement) = compute_hover_card_position_with_state(card, state) {
        update_placement_if_changed(state, placement);
        set_hover_card_content_readiness(state, FloatingReadiness::Ready);
    }

    let card_for_frame = card.clone();
    spawn_root_task(async move {
        let _ = oxidase::frame::next_frame().await;
        sync_hover_card_content_z_index(&card_for_frame, state);
        if let Some(placement) = compute_hover_card_position_with_state(&card_for_frame, state) {
            update_placement_if_changed(state, placement);
            set_hover_card_content_readiness(state, FloatingReadiness::Ready);
        }
    });

    let trigger_id = effective_trigger_id_from_state(card, state);
    let content_id = effective_content_id_from_state(card, state);
    let card_clone = card.clone();

    let watcher =
        start_floating_auto_update_monitor(&[trigger_id.as_str()], &content_id, move |event| {
            match event {
                FloatingAutoUpdateEvent::Scroll | FloatingAutoUpdateEvent::Update => {
                    sync_hover_card_content_z_index(&card_clone, state);
                    if let Some(placement) = compute_hover_card_position_with_state(&card_clone, state)
                    {
                        update_placement_if_changed(state, placement);
                        set_hover_card_content_readiness(state, FloatingReadiness::Ready);
                    }
                }
            }
        });

    let mut position_monitor = state.position_monitor;
    if let Ok(mut pm) = position_monitor.try_write() {
        *pm = Some(watcher);
    }
}

fn sync_hover_card_presence(
    card: &HoverCard,
    mut state: HoverCardRuntimeState,
    on_open_change_complete: Option<HoverCardOpenChangeHandler>,
) {
    let update = state
        .presence_lane
        .with_mut(|lane| lane.sync(card.is_open()));

    if update.invalidated_close_cycle().is_some() || !update.should_render() {
        stop_hover_card_presence_monitor(state, &effective_content_id_from_state(card, state));
    }

    if let Some(cycle_id) = update.started_close_cycle() {
        start_hover_card_presence_monitor(card, state, cycle_id, on_open_change_complete);
    }
}

fn start_hover_card_presence_monitor(
    card: &HoverCard,
    state: HoverCardRuntimeState,
    cycle_id: PresenceCloseCycleId,
    on_open_change_complete: Option<HoverCardOpenChangeHandler>,
) {
    let content_id = effective_content_id_from_state(card, state);
    stop_hover_card_presence_monitor(state, &content_id);

    let watcher = start_presence_monitor(content_id.as_str(), cycle_id, move |event| {
        match event {
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
            }
            | PresenceMonitorEvent::TransitionEnd {
                cycle_id: event_cycle,
                ..
            }
            | PresenceMonitorEvent::TransitionCancel {
                cycle_id: event_cycle,
                ..
            } => {
                complete_hover_card_presence_close_cycle(
                    state,
                    event_cycle,
                    on_open_change_complete.as_ref(),
                );
            }
            PresenceMonitorEvent::Stopped {
                cycle_id: event_cycle,
            } => {
                if state
                    .presence_monitor
                    .cycle_id
                    .with_peek(|current| *current == Some(event_cycle))
                {
                    clear_hover_card_presence_monitor_state(state);
                }
            }
        }
    });

    let mut active_monitor = state.presence_monitor.monitor;
    if let Ok(mut w) = active_monitor.try_write() {
        *w = Some(watcher);
    }
    let mut active_cycle_id = state.presence_monitor.cycle_id;
    if let Ok(mut w) = active_cycle_id.try_write() {
        *w = Some(cycle_id);
    }
}

fn complete_hover_card_presence_close_cycle(
    mut state: HoverCardRuntimeState,
    cycle_id: PresenceCloseCycleId,
    on_open_change_complete: Option<&HoverCardOpenChangeHandler>,
) {
    clear_hover_card_presence_monitor_state(state);
    let completed = state
        .presence_lane
        .with_mut(|lane| lane.complete_close_cycle(cycle_id));
    if completed {
        clear_hover_card_retained_content_state(state);
        if let Some(cb) = on_open_change_complete {
            cb(false);
        }
    }
}

fn stop_hover_card_presence_monitor(state: HoverCardRuntimeState, _content_id: &str) {
    clear_hover_card_presence_monitor_state(state);
}

fn clear_hover_card_presence_monitor_state(state: HoverCardRuntimeState) {
    let mut active_monitor = state.presence_monitor.monitor;
    if let Ok(mut m) = active_monitor.try_write() {
        *m = None;
    }
    let mut active_cycle_id = state.presence_monitor.cycle_id;
    if let Ok(mut c) = active_cycle_id.try_write() {
        *c = None;
    }
}

fn stop_hover_card_position_monitor(state: HoverCardRuntimeState) {
    let mut position_monitor = state.position_monitor;
    if let Ok(mut pm) = position_monitor.try_write() {
        *pm = None;
    }
}

fn stop_hover_card_dismiss_monitor(state: HoverCardRuntimeState) {
    let mut dismiss_monitor = state.dismiss_monitor;
    if let Ok(mut dm) = dismiss_monitor.try_write() {
        *dm = None;
    }
}

fn stop_hover_card_pointer_up_monitor(state: HoverCardRuntimeState) {
    let mut pointer_up_monitor = state.pointer_up_monitor;
    if let Ok(mut pu) = pointer_up_monitor.try_write() {
        *pu = None;
    }
}

fn stop_hover_card_grace_monitor(mut state: HoverCardRuntimeState) {
    if let Ok(mut gm) = state.grace_monitor.try_write() {
        *gm = None;
    }
}

fn sync_hover_card_content_z_index(card: &HoverCard, state: HoverCardRuntimeState) {
    let content_id = effective_content_id_from_state(card, state);
    let next = normalize_hoisted_z_index(get_element_computed_z_index(&content_id).as_deref());
    let mut sig = state.content_z_index;
    if sig.try_peek().is_ok_and(|cur| *cur != next) {
        if let Ok(mut w) = sig.try_write() {
            *w = next;
        }
    }
}

fn clear_hover_card_content_z_index(state: HoverCardRuntimeState) {
    let mut sig = state.content_z_index;
    if sig.try_peek().is_ok_and(|cur| cur.is_some()) {
        if let Ok(mut w) = sig.try_write() {
            *w = None;
        }
    }
}

fn clear_hover_card_retained_content_state(state: HoverCardRuntimeState) {
    stop_hover_card_grace_monitor(state);
    let mut content_handle = state.content_handle;
    if content_handle
        .try_peek()
        .is_ok_and(|h| h.is_some())
    {
        if let Ok(mut ch) = content_handle.try_write() {
            *ch = None;
        }
    }
    let mut placement = state.placement;
    if placement.try_peek().is_ok_and(|p| p.is_some()) {
        if let Ok(mut pl) = placement.try_write() {
            *pl = None;
        }
    }
    clear_hover_card_content_z_index(state);
    set_hover_card_content_readiness(state, FloatingReadiness::Measuring);
}

fn reset_hover_and_selection_state(state: HoverCardRuntimeState) {
    let mut ch = state.content_hovered;
    if let Ok(mut w) = ch.try_write() {
        *w = false;
    }
    let mut pdi = state.is_pointer_down_on_content;
    if let Ok(mut w) = pdi.try_write() {
        *w = false;
    }
    let mut hs = state.has_selection;
    if let Ok(mut w) = hs.try_write() {
        *w = false;
    }
    let mut cs = state.contain_selection;
    if cs.try_peek().is_ok_and(|c| *c) {
        restore_body_user_select();
        if let Ok(mut w) = cs.try_write() {
            *w = false;
        }
    }
}

fn set_hover_card_content_readiness(state: HoverCardRuntimeState, readiness: FloatingReadiness) {
    if state
        .content_readiness
        .try_peek()
        .is_ok_and(|current| *current != readiness)
    {
        let mut cr = state.content_readiness;
        if let Ok(mut w) = cr.try_write() {
            *w = readiness;
        }
    }
}

fn effective_trigger_id_from_state(card: &HoverCard, state: HoverCardRuntimeState) -> String {
    state
        .registered_trigger_id
        .try_peek()
        .ok()
        .and_then(|id| id.clone())
        .unwrap_or_else(|| card.relationships().trigger_id().to_owned())
}

fn effective_content_id_from_state(card: &HoverCard, state: HoverCardRuntimeState) -> String {
    state
        .content_id
        .try_peek()
        .ok()
        .and_then(|id| id.clone())
        .unwrap_or_else(|| card.relationships().content_id().to_owned())
}

fn register_id_signal(mut slot: Signal<Option<String>>, id: String) {
    let same = slot
        .try_peek()
        .is_ok_and(|current| current.as_deref() == Some(id.as_str()));
    if !same {
        if let Ok(mut current) = slot.try_write() {
            *current = Some(id);
        }
    }
}

fn unregister_id_signal(mut slot: Signal<Option<String>>, id: &str) {
    let matches = slot
        .try_peek()
        .is_ok_and(|current| current.as_deref() == Some(id));
    if matches {
        if let Ok(mut current) = slot.try_write() {
            *current = None;
        }
    }
}

fn advance_token(mut signal: Signal<u64>) -> u64 {
    let Ok(value) = signal.try_peek() else {
        return 0;
    };
    let next = value.saturating_add(1);
    drop(value);
    if let Ok(mut write) = signal.try_write() {
        *write = next;
    }
    next
}

fn serialize_css_custom_properties(entries: &[(String, String)]) -> String {
    let mut style = String::new();
    for (name, value) in entries {
        style.push(' ');
        style.push_str(name);
        style.push_str(": ");
        style.push_str(value);
        style.push(';');
    }
    style
}

#[cfg(test)]
mod tests {
    use super::{HoverCardContentPresenceLane, serialize_css_custom_properties};
    use crate::foundation::overlay::{GeometryVars, Presence, PresenceState};
    use crate::hover_card::{HoverCard, HOVER_CARD_GEOMETRY_NAMESPACE};

    #[test]
    fn hover_card_presence_lane_retains_content_until_close_completion() {
        let presence = Presence::new(true).with_retained_mount(true);
        let mut lane = HoverCardContentPresenceLane::new(&presence);

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

    #[test]
    fn hover_card_css_custom_properties_serialize_both_namespaces() {
        let geometry = GeometryVars::new(
            HOVER_CARD_GEOMETRY_NAMESPACE,
            10.0,
            20.0,
            15.0,
            0.0,
            200.0,
            150.0,
            60.0,
            20.0,
            180.0,
            90.0,
        );
        let vars = HoverCard::content_css_variables(&geometry);
        let serialized = serialize_css_custom_properties(&vars);

        assert!(serialized.contains("--monoxus-hover-card-floating-x: 10px;"));
        assert!(serialized.contains("--radix-hover-card-content-transform-origin: 15px 0px;"));
        assert!(serialized.contains("--radix-hover-card-trigger-width: 60px;"));
    }
}

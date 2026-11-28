use std::rc::Rc;

use dioxus::prelude::*;
use ui_sdk::kernel::{ChangeContext, ChangeEvent, ChangeReason, ChangeSource};

pub use crate::foundation::compose::{
    compose_part_event_handlers, compose_part_refs, project_as_child,
};

use crate::foundation::{
    browser::{
        document_has_text_selection, get_element_client_rect, is_element_focus_visible_by_id,
        is_element_focused_by_id, remove_from_tab_order, restore_body_user_select,
        restore_tab_order, set_body_user_select_none, spawn_root_task,
        start_document_pointer_up_observer,
    },
    keyboard::is_ime_composing_key,
    overlay::{
        CancellableDelay, DismissBehavior, DismissCallbacks, DismissConfig, DismissEvent,
        DismissLayerHandle, FloatingHandle, FloatingLayer, FloatingMonitorSpec, FloatingPlacement,
        FloatingReadiness, FocusScopeConfig, FocusScopeHandle, GeometryVars, GraceCorridorHandle,
        GraceCorridorOutcome, GraceTransit, GraceTransitStatus, OverlayLayerSpec, PlacementAlign,
        PlacementSide, Point2D, PresenceHandle, Rect, Size, StickyStrategy, WatcherSlot,
        active_overlay_layers_snapshot, is_overlay_escape_consumed, is_topmost_overlay_layer,
        mark_outside_press_propagating, mark_overlay_escape_consumed, sync_overlay_layer_spec,
        use_cancellable_delay, use_dismissable_layer_with_details, use_floating_layer,
        use_focus_scope, use_grace_corridor, use_presence_with_keep_mounted, use_watcher_slot,
    },
    shared::{
        Direction, EscapeKeyDownEvent, InteractOutsideEvent, PointerDownOutsideEvent,
        PreventableEvent,
    },
    state::{ControllableSignal, DataState, sync_signal},
};

use super::{
    attrs::{
        HoverCardArrowAttributes, HoverCardContentAttributes, HoverCardContentStaticAttributes,
        HoverCardPortalAttributes, HoverCardRootAttributes, HoverCardTriggerAttributes,
    },
    relationships::HoverCardRelationships,
    state::{HoverCard, HoverCardLifecycle},
    types::{
        HOVER_CARD_GEOMETRY_NAMESPACE, HOVER_CARD_RADIX_ANCHOR_LABEL,
        HOVER_CARD_RADIX_COMPATIBILITY_PREFIX,
    },
};

type HoverCardOpenChangeHandler = Rc<dyn Fn(bool)>;
const HOVER_CARD_GRACE_STALL_TIMEOUT_MS: u64 = 300;

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
    pub sticky: Option<StickyStrategy>,
    pub hide_when_detached: Option<bool>,
    pub custom_anchor: Option<String>,
    pub dir: Option<Direction>,
}

#[derive(Clone, Copy, Default, PartialEq)]
pub struct HoverCardOutsideConfig {
    pub escape_key_down_behavior: DismissBehavior,
    pub interact_outside_behavior: DismissBehavior,
    pub on_escape_key_down: Option<EventHandler<EscapeKeyDownEvent>>,
    pub on_pointer_down_outside: Option<EventHandler<PointerDownOutsideEvent>>,
    pub on_interact_outside: Option<EventHandler<InteractOutsideEvent>>,
}

#[derive(Clone, Copy)]
struct HoverCardRuntimeState {
    open_ctl: Option<ControllableSignal<bool>>,
    live_hover_card: Signal<HoverCard>,
    desired_open: Signal<bool>,
    trigger_handle: Signal<Option<Rc<MountedData>>>,
    content_handle: Signal<Option<Rc<MountedData>>>,
    presence: PresenceHandle,
    floating: FloatingHandle,
    dismiss: DismissLayerHandle,
    focus_scope: FocusScopeHandle,
    grace: GraceCorridorHandle,
    open_delay: CancellableDelay,
    close_delay: CancellableDelay,
    pointer_up_watcher: WatcherSlot,
    pointer_down_inside_trigger: CopyValue<bool>,
    trigger_hovered: CopyValue<bool>,
    content_hovered: CopyValue<bool>,
    opened_via_focus: CopyValue<bool>,
    last_pointer_type: CopyValue<Option<String>>,
    is_pointer_down_on_content: CopyValue<bool>,
    has_selection: CopyValue<bool>,
    contain_selection: Signal<bool>,
    handle_trigger_id: Signal<Option<String>>,
    registered_trigger_id: Signal<Option<String>>,
    content_id: Signal<Option<String>>,
    floating_overrides: Signal<HoverCardFloatingOverrides>,
    outside_config: CopyValue<HoverCardOutsideConfig>,
    keep_mounted: Signal<bool>,
    alive: CopyValue<bool>,
}

#[derive(Clone)]
pub struct HoverCardRuntime {
    hover_card: HoverCard,
    on_open_change: CopyValue<HoverCardOpenChangeHandler>,
    state: HoverCardRuntimeState,
}

fn propose_hover_card_open(
    open_ctl: Option<ControllableSignal<bool>>,
    on_open_change: CopyValue<HoverCardOpenChangeHandler>,
    next_open: bool,
    event: &ChangeEvent<bool>,
) -> bool {
    if let Some(ctl) = open_ctl {
        ctl.set_with_event(next_open, event).is_accepted()
    } else {
        if let Ok(cb) = on_open_change.try_read().map(|g| Rc::clone(&*g)) {
            cb(next_open);
        }
        true
    }
}

pub fn use_hover_card_runtime<F>(
    hover_card: HoverCard,
    on_open_change: F,
    on_open_change_complete: Option<HoverCardOpenChangeHandler>,
) -> HoverCardRuntime
where
    F: Fn(bool) + 'static,
{
    use_hover_card_runtime_inner(
        hover_card,
        None,
        Rc::new(on_open_change),
        on_open_change_complete,
        None,
        None,
    )
}

pub(crate) fn use_hover_card_runtime_with_ctl(
    hover_card: HoverCard,
    open_ctl: ControllableSignal<bool>,
    on_open_change_complete: Option<HoverCardOpenChangeHandler>,
    handle_active_trigger_id: Option<String>,
    handle_active_trigger_mounted: Option<Rc<MountedData>>,
) -> HoverCardRuntime {
    let fallback: HoverCardOpenChangeHandler = Rc::new(move |next_open: bool| {
        open_ctl.set(next_open);
    });
    use_hover_card_runtime_inner(
        hover_card,
        Some(open_ctl),
        fallback,
        on_open_change_complete,
        handle_active_trigger_id,
        handle_active_trigger_mounted,
    )
}

fn use_hover_card_runtime_inner(
    hover_card: HoverCard,
    open_ctl: Option<ControllableSignal<bool>>,
    synced_open_change: HoverCardOpenChangeHandler,
    on_open_change_complete: Option<HoverCardOpenChangeHandler>,
    handle_active_trigger_id: Option<String>,
    handle_active_trigger_mounted: Option<Rc<MountedData>>,
) -> HoverCardRuntime {
    let mut on_open_change_slot = use_hook(|| CopyValue::new(Rc::clone(&synced_open_change)));
    on_open_change_slot.set(synced_open_change);

    let mut on_open_change_complete_slot = use_hook(|| CopyValue::new(on_open_change_complete.clone()));
    on_open_change_complete_slot.set(on_open_change_complete);

    let mut live_hover_card = use_signal(|| hover_card.clone());
    let mut desired_open = use_signal(|| hover_card.is_open());
    sync_signal(&mut live_hover_card, hover_card.clone());
    sync_signal(&mut desired_open, hover_card.is_open());

    let trigger_handle = use_signal(|| None::<Rc<MountedData>>);
    let content_handle = use_signal(|| None::<Rc<MountedData>>);
    let pointer_down_inside_trigger = use_hook(|| CopyValue::new(false));
    let trigger_hovered = use_hook(|| CopyValue::new(false));
    let content_hovered = use_hook(|| CopyValue::new(false));
    let opened_via_focus = use_hook(|| CopyValue::new(false));
    let last_pointer_type = use_hook(|| CopyValue::new(Option::<String>::None));
    let is_pointer_down_on_content = use_hook(|| CopyValue::new(false));
    let has_selection = use_hook(|| CopyValue::new(false));
    let contain_selection = use_signal(|| false);
    let mut handle_trigger_id = use_signal(|| handle_active_trigger_id.clone());
    sync_signal(&mut handle_trigger_id, handle_active_trigger_id.clone());
    let registered_trigger_id = use_signal(|| None::<String>);
    let content_id = use_signal(|| None::<String>);
    let floating_overrides = use_signal(HoverCardFloatingOverrides::default);
    let outside_config = use_hook(|| CopyValue::new(HoverCardOutsideConfig::default()));
    let keep_mounted = use_signal(|| false);
    let mut alive = use_hook(|| CopyValue::new(true));

    let effective_trigger_id = handle_active_trigger_id
        .clone()
        .filter(|id| !id.is_empty())
        .or_else(|| registered_trigger_id.read().clone())
        .unwrap_or_else(|| hover_card.relationships().trigger_id().to_owned());
    let overrides_now = floating_overrides.read().clone();
    let effective_anchor_id = overrides_now
        .custom_anchor
        .clone()
        .filter(|id| !id.is_empty())
        .unwrap_or_else(|| effective_trigger_id.clone());
    let effective_content_id = content_id
        .read()
        .clone()
        .unwrap_or_else(|| hover_card.relationships().content_id().to_owned());

    let on_presence_complete = use_hook(move || {
        EventHandler::new(move |open: bool| {
            if let Ok(Some(cb)) = on_open_change_complete_slot
                .try_read()
                .map(|g| g.as_ref().map(Rc::clone))
            {
                cb(open);
            }
        })
    });

    let presence = use_presence_with_keep_mounted(
        hover_card.is_open(),
        &effective_content_id,
        hover_card.lifecycle().presence().retain_mount(),
        *keep_mounted.read(),
        Some(on_presence_complete),
    );

    let effective_floating = compute_effective_floating_layer(&hover_card, &overrides_now);
    let mut floating_spec =
        FloatingMonitorSpec::new(effective_anchor_id, effective_content_id.clone());
    floating_spec.arrow_id = Some(hover_card.relationships().arrow_id().to_owned());
    floating_spec.boundary_id = overrides_now.collision_boundary.clone();
    floating_spec.side = effective_floating.side();
    floating_spec.align = effective_floating.align();
    floating_spec.direction = effective_floating.direction();
    floating_spec.side_offset = effective_floating.side_offset();
    floating_spec.align_offset = effective_floating.align_offset();
    floating_spec.avoid_collisions = effective_floating.avoid_collisions();
    floating_spec.collision_padding = effective_floating.collision_padding();
    floating_spec.arrow_padding = effective_floating.arrow_padding();
    floating_spec.hide_when_detached = effective_floating.hide_when_detached();
    floating_spec.sticky = effective_floating.sticky() == StickyStrategy::Always;
    floating_spec.namespace = Some(HOVER_CARD_GEOMETRY_NAMESPACE.to_owned());

    let floating = use_floating_layer(
        hover_card.is_open() && presence.should_render(),
        floating_spec,
        None,
    );
    if let Some(mounted) = handle_active_trigger_mounted {
        floating.capture_anchor(mounted);
    }

    let grace = use_grace_corridor();
    let open_delay = use_cancellable_delay();
    let close_delay = use_cancellable_delay();
    let pointer_up_watcher = use_watcher_slot();

    let mut focus_cfg = FocusScopeConfig::new(effective_content_id.clone());
    focus_cfg.auto_focus_on_open = false;
    focus_cfg.restore_focus_on_close = false;
    focus_cfg.suppress_tab_order = true;
    let focus_scope = use_focus_scope(hover_card.is_open(), focus_cfg, None, None);

    let mut dismiss_boundaries = vec![effective_content_id.clone()];
    for branch in hover_card.lifecycle().dismiss_layer().branches() {
        if !dismiss_boundaries.contains(branch) {
            dismiss_boundaries.push(branch.clone());
        }
    }
    let dismiss_config = DismissConfig::new(effective_content_id.clone())
        .with_boundaries(dismiss_boundaries)
        .with_modal(false)
        .with_listen_escape(true)
        .with_listen_pointer_down_outside(true)
        .with_listen_focus_outside(false);

    let on_escape_bridge = use_hook(move || {
        EventHandler::new(move |evt: EscapeKeyDownEvent| {
            let cfg = outside_config.try_read().map(|g| *g).unwrap_or_default();
            if let Some(handler) = cfg.on_escape_key_down {
                handler.call(evt.clone());
            }
            if !cfg.escape_key_down_behavior.dismisses() {
                evt.prevent_default();
            }
        })
    });
    let on_ptr_down_bridge = use_hook(move || {
        EventHandler::new(move |evt: PointerDownOutsideEvent| {
            let cfg = outside_config.try_read().map(|g| *g).unwrap_or_default();
            if let Some(handler) = cfg.on_pointer_down_outside {
                handler.call(evt.clone());
            }
            if !cfg.interact_outside_behavior.dismisses() {
                evt.prevent_default();
            }
        })
    });
    let on_interact_bridge = use_hook(move || {
        EventHandler::new(move |evt: InteractOutsideEvent| {
            let cfg = outside_config.try_read().map(|g| *g).unwrap_or_default();
            if let Some(handler) = cfg.on_interact_outside {
                handler.call(evt.clone());
            }
            if !cfg.interact_outside_behavior.dismisses() {
                evt.prevent_default();
            }
        })
    });

    let force_close_from_dismiss_details = use_hook(move || {
        EventHandler::new(move |(_event, change_event): (DismissEvent, ChangeEvent<bool>)| {
            if !alive.try_read().is_ok_and(|a| *a) {
                return;
            }
            let mut desired = desired_open;
            if desired.try_peek().is_ok_and(|d| *d) {
                if !propose_hover_card_open(open_ctl, on_open_change_slot, false, &change_event) {
                    return;
                }
                let _ = desired.try_write().map(|mut w| *w = false);
            }
            reset_hover_and_selection_slots(
                content_hovered,
                opened_via_focus,
                is_pointer_down_on_content,
                has_selection,
                contain_selection,
            );
            open_delay.cancel();
            close_delay.cancel();
            grace.clear();
        })
    });

    let dismiss = use_dismissable_layer_with_details(
        hover_card.is_open(),
        dismiss_config,
        DismissCallbacks {
            on_escape_key_down: Some(on_escape_bridge),
            on_pointer_down_outside: Some(on_ptr_down_bridge),
            on_focus_outside: None,
            on_interact_outside: Some(on_interact_bridge),
            on_dismiss: None,
        },
        Some(force_close_from_dismiss_details),
    );

    if hover_card.is_open() && !effective_content_id.is_empty() {
        let cfg = outside_config.try_read().map(|g| *g).unwrap_or_default();
        sync_overlay_layer_spec(
            OverlayLayerSpec::new(effective_content_id)
                .with_modal(false)
                .with_escape_behavior(cfg.escape_key_down_behavior)
                .with_outside_press_behavior(cfg.interact_outside_behavior),
        );
    }

    let state = HoverCardRuntimeState {
        open_ctl,
        live_hover_card,
        desired_open,
        trigger_handle,
        content_handle,
        presence,
        floating,
        dismiss,
        focus_scope,
        grace,
        open_delay,
        close_delay,
        pointer_up_watcher,
        pointer_down_inside_trigger,
        trigger_hovered,
        content_hovered,
        opened_via_focus,
        last_pointer_type,
        is_pointer_down_on_content,
        has_selection,
        contain_selection,
        handle_trigger_id,
        registered_trigger_id,
        content_id,
        floating_overrides,
        outside_config,
        keep_mounted,
        alive,
    };

    let runtime = HoverCardRuntime {
        hover_card: hover_card.clone(),
        on_open_change: on_open_change_slot,
        state,
    };

    let is_open_now = hover_card.is_open();
    let should_render_now = presence.should_render();
    let rt_for_effect = runtime.clone();
    use_effect(use_reactive(
        (&is_open_now, &should_render_now),
        move |(open_now, render_now)| {
            if !open_now {
                reset_hover_and_selection_slots(
                    state.content_hovered,
                    state.opened_via_focus,
                    state.is_pointer_down_on_content,
                    state.has_selection,
                    state.contain_selection,
                );
                state.grace.clear();
                state.pointer_up_watcher.clear();
                if !render_now {
                    let mut ch = state.content_handle;
                    if ch.try_peek().is_ok_and(|h| h.is_some())
                        && let Ok(mut w) = ch.try_write()
                    {
                        *w = None;
                    }
                    state.floating.clear();
                }
            } else {
                arm_hover_card_pointer_up_observer(&rt_for_effect);
            }
        },
    ));

    use_drop(move || {
        if let Ok(mut flag) = alive.try_write() {
            *flag = false;
        }
        if contain_selection.try_peek().is_ok_and(|c| *c) {
            restore_body_user_select();
        }
        pointer_up_watcher.clear();
        grace.clear();
        open_delay.cancel();
        close_delay.cancel();
    });

    runtime
}

impl HoverCardRuntime {
    pub fn is_alive(&self) -> bool {
        self.state.alive.try_read().is_ok_and(|a| *a)
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
        self.state.presence.should_render()
    }

    pub fn should_render_content(&self) -> bool {
        self.state.presence.should_render()
    }

    pub fn data_starting_style(&self) -> Option<&'static str> {
        self.state.presence.data_starting_style()
    }

    pub fn data_ending_style(&self) -> Option<&'static str> {
        self.state.presence.data_ending_style()
    }

    pub fn is_presence_hidden(&self) -> bool {
        self.state.presence.is_hidden()
    }

    pub fn set_keep_mounted(&self, keep_mounted: bool) {
        let mut sig = self.state.keep_mounted;
        sync_signal(&mut sig, keep_mounted);
    }

    pub fn content(&self) -> HoverCardContentAttributes {
        let mut content = self.hover_card.content();
        if let Some(placement) = self.placement() {
            content.data_side = placement.side().as_str();
            content.data_align = placement.align().as_str();
        } else {
            let effective = self.effective_floating_layer();
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
            let effective = self.effective_floating_layer();
            arrow.data_side = effective.data_side();
            arrow.data_align = effective.data_align();
        }
        arrow
    }

    pub fn effective_arrow_padding(&self) -> f32 {
        self.state
            .floating_overrides
            .read()
            .arrow_padding
            .unwrap_or_else(|| self.hover_card.lifecycle().floating().arrow_padding())
    }

    pub fn geometry_vars(&self, anchor: Rect, content: Size) -> GeometryVars {
        self.effective_floating_layer().geometry_vars(anchor, content)
    }

    pub fn content_readiness(&self) -> FloatingReadiness {
        self.state.floating.readiness()
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
        self.state.floating.css_custom_properties(Some((
            HOVER_CARD_RADIX_COMPATIBILITY_PREFIX,
            HOVER_CARD_RADIX_ANCHOR_LABEL,
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

    pub fn is_contain_selection(&self) -> bool {
        *self.state.contain_selection.read()
    }

    pub fn has_selection(&self) -> bool {
        self.state.has_selection.try_read().map(|s| *s).unwrap_or(false)
    }

    pub fn is_pointer_down_on_content(&self) -> bool {
        self.state
            .is_pointer_down_on_content
            .try_read()
            .map(|p| *p)
            .unwrap_or(false)
    }

    pub fn register_content_id(&self, id: impl Into<String>) {
        let id = id.into();
        let mut sig = self.state.content_id;
        sync_signal(&mut sig, Some(id.clone()));
        self.state.dismiss.rename_layer_id(&id);
        self.state.focus_scope.rename_container_id(&id);
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

    pub fn register_branch(&self, branch: String) {
        self.state.dismiss.register_branch(branch);
    }

    pub fn unregister_branch(&self, branch: &str) {
        self.state.dismiss.unregister_branch(branch);
    }

    pub fn branches_signal(&self) -> Signal<Vec<String>> {
        self.state.dismiss.branches_signal()
    }

    pub fn effective_trigger_id(&self) -> String {
        self.state
            .handle_trigger_id
            .try_peek()
            .ok()
            .and_then(|id| id.clone())
            .filter(|id| !id.is_empty())
            .or_else(|| {
                self.state
                    .registered_trigger_id
                    .try_peek()
                    .ok()
                    .and_then(|id| id.clone())
            })
            .unwrap_or_else(|| self.hover_card.relationships().trigger_id().to_owned())
    }

    pub fn effective_content_id(&self) -> String {
        self.state
            .content_id
            .try_peek()
            .ok()
            .and_then(|id| id.clone())
            .unwrap_or_else(|| self.hover_card.relationships().content_id().to_owned())
    }

    pub fn set_floating_overrides(&self, overrides: HoverCardFloatingOverrides) {
        let mut sig = self.state.floating_overrides;
        sync_signal(&mut sig, overrides);
    }

    pub fn set_outside_config(&self, config: HoverCardOutsideConfig) {
        let mut slot = self.state.outside_config;
        let _ = slot.try_write().map(|mut w| *w = config);
    }

    pub fn resync_positioning_if_open(&self) {
        if self.is_live_open() {
            self.state.floating.refresh();
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
        self.state.grace.is_active()
    }

    pub fn stop_grace_monitor(&self) {
        self.state.grace.clear();
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

        let transit = GraceTransit::new(exit_point, from_rect, to_rect, Some(5.0), true);
        match transit.inspect_point(exit_point) {
            GraceTransitStatus::ExitedCorridor => return false,
            GraceTransitStatus::ReachedTarget => return true,
            GraceTransitStatus::InTransit => {}
        }

        let runtime = self.clone();
        self.state.grace.start(
            transit,
            Some(HOVER_CARD_GRACE_STALL_TIMEOUT_MS),
            move |outcome| match outcome {
                GraceCorridorOutcome::InTransit(_) => {}
                GraceCorridorOutcome::ReachedTarget(_) => {
                    runtime.cancel_close();
                }
                GraceCorridorOutcome::ExitedCorridor(_) | GraceCorridorOutcome::Stalled => {
                    runtime.schedule_close();
                }
            },
        );
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
            let content_id = runtime.effective_content_id();
            remove_from_tab_order(&content_id);
            runtime.state.floating.capture_content(event.data());

            let rt_next = runtime.clone();
            spawn_root_task(async move {
                let _ = oxidase::frame::next_frame().await;
                if rt_next.is_alive() && rt_next.is_live_open() {
                    remove_from_tab_order(&rt_next.effective_content_id());
                }
            });
        }
    }

    pub fn mount_content_static(&self) -> impl FnMut(MountedEvent) + 'static {
        let runtime = self.clone();
        move |event| {
            let mut content_handle = runtime.state.content_handle;
            let _ = content_handle.try_write().map(|mut w| *w = Some(event.data()));
            let content_id = runtime.effective_content_id();
            remove_from_tab_order(&content_id);
            let rt_next = runtime.clone();
            spawn_root_task(async move {
                let _ = oxidase::frame::next_frame().await;
                if rt_next.is_alive() && rt_next.is_live_open() {
                    remove_from_tab_order(&rt_next.effective_content_id());
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
            runtime.cancel_pending_open();
            let mut pdi = runtime.state.pointer_down_inside_trigger;
            let _ = pdi.try_write().map(|mut w| *w = true);
            let mut lpt = runtime.state.last_pointer_type;
            let _ = lpt.try_write().map(|mut w| *w = Some(event.pointer_type()));
        }
    }

    pub fn trigger_pointer_enter(
        &self,
        item_disabled: bool,
    ) -> impl FnMut(Event<PointerData>) + 'static {
        let runtime = self.clone();
        move |event| {
            if item_disabled || runtime.is_disabled() || event.pointer_type() == "touch" {
                return;
            }
            let mut th = runtime.state.trigger_hovered;
            let _ = th.try_write().map(|mut w| *w = true);
            let mut lpt = runtime.state.last_pointer_type;
            if lpt.try_read().ok().and_then(|p| p.clone()).as_deref() != Some("touch") {
                let _ = lpt.try_write().map(|mut w| *w = None);
            }
            runtime.cancel_close();
            runtime.stop_grace_monitor();
            runtime.schedule_open(false);
        }
    }

    pub fn trigger_pointer_leave(
        &self,
        item_disabled: bool,
    ) -> impl FnMut(Event<PointerData>) + 'static {
        let runtime = self.clone();
        move |event| {
            if item_disabled || event.pointer_type() == "touch" {
                return;
            }
            let mut th = runtime.state.trigger_hovered;
            let _ = th.try_write().map(|mut w| *w = false);

            runtime.cancel_pending_open();

            if !runtime.is_live_open() {
                return;
            }

            if runtime.state.opened_via_focus.try_read().is_ok_and(|f| *f)
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
                .try_read()
                .is_ok_and(|p| *p);
            let mut pdi = runtime.state.pointer_down_inside_trigger;
            let _ = pdi.try_write().map(|mut w| *w = false);

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

            let mut ovf = runtime.state.opened_via_focus;
            let _ = ovf.try_write().map(|mut w| *w = true);
            runtime.cancel_close();
            runtime.stop_grace_monitor();
            runtime.schedule_open_with_context(
                false,
                ChangeContext::new(ChangeReason::Focus, ChangeSource::Keyboard),
            );
        }
    }

    pub fn trigger_blur(&self) -> impl FnMut(Event<FocusData>) + 'static {
        let runtime = self.clone();
        move |_| {
            let mut pdi = runtime.state.pointer_down_inside_trigger;
            let _ = pdi.try_write().map(|mut w| *w = false);
            let mut lpt = runtime.state.last_pointer_type;
            let _ = lpt.try_write().map(|mut w| *w = None);
            let mut ovf = runtime.state.opened_via_focus;
            let _ = ovf.try_write().map(|mut w| *w = false);

            let trigger_hovered = runtime.state.trigger_hovered.try_read().is_ok_and(|h| *h);
            let content_hovered = runtime.state.content_hovered.try_read().is_ok_and(|h| *h);
            if !trigger_hovered
                && !content_hovered
                && !runtime.has_selection()
                && !runtime.is_pointer_down_on_content()
            {
                runtime.schedule_close_with_context(ChangeContext::new(
                    ChangeReason::FocusOut,
                    ChangeSource::Keyboard,
                ));
            }
        }
    }

    pub fn content_pointer_down(&self) -> impl FnMut(Event<PointerData>) + 'static {
        let runtime = self.clone();
        move |event| {
            event.stop_propagation();
            let mut contain = runtime.state.contain_selection;
            sync_signal(&mut contain, true);
            let mut hs = runtime.state.has_selection;
            let _ = hs.try_write().map(|mut w| *w = false);
            let mut down = runtime.state.is_pointer_down_on_content;
            let _ = down.try_write().map(|mut w| *w = true);
            set_body_user_select_none();
        }
    }

    pub fn content_pointer_enter(&self) -> impl FnMut(Event<PointerData>) + 'static {
        let runtime = self.clone();
        move |event| {
            if runtime.is_disabled() || event.pointer_type() == "touch" {
                return;
            }
            let mut ch = runtime.state.content_hovered;
            let _ = ch.try_write().map(|mut w| *w = true);
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
            let mut ch = runtime.state.content_hovered;
            let _ = ch.try_write().map(|mut w| *w = false);

            if runtime.state.opened_via_focus.try_read().is_ok_and(|f| *f)
                || runtime.has_selection()
                || runtime.is_pointer_down_on_content()
            {
                return;
            }

            let coords = event.client_coordinates();
            let exit_point = Point2D::new(coords.x, coords.y);
            if !runtime.start_hover_grace_corridor(exit_point, false)
                && !runtime.has_active_grace_monitor()
                && !runtime.state.trigger_hovered.try_read().is_ok_and(|h| *h)
            {
                runtime.schedule_close();
            }
        }
    }

    pub fn escape_keydown(&self) -> impl FnMut(Event<KeyboardData>) + 'static {
        let runtime = self.clone();
        move |event| {
            if is_ime_composing_key(&event) {
                return;
            }
            let _ = runtime.state.dismiss.handle_keydown(&event);
        }
    }

    pub fn dismiss_escape(&self) -> bool {
        if !self.is_live_open() || is_overlay_escape_consumed() {
            return false;
        }
        let effective_content_id = self.effective_content_id();
        if !is_topmost_overlay_layer(&effective_content_id) {
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
            .try_read()
            .map(|c| *c)
            .unwrap_or_default();
        let evt: EscapeKeyDownEvent = PreventableEvent::empty();
        if let Some(handler) = cfg.on_escape_key_down {
            handler.call(evt.clone());
        }
        if cfg.escape_key_down_behavior.dismisses() && !evt.is_default_prevented() {
            let change_event = ChangeEvent::new(
                false,
                ChangeContext::new(ChangeReason::EscapeKey, ChangeSource::Keyboard),
            );
            if !self.force_close_with_event(&change_event) {
                return false;
            }
            if !cfg.escape_key_down_behavior.bubbles() && !change_event.is_propagation_allowed() {
                mark_overlay_escape_consumed();
            }
            return true;
        }
        false
    }

    pub fn dismiss_outside_pointer(&self, target: Option<&str>) -> bool {
        if !self.is_live_open() {
            return false;
        }
        let target_owned = target.map(str::to_owned);
        let stack = self.dismiss_stack();
        let effective_content_id = self.effective_content_id();
        let branches_sig = self.state.dismiss.branches_signal();
        let should_dismiss_layer = self.state.live_hover_card.with_peek(|c| {
            let mut layer = c.lifecycle().dismiss_layer().clone();
            layer.set_id(effective_content_id.clone());
            if let Ok(branches) = branches_sig.try_peek() {
                for branch in branches.iter() {
                    layer.register_branch(branch.clone());
                }
            }
            layer.should_dismiss_outside_pointer(target_owned.as_ref(), &stack)
        });
        if !should_dismiss_layer {
            return false;
        }

        let cfg = self
            .state
            .outside_config
            .try_read()
            .map(|c| *c)
            .unwrap_or_default();
        let out_evt: PointerDownOutsideEvent = PreventableEvent::new(target_owned);
        if let Some(handler) = cfg.on_pointer_down_outside {
            handler.call(out_evt.clone());
        }
        if let Some(handler) = cfg.on_interact_outside {
            handler.call(out_evt.clone());
        }
        if cfg.interact_outside_behavior.dismisses() && !out_evt.is_default_prevented() {
            let change_event = ChangeEvent::new(
                false,
                ChangeContext::new(ChangeReason::OutsidePress, ChangeSource::Mouse),
            );
            if !self.force_close_with_event(&change_event) {
                return false;
            }
            if change_event.is_propagation_allowed() {
                mark_outside_press_propagating(&effective_content_id);
            }
            return true;
        }
        false
    }

    pub fn schedule_open(&self, immediate: bool) {
        self.schedule_open_with_context(
            immediate,
            ChangeContext::new(ChangeReason::Hover, ChangeSource::Mouse),
        );
    }

    fn schedule_open_with_context(&self, immediate: bool, ctx: ChangeContext) {
        if !self.is_alive() || self.is_disabled() {
            return;
        }
        self.cancel_close();
        if self.is_live_open() {
            return;
        }

        let open_delay_ms = self.open_delay_ms();
        if immediate || open_delay_ms == 0 {
            self.state.open_delay.cancel();
            self.commit_open_with_context(true, ctx);
            return;
        }

        let runtime = self.clone();
        self.state.open_delay.schedule(open_delay_ms, move || {
            if runtime.is_alive() && !runtime.is_disabled() {
                runtime.commit_open_with_context(true, ctx);
            }
        });
    }

    pub fn cancel_pending_open(&self) {
        self.state.open_delay.cancel();
    }

    pub fn cancel_close(&self) {
        self.state.close_delay.cancel();
    }

    pub fn schedule_close(&self) {
        self.schedule_close_with_context(ChangeContext::new(
            ChangeReason::Hover,
            ChangeSource::Mouse,
        ));
    }

    fn schedule_close_with_context(&self, ctx: ChangeContext) {
        if !self.is_alive() {
            return;
        }
        self.cancel_pending_open();

        if self.has_selection() || self.is_pointer_down_on_content() {
            return;
        }

        let close_delay_ms = self.close_delay_ms();
        if close_delay_ms == 0 {
            self.state.close_delay.cancel();
            self.try_commit_close_with_context(ctx);
            return;
        }

        let runtime = self.clone();
        self.state.close_delay.schedule(close_delay_ms, move || {
            if runtime.is_alive() {
                runtime.try_commit_close_with_context(ctx);
            }
        });
    }

    pub fn force_close(&self) {
        let change_event = ChangeEvent::new(
            false,
            ChangeContext::new(ChangeReason::Programmatic, ChangeSource::Programmatic),
        );
        let _ = self.force_close_with_event(&change_event);
    }

    fn force_close_with_event(&self, change_event: &ChangeEvent<bool>) -> bool {
        if !self.is_alive() {
            return false;
        }
        if self.is_live_open() && !self.commit_open_with_event(false, change_event) {
            return false;
        }
        reset_hover_and_selection_slots(
            self.state.content_hovered,
            self.state.opened_via_focus,
            self.state.is_pointer_down_on_content,
            self.state.has_selection,
            self.state.contain_selection,
        );
        self.cancel_pending_open();
        self.cancel_close();
        self.stop_grace_monitor();
        let cid = self.effective_content_id();
        restore_tab_order(&cid);
        true
    }

    fn try_commit_close_with_context(&self, ctx: ChangeContext) {
        let trigger_hovered = self.state.trigger_hovered.try_read().is_ok_and(|h| *h);
        let content_hovered = self.state.content_hovered.try_read().is_ok_and(|h| *h);
        let opened_via_focus = self.state.opened_via_focus.try_read().is_ok_and(|f| *f);

        if !trigger_hovered
            && !content_hovered
            && !opened_via_focus
            && !self.has_selection()
            && !self.is_pointer_down_on_content()
            && self.is_live_open()
        {
            self.commit_open_with_context(false, ctx);
        }
    }

    fn commit_open_with_context(&self, next_open: bool, ctx: ChangeContext) -> bool {
        let change_event = ChangeEvent::new(next_open, ctx);
        self.commit_open_with_event(next_open, &change_event)
    }

    fn commit_open_with_event(&self, next_open: bool, change_event: &ChangeEvent<bool>) -> bool {
        let mut desired = self.state.desired_open;
        if desired.try_peek().is_ok_and(|d| *d == next_open) {
            return true;
        }
        if !propose_hover_card_open(
            self.state.open_ctl,
            self.on_open_change,
            next_open,
            change_event,
        ) {
            return false;
        }
        let _ = desired.try_write().map(|mut w| *w = next_open);
        true
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

    fn effective_floating_layer(&self) -> FloatingLayer {
        let overrides = self
            .state
            .floating_overrides
            .try_peek()
            .map(|o| o.clone())
            .unwrap_or_default();
        compute_effective_floating_layer(&self.hover_card, &overrides)
    }

    fn dismiss_stack(&self) -> Vec<String> {
        let active = active_overlay_layers_snapshot();
        if active.is_empty() {
            vec![self.effective_content_id()]
        } else {
            active
        }
    }

    #[allow(dead_code)]
    pub(crate) fn focus_scope(&self) -> FocusScopeHandle {
        self.state.focus_scope
    }
}

fn arm_hover_card_pointer_up_observer(runtime: &HoverCardRuntime) {
    let rt = runtime.clone();
    runtime.state.pointer_up_watcher.arm_with(move |expected_gen| {
        let rt_cb = rt.clone();
        start_document_pointer_up_observer(move || {
            if !rt_cb.is_alive()
                || rt_cb.state.pointer_up_watcher.generation() != expected_gen
            {
                return;
            }
            let was_containing = rt_cb.state.contain_selection.try_peek().is_ok_and(|c| *c);
            let was_pointer_down = rt_cb
                .state
                .is_pointer_down_on_content
                .try_read()
                .is_ok_and(|p| *p);

            if was_containing {
                restore_body_user_select();
                let mut contain = rt_cb.state.contain_selection;
                sync_signal(&mut contain, false);
            }
            if was_pointer_down {
                let mut down = rt_cb.state.is_pointer_down_on_content;
                let _ = down.try_write().map(|mut w| *w = false);
            }

            if was_containing || was_pointer_down {
                let rt_frame = rt_cb.clone();
                spawn_root_task(async move {
                    let _ = oxidase::frame::next_frame().await;
                    if !rt_frame.is_alive() {
                        return;
                    }
                    let has_sel = document_has_text_selection();
                    let mut hs = rt_frame.state.has_selection;
                    let _ = hs.try_write().map(|mut w| *w = has_sel);
                    if !has_sel {
                        let trigger_hovered =
                            rt_frame.state.trigger_hovered.try_read().is_ok_and(|h| *h);
                        let content_hovered =
                            rt_frame.state.content_hovered.try_read().is_ok_and(|h| *h);
                        let opened_via_focus =
                            rt_frame.state.opened_via_focus.try_read().is_ok_and(|f| *f);
                        if !trigger_hovered && !content_hovered && !opened_via_focus {
                            rt_frame.schedule_close();
                        }
                    }
                });
            }
        })
    });
}

fn compute_effective_floating_layer(
    card: &HoverCard,
    overrides: &HoverCardFloatingOverrides,
) -> FloatingLayer {
    let base = card.lifecycle().floating().clone();
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
    if let Some(sticky) = overrides.sticky {
        layer = layer.with_sticky(sticky);
    }
    if let Some(hide_when_detached) = overrides.hide_when_detached {
        layer = layer.with_hide_when_detached(hide_when_detached);
    }
    if let Some(dir) = overrides.dir {
        layer = layer.with_direction(dir);
    }
    layer
}

fn reset_hover_and_selection_slots(
    mut content_hovered: CopyValue<bool>,
    mut opened_via_focus: CopyValue<bool>,
    mut is_pointer_down_on_content: CopyValue<bool>,
    mut has_selection: CopyValue<bool>,
    mut contain_selection: Signal<bool>,
) {
    let _ = content_hovered.try_write().map(|mut w| *w = false);
    let _ = opened_via_focus.try_write().map(|mut w| *w = false);
    let _ = is_pointer_down_on_content.try_write().map(|mut w| *w = false);
    let _ = has_selection.try_write().map(|mut w| *w = false);
    if contain_selection.try_peek().is_ok_and(|c| *c) {
        restore_body_user_select();
        sync_signal(&mut contain_selection, false);
    }
}

#[cfg(test)]
mod tests {
    use crate::foundation::overlay::{
        GeometryVars, Presence, PresenceController, PresenceState, serialize_css_custom_properties,
    };
    use crate::hover_card::{HOVER_CARD_GEOMETRY_NAMESPACE, HoverCard};

    #[test]
    fn hover_card_presence_lane_retains_content_until_close_completion() {
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

use std::{collections::HashMap, rc::Rc};

use dioxus::prelude::*;
use ui_sdk::kernel::{ChangeContext, ChangeEvent, ChangeReason, ChangeSource};

pub use crate::foundation::compose::{
    MountedHandle as PopoverMountedHandle, compose_part_event_handlers, compose_part_refs,
    project_as_child,
};

use crate::foundation::{
    browser::focus_mounted_handle,
    keyboard::is_ime_composing_key,
    overlay::{
        DismissBehavior, DismissCallbacks, DismissConfig, DismissEvent, DismissLayerHandle,
        FloatingHandle, FloatingMonitorSpec, FloatingPlacement, FloatingReadiness,
        FocusScopeConfig, FocusScopeHandle, GeometryVars, ModalLayerConfig, ModalLayerHandle,
        ModalShieldPolicy, OverlayEscapeGuardFn, OverlayLayerSpec, PlacementAlign, PlacementSide,
        PresenceHandle, Rect, Size, StickyStrategy, active_overlay_layers_snapshot,
        is_overlay_escape_consumed, is_topmost_overlay_layer, mark_outside_press_propagating,
        mark_overlay_escape_consumed, serialize_css_custom_properties, sync_overlay_layer_spec,
        use_dismissable_layer_with_details, use_floating_layer, use_focus_scope, use_modal_layer,
        use_presence_with_keep_mounted,
    },
    shared::PreventableEvent,
    state::{ControllableSignal, DataState, sync_signal},
};

use super::{
    attrs::{
        PopoverAnchorAttributes, PopoverArrowAttributes, PopoverCloseAttributes,
        PopoverContentAttributes, PopoverDescriptionAttributes, PopoverPortalAttributes,
        PopoverRootAttributes, PopoverTitleAttributes, PopoverTriggerAttributes,
    },
    relationships::PopoverRelationships,
    state::{Popover, PopoverLifecycle},
    types::{
        CloseAutoFocusEvent, EscapeKeyDownEvent, InteractOutsideEvent, OpenAutoFocusEvent,
        POPOVER_GEOMETRY_NAMESPACE, PointerDownOutsideEvent, PopoverCloseFocusPolicy,
        PopoverOpenFocusPolicy, PopoverOutsideInteractionPolicy, PopoverScrollLockPolicy,
        PopoverStateRequest,
    },
};

type PopoverOpenChangeHandler = Rc<dyn Fn(bool)>;
const POPOVER_RADIX_COMPATIBILITY_PREFIX: &str = "radix-popover";
const POPOVER_RADIX_ANCHOR_LABEL: &str = "trigger";

#[derive(Clone, Copy)]
struct PopoverRuntimeState {
    open_ctl: Option<ControllableSignal<bool>>,
    trigger_handle: Signal<Option<PopoverMountedHandle>>,
    anchor_handle: Signal<Option<PopoverMountedHandle>>,
    content_handle: Signal<Option<PopoverMountedHandle>>,
    focus_targets: Signal<HashMap<String, PopoverMountedHandle>>,
    presence: PresenceHandle,
    floating: FloatingHandle,
    dismiss: DismissLayerHandle,
    modal: ModalLayerHandle,
    focus_scope: FocusScopeHandle,
    last_open: CopyValue<bool>,
    last_dismiss_was_pointer: CopyValue<bool>,
    last_dismiss_was_outside_pointer: CopyValue<bool>,
    handle_trigger_id: Signal<Option<String>>,
    trigger_id: Signal<Option<String>>,
    content_id: Signal<Option<String>>,
    wrapper_id_override: Signal<Option<String>>,
    escape_guards: Signal<Vec<(String, OverlayEscapeGuardFn)>>,
    keep_mounted: Signal<bool>,
    side_override: Signal<Option<PlacementSide>>,
    align_override: Signal<Option<PlacementAlign>>,
    side_offset_override: Signal<Option<f32>>,
    align_offset_override: Signal<Option<f32>>,
    avoid_collisions_override: Signal<Option<bool>>,
    collision_boundary_override: Signal<Option<String>>,
    collision_padding_override: Signal<Option<f32>>,
    arrow_padding_override: Signal<Option<f32>>,
    sticky_override: Signal<Option<StickyStrategy>>,
    hide_when_detached_override: Signal<Option<bool>>,
    custom_anchor_override: Signal<Option<String>>,
    open_focus_override: Signal<Option<PopoverOpenFocusPolicy>>,
    close_focus_override: Signal<Option<PopoverCloseFocusPolicy>>,
    scroll_lock_override: Signal<Option<PopoverScrollLockPolicy>>,
    outside_interaction_override: Signal<Option<PopoverOutsideInteractionPolicy>>,
    prevent_body_pointer_events_override: Signal<Option<bool>>,
    escape_key_down_behavior: Signal<Option<DismissBehavior>>,
    interact_outside_behavior: Signal<Option<DismissBehavior>>,
    on_open_auto_focus: CopyValue<Option<EventHandler<OpenAutoFocusEvent>>>,
    on_close_auto_focus: CopyValue<Option<EventHandler<CloseAutoFocusEvent>>>,
    on_escape_key_down: CopyValue<Option<EventHandler<EscapeKeyDownEvent>>>,
    on_pointer_down_outside: CopyValue<Option<EventHandler<PointerDownOutsideEvent>>>,
    on_interact_outside: CopyValue<Option<EventHandler<InteractOutsideEvent>>>,
    on_open_change_complete: CopyValue<Option<EventHandler<bool>>>,
}

#[derive(Clone)]
pub struct PopoverRuntime {
    popover: Popover,
    on_open_change: CopyValue<PopoverOpenChangeHandler>,
    state: PopoverRuntimeState,
}

fn propose_popover_open(
    open_ctl: Option<ControllableSignal<bool>>,
    on_open_change: CopyValue<PopoverOpenChangeHandler>,
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

pub fn use_popover_runtime<F>(popover: Popover, on_open_change: F) -> PopoverRuntime
where
    F: Fn(bool) + 'static,
{
    use_popover_runtime_inner(popover, None, Rc::new(on_open_change), None, None)
}

pub(crate) fn use_popover_runtime_with_ctl(
    popover: Popover,
    open_ctl: ControllableSignal<bool>,
    handle_active_trigger_id: Option<String>,
    handle_active_trigger_mounted: Option<PopoverMountedHandle>,
) -> PopoverRuntime {
    let fallback: PopoverOpenChangeHandler = Rc::new(move |next_open: bool| {
        open_ctl.set(next_open);
    });
    use_popover_runtime_inner(
        popover,
        Some(open_ctl),
        fallback,
        handle_active_trigger_id,
        handle_active_trigger_mounted,
    )
}

fn use_popover_runtime_inner(
    popover: Popover,
    open_ctl: Option<ControllableSignal<bool>>,
    synced_open_change: PopoverOpenChangeHandler,
    handle_active_trigger_id: Option<String>,
    handle_active_trigger_mounted: Option<PopoverMountedHandle>,
) -> PopoverRuntime {
    let mut on_open_change_slot = use_hook(|| CopyValue::new(Rc::clone(&synced_open_change)));
    on_open_change_slot.set(synced_open_change);

    let trigger_handle = use_signal(|| None::<PopoverMountedHandle>);
    let anchor_handle = use_signal(|| None::<PopoverMountedHandle>);
    let content_handle = use_signal(|| None::<PopoverMountedHandle>);
    let focus_targets = use_signal(HashMap::new);
    let last_open = use_hook(|| CopyValue::new(popover.is_open()));
    let last_dismiss_was_pointer = use_hook(|| CopyValue::new(false));
    let last_dismiss_was_outside_pointer = use_hook(|| CopyValue::new(false));
    let mut handle_trigger_id = use_signal(|| handle_active_trigger_id.clone());
    sync_signal(&mut handle_trigger_id, handle_active_trigger_id.clone());
    let trigger_id = use_signal(|| None::<String>);
    let content_id = use_signal(|| None::<String>);
    let wrapper_id_override = use_signal(|| None::<String>);
    let escape_guards = use_signal(Vec::<(String, OverlayEscapeGuardFn)>::new);
    let keep_mounted = use_signal(|| false);

    let side_override = use_signal(|| None::<PlacementSide>);
    let align_override = use_signal(|| None::<PlacementAlign>);
    let side_offset_override = use_signal(|| None::<f32>);
    let align_offset_override = use_signal(|| None::<f32>);
    let avoid_collisions_override = use_signal(|| None::<bool>);
    let collision_boundary_override = use_signal(|| None::<String>);
    let collision_padding_override = use_signal(|| None::<f32>);
    let arrow_padding_override = use_signal(|| None::<f32>);
    let sticky_override = use_signal(|| None::<StickyStrategy>);
    let hide_when_detached_override = use_signal(|| None::<bool>);
    let custom_anchor_override = use_signal(|| None::<String>);

    let open_focus_override = use_signal(|| None::<PopoverOpenFocusPolicy>);
    let close_focus_override = use_signal(|| None::<PopoverCloseFocusPolicy>);
    let scroll_lock_override = use_signal(|| None::<PopoverScrollLockPolicy>);
    let outside_interaction_override = use_signal(|| None::<PopoverOutsideInteractionPolicy>);
    let prevent_body_pointer_events_override = use_signal(|| None::<bool>);
    let escape_key_down_behavior = use_signal(|| None::<DismissBehavior>);
    let interact_outside_behavior = use_signal(|| None::<DismissBehavior>);

    let on_open_auto_focus =
        use_hook(|| CopyValue::new(Option::<EventHandler<OpenAutoFocusEvent>>::None));
    let on_close_auto_focus =
        use_hook(|| CopyValue::new(Option::<EventHandler<CloseAutoFocusEvent>>::None));
    let on_escape_key_down =
        use_hook(|| CopyValue::new(Option::<EventHandler<EscapeKeyDownEvent>>::None));
    let on_pointer_down_outside =
        use_hook(|| CopyValue::new(Option::<EventHandler<PointerDownOutsideEvent>>::None));
    let on_interact_outside =
        use_hook(|| CopyValue::new(Option::<EventHandler<InteractOutsideEvent>>::None));
    let on_open_change_complete = use_hook(|| CopyValue::new(Option::<EventHandler<bool>>::None));

    let effective_content_id = content_id
        .read()
        .clone()
        .unwrap_or_else(|| popover.relationships().content_id().to_owned());
    let effective_trigger_id = handle_active_trigger_id
        .clone()
        .filter(|id| !id.is_empty())
        .or_else(|| trigger_id.read().clone())
        .unwrap_or_else(|| popover.relationships().trigger_id().to_owned());
    let custom_anchor = custom_anchor_override
        .read()
        .clone()
        .filter(|id| !id.is_empty());
    let effective_anchor_id = if let Some(anchor_id) = custom_anchor {
        anchor_id
    } else if anchor_handle.read().is_some() {
        popover.relationships().anchor_id().to_owned()
    } else {
        effective_trigger_id.clone()
    };
    let effective_wrapper_id = wrapper_id_override
        .read()
        .clone()
        .unwrap_or_else(|| format!("{effective_content_id}-wrapper"));

    let on_presence_complete = use_hook(move || {
        EventHandler::new(move |open: bool| {
            if let Some(cb) = on_open_change_complete.try_read().ok().and_then(|s| *s) {
                cb.call(open);
            }
        })
    });

    let presence = use_presence_with_keep_mounted(
        popover.is_open(),
        &effective_content_id,
        popover.lifecycle().presence().retain_mount(),
        *keep_mounted.read(),
        Some(on_presence_complete),
    );

    let floating_cfg = popover.lifecycle().floating();
    let mut floating_spec =
        FloatingMonitorSpec::new(effective_anchor_id.clone(), effective_content_id.clone());
    floating_spec.arrow_id = Some(popover.relationships().arrow_id().to_owned());
    floating_spec.boundary_id = collision_boundary_override.read().clone();
    floating_spec.side = (*side_override.read()).unwrap_or_else(|| floating_cfg.side());
    floating_spec.align = (*align_override.read()).unwrap_or_else(|| floating_cfg.align());
    floating_spec.direction = floating_cfg.direction();
    floating_spec.side_offset =
        (*side_offset_override.read()).unwrap_or_else(|| floating_cfg.side_offset());
    floating_spec.align_offset =
        (*align_offset_override.read()).unwrap_or_else(|| floating_cfg.align_offset());
    floating_spec.avoid_collisions =
        (*avoid_collisions_override.read()).unwrap_or_else(|| floating_cfg.avoid_collisions());
    floating_spec.collision_padding =
        (*collision_padding_override.read()).unwrap_or_else(|| floating_cfg.collision_padding());
    floating_spec.arrow_padding =
        (*arrow_padding_override.read()).unwrap_or_else(|| floating_cfg.arrow_padding());
    floating_spec.hide_when_detached =
        (*hide_when_detached_override.read()).unwrap_or_else(|| floating_cfg.hide_when_detached());
    floating_spec.sticky =
        (*sticky_override.read()).unwrap_or_else(|| floating_cfg.sticky()) == StickyStrategy::Always;
    floating_spec.namespace = Some(POPOVER_GEOMETRY_NAMESPACE.to_owned());

    let floating = use_floating_layer(
        popover.is_open() && presence.should_render(),
        floating_spec,
        None,
    );
    if let Some(mounted) = handle_active_trigger_mounted
        && anchor_handle.try_peek().is_ok_and(|a| a.is_none())
    {
        floating.capture_anchor(mounted);
    }

    let mut dismiss_boundaries = vec![
        effective_content_id.clone(),
        effective_trigger_id.clone(),
        popover.relationships().anchor_id().to_owned(),
        popover.lifecycle().focus_guards().before().to_owned(),
        popover.lifecycle().focus_guards().after().to_owned(),
    ];
    if !dismiss_boundaries.contains(&effective_anchor_id) {
        dismiss_boundaries.push(effective_anchor_id);
    }
    for b in popover.lifecycle().dismiss_layer().branches() {
        if !dismiss_boundaries.contains(b) {
            dismiss_boundaries.push(b.clone());
        }
    }

    let is_modal = popover.lifecycle().is_modal();
    let effective_scroll_lock = scroll_lock_override
        .read()
        .clone()
        .unwrap_or_else(|| popover.lifecycle().scroll_lock_policy().clone());
    let effective_pbpe = prevent_body_pointer_events_override.read().unwrap_or(true);
    let outside_policy = outside_interaction_override
        .read()
        .unwrap_or_else(|| *popover.lifecycle().outside_interaction_policy());
    let escape_behavior = escape_key_down_behavior
        .read()
        .unwrap_or(DismissBehavior::Close);
    let outside_behavior = interact_outside_behavior.read().unwrap_or_else(|| {
        if outside_policy.pointer_down_outside().dismisses() {
            DismissBehavior::Close
        } else {
            DismissBehavior::Ignore
        }
    });

    let dismiss_config = DismissConfig::new(effective_content_id.clone())
        .with_boundaries(dismiss_boundaries)
        .with_modal(is_modal)
        .with_disable_outside_pointer_events(
            is_modal && effective_scroll_lock.is_enabled() && effective_pbpe,
        )
        .with_listen_escape(escape_behavior.dismisses())
        .with_listen_pointer_down_outside(outside_behavior.dismisses())
        .with_listen_focus_outside(
            outside_behavior.dismisses() && outside_policy.focus_outside().dismisses(),
        );

    let on_escape_bridge = use_hook(move || {
        EventHandler::new(move |evt: EscapeKeyDownEvent| {
            if run_popover_escape_guards(escape_guards) {
                evt.prevent_default();
                return;
            }
            if let Some(handler) = on_escape_key_down.try_read().ok().and_then(|s| *s) {
                handler.call(evt);
            }
        })
    });
    let on_pointer_down_outside_bridge = use_hook(move || {
        EventHandler::new(move |evt: PointerDownOutsideEvent| {
            if let Some(handler) = on_pointer_down_outside.try_read().ok().and_then(|s| *s) {
                handler.call(evt);
            }
        })
    });
    let on_interact_bridge = use_hook(move || {
        EventHandler::new(move |evt: InteractOutsideEvent| {
            if let Some(handler) = on_interact_outside.try_read().ok().and_then(|s| *s) {
                handler.call(evt);
            }
        })
    });
    let on_dismiss_details_bridge = use_hook(move || {
        EventHandler::new(move |(event, change_event): (DismissEvent, ChangeEvent<bool>)| {
            let accepted =
                propose_popover_open(open_ctl, on_open_change_slot, false, &change_event);
            if accepted {
                let is_pointer = matches!(event, DismissEvent::PointerDownOutside);
                let mut ptr = last_dismiss_was_pointer;
                let _ = ptr.try_write().map(|mut w| *w = is_pointer);
                let mut out_ptr = last_dismiss_was_outside_pointer;
                let _ = out_ptr.try_write().map(|mut w| *w = is_pointer);
            }
        })
    });

    let dismiss = use_dismissable_layer_with_details(
        popover.is_open(),
        dismiss_config,
        DismissCallbacks {
            on_escape_key_down: Some(on_escape_bridge),
            on_pointer_down_outside: Some(on_pointer_down_outside_bridge),
            on_focus_outside: None,
            on_interact_outside: Some(on_interact_bridge),
            on_dismiss: None,
        },
        Some(on_dismiss_details_bridge),
    );

    if popover.is_open() && !effective_content_id.is_empty() {
        sync_overlay_layer_spec(
            OverlayLayerSpec::new(effective_content_id.clone())
                .with_modal(is_modal)
                .with_escape_behavior(escape_behavior)
                .with_outside_press_behavior(outside_behavior),
        );
    }

    let dynamic_branches = dismiss.branches_signal().read().clone();
    let mut modal_keep_ids = vec![effective_wrapper_id, effective_content_id.clone()];
    for branch in popover.lifecycle().dismiss_layer().branches() {
        if !modal_keep_ids.contains(branch) {
            modal_keep_ids.push(branch.clone());
        }
    }
    for branch in &dynamic_branches {
        if !modal_keep_ids.contains(branch) {
            modal_keep_ids.push(branch.clone());
        }
    }
    let modal_config = ModalLayerConfig {
        lock_id: popover.relationships().root_id().to_owned(),
        keep_ids: modal_keep_ids,
        shield_policy: if is_modal {
            ModalShieldPolicy::InertAndAriaHidden
        } else {
            ModalShieldPolicy::None
        },
        prevent_scroll: is_modal && effective_scroll_lock.is_enabled(),
        prevent_body_pointer_events: effective_pbpe,
        restore_delay: effective_scroll_lock.restore_delay(),
    };
    let modal = use_modal_layer(popover.is_open() && is_modal, modal_config);

    let open_focus_policy = open_focus_override
        .read()
        .clone()
        .unwrap_or_else(|| popover.lifecycle().open_focus_policy().clone());
    let close_focus_policy = close_focus_override
        .read()
        .clone()
        .unwrap_or_else(|| popover.lifecycle().close_focus_policy().clone());
    let mut focus_config = FocusScopeConfig::new(effective_content_id.clone());
    focus_config.trap_focus = popover.lifecycle().focus_scope().traps_focus();
    focus_config.loop_focus = popover.lifecycle().focus_scope().loops_focus();
    focus_config.auto_focus_on_open = popover.lifecycle().focus_scope().autofocus_enabled()
        && !matches!(open_focus_policy, PopoverOpenFocusPolicy::Suppress);
    focus_config.restore_focus_on_close =
        !matches!(close_focus_policy, PopoverCloseFocusPolicy::None);
    focus_config.open_focus_target_id = match &open_focus_policy {
        PopoverOpenFocusPolicy::Target(target) => Some(target.clone()),
        _ => None,
    };
    focus_config.focusable_selector = match &open_focus_policy {
        PopoverOpenFocusPolicy::Selector(selector) => Some(selector.clone()),
        _ => None,
    };
    focus_config.restore_target_id = match &close_focus_policy {
        PopoverCloseFocusPolicy::Trigger => Some(effective_trigger_id),
        PopoverCloseFocusPolicy::Target(target) => Some(target.clone()),
        PopoverCloseFocusPolicy::None => None,
    };

    let on_open_focus_bridge = use_hook(move || {
        EventHandler::new(move |evt: OpenAutoFocusEvent| {
            if let Some(handler) = on_open_auto_focus.try_read().ok().and_then(|s| *s) {
                handler.call(evt);
            }
        })
    });
    let on_close_focus_bridge = use_hook(move || {
        EventHandler::new(move |evt: CloseAutoFocusEvent| {
            if let Some(handler) = on_close_auto_focus.try_read().ok().and_then(|s| *s) {
                handler.call(evt);
            }
        })
    });

    let focus_scope = use_focus_scope(
        popover.is_open(),
        focus_config,
        Some(on_open_focus_bridge),
        Some(on_close_focus_bridge),
    );
    if matches!(close_focus_policy, PopoverCloseFocusPolicy::Trigger)
        && let Some(ref active_tid) = handle_active_trigger_id
        && !active_tid.is_empty()
    {
        focus_scope.set_restore_target(Some(active_tid.clone()));
    }

    let state = PopoverRuntimeState {
        open_ctl,
        trigger_handle,
        anchor_handle,
        content_handle,
        focus_targets,
        presence,
        floating,
        dismiss,
        modal,
        focus_scope,
        last_open,
        last_dismiss_was_pointer,
        last_dismiss_was_outside_pointer,
        handle_trigger_id,
        trigger_id,
        content_id,
        wrapper_id_override,
        escape_guards,
        keep_mounted,
        side_override,
        align_override,
        side_offset_override,
        align_offset_override,
        avoid_collisions_override,
        collision_boundary_override,
        collision_padding_override,
        arrow_padding_override,
        sticky_override,
        hide_when_detached_override,
        custom_anchor_override,
        open_focus_override,
        close_focus_override,
        scroll_lock_override,
        outside_interaction_override,
        prevent_body_pointer_events_override,
        escape_key_down_behavior,
        interact_outside_behavior,
        on_open_auto_focus,
        on_close_auto_focus,
        on_escape_key_down,
        on_pointer_down_outside,
        on_interact_outside,
        on_open_change_complete,
    };

    // Coordinate open->closed focus restoration and post-exit floating state cleanup.
    let is_open_now = popover.is_open();
    let should_render_now = presence.should_render();
    let close_policy_for_effect = close_focus_policy;
    let is_modal_for_effect = is_modal;
    use_effect(use_reactive(
        (&is_open_now, &should_render_now),
        move |(open_now, render_now)| {
            let was_open = state.last_open.try_read().map(|g| *g).unwrap_or(false);
            if open_now && !was_open {
                let mut ptr = state.last_dismiss_was_pointer;
                let _ = ptr.try_write().map(|mut w| *w = false);
                let mut out_ptr = state.last_dismiss_was_outside_pointer;
                let _ = out_ptr.try_write().map(|mut w| *w = false);
            } else if !open_now && was_open {
                let was_pointer = state
                    .last_dismiss_was_pointer
                    .try_read()
                    .map(|g| *g)
                    .unwrap_or(false);
                let was_outside_pointer = state
                    .last_dismiss_was_outside_pointer
                    .try_read()
                    .map(|g| *g)
                    .unwrap_or(false);
                let mut ptr = state.last_dismiss_was_pointer;
                let _ = ptr.try_write().map(|mut w| *w = false);
                let mut out_ptr = state.last_dismiss_was_outside_pointer;
                let _ = out_ptr.try_write().map(|mut w| *w = false);

                let skip_non_modal_trigger_restore = was_outside_pointer
                    && !is_modal_for_effect
                    && matches!(close_policy_for_effect, PopoverCloseFocusPolicy::Trigger);
                if skip_non_modal_trigger_restore {
                    if let Some(handler) =
                        state.on_close_auto_focus.try_read().ok().and_then(|s| *s)
                    {
                        let evt: CloseAutoFocusEvent = PreventableEvent::empty();
                        handler.call(evt);
                    }
                } else {
                    state.focus_scope.apply_close_focus(was_pointer);
                }
            }

            if was_open != open_now {
                let mut lo = state.last_open;
                let _ = lo.try_write().map(|mut w| *w = open_now);
            }

            if !open_now && !render_now {
                let mut ch = state.content_handle;
                if ch.try_peek().is_ok_and(|h| h.is_some())
                    && let Ok(mut w) = ch.try_write()
                {
                    *w = None;
                }
                state.floating.clear();
            }
        },
    ));

    PopoverRuntime {
        popover,
        on_open_change: on_open_change_slot,
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
        let mut attrs = self.popover.trigger();
        let content_id = self
            .state
            .content_id
            .read()
            .clone()
            .unwrap_or_else(|| self.relationships().content_id().to_string());
        attrs.target_content_id = content_id.clone();
        attrs.aria_controls = if self.is_open() || self.should_render_content() {
            Some(content_id)
        } else {
            None
        };
        attrs
    }

    pub fn anchor(&self) -> PopoverAnchorAttributes {
        self.popover.anchor()
    }

    pub fn portal(&self) -> PopoverPortalAttributes {
        self.popover.portal()
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

    pub fn is_hidden(&self) -> bool {
        self.state.presence.is_hidden()
    }

    pub fn set_keep_mounted(&self, keep_mounted: bool) {
        let mut sig = self.state.keep_mounted;
        sync_signal(&mut sig, keep_mounted);
    }

    pub fn effective_arrow_padding(&self) -> f32 {
        (*self.state.arrow_padding_override.read())
            .unwrap_or_else(|| self.popover.lifecycle().floating().arrow_padding())
    }

    pub fn content(&self) -> PopoverContentAttributes {
        let mut content = self.popover.content();
        if let Some(placement) = self.placement() {
            content.data_side = placement.side().as_str();
            content.data_align = placement.align().as_str();
        } else {
            if let Some(side) = *self.state.side_override.read() {
                content.data_side = side.as_str();
            }
            if let Some(align) = *self.state.align_override.read() {
                content.data_align = align.as_str();
            }
        }
        content
    }

    pub fn arrow(&self) -> PopoverArrowAttributes {
        let mut arrow = self.popover.arrow();
        if let Some(placement) = self.placement() {
            arrow.data_side = placement.side().as_str();
            arrow.data_align = placement.align().as_str();
        } else {
            if let Some(side) = *self.state.side_override.read() {
                arrow.data_side = side.as_str();
            }
            if let Some(align) = *self.state.align_override.read() {
                arrow.data_align = align.as_str();
            }
        }
        arrow
    }

    pub fn close(&self) -> PopoverCloseAttributes {
        self.popover.close()
    }

    pub fn title(&self) -> PopoverTitleAttributes {
        self.popover.title()
    }

    pub fn description(&self) -> PopoverDescriptionAttributes {
        self.popover.description()
    }

    pub fn geometry_vars(&self, anchor: Rect, content: Size) -> GeometryVars {
        self.popover.geometry_vars(anchor, content)
    }

    pub fn content_readiness(&self) -> FloatingReadiness {
        self.state.floating.readiness()
    }

    pub fn content_positioning_state(&self) -> &'static str {
        self.content_readiness().positioning_state()
    }

    pub fn content_css_variables(&self) -> Vec<(String, String)> {
        self.placement()
            .map(|placement| popover_content_css_variables(placement.geometry()))
            .unwrap_or_default()
    }

    pub fn content_css_custom_properties(&self) -> String {
        let vars = self.content_css_variables();
        if vars.is_empty() {
            String::new()
        } else {
            serialize_css_custom_properties(&vars)
        }
    }

    pub fn wrapper_style(&self) -> String {
        self.state.floating.wrapper_style()
    }

    pub fn clear_content_z_index(&self) {
        self.state.floating.clear_z_index();
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
        self.state.floating.placement()
    }

    pub fn mounted_focus_target(&self, id: &str) -> Option<PopoverMountedHandle> {
        self.state
            .focus_targets
            .with_peek(|targets| targets.get(id).cloned())
    }

    pub fn request_state(&self, request: PopoverStateRequest) -> bool {
        self.request_state_with_context(
            request,
            ChangeContext::new(ChangeReason::Programmatic, ChangeSource::Programmatic),
        )
    }

    pub fn request_state_with_context(
        &self,
        request: PopoverStateRequest,
        ctx: ChangeContext,
    ) -> bool {
        let next_open = request.next_open(self.is_open());
        if next_open != self.is_open() {
            let change_event = ChangeEvent::new(next_open, ctx);
            if propose_popover_open(
                self.state.open_ctl,
                self.on_open_change,
                next_open,
                &change_event,
            ) {
                return next_open;
            }
            return self.is_open();
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

    pub fn register_trigger_id(&self, id: impl Into<String>) {
        let mut trigger_id = self.state.trigger_id;
        sync_signal(&mut trigger_id, Some(id.into()));
    }

    pub fn unregister_trigger_id(&self, id: &str) {
        let mut trigger_id = self.state.trigger_id;
        if trigger_id.try_peek().is_ok_and(|c| c.as_deref() == Some(id))
            && let Ok(mut current) = trigger_id.try_write()
        {
            *current = None;
        }
    }

    pub fn register_content_id(&self, id: impl Into<String>) {
        let id = id.into();
        let mut content_id = self.state.content_id;
        sync_signal(&mut content_id, Some(id.clone()));
        self.state.dismiss.rename_layer_id(&id);
        self.state.focus_scope.rename_container_id(&id);
    }

    pub fn unregister_content_id(&self, id: &str) {
        let mut content_id = self.state.content_id;
        if content_id.try_peek().is_ok_and(|c| c.as_deref() == Some(id))
            && let Ok(mut current) = content_id.try_write()
        {
            *current = None;
        }
    }

    pub fn register_arrow_id(&self, id: impl Into<String>) {
        self.state.floating.register_arrow_id(id);
    }

    pub fn unregister_arrow_id(&self, id: &str) {
        self.state.floating.unregister_arrow_id(id);
    }

    pub fn register_branch(&self, id: impl Into<String>) {
        self.state.dismiss.register_branch(id);
    }

    pub fn unregister_branch(&self, id: &str) {
        self.state.dismiss.unregister_branch(id);
    }

    pub fn branches(&self) -> Vec<String> {
        self.state.dismiss.branches_signal().cloned()
    }

    pub fn register_escape_guard(&self, id: impl Into<String>, guard: OverlayEscapeGuardFn) {
        let id = id.into();
        let mut guards = self.state.escape_guards;
        if let Ok(mut list) = guards.try_write() {
            list.retain(|(existing_id, _)| existing_id != &id);
            list.push((id, guard));
        }
    }

    pub fn unregister_escape_guard(&self, id: &str) {
        let mut guards = self.state.escape_guards;
        if let Ok(mut list) = guards.try_write() {
            list.retain(|(existing_id, _)| existing_id != id);
        }
    }

    pub fn run_escape_guards(&self) -> bool {
        run_popover_escape_guards(self.state.escape_guards)
    }

    pub fn should_force_pointer_events_auto(&self) -> bool {
        self.state.dismiss.should_force_pointer_events_auto()
            || self.state.modal.should_force_pointer_events_auto()
    }

    pub(crate) fn sync_modal_shield_with_wrapper(&self, wrapper_id: Option<&str>) {
        let mut sig = self.state.wrapper_id_override;
        sync_signal(&mut sig, wrapper_id.map(str::to_owned));
    }

    #[allow(clippy::too_many_arguments)]
    pub fn set_content_placement_overrides(
        &self,
        side: Option<PlacementSide>,
        align: Option<PlacementAlign>,
        side_offset: Option<f32>,
        align_offset: Option<f32>,
        avoid_collisions: Option<bool>,
        collision_boundary: Option<String>,
        collision_padding: Option<f32>,
        arrow_padding: Option<f32>,
        sticky: Option<StickyStrategy>,
        hide_when_detached: Option<bool>,
        custom_anchor: Option<String>,
    ) {
        let mut s = self.state.side_override;
        sync_signal(&mut s, side);
        let mut a = self.state.align_override;
        sync_signal(&mut a, align);
        let mut so = self.state.side_offset_override;
        sync_signal(&mut so, side_offset);
        let mut ao = self.state.align_offset_override;
        sync_signal(&mut ao, align_offset);
        let mut ac = self.state.avoid_collisions_override;
        sync_signal(&mut ac, avoid_collisions);
        let mut cb = self.state.collision_boundary_override;
        sync_signal(&mut cb, collision_boundary);
        let mut cp = self.state.collision_padding_override;
        sync_signal(&mut cp, collision_padding);
        let mut ap = self.state.arrow_padding_override;
        sync_signal(&mut ap, arrow_padding);
        let mut st = self.state.sticky_override;
        sync_signal(&mut st, sticky);
        let mut hwd = self.state.hide_when_detached_override;
        sync_signal(&mut hwd, hide_when_detached);
        let mut ca = self.state.custom_anchor_override;
        sync_signal(&mut ca, custom_anchor);
    }

    #[allow(clippy::too_many_arguments)]
    pub fn set_content_policy_overrides(
        &self,
        open_focus: Option<PopoverOpenFocusPolicy>,
        close_focus: Option<PopoverCloseFocusPolicy>,
        scroll_lock: Option<PopoverScrollLockPolicy>,
        outside_interaction: Option<PopoverOutsideInteractionPolicy>,
        prevent_body_pointer_events: Option<bool>,
        escape_key_down_behavior: Option<DismissBehavior>,
        interact_outside_behavior: Option<DismissBehavior>,
    ) {
        let mut of = self.state.open_focus_override;
        sync_signal(&mut of, open_focus);
        let mut cf = self.state.close_focus_override;
        sync_signal(&mut cf, close_focus);
        let mut sl = self.state.scroll_lock_override;
        sync_signal(&mut sl, scroll_lock);
        let mut oi = self.state.outside_interaction_override;
        sync_signal(&mut oi, outside_interaction);
        let mut pbpe = self.state.prevent_body_pointer_events_override;
        sync_signal(&mut pbpe, prevent_body_pointer_events);
        let mut eb = self.state.escape_key_down_behavior;
        sync_signal(&mut eb, escape_key_down_behavior);
        let mut ob = self.state.interact_outside_behavior;
        sync_signal(&mut ob, interact_outside_behavior);
    }

    pub fn set_content_event_handlers(
        &self,
        on_open_auto_focus: Option<EventHandler<OpenAutoFocusEvent>>,
        on_close_auto_focus: Option<EventHandler<CloseAutoFocusEvent>>,
        on_escape_key_down: Option<EventHandler<EscapeKeyDownEvent>>,
        on_pointer_down_outside: Option<EventHandler<PointerDownOutsideEvent>>,
        on_interact_outside: Option<EventHandler<InteractOutsideEvent>>,
    ) {
        let mut oa = self.state.on_open_auto_focus;
        let _ = oa.try_write().map(|mut w| *w = on_open_auto_focus);
        let mut ca = self.state.on_close_auto_focus;
        let _ = ca.try_write().map(|mut w| *w = on_close_auto_focus);
        let mut ek = self.state.on_escape_key_down;
        let _ = ek.try_write().map(|mut w| *w = on_escape_key_down);
        let mut pd = self.state.on_pointer_down_outside;
        let _ = pd.try_write().map(|mut w| *w = on_pointer_down_outside);
        let mut io = self.state.on_interact_outside;
        let _ = io.try_write().map(|mut w| *w = on_interact_outside);
    }

    pub fn set_on_open_change_complete(&self, handler: Option<EventHandler<bool>>) {
        let mut slot = self.state.on_open_change_complete;
        let _ = slot.try_write().map(|mut w| *w = handler);
    }

    pub fn capture_trigger(&self, mounted: PopoverMountedHandle) {
        let mut trigger_handle = self.state.trigger_handle;
        let _ = trigger_handle.try_write().map(|mut w| *w = Some(mounted.clone()));
        self.capture_focus_target(effective_trigger_id(&self.popover, self.state), mounted.clone());
        if self.state.anchor_handle.try_peek().is_ok_and(|a| a.is_none()) {
            self.state.floating.capture_anchor(mounted);
        }
    }

    pub fn capture_anchor(&self, mounted: PopoverMountedHandle) {
        let mut anchor_handle = self.state.anchor_handle;
        let _ = anchor_handle.try_write().map(|mut w| *w = Some(mounted.clone()));
        self.state.floating.capture_anchor(mounted);
    }

    pub fn capture_content(&self, mounted: PopoverMountedHandle) {
        let mut content_handle = self.state.content_handle;
        let _ = content_handle.try_write().map(|mut w| *w = Some(mounted.clone()));
        self.capture_focus_target(effective_content_id(&self.popover, self.state), mounted.clone());
        self.state.floating.capture_content(mounted);
    }

    pub fn capture_close(&self, mounted: PopoverMountedHandle) {
        self.capture_focus_target(self.relationships().close_id().to_owned(), mounted);
    }

    pub fn capture_focus_target(&self, id: impl Into<String>, mounted: PopoverMountedHandle) {
        let id = id.into();
        let mut focus_targets = self.state.focus_targets;
        if let Ok(mut targets) = focus_targets.try_write() {
            targets.insert(id, mounted);
        }
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
            if is_ime_composing_key(&event) {
                return;
            }
            if runtime.state.focus_scope.handle_tab_keydown(&event) {
                return;
            }
            let _ = runtime.state.dismiss.handle_keydown(&event);
        }
    }

    pub fn dismiss_escape(&self) -> bool {
        if is_overlay_escape_consumed() {
            return false;
        }
        let escape_behavior = self
            .state
            .escape_key_down_behavior
            .try_peek()
            .ok()
            .and_then(|b| *b)
            .unwrap_or(DismissBehavior::Close);
        if !escape_behavior.dismisses() {
            return false;
        }
        let content_id = effective_content_id(&self.popover, self.state);
        if !is_topmost_overlay_layer(content_id.as_str()) {
            return false;
        }
        let stack = self.dismiss_stack();
        let dismiss_layer = effective_dismiss_layer(&self.popover, self.state);
        if !dismiss_layer.should_dismiss_escape(&stack) {
            return false;
        }
        if self.run_escape_guards() {
            return false;
        }
        if let Some(handler) = self.state.on_escape_key_down.try_read().ok().and_then(|h| *h) {
            let evt: EscapeKeyDownEvent = PreventableEvent::empty();
            handler.call(evt.clone());
            if evt.is_default_prevented() {
                mark_overlay_escape_consumed();
                return false;
            }
        }

        let change_event = ChangeEvent::new(
            false,
            ChangeContext::new(ChangeReason::EscapeKey, ChangeSource::Keyboard),
        );
        if !propose_popover_open(self.state.open_ctl, self.on_open_change, false, &change_event) {
            return false;
        }
        if !escape_behavior.bubbles() && !change_event.is_propagation_allowed() {
            mark_overlay_escape_consumed();
        }
        let mut ptr = self.state.last_dismiss_was_pointer;
        let _ = ptr.try_write().map(|mut w| *w = false);
        let mut out_ptr = self.state.last_dismiss_was_outside_pointer;
        let _ = out_ptr.try_write().map(|mut w| *w = false);
        true
    }

    pub fn dismiss_outside_pointer(&self, target: Option<&str>) -> bool {
        let outside_behavior = self
            .state
            .interact_outside_behavior
            .try_peek()
            .ok()
            .and_then(|b| *b);
        if let Some(b) = outside_behavior
            && !b.dismisses()
        {
            return false;
        }
        let target_owned = target.map(str::to_owned);
        let stack = self.dismiss_stack();
        let dismiss_layer = effective_dismiss_layer(&self.popover, self.state);
        if dismiss_layer.should_dismiss_outside_pointer(target_owned.as_ref(), &stack) {
            let evt: PointerDownOutsideEvent = PreventableEvent::new(target_owned);
            if let Some(handler) = self
                .state
                .on_pointer_down_outside
                .try_read()
                .ok()
                .and_then(|h| *h)
            {
                handler.call(evt.clone());
            }
            if let Some(handler) = self.state.on_interact_outside.try_read().ok().and_then(|h| *h) {
                handler.call(evt.clone());
            }
            if evt.is_default_prevented() {
                return false;
            }
            let change_event = ChangeEvent::new(
                false,
                ChangeContext::new(ChangeReason::OutsidePress, ChangeSource::Mouse),
            );
            if !propose_popover_open(self.state.open_ctl, self.on_open_change, false, &change_event)
            {
                return false;
            }
            if change_event.is_propagation_allowed() {
                let cid = effective_content_id(&self.popover, self.state);
                mark_outside_press_propagating(&cid);
            }
            let mut ptr = self.state.last_dismiss_was_pointer;
            let _ = ptr.try_write().map(|mut w| *w = true);
            let mut out_ptr = self.state.last_dismiss_was_outside_pointer;
            let _ = out_ptr.try_write().map(|mut w| *w = true);
            return true;
        }
        false
    }

    pub fn dismiss_outside_focus(&self, target: Option<&str>) -> bool {
        let outside_behavior = self
            .state
            .interact_outside_behavior
            .try_peek()
            .ok()
            .and_then(|b| *b);
        if let Some(b) = outside_behavior
            && !b.dismisses()
        {
            return false;
        }
        let target_owned = target.map(str::to_owned);
        let stack = self.dismiss_stack();
        let dismiss_layer = effective_dismiss_layer(&self.popover, self.state);
        if dismiss_layer.should_dismiss_outside_focus(target_owned.as_ref(), &stack) {
            if let Some(handler) = self.state.on_interact_outside.try_read().ok().and_then(|h| *h) {
                let evt: InteractOutsideEvent = PreventableEvent::new(target_owned);
                handler.call(evt.clone());
                if evt.is_default_prevented() {
                    return false;
                }
            }
            let change_event = ChangeEvent::new(
                false,
                ChangeContext::new(ChangeReason::FocusOut, ChangeSource::Programmatic),
            );
            if !propose_popover_open(self.state.open_ctl, self.on_open_change, false, &change_event)
            {
                return false;
            }
            if change_event.is_propagation_allowed() {
                let cid = effective_content_id(&self.popover, self.state);
                mark_outside_press_propagating(&cid);
            }
            let mut ptr = self.state.last_dismiss_was_pointer;
            let _ = ptr.try_write().map(|mut w| *w = false);
            let mut out_ptr = self.state.last_dismiss_was_outside_pointer;
            let _ = out_ptr.try_write().map(|mut w| *w = false);
            return true;
        }
        false
    }

    pub fn trigger_pointer_down(&self) -> impl FnMut(Event<PointerData>) + 'static {
        let runtime = self.clone();
        move |_| {
            if runtime.is_open() {
                let mut ptr = runtime.state.last_dismiss_was_pointer;
                let _ = ptr.try_write().map(|mut w| *w = true);
                let mut out_ptr = runtime.state.last_dismiss_was_outside_pointer;
                let _ = out_ptr.try_write().map(|mut w| *w = false);
            }
        }
    }

    pub fn close_pointer_down(&self) -> impl FnMut(Event<PointerData>) + 'static {
        let runtime = self.clone();
        move |_| {
            let mut ptr = runtime.state.last_dismiss_was_pointer;
            let _ = ptr.try_write().map(|mut w| *w = true);
            let mut out_ptr = runtime.state.last_dismiss_was_outside_pointer;
            let _ = out_ptr.try_write().map(|mut w| *w = false);
        }
    }

    pub fn trigger_click(&self) -> impl FnMut(Event<MouseData>) + 'static {
        let runtime = self.clone();
        move |_| {
            runtime.request_state_with_context(
                PopoverStateRequest::Toggle,
                ChangeContext::new(ChangeReason::Activation, ChangeSource::Mouse),
            );
        }
    }

    pub fn close_click(&self) -> impl FnMut(Event<MouseData>) + 'static {
        let runtime = self.clone();
        move |_| {
            runtime.request_state_with_context(
                PopoverStateRequest::Close,
                ChangeContext::new(ChangeReason::Close, ChangeSource::Mouse),
            );
        }
    }

    #[allow(dead_code)]
    pub(crate) fn focus_registered_target(&self, target: &str) -> bool {
        focus_mounted_handle(self.mounted_focus_target(target))
    }

    fn dismiss_stack(&self) -> Vec<String> {
        let active = active_overlay_layers_snapshot();
        if active.is_empty() {
            vec![effective_content_id(&self.popover, self.state)]
        } else {
            active
        }
    }
}

fn effective_dismiss_layer(
    popover: &Popover,
    state: PopoverRuntimeState,
) -> crate::foundation::overlay::DismissLayer<String> {
    let mut layer = popover.lifecycle().dismiss_layer().clone();
    layer.set_id(effective_content_id(popover, state));
    layer
}

fn effective_trigger_id(popover: &Popover, state: PopoverRuntimeState) -> String {
    state
        .handle_trigger_id
        .try_peek()
        .ok()
        .and_then(|id| id.clone())
        .filter(|id| !id.is_empty())
        .or_else(|| state.trigger_id.try_peek().ok().and_then(|id| id.clone()))
        .unwrap_or_else(|| popover.relationships().trigger_id().to_owned())
}

fn effective_content_id(popover: &Popover, state: PopoverRuntimeState) -> String {
    state
        .content_id
        .try_peek()
        .ok()
        .and_then(|id| id.clone())
        .unwrap_or_else(|| popover.relationships().content_id().to_owned())
}

fn run_popover_escape_guards(escape_guards: Signal<Vec<(String, OverlayEscapeGuardFn)>>) -> bool {
    let guards: Vec<OverlayEscapeGuardFn> = escape_guards
        .try_peek()
        .map(|list| list.iter().rev().map(|(_, guard)| Rc::clone(guard)).collect())
        .unwrap_or_default();
    for guard in guards {
        if let Ok(mut cb) = guard.try_borrow_mut()
            && cb()
        {
            return true;
        }
    }
    false
}

fn popover_content_css_variables(geometry: &GeometryVars) -> Vec<(String, String)> {
    let mut vars: Vec<(String, String)> = geometry
        .css_iter()
        .chain(geometry.compatibility_alias_iter(
            POPOVER_RADIX_COMPATIBILITY_PREFIX,
            POPOVER_RADIX_ANCHOR_LABEL,
        ))
        .collect();
    let avail_w = geometry
        .css_get("--monoxus-popover-available-width")
        .unwrap_or_else(|| format!("{}px", geometry.available_width()));
    let avail_h = geometry
        .css_get("--monoxus-popover-available-height")
        .unwrap_or_else(|| format!("{}px", geometry.available_height()));
    vars.push((
        "--monoxus-popover-content-transform-origin".to_string(),
        geometry.transform_origin_css_value(),
    ));
    vars.push((
        "--monoxus-popover-content-available-width".to_string(),
        avail_w,
    ));
    vars.push((
        "--monoxus-popover-content-available-height".to_string(),
        avail_h,
    ));
    vars
}

#[cfg(test)]
pub(crate) fn popover_document_path_is_outside(popover: &Popover, path_ids: &[String]) -> bool {
    !popover_document_path_is_inside(popover, path_ids)
}

#[cfg(test)]
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

#[cfg(test)]
mod tests {
    use super::popover_content_css_variables;
    use crate::foundation::overlay::{
        GeometryVars, Presence, PresenceController, PresenceState, serialize_css_custom_properties,
    };

    #[test]
    fn phase_3_7_step_4_popover_presence_lane_retains_content_until_close_completion() {
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
    fn popover_content_css_variables_include_monoxus_and_radix_compatibility_names() {
        let default_geometry = GeometryVars::new(
            "popover", 24.0, 44.0, 26.0, 0.0, 120.0, 80.0, 40.0, 16.0, 30.0, 12.0,
        );
        assert_eq!(
            default_geometry.css_get("--ui-popover-floating-x"),
            Some("24px".to_string())
        );

        let geometry = default_geometry.with_css_prefix("monoxus");
        let variables = popover_content_css_variables(&geometry);
        let serialized = serialize_css_custom_properties(&variables);

        assert_eq!(
            variables,
            vec![
                (
                    "--monoxus-popover-floating-x".to_string(),
                    "24px".to_string()
                ),
                (
                    "--monoxus-popover-floating-y".to_string(),
                    "44px".to_string()
                ),
                (
                    "--monoxus-popover-transform-origin-x".to_string(),
                    "26px".to_string(),
                ),
                (
                    "--monoxus-popover-transform-origin-y".to_string(),
                    "0px".to_string(),
                ),
                (
                    "--monoxus-popover-available-width".to_string(),
                    "120px".to_string(),
                ),
                (
                    "--monoxus-popover-available-height".to_string(),
                    "80px".to_string(),
                ),
                (
                    "--monoxus-popover-anchor-width".to_string(),
                    "40px".to_string(),
                ),
                (
                    "--monoxus-popover-anchor-height".to_string(),
                    "16px".to_string(),
                ),
                (
                    "--monoxus-popover-content-width".to_string(),
                    "30px".to_string(),
                ),
                (
                    "--monoxus-popover-content-height".to_string(),
                    "12px".to_string(),
                ),
                (
                    "--radix-popover-content-transform-origin".to_string(),
                    "26px 0px".to_string(),
                ),
                (
                    "--radix-popover-content-available-width".to_string(),
                    "120px".to_string(),
                ),
                (
                    "--radix-popover-content-available-height".to_string(),
                    "80px".to_string(),
                ),
                (
                    "--radix-popover-trigger-width".to_string(),
                    "40px".to_string()
                ),
                (
                    "--radix-popover-trigger-height".to_string(),
                    "16px".to_string(),
                ),
                (
                    "--monoxus-popover-content-transform-origin".to_string(),
                    "26px 0px".to_string(),
                ),
                (
                    "--monoxus-popover-content-available-width".to_string(),
                    "120px".to_string(),
                ),
                (
                    "--monoxus-popover-content-available-height".to_string(),
                    "80px".to_string(),
                ),
            ],
        );
        assert!(serialized.contains("--monoxus-popover-floating-x: 24px;"));
        assert!(serialized.contains("--monoxus-popover-content-transform-origin: 26px 0px;"));
        assert!(serialized.contains("--monoxus-popover-content-available-width: 120px;"));
        assert!(serialized.contains("--monoxus-popover-content-available-height: 80px;"));
        assert!(serialized.contains("--radix-popover-content-transform-origin: 26px 0px;"));
        assert!(serialized.contains("--radix-popover-trigger-height: 16px;"));
    }

    #[test]
    fn popover_readiness_guard_requires_positive_content_dimensions() {
        let zero_geom = GeometryVars::new(
            "popover", 0.0, 0.0, 0.0, 0.0, 100.0, 100.0, 50.0, 20.0, 0.0, 0.0,
        );
        assert!(!(zero_geom.content_width() > 0.0 && zero_geom.content_height() > 0.0));

        let zero_height_geom = GeometryVars::new(
            "popover", 0.0, 0.0, 0.0, 0.0, 100.0, 100.0, 50.0, 20.0, 100.0, 0.0,
        );
        assert!(!(zero_height_geom.content_width() > 0.0 && zero_height_geom.content_height() > 0.0));

        let non_zero_geom = GeometryVars::new(
            "popover", 0.0, 0.0, 0.0, 0.0, 100.0, 100.0, 50.0, 20.0, 100.0, 50.0,
        );
        assert!(non_zero_geom.content_width() > 0.0 && non_zero_geom.content_height() > 0.0);
    }
}

use std::cell::Cell;

use dioxus::prelude::keyboard_types::{Code, Key, Location, Modifiers, NamedKey};
use dioxus::prelude::*;
use oxidase::watcher::DismissOptions;

pub use ui_sdk::foundation::overlay::dismiss::*;
use ui_sdk::kernel::{ChangeContext, ChangeEvent, ChangeReason, ChangeSource};

use crate::foundation::{
    browser::{
        DocumentDismissEvent, is_body_pointer_events_locked,
        start_document_dismiss_monitor_with_options,
    },
    keyboard::{is_escape_key, is_ime_composing_key},
    shared::{
        EscapeKeyDownEvent, FocusOutsideEvent, InteractOutsideEvent, PointerDownOutsideEvent,
        PreventableEvent,
    },
};

use super::{
    OverlayBranchContext, OverlayLayerSpec, ensure_active_overlay_layer,
    has_overlay_escape_handler, is_overlay_escape_consumed, is_topmost_overlay_layer,
    is_topmost_overlay_layer_for_outside_press, mark_outside_press_propagating,
    mark_overlay_escape_consumed, overlay_layer_bubbles_escape,
    register_overlay_escape_handler_with_result, replace_active_overlay_layer,
    sync_overlay_layer_spec, unregister_active_overlay_layer, unregister_overlay_escape_handler,
    watcher::{WatcherSlot, use_watcher_slot},
};

/// Result of dispatching an Escape keydown against an overlay dismiss layer.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, Hash)]
pub struct EscapeDispatchResult {
    /// `true` when the layer handled the Escape event.
    pub consumed: bool,
    /// `true` when `on_escape_key_down` prevented default or `on_dismiss_details` canceled the
    /// transition.
    pub canceled: bool,
    /// `true` when `on_dismiss_details` explicitly allowed propagation to ancestor layers via
    /// [`ChangeEvent::allow_propagation`].
    pub propagation_allowed: bool,
}

thread_local! {
    /// Render-time modal pointer-events lock counter (`radix#4105`), ensuring modal content and
    /// nested overlays emit `pointer-events: auto` on frame 0 before `use_effect` runs.
    static MODAL_POINTER_LOCK_COUNT: Cell<usize> = const { Cell::new(0) };
}

/// Increments the render-time modal pointer-events lock reference count (`radix#4105`).
pub fn acquire_modal_pointer_lock() {
    MODAL_POINTER_LOCK_COUNT.with(|count| count.set(count.get().saturating_add(1)));
}

/// Decrements the render-time modal pointer-events lock reference count (`radix#4105`).
pub fn release_modal_pointer_lock() {
    MODAL_POINTER_LOCK_COUNT.with(|count| count.set(count.get().saturating_sub(1)));
}

/// Returns `true` when any mounted modal layer holds a render-time or DOM body pointer lock.
pub fn is_modal_pointer_lock_active() -> bool {
    MODAL_POINTER_LOCK_COUNT.with(|count| count.get() > 0) || is_body_pointer_events_locked()
}

/// Synthetic `Escape` `KeyboardData` payload for primitives needing a Dioxus `KeyboardEvent`
/// facade when bridging document-level Escape dismissal.
pub struct SyntheticEscapeKey;

impl dioxus::html::ModifiersInteraction for SyntheticEscapeKey {
    fn modifiers(&self) -> Modifiers {
        Modifiers::empty()
    }
}

impl dioxus::events::HasKeyboardData for SyntheticEscapeKey {
    fn key(&self) -> Key {
        Key::Named(NamedKey::Escape)
    }

    fn code(&self) -> Code {
        Code::Escape
    }

    fn location(&self) -> Location {
        Location::Standard
    }

    fn is_auto_repeating(&self) -> bool {
        false
    }

    fn is_composing(&self) -> bool {
        false
    }

    fn as_any(&self) -> &dyn std::any::Any {
        self
    }
}

fn use_overlay_layer_registration_slot(layer_id: &str, active: bool) -> CopyValue<Option<String>> {
    let mut registered = use_hook(|| CopyValue::new(Option::<String>::None));

    if active && !layer_id.is_empty() {
        let same = registered
            .try_read()
            .is_ok_and(|cur| cur.as_deref() == Some(layer_id));
        if !same {
            let prev = registered.try_write().ok().and_then(|mut cur| cur.take());
            if let Some(prev_id) = prev {
                replace_active_overlay_layer(&prev_id, layer_id);
            } else {
                ensure_active_overlay_layer(layer_id);
            }
            if let Ok(mut cur) = registered.try_write() {
                *cur = Some(layer_id.to_owned());
            }
        }
    } else {
        let prev = registered.try_write().ok().and_then(|mut cur| cur.take());
        if let Some(prev_id) = prev {
            unregister_active_overlay_layer(&prev_id);
        }
    }

    use_drop(move || {
        let prev = registered.try_write().ok().and_then(|mut cur| cur.take());
        if let Some(prev_id) = prev {
            unregister_active_overlay_layer(&prev_id);
        }
    });

    registered
}

/// Registers `layer_id` on the global LIFO `ACTIVE_OVERLAY_LAYERS` stack while `active` is `true`,
/// preserving stack order as a no-op when re-rendered with the same `layer_id` (`bits#2143`),
/// and cleaning up automatically on close or unmount (`use_drop`).
pub fn use_overlay_layer_registration(layer_id: &str, active: bool) {
    let _ = use_overlay_layer_registration_slot(layer_id, active);
}

/// Provides an [`OverlayBranchContext`] that records child overlay branches into the current
/// layer and forwards registrations up to any enclosing [`OverlayBranchContext`].
pub fn use_overlay_branch_provider(
    register: impl FnMut(String) + 'static,
    unregister: impl FnMut(String) + 'static,
) -> OverlayBranchContext {
    let parent_branch = try_consume_context::<OverlayBranchContext>();
    let parent_for_reg = parent_branch.clone();
    let parent_for_unreg = parent_branch;
    let mut reg_fn = register;
    let mut unreg_fn = unregister;

    use_context_provider(move || {
        OverlayBranchContext::new(
            move |branch: String| {
                reg_fn(branch.clone());
                if let Some(ref parent) = parent_for_reg {
                    parent.register_branch(branch);
                }
            },
            move |branch: String| {
                unreg_fn(branch.clone());
                if let Some(ref parent) = parent_for_unreg {
                    parent.unregister_branch(branch);
                }
            },
        )
    })
}

/// Configuration for [`use_dismissable_layer`].
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct DismissConfig {
    pub layer_id: String,
    pub boundaries: Vec<String>,
    pub modal: bool,
    pub listen_escape: bool,
    pub listen_pointer_down_outside: bool,
    pub listen_focus_outside: bool,
    pub disable_outside_pointer_events: bool,
}

impl DismissConfig {
    pub fn new(layer_id: impl Into<String>) -> Self {
        let id = layer_id.into();
        Self {
            boundaries: vec![id.clone()],
            layer_id: id,
            modal: false,
            listen_escape: true,
            listen_pointer_down_outside: true,
            listen_focus_outside: true,
            disable_outside_pointer_events: false,
        }
    }

    pub fn with_boundaries(mut self, boundaries: Vec<String>) -> Self {
        self.boundaries = boundaries;
        self
    }

    pub fn with_modal(mut self, modal: bool) -> Self {
        self.modal = modal;
        self
    }

    pub fn with_disable_outside_pointer_events(mut self, disable: bool) -> Self {
        self.disable_outside_pointer_events = disable;
        self
    }

    pub fn with_listen_escape(mut self, enabled: bool) -> Self {
        self.listen_escape = enabled;
        self
    }

    pub fn with_listen_pointer_down_outside(mut self, enabled: bool) -> Self {
        self.listen_pointer_down_outside = enabled;
        self
    }

    pub fn with_listen_focus_outside(mut self, enabled: bool) -> Self {
        self.listen_focus_outside = enabled;
        self
    }
}

/// Optional callback set for [`use_dismissable_layer`], stored in a non-reactive [`CopyValue`].
#[derive(Clone, Copy, Default, PartialEq)]
pub struct DismissCallbacks {
    pub on_escape_key_down: Option<EventHandler<EscapeKeyDownEvent>>,
    pub on_pointer_down_outside: Option<EventHandler<PointerDownOutsideEvent>>,
    pub on_focus_outside: Option<EventHandler<FocusOutsideEvent>>,
    pub on_interact_outside: Option<EventHandler<InteractOutsideEvent>>,
    pub on_dismiss: Option<EventHandler<DismissEvent>>,
}

/// Zero-overhead `Copy` handle returned by [`use_dismissable_layer`].
#[derive(Clone, Copy, PartialEq)]
pub struct DismissLayerHandle {
    layer_id: CopyValue<String>,
    registered_layer: CopyValue<Option<String>>,
    registered_escape_layer: CopyValue<Option<String>>,
    branches: Signal<Vec<String>>,
    watcher: WatcherSlot,
    alive: CopyValue<bool>,
    interaction_settled: CopyValue<bool>,
    pointer_lock_held: CopyValue<bool>,
    config: CopyValue<DismissConfig>,
    callbacks: CopyValue<DismissCallbacks>,
    on_dismiss_details: CopyValue<Option<EventHandler<(DismissEvent, ChangeEvent<bool>)>>>,
}

impl DismissLayerHandle {
    pub fn is_alive(&self) -> bool {
        self.alive.try_read().map(|g| *g).unwrap_or(false)
    }

    /// Updates the context-aware dismissal callback without breaking [`DismissCallbacks`] literals.
    pub fn set_on_dismiss_details(
        &self,
        handler: Option<EventHandler<(DismissEvent, ChangeEvent<bool>)>>,
    ) {
        let mut slot = self.on_dismiss_details;
        let _ = slot.try_write().map(|mut w| *w = handler);
    }

    /// Synchronously renames the active layer ID in place (preserving LIFO stack position) when
    /// a child `*Content` registers a custom DOM `id` during the initial render pass.
    pub fn rename_layer_id(&self, new_id: &str) {
        if new_id.is_empty() {
            return;
        }
        let mut lid = self.layer_id;
        let old_id = lid.try_read().ok().map(|g| g.clone());
        if old_id.as_deref() != Some(new_id) {
            let _ = lid.try_write().map(|mut w| *w = new_id.to_owned());
        }
        let mut cfg_slot = self.config;
        if let Ok(mut cfg) = cfg_slot.try_write() {
            if cfg.layer_id != new_id {
                cfg.layer_id = new_id.to_owned();
            }
            if !cfg.boundaries.iter().any(|b| b == new_id) {
                cfg.boundaries.push(new_id.to_owned());
            }
        }
        let mut reg_slot = self.registered_layer;
        let prev_reg = reg_slot.try_read().ok().and_then(|g| g.clone());
        if let Some(prev_id) = prev_reg
            && prev_id != new_id
        {
            replace_active_overlay_layer(&prev_id, new_id);
            let _ = reg_slot.try_write().map(|mut w| *w = Some(new_id.to_owned()));
        }
        let mut esc_slot = self.registered_escape_layer;
        let prev_esc = esc_slot.try_read().ok().and_then(|g| g.clone());
        if let Some(prev_id) = prev_esc
            && prev_id != new_id
        {
            let _ = esc_slot.try_write().map(|mut w| *w = Some(new_id.to_owned()));
        }
    }

    /// Evaluates live LIFO topmost membership against `ACTIVE_OVERLAY_LAYERS` (`radix#4143/#4253`).
    pub fn is_topmost(&self) -> bool {
        self.layer_id
            .try_read()
            .map(|id| is_topmost_overlay_layer(id.as_str()))
            .unwrap_or(false)
    }

    /// Evaluates live LIFO topmost membership for outside-press dispatch, treating any
    /// descendant layers that called [`ChangeEvent::allow_propagation`] on the current
    /// outside-press tick as transparent.
    pub fn is_topmost_for_outside_press(&self) -> bool {
        self.layer_id
            .try_read()
            .map(|id| is_topmost_overlay_layer_for_outside_press(id.as_str()))
            .unwrap_or(false)
    }

    pub fn register_branch(&self, id: impl Into<String>) {
        let id = id.into();
        if id.is_empty() {
            return;
        }
        let mut branches = self.branches;
        if branches.try_peek().is_ok_and(|list| !list.contains(&id))
            && let Ok(mut list) = branches.try_write()
            && !list.contains(&id)
        {
            list.push(id);
        }
    }

    pub fn unregister_branch(&self, id: &str) {
        let mut branches = self.branches;
        if branches
            .try_peek()
            .is_ok_and(|list| list.iter().any(|entry| entry == id))
            && let Ok(mut list) = branches.try_write()
        {
            list.retain(|entry| entry != id);
        }
    }

    pub fn branches_signal(&self) -> Signal<Vec<String>> {
        self.branches
    }

    pub fn should_force_pointer_events_auto(&self) -> bool {
        is_modal_pointer_lock_active()
    }

    /// Synchronously resets outside pointer interaction settlement state (`bits#2111/#2229`).
    pub fn reset_interaction(&self) {
        let mut settled = self.interaction_settled;
        if let Ok(mut w) = settled.try_write() {
            *w = false;
        }
    }

    /// Handles a content-scoped `onkeydown` event, filtering IME composition (`radix#4153`),
    /// verifying live LIFO topmost state, firing `on_escape_key_down`, and triggering
    /// `on_dismiss_details` / `on_dismiss(DismissEvent::Escape)` unless prevented.
    pub fn handle_keydown(&self, evt: &KeyboardEvent) -> bool {
        if !self.is_alive() || is_ime_composing_key(evt) || !is_escape_key(&evt.key()) {
            return false;
        }
        let (listen_escape, layer_id) = self
            .config
            .try_read()
            .map(|c| (c.listen_escape, c.layer_id.clone()))
            .unwrap_or((true, String::new()));
        if !listen_escape || is_overlay_escape_consumed() || !self.is_topmost() {
            return false;
        }

        let bubbles_escape = overlay_layer_bubbles_escape(&layer_id);
        let res = self.dispatch_escape();
        if !res.consumed {
            return false;
        }
        if !res.canceled {
            evt.prevent_default();
        }
        if !bubbles_escape && !res.propagation_allowed {
            evt.stop_propagation();
            mark_overlay_escape_consumed();
        }
        true
    }

    pub fn stop(&self) {
        self.watcher.clear();
        self.reset_interaction();
    }

    fn is_inside_path(&self, path_ids: &[String]) -> bool {
        let in_config = self.config.try_read().is_ok_and(|cfg| {
            path_ids.iter().any(|pid| {
                pid == &cfg.layer_id || cfg.boundaries.iter().any(|b| b == pid)
            })
        });
        if in_config {
            return true;
        }
        self.branches
            .try_peek()
            .is_ok_and(|branches| path_ids.iter().any(|pid| branches.contains(pid)))
    }

    fn dispatch_escape(&self) -> EscapeDispatchResult {
        if !self.is_alive() || is_overlay_escape_consumed() || !self.is_topmost() {
            return EscapeDispatchResult::default();
        }
        self.dispatch_escape_from_coordinator()
    }

    fn dispatch_escape_from_coordinator(&self) -> EscapeDispatchResult {
        if !self.is_alive() {
            return EscapeDispatchResult::default();
        }
        let cbs = self
            .callbacks
            .try_read()
            .map(|g| *g)
            .unwrap_or_default();
        let esc_evt: EscapeKeyDownEvent = PreventableEvent::empty();
        if let Some(handler) = cbs.on_escape_key_down {
            handler.call(esc_evt.clone());
        }
        if esc_evt.is_default_prevented() {
            return EscapeDispatchResult {
                consumed: true,
                canceled: true,
                propagation_allowed: false,
            };
        }
        if !self.is_alive() {
            return EscapeDispatchResult {
                consumed: true,
                canceled: false,
                propagation_allowed: false,
            };
        }

        let change_event = ChangeEvent::new(
            false,
            ChangeContext::new(ChangeReason::EscapeKey, ChangeSource::Keyboard),
        );
        let details_cb = self
            .on_dismiss_details
            .try_read()
            .ok()
            .and_then(|g| *g);
        if let Some(on_dismiss_details) = details_cb {
            on_dismiss_details.call((DismissEvent::Escape, change_event.clone()));
        } else if let Some(on_dismiss) = cbs.on_dismiss {
            on_dismiss.call(DismissEvent::Escape);
        }

        EscapeDispatchResult {
            consumed: true,
            canceled: change_event.is_canceled(),
            propagation_allowed: change_event.is_propagation_allowed(),
        }
    }

    fn dispatch_pointer_down_outside(&self, path_ids: &[String]) {
        if !self.is_alive() || !self.is_topmost_for_outside_press() || self.is_inside_path(path_ids)
        {
            return;
        }

        // Capture-start flush (`bits#2111/#2229`): settle rapid sequential outside PointerDown
        // events synchronously before dispatching callbacks so follow-up Click deduplicates cleanly.
        let mut settled = self.interaction_settled;
        if let Ok(mut w) = settled.try_write() {
            *w = true;
        }

        let cbs = self
            .callbacks
            .try_read()
            .map(|g| *g)
            .unwrap_or_default();
        let out_evt: PointerDownOutsideEvent = PreventableEvent::new(path_ids.first().cloned());
        if let Some(handler) = cbs.on_pointer_down_outside {
            handler.call(out_evt.clone());
        }
        if let Some(handler) = cbs.on_interact_outside {
            handler.call(out_evt.clone());
        }
        if !out_evt.is_default_prevented() && self.is_alive() {
            let change_event = ChangeEvent::new(
                false,
                ChangeContext::new(ChangeReason::OutsidePress, ChangeSource::Mouse),
            );
            let details_cb = self
                .on_dismiss_details
                .try_read()
                .ok()
                .and_then(|g| *g);
            if let Some(on_dismiss_details) = details_cb {
                on_dismiss_details.call((DismissEvent::PointerDownOutside, change_event.clone()));
            } else if let Some(on_dismiss) = cbs.on_dismiss {
                on_dismiss.call(DismissEvent::PointerDownOutside);
            }
            if change_event.is_propagation_allowed()
                && let Ok(id) = self.layer_id.try_read().map(|g| g.clone())
            {
                mark_outside_press_propagating(&id);
            }
        }
    }

    fn dispatch_click_outside(&self, path_ids: &[String]) {
        let mut settled = self.interaction_settled;
        let was_settled = settled
            .try_write()
            .map(|mut w| {
                let prev = *w;
                *w = false;
                prev
            })
            .unwrap_or(false);
        if was_settled {
            return;
        }
        self.dispatch_pointer_down_outside(path_ids);
        self.reset_interaction();
    }

    fn dispatch_focus_outside(&self, path_ids: &[String]) {
        if !self.is_alive() || !self.is_topmost_for_outside_press() || self.is_inside_path(path_ids)
        {
            return;
        }
        let cbs = self
            .callbacks
            .try_read()
            .map(|g| *g)
            .unwrap_or_default();
        let out_evt: FocusOutsideEvent = PreventableEvent::new(path_ids.first().cloned());
        if let Some(handler) = cbs.on_focus_outside {
            handler.call(out_evt.clone());
        }
        if let Some(handler) = cbs.on_interact_outside {
            handler.call(out_evt.clone());
        }
        if !out_evt.is_default_prevented() && self.is_alive() {
            let change_event = ChangeEvent::new(
                false,
                ChangeContext::new(ChangeReason::FocusOut, ChangeSource::Programmatic),
            );
            let details_cb = self
                .on_dismiss_details
                .try_read()
                .ok()
                .and_then(|g| *g);
            if let Some(on_dismiss_details) = details_cb {
                on_dismiss_details.call((DismissEvent::FocusOutside, change_event.clone()));
            } else if let Some(on_dismiss) = cbs.on_dismiss {
                on_dismiss.call(DismissEvent::FocusOutside);
            }
            if change_event.is_propagation_allowed()
                && let Ok(id) = self.layer_id.try_read().map(|g| g.clone())
            {
                mark_outside_press_propagating(&id);
            }
        }
    }

    #[cfg(test)]
    pub(crate) fn dispatch_pointer_down_outside_for_test(&self, path_ids: &[String]) {
        self.dispatch_pointer_down_outside(path_ids);
    }
}

/// Shared zero-overhead dismissable layer hook implementing:
/// - same-node LIFO registration no-op (`bits#2143`),
/// - live event-time `ACTIVE_OVERLAY_LAYERS` check (`radix#4143/#4253`),
/// - synchronous interaction reset & capture-start flush (`bits#2111/#2229`),
/// - teardown guard during parent outro (`bits#2295`),
/// - render-time `MODAL_POINTER_LOCK_COUNT` (`radix#4105`),
/// - minimal `CaptureEventMask` subscription via `oxidase v0.4.7`.
pub fn use_dismissable_layer(
    active: bool,
    config: DismissConfig,
    callbacks: DismissCallbacks,
) -> DismissLayerHandle {
    use_dismissable_layer_with_details(active, config, callbacks, None)
}

/// Like [`use_dismissable_layer`], with a context-aware dismissal callback receiving
/// `(DismissEvent, ChangeEvent<bool>)` for live `cancel()` and `allow_propagation()`.
pub fn use_dismissable_layer_with_details(
    active: bool,
    config: DismissConfig,
    callbacks: DismissCallbacks,
    on_dismiss_details: Option<EventHandler<(DismissEvent, ChangeEvent<bool>)>>,
) -> DismissLayerHandle {
    let registered_layer = use_overlay_layer_registration_slot(&config.layer_id, active);

    let mut layer_id = use_hook(|| CopyValue::new(config.layer_id.clone()));
    let mut registered_escape_layer = use_hook(|| CopyValue::new(Option::<String>::None));
    let branches = use_signal(Vec::new);
    let watcher = use_watcher_slot();
    let mut alive = use_hook(|| CopyValue::new(true));
    let interaction_settled = use_hook(|| CopyValue::new(false));
    let mut pointer_lock_held = use_hook(|| CopyValue::new(false));
    let mut config_slot = use_hook(|| CopyValue::new(config.clone()));
    let mut callbacks_slot = use_hook(|| CopyValue::new(callbacks));
    let mut on_dismiss_details_slot = use_hook(|| CopyValue::new(on_dismiss_details));

    if layer_id
        .try_read()
        .is_ok_and(|cur| cur.as_str() != config.layer_id.as_str())
    {
        layer_id.set(config.layer_id.clone());
    }
    config_slot.set(config.clone());
    callbacks_slot.set(callbacks);
    on_dismiss_details_slot.set(on_dismiss_details);

    // Render-time modal pointer lock (`radix#4105`) so frame-0 renders emit `pointer-events: auto`.
    let wants_pointer_lock = active && config.modal && config.disable_outside_pointer_events;
    let currently_locked = pointer_lock_held.try_read().map(|g| *g).unwrap_or(false);
    if wants_pointer_lock && !currently_locked {
        acquire_modal_pointer_lock();
        pointer_lock_held.set(true);
    } else if !wants_pointer_lock && currently_locked {
        release_modal_pointer_lock();
        pointer_lock_held.set(false);
    }

    let handle = DismissLayerHandle {
        layer_id,
        registered_layer,
        registered_escape_layer,
        branches,
        watcher,
        alive,
        interaction_settled,
        pointer_lock_held,
        config: config_slot,
        callbacks: callbacks_slot,
        on_dismiss_details: on_dismiss_details_slot,
    };

    // Synchronize layer spec and centralized Escape capture handler (`T2.3`).
    let effective_layer_id = registered_layer
        .try_read()
        .ok()
        .and_then(|g| g.clone())
        .unwrap_or_else(|| config.layer_id.clone());
    if active && !effective_layer_id.is_empty() {
        let layer_spec = OverlayLayerSpec::new(effective_layer_id.clone())
            .with_modal(config.modal)
            .with_dismiss_on_escape(config.listen_escape)
            .with_dismiss_on_outside_press(config.listen_pointer_down_outside);
        if config.listen_escape {
            let prev_esc = registered_escape_layer
                .try_read()
                .ok()
                .and_then(|g| g.clone());
            if prev_esc.as_deref() != Some(effective_layer_id.as_str())
                || !has_overlay_escape_handler(&effective_layer_id)
            {
                if let Some(old_id) = prev_esc
                    && old_id != effective_layer_id
                {
                    unregister_overlay_escape_handler(&old_id);
                }
                let esc_handle = handle;
                register_overlay_escape_handler_with_result(
                    &effective_layer_id,
                    layer_spec,
                    move || esc_handle.dispatch_escape_from_coordinator(),
                );
                if let Ok(mut w) = registered_escape_layer.try_write() {
                    *w = Some(effective_layer_id);
                }
            } else {
                sync_overlay_layer_spec(layer_spec);
            }
        } else {
            let prev_esc = registered_escape_layer
                .try_write()
                .ok()
                .and_then(|mut w| w.take());
            if let Some(old_id) = prev_esc {
                unregister_overlay_escape_handler(&old_id);
            }
            sync_overlay_layer_spec(layer_spec);
        }
    } else {
        let prev_esc = registered_escape_layer
            .try_write()
            .ok()
            .and_then(|mut w| w.take());
        if let Some(old_id) = prev_esc {
            unregister_overlay_escape_handler(&old_id);
        }
    }

    use_effect(use_reactive((&active, &config), move |(is_active, cur_cfg)| {
        let current_branches = handle.branches.read().clone();
        if !is_active {
            handle.stop();
            return;
        }
        if !cur_cfg.listen_pointer_down_outside && !cur_cfg.listen_focus_outside {
            handle.stop();
            return;
        }

        let mut all_boundaries = Vec::with_capacity(
            1 + cur_cfg.boundaries.len() + current_branches.len(),
        );
        if !cur_cfg.layer_id.is_empty() {
            all_boundaries.push(cur_cfg.layer_id.clone());
        }
        for b in &cur_cfg.boundaries {
            if !b.is_empty() && !all_boundaries.contains(b) {
                all_boundaries.push(b.clone());
            }
        }
        for b in &current_branches {
            if !b.is_empty() && !all_boundaries.contains(b) {
                all_boundaries.push(b.clone());
            }
        }

        let options = DismissOptions {
            boundary_ids: all_boundaries,
            listen_escape: false,
            listen_pointer: cur_cfg.listen_pointer_down_outside,
            listen_focusin: cur_cfg.listen_focus_outside,
        };

        handle.reset_interaction();
        handle.watcher.arm_with(move |expected_gen| {
            start_document_dismiss_monitor_with_options(options, move |event| {
                if !handle.is_alive() || handle.watcher.generation() != expected_gen {
                    return;
                }
                match event {
                    DocumentDismissEvent::Escape => {}
                    DocumentDismissEvent::PointerDown { path_ids } => {
                        handle.dispatch_pointer_down_outside(&path_ids);
                    }
                    DocumentDismissEvent::Click { path_ids } => {
                        handle.dispatch_click_outside(&path_ids);
                    }
                    DocumentDismissEvent::FocusIn { path_ids } => {
                        handle.dispatch_focus_outside(&path_ids);
                    }
                }
            })
        });
    }));

    use_drop(move || {
        if let Ok(mut flag) = alive.try_write() {
            *flag = false;
        }
        let prev_esc = registered_escape_layer
            .try_write()
            .ok()
            .and_then(|mut w| w.take());
        if let Some(old_id) = prev_esc {
            unregister_overlay_escape_handler(&old_id);
        }
        let was_locked = pointer_lock_held
            .try_write()
            .map(|mut w| {
                let prev = *w;
                *w = false;
                prev
            })
            .unwrap_or(false);
        if was_locked {
            release_modal_pointer_lock();
        }
        handle.watcher.clear();
    });

    handle
}

#[cfg(test)]
mod tests {
    use std::{cell::RefCell, rc::Rc};

    use dioxus::core::ScopeId;

    use super::*;
    use crate::foundation::overlay::{
        clear_outside_press_propagating, dispatch_central_escape_event_for_test,
        reset_overlay_coordinator_for_test,
    };

    #[derive(Clone, Props)]
    struct PropagatingNestedProps {
        child_allow_propagation: bool,
        child_cancel: bool,
        parent_dismissed: Rc<RefCell<usize>>,
        child_dismissed: Rc<RefCell<usize>>,
        parent_handle: Rc<RefCell<Option<DismissLayerHandle>>>,
        child_handle: Rc<RefCell<Option<DismissLayerHandle>>>,
    }

    impl PartialEq for PropagatingNestedProps {
        fn eq(&self, other: &Self) -> bool {
            self.child_allow_propagation == other.child_allow_propagation
                && self.child_cancel == other.child_cancel
                && Rc::ptr_eq(&self.parent_dismissed, &other.parent_dismissed)
                && Rc::ptr_eq(&self.child_dismissed, &other.child_dismissed)
                && Rc::ptr_eq(&self.parent_handle, &other.parent_handle)
                && Rc::ptr_eq(&self.child_handle, &other.child_handle)
        }
    }

    #[component]
    fn PropagatingNestedScenario(props: PropagatingNestedProps) -> Element {
        let mut parent_open = use_signal(|| true);
        let mut child_open = use_signal(|| true);

        let p_hits = Rc::clone(&props.parent_dismissed);
        let parent = use_dismissable_layer_with_details(
            parent_open(),
            DismissConfig::new("prop-parent").with_modal(true),
            DismissCallbacks::default(),
            Some(EventHandler::new(
                move |(_evt, change): (DismissEvent, ChangeEvent<bool>)| {
                    if !change.is_canceled() {
                        *p_hits.borrow_mut() += 1;
                        parent_open.set(false);
                    }
                },
            )),
        );
        *props.parent_handle.borrow_mut() = Some(parent);

        let c_hits = Rc::clone(&props.child_dismissed);
        let allow_prop = props.child_allow_propagation;
        let cancel_child = props.child_cancel;
        let child = use_dismissable_layer_with_details(
            child_open(),
            DismissConfig::new("prop-child"),
            DismissCallbacks::default(),
            Some(EventHandler::new(
                move |(_evt, change): (DismissEvent, ChangeEvent<bool>)| {
                    if allow_prop {
                        change.allow_propagation();
                    }
                    if cancel_child {
                        change.cancel();
                        return;
                    }
                    *c_hits.borrow_mut() += 1;
                    child_open.set(false);
                },
            )),
        );
        *props.child_handle.borrow_mut() = Some(child);

        rsx! {}
    }

    #[test]
    fn nested_escape_respects_dynamic_allow_propagation() {
        reset_overlay_coordinator_for_test();

        // 1. Without allow_propagation(): child closes, parent stays open, Escape consumed.
        let parent_hits = Rc::new(RefCell::new(0usize));
        let child_hits = Rc::new(RefCell::new(0usize));
        let mut dom = VirtualDom::new_with_props(
            PropagatingNestedScenario,
            PropagatingNestedProps {
                child_allow_propagation: false,
                child_cancel: false,
                parent_dismissed: Rc::clone(&parent_hits),
                child_dismissed: Rc::clone(&child_hits),
                parent_handle: Rc::new(RefCell::new(None)),
                child_handle: Rc::new(RefCell::new(None)),
            },
        );
        dom.rebuild_in_place();

        let outcome = dispatch_central_escape_event_for_test();
        assert!(outcome.consumed);
        assert_eq!(outcome.dismissed_layers, vec!["prop-child".to_string()]);
        assert_eq!(*child_hits.borrow(), 1);
        assert_eq!(*parent_hits.borrow(), 0);
        assert!(is_overlay_escape_consumed());
        drop(dom);

        // 2. With allow_propagation(): BOTH child and parent close in the same Escape dispatch.
        reset_overlay_coordinator_for_test();
        let parent_hits2 = Rc::new(RefCell::new(0usize));
        let child_hits2 = Rc::new(RefCell::new(0usize));
        let mut dom2 = VirtualDom::new_with_props(
            PropagatingNestedScenario,
            PropagatingNestedProps {
                child_allow_propagation: true,
                child_cancel: false,
                parent_dismissed: Rc::clone(&parent_hits2),
                child_dismissed: Rc::clone(&child_hits2),
                parent_handle: Rc::new(RefCell::new(None)),
                child_handle: Rc::new(RefCell::new(None)),
            },
        );
        dom2.rebuild_in_place();

        let outcome2 = dispatch_central_escape_event_for_test();
        assert!(outcome2.consumed);
        assert_eq!(
            outcome2.dismissed_layers,
            vec!["prop-child".to_string(), "prop-parent".to_string()]
        );
        assert_eq!(*child_hits2.borrow(), 1);
        assert_eq!(*parent_hits2.borrow(), 1);
        assert!(is_overlay_escape_consumed());
        drop(dom2);

        // 3. With cancel() + allow_propagation(): child stays open, parent closes.
        reset_overlay_coordinator_for_test();
        let parent_hits3 = Rc::new(RefCell::new(0usize));
        let child_hits3 = Rc::new(RefCell::new(0usize));
        let mut dom3 = VirtualDom::new_with_props(
            PropagatingNestedScenario,
            PropagatingNestedProps {
                child_allow_propagation: true,
                child_cancel: true,
                parent_dismissed: Rc::clone(&parent_hits3),
                child_dismissed: Rc::clone(&child_hits3),
                parent_handle: Rc::new(RefCell::new(None)),
                child_handle: Rc::new(RefCell::new(None)),
            },
        );
        dom3.rebuild_in_place();

        let _ = dispatch_central_escape_event_for_test();
        assert_eq!(*child_hits3.borrow(), 0);
        assert_eq!(*parent_hits3.borrow(), 1);
        drop(dom3);
        reset_overlay_coordinator_for_test();
    }

    #[test]
    fn nested_outside_press_respects_dynamic_allow_propagation() {
        reset_overlay_coordinator_for_test();

        let outside = vec!["outside-backdrop".to_string()];

        // 1. Without allow_propagation(): child dismisses, parent is NOT topmost and stays open.
        let parent_hits = Rc::new(RefCell::new(0usize));
        let child_hits = Rc::new(RefCell::new(0usize));
        let parent_slot = Rc::new(RefCell::new(None));
        let child_slot = Rc::new(RefCell::new(None));
        let mut dom = VirtualDom::new_with_props(
            PropagatingNestedScenario,
            PropagatingNestedProps {
                child_allow_propagation: false,
                child_cancel: false,
                parent_dismissed: Rc::clone(&parent_hits),
                child_dismissed: Rc::clone(&child_hits),
                parent_handle: Rc::clone(&parent_slot),
                child_handle: Rc::clone(&child_slot),
            },
        );
        dom.rebuild_in_place();

        dom.in_scope(ScopeId::ROOT, || {
            let child = child_slot.borrow().unwrap();
            let parent = parent_slot.borrow().unwrap();
            child.dispatch_pointer_down_outside_for_test(&outside);
            parent.dispatch_pointer_down_outside_for_test(&outside);
        });
        assert_eq!(*child_hits.borrow(), 1);
        assert_eq!(*parent_hits.borrow(), 0);
        drop(dom);

        // 2. With allow_propagation(): child marks propagating -> parent receives same tick!
        reset_overlay_coordinator_for_test();
        let parent_hits2 = Rc::new(RefCell::new(0usize));
        let child_hits2 = Rc::new(RefCell::new(0usize));
        let parent_slot2 = Rc::new(RefCell::new(None));
        let child_slot2 = Rc::new(RefCell::new(None));
        let mut dom2 = VirtualDom::new_with_props(
            PropagatingNestedScenario,
            PropagatingNestedProps {
                child_allow_propagation: true,
                child_cancel: false,
                parent_dismissed: Rc::clone(&parent_hits2),
                child_dismissed: Rc::clone(&child_hits2),
                parent_handle: Rc::clone(&parent_slot2),
                child_handle: Rc::clone(&child_slot2),
            },
        );
        dom2.rebuild_in_place();

        dom2.in_scope(ScopeId::ROOT, || {
            let child = child_slot2.borrow().unwrap();
            let parent = parent_slot2.borrow().unwrap();
            assert!(!parent.is_topmost_for_outside_press());
            child.dispatch_pointer_down_outside_for_test(&outside);
            assert!(parent.is_topmost_for_outside_press());
            parent.dispatch_pointer_down_outside_for_test(&outside);
        });
        assert_eq!(*child_hits2.borrow(), 1);
        assert_eq!(*parent_hits2.borrow(), 1);

        clear_outside_press_propagating();
        drop(dom2);
        reset_overlay_coordinator_for_test();
    }
}

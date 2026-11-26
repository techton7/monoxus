use std::cell::Cell;

use dioxus::prelude::keyboard_types::{Code, Key, Location, Modifiers, NamedKey};
use dioxus::prelude::*;
use oxidase::watcher::DismissOptions;

pub use monoxus_core::foundation::overlay::dismiss::*;

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
    OverlayBranchContext, is_overlay_escape_consumed, is_topmost_overlay_layer,
    mark_overlay_escape_consumed, register_active_overlay_layer, replace_active_overlay_layer,
    unregister_active_overlay_layer,
    watcher::{WatcherSlot, use_watcher_slot},
};

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
                register_active_overlay_layer(layer_id);
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
    branches: Signal<Vec<String>>,
    watcher: WatcherSlot,
    alive: CopyValue<bool>,
    interaction_settled: CopyValue<bool>,
    pointer_lock_held: CopyValue<bool>,
    config: CopyValue<DismissConfig>,
    callbacks: CopyValue<DismissCallbacks>,
}

impl DismissLayerHandle {
    pub fn is_alive(&self) -> bool {
        self.alive.try_read().map(|g| *g).unwrap_or(false)
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
    }

    /// Evaluates live LIFO topmost membership against `ACTIVE_OVERLAY_LAYERS` (`radix#4143/#4253`).
    pub fn is_topmost(&self) -> bool {
        self.layer_id
            .try_read()
            .map(|id| is_topmost_overlay_layer(id.as_str()))
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
    /// `on_dismiss(DismissEvent::Escape)` unless prevented.
    pub fn handle_keydown(&self, evt: &KeyboardEvent) -> bool {
        if !self.is_alive() || is_ime_composing_key(evt) || !is_escape_key(&evt.key()) {
            return false;
        }
        let listen_escape = self
            .config
            .try_read()
            .map(|c| c.listen_escape)
            .unwrap_or(true);
        if !listen_escape || is_overlay_escape_consumed() || !self.is_topmost() {
            return false;
        }

        evt.stop_propagation();
        self.dispatch_escape();
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

    fn dispatch_escape(&self) {
        if !self.is_alive() || is_overlay_escape_consumed() || !self.is_topmost() {
            return;
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
        mark_overlay_escape_consumed();
        if !esc_evt.is_default_prevented()
            && self.is_alive()
            && let Some(on_dismiss) = cbs.on_dismiss
        {
            on_dismiss.call(DismissEvent::Escape);
        }
    }

    fn dispatch_pointer_down_outside(&self, path_ids: &[String]) {
        if !self.is_alive() || !self.is_topmost() || self.is_inside_path(path_ids) {
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
        if !out_evt.is_default_prevented()
            && self.is_alive()
            && let Some(on_dismiss) = cbs.on_dismiss
        {
            on_dismiss.call(DismissEvent::PointerDownOutside);
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
        if !self.is_alive() || !self.is_topmost() || self.is_inside_path(path_ids) {
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
        if !out_evt.is_default_prevented()
            && self.is_alive()
            && let Some(on_dismiss) = cbs.on_dismiss
        {
            on_dismiss.call(DismissEvent::FocusOutside);
        }
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
    let registered_layer = use_overlay_layer_registration_slot(&config.layer_id, active);

    let mut layer_id = use_hook(|| CopyValue::new(config.layer_id.clone()));
    let branches = use_signal(Vec::new);
    let watcher = use_watcher_slot();
    let mut alive = use_hook(|| CopyValue::new(true));
    let interaction_settled = use_hook(|| CopyValue::new(false));
    let mut pointer_lock_held = use_hook(|| CopyValue::new(false));
    let mut config_slot = use_hook(|| CopyValue::new(config.clone()));
    let mut callbacks_slot = use_hook(|| CopyValue::new(callbacks));

    if layer_id
        .try_read()
        .is_ok_and(|cur| cur.as_str() != config.layer_id.as_str())
    {
        layer_id.set(config.layer_id.clone());
    }
    config_slot.set(config.clone());
    callbacks_slot.set(callbacks);

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
        branches,
        watcher,
        alive,
        interaction_settled,
        pointer_lock_held,
        config: config_slot,
        callbacks: callbacks_slot,
    };

    use_effect(use_reactive((&active, &config), move |(is_active, cur_cfg)| {
        let current_branches = handle.branches.read().clone();
        if !is_active {
            handle.stop();
            return;
        }
        if !cur_cfg.listen_escape
            && !cur_cfg.listen_pointer_down_outside
            && !cur_cfg.listen_focus_outside
        {
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
            listen_escape: cur_cfg.listen_escape,
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
                    DocumentDismissEvent::Escape => handle.dispatch_escape(),
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

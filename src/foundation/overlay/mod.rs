pub(crate) mod arrow;
pub mod dismiss;
pub mod floating;
pub mod focus;
pub mod grace;
pub mod handle;
pub mod portal;
pub mod presence;
pub(crate) mod watcher;

pub use ui_sdk::foundation::overlay::coordinator::*;
#[allow(unused_imports)]
pub(crate) use self::arrow::*;
pub use self::dismiss::*;
pub use self::floating::*;
pub use self::focus::*;
pub use self::grace::*;
pub use self::handle::*;
pub use self::portal::*;
pub use self::presence::*;
#[allow(unused_imports)]
pub(crate) use self::watcher::*;

#[doc(hidden)]
pub use crate::foundation::browser::{
    PortalOp, start_portal_op_recording, stop_portal_op_recording, take_portal_ops,
};

use std::{
    cell::RefCell,
    collections::{HashMap, HashSet},
    future::Future,
    pin::Pin,
    rc::{Rc, Weak},
    task::{Context, Poll},
};

use crate::foundation::browser::WatcherGuard;

pub type OverlayEscapeHandler = Rc<RefCell<dyn FnMut()>>;
pub type OverlayEscapeResultHandler = Rc<RefCell<dyn FnMut() -> EscapeDispatchResult>>;

#[derive(Default)]
struct EscapeConsumedState {
    epoch: Option<u64>,
    runtime: Option<Weak<dioxus::core::Runtime>>,
}

#[derive(Default)]
struct PropagatingOutsideState {
    generation: u64,
    layers: HashSet<String>,
    runtime: Option<Weak<dioxus::core::Runtime>>,
}

struct RegisteredEscapeHandler {
    handler: OverlayEscapeResultHandler,
    runtime: Option<Weak<dioxus::core::Runtime>>,
    scope_id: Option<dioxus::core::ScopeId>,
}

#[derive(Default)]
struct EscapeHandlerRegistry {
    handlers: HashMap<String, RegisteredEscapeHandler>,
    observer: Option<WatcherGuard>,
    last_runtime: Option<Weak<dioxus::core::Runtime>>,
}

#[derive(Default)]
struct YieldOnce {
    yielded: bool,
}

impl Future for YieldOnce {
    type Output = ();

    fn poll(mut self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<Self::Output> {
        if self.yielded {
            Poll::Ready(())
        } else {
            self.yielded = true;
            cx.waker().wake_by_ref();
            Poll::Pending
        }
    }
}

thread_local! {
    static OVERLAY_COORDINATOR: RefCell<OverlayCoordinator> = const { RefCell::new(OverlayCoordinator::new()) };
    static ESCAPE_CONSUMED_STATE: RefCell<EscapeConsumedState> = const { RefCell::new(EscapeConsumedState { epoch: None, runtime: None }) };
    static PROPAGATING_OUTSIDE_LAYERS: RefCell<PropagatingOutsideState> = RefCell::new(PropagatingOutsideState::default());
    static ESCAPE_HANDLERS: RefCell<EscapeHandlerRegistry> = RefCell::new(EscapeHandlerRegistry::default());
}

fn clear_propagating_outside_if_gen(generation: u64) {
    PROPAGATING_OUTSIDE_LAYERS.with(|state| {
        let mut state = state.borrow_mut();
        if state.generation == generation {
            state.layers.clear();
            state.runtime = None;
        }
    });
}

/// Marks `layer_id` as transparent for outside-press topmost evaluation during the current
/// synchronous pointer/focus event tick, clearing automatically on the next microtask.
pub fn mark_outside_press_propagating(layer_id: &str) {
    if layer_id.is_empty() {
        return;
    }
    let current_runtime = dioxus::core::Runtime::try_current();
    let runtime_weak = current_runtime.as_ref().map(Rc::downgrade);
    let generation = PROPAGATING_OUTSIDE_LAYERS.with(|state| {
        let mut state = state.borrow_mut();
        if is_recorded_runtime_stale(&state.runtime) {
            state.layers.clear();
        }
        state.generation = state.generation.wrapping_add(1).max(1);
        state.runtime = runtime_weak;
        state.layers.insert(layer_id.to_owned());
        state.generation
    });

    if current_runtime.is_some() {
        crate::foundation::browser::spawn_root_task(async move {
            YieldOnce::default().await;
            clear_propagating_outside_if_gen(generation);
        });
    }
}

/// Immediately clears the propagating outside-press skip set.
pub fn clear_outside_press_propagating() {
    PROPAGATING_OUTSIDE_LAYERS.with(|state| {
        let mut state = state.borrow_mut();
        state.layers.clear();
        state.runtime = None;
    });
}

/// Returns `true` when `layer_id` is the topmost non-propagating active overlay layer for
/// outside-press / focus-outside dispatch.
pub fn is_topmost_overlay_layer_for_outside_press(layer_id: &str) -> bool {
    let skip_empty = PROPAGATING_OUTSIDE_LAYERS.with(|state| {
        let mut state = state.borrow_mut();
        if !state.layers.is_empty() && is_recorded_runtime_stale(&state.runtime) {
            state.layers.clear();
            state.runtime = None;
        }
        state.layers.is_empty()
    });
    if skip_empty {
        return is_topmost_overlay_layer(layer_id);
    }

    let snapshot = active_overlay_layers_snapshot();
    if snapshot.is_empty() {
        return true;
    }

    PROPAGATING_OUTSIDE_LAYERS.with(|state| {
        let state = state.borrow();
        match snapshot.iter().rev().find(|id| !state.layers.contains(*id)) {
            Some(top) => top == layer_id,
            None => true,
        }
    })
}

pub(crate) fn overlay_layer_bubbles_escape(layer_id: &str) -> bool {
    if layer_id.is_empty() {
        return false;
    }
    OVERLAY_COORDINATOR.with(|coord| {
        coord
            .borrow()
            .spec(layer_id)
            .is_some_and(|spec| spec.bubbles_escape)
    })
}

fn is_recorded_runtime_stale(recorded: &Option<Weak<dioxus::core::Runtime>>) -> bool {
    match recorded {
        None => dioxus::core::Runtime::try_current().is_some(),
        Some(weak) => match weak.upgrade() {
            None => true,
            Some(rec) => dioxus::core::Runtime::try_current()
                .is_some_and(|cur| !Rc::ptr_eq(&rec, &cur)),
        },
    }
}

fn clear_escape_consumed_if_epoch(epoch: u64) {
    let cleared =
        OVERLAY_COORDINATOR.with(|coord| coord.borrow_mut().reset_escape_consumed_if_epoch(epoch));
    if cleared {
        ESCAPE_CONSUMED_STATE.with(|state| {
            let mut state = state.borrow_mut();
            if state.epoch == Some(epoch) {
                state.epoch = None;
                state.runtime = None;
            }
        });
    }
}

pub(crate) fn begin_overlay_escape_event() -> u64 {
    let epoch = OVERLAY_COORDINATOR.with(|coord| coord.borrow_mut().begin_escape_event());
    ESCAPE_CONSUMED_STATE.with(|state| {
        let mut state = state.borrow_mut();
        state.epoch = None;
        state.runtime = None;
    });
    clear_outside_press_propagating();
    epoch
}

pub fn mark_overlay_escape_consumed() {
    let current_runtime = dioxus::core::Runtime::try_current();
    let (is_consumed, current_epoch) = OVERLAY_COORDINATOR.with(|coord| {
        let coord = coord.borrow();
        (coord.is_escape_consumed(), coord.current_escape_epoch())
    });
    if is_consumed {
        let same_turn = ESCAPE_CONSUMED_STATE.with(|state| {
            let state = state.borrow();
            state.epoch == Some(current_epoch) && !is_recorded_runtime_stale(&state.runtime)
        });
        if same_turn {
            return;
        }
    }

    let runtime_weak = current_runtime.as_ref().map(Rc::downgrade);
    let epoch = OVERLAY_COORDINATOR.with(|coord| coord.borrow_mut().mark_escape_consumed());
    ESCAPE_CONSUMED_STATE.with(|state| {
        let mut state = state.borrow_mut();
        state.epoch = Some(epoch);
        state.runtime = runtime_weak;
    });

    if current_runtime.is_some() {
        crate::foundation::browser::spawn_root_task(async move {
            YieldOnce::default().await;
            clear_escape_consumed_if_epoch(epoch);
        });
    }
}

pub fn is_overlay_escape_consumed() -> bool {
    let consumed = OVERLAY_COORDINATOR.with(|coord| coord.borrow().is_escape_consumed());
    if !consumed {
        return false;
    }
    let stale = ESCAPE_CONSUMED_STATE
        .with(|state| is_recorded_runtime_stale(&state.borrow().runtime));
    if stale {
        OVERLAY_COORDINATOR.with(|coord| coord.borrow_mut().reset_escape_consumed());
        ESCAPE_CONSUMED_STATE.with(|state| {
            let mut state = state.borrow_mut();
            state.epoch = None;
            state.runtime = None;
        });
        return false;
    }
    true
}

fn prune_expired_escape_handlers_locked(
    reg: &mut EscapeHandlerRegistry,
) -> Vec<RegisteredEscapeHandler> {
    let dead_keys: Vec<String> = reg
        .handlers
        .iter()
        .filter_map(|(key, entry)| {
            entry
                .runtime
                .as_ref()
                .is_some_and(|weak| weak.upgrade().is_none())
                .then(|| key.clone())
        })
        .collect();
    let mut expired = Vec::with_capacity(dead_keys.len());
    for key in dead_keys {
        if let Some(removed) = reg.handlers.remove(&key) {
            expired.push(removed);
        }
    }
    expired
}

fn ensure_central_escape_observer() {
    let needs_observer = ESCAPE_HANDLERS.with(|reg| {
        let reg = reg.borrow();
        !reg.handlers.is_empty() && reg.observer.is_none()
    });
    if !needs_observer {
        return;
    }

    let Ok(guard) = oxidase::dom::observer::observe_capture_events_with_mask(
        oxidase::dom::observer::CaptureEventMask::KEY_DOWN,
        |event| {
            if let oxidase::dom::observer::CaptureEventKind::KeyDown { key, is_composing } =
                event.kind
            {
                let _ = handle_central_escape_keydown(&key, is_composing);
            }
        },
    ) else {
        return;
    };

    let redundant = ESCAPE_HANDLERS.with(|reg| {
        let mut reg = reg.borrow_mut();
        if reg.handlers.is_empty() || reg.observer.is_some() {
            Some(guard)
        } else {
            reg.observer = Some(guard);
            None
        }
    });
    if let Some(extra) = redundant {
        extra.stop();
    }
}

pub fn sync_overlay_layer_spec(spec: OverlayLayerSpec) {
    if spec.id.is_empty() {
        return;
    }
    let runtime_weak = dioxus::core::Runtime::try_current().map(|rt| Rc::downgrade(&rt));
    OVERLAY_COORDINATOR.with(|coord| {
        coord.borrow_mut().ensure_open(spec);
    });
    if runtime_weak.is_some() {
        ESCAPE_HANDLERS.with(|reg| {
            reg.borrow_mut().last_runtime = runtime_weak;
        });
    }
}

pub fn register_overlay_escape_handler(
    layer_id: &str,
    spec: OverlayLayerSpec,
    mut handler: impl FnMut() + 'static,
) {
    register_overlay_escape_handler_with_result(layer_id, spec, move || {
        handler();
        EscapeDispatchResult {
            consumed: true,
            canceled: false,
            propagation_allowed: false,
        }
    });
}

pub fn register_overlay_escape_handler_with_result(
    layer_id: &str,
    spec: OverlayLayerSpec,
    handler: impl FnMut() -> EscapeDispatchResult + 'static,
) {
    if layer_id.is_empty() {
        return;
    }
    let current_runtime = dioxus::core::Runtime::try_current();
    let scope_id = current_runtime
        .as_ref()
        .and_then(|rt| rt.try_current_scope_id());
    let runtime_weak = current_runtime.as_ref().map(Rc::downgrade);

    OVERLAY_COORDINATOR.with(|coord| {
        coord.borrow_mut().ensure_open(spec);
    });

    let entry = RegisteredEscapeHandler {
        handler: Rc::new(RefCell::new(handler)),
        runtime: runtime_weak.clone(),
        scope_id,
    };

    let (replaced, expired) = ESCAPE_HANDLERS.with(|reg| {
        let mut reg = reg.borrow_mut();
        if runtime_weak.is_some() {
            reg.last_runtime = runtime_weak;
        }
        let expired = prune_expired_escape_handlers_locked(&mut reg);
        let replaced = reg.handlers.insert(layer_id.to_owned(), entry);
        (replaced, expired)
    });

    drop(replaced);
    drop(expired);

    ensure_central_escape_observer();
}

pub fn unregister_overlay_escape_handler(layer_id: &str) {
    if layer_id.is_empty() {
        return;
    }
    let (removed, expired, observer_to_drop) = ESCAPE_HANDLERS.with(|reg| {
        let mut reg = reg.borrow_mut();
        let removed = reg.handlers.remove(layer_id);
        let expired = prune_expired_escape_handlers_locked(&mut reg);
        let observer = if reg.handlers.is_empty() {
            reg.observer.take()
        } else {
            None
        };
        (removed, expired, observer)
    });

    drop(removed);
    drop(expired);
    if let Some(guard) = observer_to_drop {
        guard.stop();
    }
}

pub(crate) fn has_overlay_escape_handler(layer_id: &str) -> bool {
    ESCAPE_HANDLERS.with(|reg| reg.borrow().handlers.contains_key(layer_id))
}

pub(crate) fn handle_central_escape_keydown(
    key: &str,
    is_composing: bool,
) -> OverlayDismissOutcome {
    if is_composing || key == "Process" || (key != "Escape" && key != "Esc") {
        return OverlayDismissOutcome::default();
    }
    dispatch_central_escape_event()
}

pub(crate) fn dispatch_central_escape_event() -> OverlayDismissOutcome {
    let _epoch = begin_overlay_escape_event();

    let layer_specs: Vec<OverlayLayerSpec> = OVERLAY_COORDINATOR.with(|coord| {
        let coord = coord.borrow();
        let ids = coord.active_layers_snapshot();
        ids.into_iter()
            .rev()
            .filter_map(|id| coord.spec(&id).cloned())
            .collect()
    });
    if layer_specs.is_empty() {
        return OverlayDismissOutcome::default();
    }

    let mut outcome = OverlayDismissOutcome::default();
    let mut should_mark_consumed = false;
    let mut mark_runtime: Option<(Rc<dioxus::core::Runtime>, dioxus::core::ScopeId)> = None;

    for spec in layer_specs {
        if !spec.dismiss_on_escape {
            if spec.modal && !spec.bubbles_escape {
                outcome.consumed = true;
                should_mark_consumed = true;
                break;
            }
            if !spec.bubbles_escape {
                break;
            }
            continue;
        }

        let mut expired_handlers: Vec<RegisteredEscapeHandler> = Vec::new();
        let mut observer_to_drop: Option<WatcherGuard> = None;
        let handler_entry = ESCAPE_HANDLERS.with(|reg| {
            let mut reg = reg.borrow_mut();
            expired_handlers = prune_expired_escape_handlers_locked(&mut reg);
            let found = reg.handlers.get(&spec.id).and_then(|entry| {
                let rt = match &entry.runtime {
                    Some(weak) => Some(weak.upgrade()?),
                    None => None,
                };
                Some((Rc::clone(&entry.handler), rt, entry.scope_id))
            });
            if reg.handlers.is_empty() {
                observer_to_drop = reg.observer.take();
            }
            found
        });
        drop(expired_handlers);
        if let Some(guard) = observer_to_drop {
            guard.stop();
        }

        let Some((handler, rt, scope_id)) = handler_entry else {
            outcome.dismissed_layers.push(spec.id.clone());
            outcome.consumed = true;
            if !spec.bubbles_escape {
                should_mark_consumed = true;
                break;
            }
            continue;
        };

        let mut res = EscapeDispatchResult::default();
        let mut invoke = || {
            if let Ok(mut cb) = handler.try_borrow_mut() {
                res = cb();
            }
        };
        if let Some(ref runtime) = rt {
            let scope = scope_id.unwrap_or(dioxus::core::ScopeId::ROOT);
            runtime.in_scope(scope, invoke);
            mark_runtime = Some((Rc::clone(runtime), scope));
        } else {
            invoke();
        }

        if res.consumed {
            outcome.dismissed_layers.push(spec.id.clone());
            outcome.consumed = true;
            if !res.propagation_allowed && !spec.bubbles_escape {
                should_mark_consumed = true;
                break;
            }
        }
    }

    if should_mark_consumed {
        if dioxus::core::Runtime::try_current().is_some() {
            mark_overlay_escape_consumed();
        } else if let Some((rt, scope)) = mark_runtime {
            rt.in_scope(scope, mark_overlay_escape_consumed);
        } else {
            let fallback_runtime = ESCAPE_HANDLERS
                .with(|reg| reg.borrow().last_runtime.as_ref().and_then(Weak::upgrade));
            if let Some(rt) = fallback_runtime {
                rt.in_scope(dioxus::core::ScopeId::ROOT, mark_overlay_escape_consumed);
            } else {
                mark_overlay_escape_consumed();
            }
        }
    }

    outcome
}

pub fn register_active_overlay_layer(layer_id: &str) {
    if layer_id.is_empty() {
        return;
    }
    OVERLAY_COORDINATOR.with(|coord| {
        coord.borrow_mut().promote_or_open(layer_id);
    });
}

/// Ensures `layer_id` is registered on `OVERLAY_COORDINATOR` without reordering if already present (`bits#2143`).
pub fn ensure_active_overlay_layer(layer_id: &str) {
    if layer_id.is_empty() {
        return;
    }
    OVERLAY_COORDINATOR.with(|coord| {
        coord.borrow_mut().ensure_registered(layer_id);
    });
}

/// Replaces `old_id` with `new_id` in place on `OVERLAY_COORDINATOR` without disturbing LIFO
/// stack order, migrating any registered Escape handler from `old_id` to `new_id`.
/// If `old_id` is not present, ensures `new_id` is registered unless empty.
pub fn replace_active_overlay_layer(old_id: &str, new_id: &str) {
    if new_id.is_empty() {
        if !old_id.is_empty() {
            unregister_active_overlay_layer(old_id);
        }
        return;
    }
    if old_id == new_id {
        ensure_active_overlay_layer(new_id);
        return;
    }
    OVERLAY_COORDINATOR.with(|coord| {
        let mut coord = coord.borrow_mut();
        if !coord.rename_layer(old_id, new_id) {
            coord.ensure_registered(new_id);
        }
    });
    let displaced = ESCAPE_HANDLERS.with(|reg| {
        let mut reg = reg.borrow_mut();
        if let Some(handler) = reg.handlers.remove(old_id) {
            if reg.handlers.contains_key(new_id) {
                Some(handler)
            } else {
                reg.handlers.insert(new_id.to_owned(), handler);
                None
            }
        } else {
            None
        }
    });
    drop(displaced);
}

pub fn unregister_active_overlay_layer(layer_id: &str) {
    if layer_id.is_empty() {
        return;
    }
    OVERLAY_COORDINATOR.with(|coord| {
        coord.borrow_mut().close_layer(layer_id);
    });
    unregister_overlay_escape_handler(layer_id);
}

pub fn is_topmost_overlay_layer(layer_id: &str) -> bool {
    OVERLAY_COORDINATOR.with(|coord| coord.borrow().is_topmost(layer_id))
}

pub fn active_overlay_layers_snapshot() -> Vec<String> {
    OVERLAY_COORDINATOR.with(|coord| coord.borrow().active_layers_snapshot())
}

#[cfg(test)]
pub(crate) fn dispatch_central_escape_event_for_test() -> OverlayDismissOutcome {
    dispatch_central_escape_event()
}

#[cfg(test)]
pub(crate) fn reset_overlay_coordinator_for_test() {
    OVERLAY_COORDINATOR.with(|coord| {
        *coord.borrow_mut() = OverlayCoordinator::new();
    });
    ESCAPE_CONSUMED_STATE.with(|state| {
        let mut state = state.borrow_mut();
        state.epoch = None;
        state.runtime = None;
    });
    clear_outside_press_propagating();
    let (drained_handlers, observer_to_drop) = ESCAPE_HANDLERS.with(|reg| {
        let mut reg = reg.borrow_mut();
        let handlers: Vec<RegisteredEscapeHandler> = reg.handlers.drain().map(|(_, v)| v).collect();
        let observer = reg.observer.take();
        reg.last_runtime = None;
        (handlers, observer)
    });
    drop(drained_handlers);
    if let Some(guard) = observer_to_drop {
        guard.stop();
    }
}

pub type OverlayEscapeGuardFn = Rc<RefCell<dyn FnMut() -> bool>>;

#[derive(Clone)]
pub struct OverlayEscapeGuardContext {
    register: Rc<RefCell<dyn FnMut(String, OverlayEscapeGuardFn)>>,
    unregister: Rc<RefCell<dyn FnMut(String)>>,
}

impl PartialEq for OverlayEscapeGuardContext {
    fn eq(&self, other: &Self) -> bool {
        Rc::ptr_eq(&self.register, &other.register)
            && Rc::ptr_eq(&self.unregister, &other.unregister)
    }
}

impl OverlayEscapeGuardContext {
    pub fn new(
        register_guard: impl FnMut(String, OverlayEscapeGuardFn) + 'static,
        unregister_guard: impl FnMut(String) + 'static,
    ) -> Self {
        Self {
            register: Rc::new(RefCell::new(register_guard)),
            unregister: Rc::new(RefCell::new(unregister_guard)),
        }
    }

    pub fn register_guard(&self, id: impl Into<String>, guard: impl FnMut() -> bool + 'static) {
        if let Ok(mut cb) = self.register.try_borrow_mut() {
            cb(id.into(), Rc::new(RefCell::new(guard)));
        }
    }

    pub fn unregister_guard(&self, id: impl Into<String>) {
        if let Ok(mut cb) = self.unregister.try_borrow_mut() {
            cb(id.into());
        }
    }
}

#[derive(Clone)]
pub struct OverlayBranchContext {
    register: Rc<RefCell<dyn FnMut(String)>>,
    unregister: Rc<RefCell<dyn FnMut(String)>>,
}

impl PartialEq for OverlayBranchContext {
    fn eq(&self, other: &Self) -> bool {
        Rc::ptr_eq(&self.register, &other.register)
            && Rc::ptr_eq(&self.unregister, &other.unregister)
    }
}

impl OverlayBranchContext {
    pub fn new(
        register_branch: impl FnMut(String) + 'static,
        unregister_branch: impl FnMut(String) + 'static,
    ) -> Self {
        Self {
            register: Rc::new(RefCell::new(register_branch)),
            unregister: Rc::new(RefCell::new(unregister_branch)),
        }
    }

    pub fn register_branch(&self, branch: impl Into<String>) {
        if let Ok(mut cb) = self.register.try_borrow_mut() {
            cb(branch.into());
        }
    }

    pub fn unregister_branch(&self, branch: impl Into<String>) {
        if let Ok(mut cb) = self.unregister.try_borrow_mut() {
            cb(branch.into());
        }
    }
}

/// Shared portal teleportation & drop cleanup hook for all `*Portal` components (`PRT-1`..`PRT-7`).
pub(crate) fn use_portal_teleport(
    pid: String,
    should_render: bool,
    disabled: bool,
    resolved_host: &PortalHost,
) {
    use dioxus::prelude::*;

    let state: Rc<RefCell<(String, bool)>> =
        use_hook(|| Rc::new(RefCell::new((pid.clone(), false))));
    let floating_recalc = try_use_context::<floating::PortalTeleportRecalcContext>();

    {
        let state_effect = Rc::clone(&state);
        use_effect(use_reactive(
            (&should_render, &disabled, &pid, resolved_host),
            move |(render, is_disabled, cur_pid, host)| {
                let mut tracked = state_effect.borrow_mut();
                let should_teleport = render && !is_disabled && !host.is_inline();
                if should_teleport {
                    if tracked.1 && tracked.0 != cur_pid {
                        crate::foundation::browser::remove_element_by_id(&tracked.0);
                    }
                    let host_id = match &host {
                        PortalHost::Default => None,
                        PortalHost::Named(name) => Some(name.as_ref()),
                        PortalHost::Inline => None,
                    };
                    crate::foundation::browser::teleport_element_to_host(&cur_pid, host_id);
                    if let Some(ctx) = floating_recalc {
                        let _ = ctx.0.recalculate_sync();
                    }
                    tracked.0 = cur_pid;
                    tracked.1 = true;
                } else {
                    if tracked.1 && tracked.0 != cur_pid {
                        crate::foundation::browser::remove_element_by_id(&tracked.0);
                    }
                    tracked.0 = cur_pid;
                    tracked.1 = false;
                }
            },
        ));
    }

    {
        let state_drop = Rc::clone(&state);
        use_drop(move || {
            let (cur_pid, was_teleported) = state_drop.borrow().clone();
            if was_teleported {
                crate::foundation::browser::remove_element_by_id(&cur_pid);
            }
        });
    }
}

/// Registers a set of DOM element IDs (e.g. `[content_id, wrapper_id, portal_id]`) into an
/// enclosing [`OverlayBranchContext`] while `active` is `true`, cleaning up stale IDs on
/// ID change, close (`active == false`), and unmount (`use_drop`).
pub(crate) fn use_register_parent_overlay_branches(
    parent_branch: Option<OverlayBranchContext>,
    active: bool,
    ids: Vec<String>,
) {
    use dioxus::prelude::*;

    let tracked: Rc<RefCell<Vec<String>>> = use_hook(|| Rc::new(RefCell::new(Vec::new())));

    if let Some(ref bctx) = parent_branch {
        let mut prev = tracked.borrow_mut();
        if !active {
            for old in prev.drain(..) {
                bctx.unregister_branch(old);
            }
        } else {
            let stale: Vec<String> = prev
                .iter()
                .filter(|old| !ids.iter().any(|n| n == *old))
                .cloned()
                .collect();
            for old in &stale {
                bctx.unregister_branch(old.clone());
            }
            prev.retain(|old| !stale.contains(old));

            for id in &ids {
                if !id.is_empty() && !prev.contains(id) {
                    bctx.register_branch(id.clone());
                    prev.push(id.clone());
                }
            }
        }
    } else {
        tracked.borrow_mut().clear();
    }

    let drop_tracked = Rc::clone(&tracked);
    let drop_branch = parent_branch;
    use_drop(move || {
        if let Some(ref bctx) = drop_branch {
            for old in drop_tracked.borrow_mut().drain(..) {
                bctx.unregister_branch(old);
            }
        }
    });
}

#[cfg(test)]
mod tests;

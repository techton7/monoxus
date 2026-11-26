use dioxus::prelude::*;

pub use monoxus_core::foundation::overlay::presence::*;

use crate::foundation::browser::{
    PresenceMonitorEvent, set_element_style_property, start_presence_monitor,
};

use super::watcher::{WatcherSlot, use_watcher_slot};

/// Zero-overhead `Copy` presence handle uniting [`PresenceController`], a [`WatcherSlot`],
/// mount-flush open safety (`bits#2274`), monotonic cycle invalidation, and `animation-fill-mode: forwards`
/// exit flash protection (`radix#1849/#2763`).
#[derive(Clone, Copy, PartialEq)]
pub struct PresenceHandle {
    controller: Signal<PresenceController>,
    watcher: WatcherSlot,
    has_mounted: CopyValue<bool>,
    alive: CopyValue<bool>,
    element_id: CopyValue<String>,
    on_complete: CopyValue<Option<EventHandler<bool>>>,
}

impl PresenceHandle {
    /// Returns the current [`PresenceState`] (subscribing the calling reactive scope).
    pub fn state(&self) -> PresenceState {
        self.controller.read().state()
    }

    /// Returns `true` when the element should remain mounted in the DOM (subscribing).
    pub fn should_render(&self) -> bool {
        self.controller.read().should_render()
    }

    /// Alias for [`Self::should_render`].
    pub fn is_present(&self) -> bool {
        self.should_render()
    }

    /// Returns the last synchronized `desired_present` target (subscribing).
    pub fn desired_present(&self) -> bool {
        self.controller.read().desired_present()
    }

    /// Returns the active close cycle ID if currently exiting (`PresenceState::Suspended`).
    pub fn active_close_cycle(&self) -> Option<PresenceCloseCycleId> {
        self.controller
            .try_peek()
            .ok()
            .and_then(|c| c.active_close_cycle())
    }

    /// Returns `true` when the owning component has completed its first mount effect flush.
    pub fn has_mounted(&self) -> bool {
        self.has_mounted.try_read().map(|g| *g).unwrap_or(false)
    }

    /// Returns `true` while the owning component has not been dropped.
    pub fn is_alive(&self) -> bool {
        self.alive.try_read().map(|g| *g).unwrap_or(false)
    }

    /// Stops any active presence watcher without mutating controller state.
    pub fn stop(&self) {
        self.watcher.clear();
    }

    /// Synchronizes `desired_present` against the underlying [`PresenceController`], arming or
    /// invalidating the exit animation watcher strictly outside signal borrows.
    pub fn sync(&self, desired_present: bool, element_id: &str) -> PresenceControllerUpdate {
        let mut id_slot = self.element_id;
        if let Ok(mut current_id) = id_slot.try_write()
            && *current_id != element_id
        {
            *current_id = element_id.to_owned();
        }

        let (prev_desired, prev_state, update) = {
            let mut ctrl_sig = self.controller;
            let Ok(mut ctrl) = ctrl_sig.try_write() else {
                return PresenceController::new(desired_present).sync(desired_present);
            };
            let prev_d = ctrl.desired_present();
            let prev_s = ctrl.state();
            let upd = ctrl.sync(desired_present);
            (prev_d, prev_s, upd)
        };

        if let Some(cycle_id) = update.started_close_cycle() {
            self.arm_exit_watcher(element_id, cycle_id);
        } else if !update.should_render() || update.invalidated_close_cycle().is_some() {
            self.watcher.clear();
            if !desired_present && prev_desired && prev_state != PresenceState::Unmounted {
                self.notify_complete(false);
            }
        }

        if desired_present && !prev_desired && self.has_mounted() {
            self.notify_complete(true);
        }

        update
    }

    /// Completes `cycle_id` if it is still the active close cycle, clearing the watcher and
    /// invoking `on_open_change_complete(false)` once.
    pub fn complete_close_cycle(&self, cycle_id: PresenceCloseCycleId) -> bool {
        if !self.is_alive() {
            return false;
        }
        let completed = {
            let mut ctrl_sig = self.controller;
            let Ok(mut ctrl) = ctrl_sig.try_write() else {
                return false;
            };
            ctrl.complete_close_cycle(cycle_id)
        };
        if completed {
            self.watcher.clear();
            self.notify_complete(false);
        }
        completed
    }

    /// Completes the currently active exit cycle (if any) after applying the `forwards` flash guard.
    pub fn finish_active_exit(&self) -> bool {
        let Some(cycle_id) = self.active_close_cycle() else {
            return false;
        };
        let cid = self
            .element_id
            .try_read()
            .map(|g| g.clone())
            .unwrap_or_default();
        if !cid.is_empty() {
            set_element_style_property(&cid, "animation-fill-mode", "forwards");
        }
        self.complete_close_cycle(cycle_id)
    }

    fn arm_exit_watcher(&self, element_id: &str, cycle_id: PresenceCloseCycleId) {
        let handle = *self;
        let cid = element_id.to_owned();
        if cid.is_empty() {
            self.complete_close_cycle(cycle_id);
            return;
        }
        self.watcher.arm_with(move |expected_gen| {
            let cid_cb = cid.clone();
            start_presence_monitor(&cid, cycle_id, move |event| {
                if !handle.is_alive() || handle.watcher.generation() != expected_gen {
                    return;
                }
                match event {
                    PresenceMonitorEvent::AnimationEnd {
                        cycle_id: event_cycle,
                        ..
                    } => {
                        // Guard against 1-frame snap-back flash before VDOM unmount (`radix#1849/#2763`).
                        if handle.active_close_cycle() == Some(event_cycle) {
                            set_element_style_property(&cid_cb, "animation-fill-mode", "forwards");
                            handle.complete_close_cycle(event_cycle);
                        }
                    }
                    PresenceMonitorEvent::AnimationCancel {
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
                    }
                    | PresenceMonitorEvent::Fallback {
                        cycle_id: event_cycle,
                        ..
                    } => {
                        handle.complete_close_cycle(event_cycle);
                    }
                    PresenceMonitorEvent::Stopped { .. } => {}
                }
            })
        });
    }

    fn notify_complete(&self, open: bool) {
        let cb = self.on_complete.try_read().ok().and_then(|slot| *slot);
        if let Some(handler) = cb {
            handler.call(open);
        }
    }
}

/// Shared zero-overhead presence hook with mount-flush open guard (`bits#2274`), monotonic
/// close-cycle invalidation, `animation-fill-mode: forwards` exit flash guard (`radix#1849/#2763`),
/// and immediate-unmount fallback when no CSS animation/transition is active.
pub fn use_presence(
    desired_present: bool,
    element_id: &str,
    retain_mount: bool,
    on_open_change_complete: Option<EventHandler<bool>>,
) -> PresenceHandle {
    let controller = use_signal(|| {
        PresenceController::new(desired_present).with_retained_mount(retain_mount)
    });
    let watcher = use_watcher_slot();
    let has_mounted = use_hook(|| CopyValue::new(false));
    let mut alive = use_hook(|| CopyValue::new(true));
    let mut element_id_slot = use_hook(|| CopyValue::new(element_id.to_owned()));
    let mut on_complete = use_hook(|| CopyValue::new(on_open_change_complete));

    if element_id_slot.try_read().is_ok_and(|cur| cur.as_str() != element_id) {
        element_id_slot.set(element_id.to_owned());
    }
    on_complete.set(on_open_change_complete);

    let handle = PresenceHandle {
        controller,
        watcher,
        has_mounted,
        alive,
        element_id: element_id_slot,
        on_complete,
    };

    // Synchronous render-time reconciliation prevents mount-closed -> immediate-open drops (`bits#2274`).
    let needs_sync = controller
        .try_peek()
        .is_ok_and(|c| c.desired_present() != desired_present);
    if needs_sync {
        handle.sync(desired_present, element_id);
    }

    let mut mounted_effect = has_mounted;
    use_effect(move || {
        if mounted_effect.try_read().is_ok_and(|m| !*m) {
            mounted_effect.set(true);
        }
    });

    use_drop(move || {
        if let Ok(mut flag) = alive.try_write() {
            *flag = false;
        }
        watcher.clear();
    });

    handle
}

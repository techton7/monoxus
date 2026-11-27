use dioxus::prelude::*;

pub use ui_sdk::foundation::overlay::presence::*;

use crate::foundation::browser::{PresenceMonitorEvent, start_presence_monitor};

use super::watcher::{WatcherSlot, use_watcher_slot};

// [SYNC1-REPLACE]: re-exported from ui_sdk::foundation::overlay::presence::TransitionStatus after Sync #1
/// Animation / transition lifecycle phase exposed as `data-starting-style` / `data-ending-style`.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, Hash)]
pub enum TransitionStatus {
    #[default]
    None,
    Starting,
    Idle,
    Ending,
}

impl TransitionStatus {
    #[inline]
    pub const fn as_str(&self) -> Option<&'static str> {
        match self {
            Self::None => None,
            Self::Starting => Some("starting"),
            Self::Idle => Some("idle"),
            Self::Ending => Some("ending"),
        }
    }

    #[inline]
    pub const fn is_starting(self) -> bool {
        matches!(self, Self::Starting)
    }

    #[inline]
    pub const fn is_idle(self) -> bool {
        matches!(self, Self::Idle)
    }

    #[inline]
    pub const fn is_ending(self) -> bool {
        matches!(self, Self::Ending)
    }
}

fn extract_inline_style_property(inline_style: &str, property: &str) -> Option<String> {
    let mut found = None;
    for decl in inline_style.split(';') {
        let Some((name, val)) = decl.split_once(':') else {
            continue;
        };
        if name.trim().eq_ignore_ascii_case(property) {
            let trimmed = val.trim();
            if !trimmed.is_empty() {
                found = Some(trimmed.to_owned());
            }
        }
    }
    found
}

fn restore_element_fill_mode(element_id: &str, prior_value: Option<&str>) {
    if element_id.is_empty() {
        return;
    }
    if let Some(doc) = oxidase::dom::Document::current()
        && let Some(el) = doc.element_by_id(element_id)
    {
        match prior_value {
            Some(prev) => {
                let _ = el.set_style_property("animation-fill-mode", prev);
            }
            None => {
                let _ = el.remove_style_property("animation-fill-mode");
            }
        }
    }
}

/// Zero-overhead `Copy` presence handle uniting [`PresenceController`], a [`WatcherSlot`],
/// mount-flush open safety (`bits#2274`), monotonic cycle invalidation, and `animation-fill-mode: forwards`
/// exit flash protection (`radix#1849/#2763`).
#[derive(Clone, Copy, PartialEq)]
pub struct PresenceHandle {
    controller: Signal<PresenceController>,
    transition_status: Signal<TransitionStatus>,
    watcher: WatcherSlot,
    has_mounted: CopyValue<bool>,
    alive: CopyValue<bool>,
    element_id: CopyValue<String>,
    on_complete: CopyValue<Option<EventHandler<bool>>>,
    fill_mode_guard_gen: CopyValue<u64>,
    pending_fill_mode_restore: CopyValue<Option<(String, Option<String>)>>,
    fill_mode_restore_frame: CopyValue<Option<oxidase::frame::FrameRequestGuard>>,
    keep_mounted: CopyValue<bool>,
    starting_frame_gen: CopyValue<u64>,
    starting_frame_guard: CopyValue<Option<oxidase::frame::FrameRequestGuard>>,
}

impl PresenceHandle {
    /// Returns the current [`PresenceState`] (subscribing the calling reactive scope).
    pub fn state(&self) -> PresenceState {
        self.controller.read().state()
    }

    /// Returns the current [`TransitionStatus`] (subscribing the calling reactive scope).
    pub fn transition_status(&self) -> TransitionStatus {
        *self.transition_status.read()
    }

    /// Returns `Some("")` while the enter starting-style phase is active (`data-starting-style`).
    pub fn data_starting_style(&self) -> Option<&'static str> {
        self.transition_status().is_starting().then_some("")
    }

    /// Returns `Some("")` while the exit ending-style phase is active (`data-ending-style`).
    pub fn data_ending_style(&self) -> Option<&'static str> {
        self.transition_status().is_ending().then_some("")
    }

    /// Returns `true` when the element is configured to remain mounted even when hidden.
    pub fn keep_mounted(&self) -> bool {
        self.keep_mounted.try_read().map(|g| *g).unwrap_or(false)
    }

    /// Returns `true` when the element is kept mounted in the DOM while logically unmounted.
    pub fn is_hidden(&self) -> bool {
        self.keep_mounted() && !self.controller.read().should_render()
    }

    /// Synchronizes `keep_mounted` reactively if the prop changes post-mount.
    pub fn sync_keep_mounted(&self, keep_mounted: bool) {
        let mut slot = self.keep_mounted;
        if let Ok(mut current) = slot.try_write()
            && *current != keep_mounted
        {
            *current = keep_mounted;
        }
    }

    /// Returns `true` when the element should remain mounted in the DOM (subscribing).
    pub fn should_render(&self) -> bool {
        self.keep_mounted() || self.controller.read().should_render()
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

    /// Synchronizes `retain_mount` reactively if the prop changes post-mount.
    ///
    /// When `retain_mount` transitions to `false` while an exit cycle is `Suspended`,
    /// completes the unmount immediately and cancels the active presence watcher.
    pub fn sync_retain_mount(&self, retain_mount: bool) {
        let should_complete_unmount = {
            let mut ctrl_sig = self.controller;
            let Ok(mut ctrl) = ctrl_sig.try_write() else {
                return;
            };
            if ctrl.retain_mount() == retain_mount {
                return;
            }
            let was_suspended = ctrl.state() == PresenceState::Suspended;
            let desired = ctrl.desired_present();
            *ctrl = (*ctrl).with_retained_mount(retain_mount);
            if !retain_mount && !desired && was_suspended {
                let upd = ctrl.sync(false);
                !upd.should_render()
            } else {
                false
            }
        };

        if should_complete_unmount {
            self.watcher.clear();
            self.cancel_starting_frame();
            self.set_transition_status(TransitionStatus::None);
            self.restore_pending_fill_mode();
            self.notify_complete(false);
        }
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

        if desired_present {
            self.restore_pending_fill_mode();
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
            self.cancel_starting_frame();
            self.set_transition_status(TransitionStatus::Ending);
            self.arm_exit_watcher(element_id, cycle_id);
        } else if !desired_present {
            self.cancel_starting_frame();
            if !update.should_render() {
                self.watcher.clear();
                self.set_transition_status(TransitionStatus::None);
                if prev_desired && prev_state != PresenceState::Unmounted {
                    self.notify_complete(false);
                }
            }
        } else {
            if update.invalidated_close_cycle().is_some() {
                self.watcher.clear();
            }
            if !prev_desired {
                self.set_transition_status(TransitionStatus::Starting);
                self.schedule_starting_frame();
                if self.has_mounted() {
                    self.notify_complete(true);
                }
            }
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
            self.set_transition_status(TransitionStatus::None);
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
            self.apply_exit_fill_mode_flash_guard(&cid);
        }
        self.complete_close_cycle(cycle_id)
    }

    fn apply_exit_fill_mode_flash_guard(&self, element_id: &str) {
        if element_id.is_empty() {
            return;
        }
        let Some(doc) = oxidase::dom::Document::current() else {
            return;
        };
        let Some(el) = doc.element_by_id(element_id) else {
            return;
        };

        // Preserve any original inline `animation-fill-mode` prior to applying the 1-frame
        // `forwards` flash guard so retained (`force_mount`) or reopened nodes are restored cleanly.
        let mut pending_slot = self.pending_fill_mode_restore;
        let existing_prior = pending_slot.try_read().ok().and_then(|existing| {
            let (saved_id, saved_val) = existing.as_ref()?;
            (saved_id == element_id).then(|| saved_val.clone())
        });
        let prior_value = match existing_prior {
            Some(val) => val,
            None => {
                let captured = el
                    .get_attribute("style")
                    .and_then(|s| extract_inline_style_property(&s, "animation-fill-mode"));
                if let Ok(mut w) = pending_slot.try_write() {
                    *w = Some((element_id.to_owned(), captured.clone()));
                }
                captured
            }
        };

        let _ = el.set_style_property("animation-fill-mode", "forwards");

        let mut gen_slot = self.fill_mode_guard_gen;
        let expected_gen = gen_slot
            .try_write()
            .map(|mut g| {
                *g = g.wrapping_add(1);
                *g
            })
            .unwrap_or(0);

        let restore_id = element_id.to_owned();
        let handle = *self;
        let mut frame_slot = self.fill_mode_restore_frame;
        let _ = frame_slot.try_write().map(|mut w| w.take());
        // Schedule via `oxidase::frame::request_next_frame` (not `dioxus::spawn`) so exit
        // completion works safely even when invoked outside an active Dioxus runtime scope.
        if let Ok(guard) = oxidase::frame::request_next_frame(move |_| {
            let gen_matches = handle
                .fill_mode_guard_gen
                .try_read()
                .is_ok_and(|g| *g == expected_gen);
            if !gen_matches {
                return;
            }
            let mut slot = handle.pending_fill_mode_restore;
            let pending = slot.try_write().ok().and_then(|mut w| w.take());
            if let Some((saved_id, saved_val)) = pending {
                restore_element_fill_mode(&saved_id, saved_val.as_deref());
            } else {
                restore_element_fill_mode(&restore_id, prior_value.as_deref());
            }
            let mut frame_done = handle.fill_mode_restore_frame;
            let _ = frame_done.try_write().map(|mut w| w.take());
        }) && let Ok(mut w) = frame_slot.try_write()
        {
            *w = Some(guard);
        }
    }

    fn restore_pending_fill_mode(&self) {
        let mut gen_slot = self.fill_mode_guard_gen;
        let _ = gen_slot.try_write().map(|mut g| *g = g.wrapping_add(1));
        let mut frame_slot = self.fill_mode_restore_frame;
        let _ = frame_slot.try_write().map(|mut w| w.take());
        let mut pending_slot = self.pending_fill_mode_restore;
        let pending = pending_slot.try_write().ok().and_then(|mut w| w.take());
        if let Some((saved_id, saved_val)) = pending {
            restore_element_fill_mode(&saved_id, saved_val.as_deref());
        }
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
                            handle.apply_exit_fill_mode_flash_guard(&cid_cb);
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

    fn set_transition_status(&self, status: TransitionStatus) {
        let mut sig = self.transition_status;
        if let Ok(mut current) = sig.try_write()
            && *current != status
        {
            *current = status;
        }
    }

    fn cancel_starting_frame(&self) {
        let mut gen_slot = self.starting_frame_gen;
        let _ = gen_slot.try_write().map(|mut g| *g = g.wrapping_add(1));
        let mut guard_slot = self.starting_frame_guard;
        let _ = guard_slot.try_write().map(|mut g| g.take());
    }

    fn schedule_starting_frame(&self) {
        let mut gen_slot = self.starting_frame_gen;
        let expected_gen = gen_slot
            .try_write()
            .map(|mut g| {
                *g = g.wrapping_add(1);
                *g
            })
            .unwrap_or(0);

        let mut guard_slot = self.starting_frame_guard;
        let _ = guard_slot.try_write().map(|mut g| g.take());

        let handle = *self;
        match oxidase::frame::request_next_frame(move |_| {
            if !handle.is_alive() {
                return;
            }
            let gen_matches = handle
                .starting_frame_gen
                .try_read()
                .is_ok_and(|g| *g == expected_gen);
            if !gen_matches {
                return;
            }
            let still_desired = handle
                .controller
                .try_peek()
                .is_ok_and(|c| c.desired_present());
            if still_desired {
                handle.set_transition_status(TransitionStatus::Idle);
            }
            let mut done_slot = handle.starting_frame_guard;
            let _ = done_slot.try_write().map(|mut g| g.take());
        }) {
            Ok(guard) => {
                if let Ok(mut g) = guard_slot.try_write() {
                    *g = Some(guard);
                }
            }
            Err(_) => {
                self.set_transition_status(TransitionStatus::Idle);
            }
        }
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
    use_presence_with_keep_mounted(
        desired_present,
        element_id,
        retain_mount,
        false,
        on_open_change_complete,
    )
}

/// Extended presence hook supporting `keep_mounted` (preserving DOM mount while hidden) in
/// addition to [`TransitionStatus`] enter/exit lifecycle tracking.
pub fn use_presence_with_keep_mounted(
    desired_present: bool,
    element_id: &str,
    retain_mount: bool,
    keep_mounted: bool,
    on_open_change_complete: Option<EventHandler<bool>>,
) -> PresenceHandle {
    let controller = use_signal(|| {
        PresenceController::new(desired_present).with_retained_mount(retain_mount)
    });
    let transition_status = use_signal(|| {
        if desired_present {
            TransitionStatus::Idle
        } else {
            TransitionStatus::None
        }
    });
    let watcher = use_watcher_slot();
    let has_mounted = use_hook(|| CopyValue::new(false));
    let mut alive = use_hook(|| CopyValue::new(true));
    let mut element_id_slot = use_hook(|| CopyValue::new(element_id.to_owned()));
    let mut on_complete = use_hook(|| CopyValue::new(on_open_change_complete));
    let fill_mode_guard_gen = use_hook(|| CopyValue::new(0_u64));
    let pending_fill_mode_restore =
        use_hook(|| CopyValue::new(Option::<(String, Option<String>)>::None));
    let fill_mode_restore_frame =
        use_hook(|| CopyValue::new(Option::<oxidase::frame::FrameRequestGuard>::None));
    let keep_mounted_slot = use_hook(|| CopyValue::new(keep_mounted));
    let starting_frame_gen = use_hook(|| CopyValue::new(0_u64));
    let starting_frame_guard =
        use_hook(|| CopyValue::new(Option::<oxidase::frame::FrameRequestGuard>::None));

    if element_id_slot
        .try_read()
        .is_ok_and(|cur| cur.as_str() != element_id)
    {
        element_id_slot.set(element_id.to_owned());
    }
    on_complete.set(on_open_change_complete);

    let handle = PresenceHandle {
        controller,
        transition_status,
        watcher,
        has_mounted,
        alive,
        element_id: element_id_slot,
        on_complete,
        fill_mode_guard_gen,
        pending_fill_mode_restore,
        fill_mode_restore_frame,
        keep_mounted: keep_mounted_slot,
        starting_frame_gen,
        starting_frame_guard,
    };

    if handle.keep_mounted() != keep_mounted {
        handle.sync_keep_mounted(keep_mounted);
    }

    // Reactively sync `retain_mount` changes before reconciling `desired_present`.
    let retain_changed = controller
        .try_peek()
        .is_ok_and(|c| c.retain_mount() != retain_mount);
    if retain_changed {
        handle.sync_retain_mount(retain_mount);
    }

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
        handle.cancel_starting_frame();
        handle.restore_pending_fill_mode();
    });

    handle
}

#[cfg(test)]
mod tests {
    use super::{TransitionStatus, extract_inline_style_property};

    #[test]
    fn transition_status_default_and_accessors() {
        assert_eq!(TransitionStatus::default(), TransitionStatus::None);

        assert_eq!(TransitionStatus::None.as_str(), None);
        assert!(!TransitionStatus::None.is_starting());
        assert!(!TransitionStatus::None.is_idle());
        assert!(!TransitionStatus::None.is_ending());

        assert_eq!(TransitionStatus::Starting.as_str(), Some("starting"));
        assert!(TransitionStatus::Starting.is_starting());
        assert!(!TransitionStatus::Starting.is_idle());
        assert!(!TransitionStatus::Starting.is_ending());

        assert_eq!(TransitionStatus::Idle.as_str(), Some("idle"));
        assert!(!TransitionStatus::Idle.is_starting());
        assert!(TransitionStatus::Idle.is_idle());
        assert!(!TransitionStatus::Idle.is_ending());

        assert_eq!(TransitionStatus::Ending.as_str(), Some("ending"));
        assert!(!TransitionStatus::Ending.is_starting());
        assert!(!TransitionStatus::Ending.is_idle());
        assert!(TransitionStatus::Ending.is_ending());
    }

    #[test]
    fn extract_inline_style_property_edge_cases() {
        assert_eq!(
            extract_inline_style_property("", "animation-fill-mode"),
            None
        );
        assert_eq!(
            extract_inline_style_property("opacity: 1; color: red", "animation-fill-mode"),
            None
        );
        assert_eq!(
            extract_inline_style_property(
                "  Animation-Fill-Mode :  both ; color: red ",
                "animation-fill-mode"
            ),
            Some("both".to_owned())
        );
        assert_eq!(
            extract_inline_style_property(
                "animation-fill-mode: none; animation-fill-mode: backwards",
                "animation-fill-mode"
            ),
            Some("backwards".to_owned())
        );
        assert_eq!(
            extract_inline_style_property(
                "animation-fill-mode: forwards; animation-fill-mode:   ;",
                "animation-fill-mode"
            ),
            Some("forwards".to_owned())
        );
        assert_eq!(
            extract_inline_style_property(
                "malformed-entry; animation-fill-mode: forwards; trailing",
                "animation-fill-mode"
            ),
            Some("forwards".to_owned())
        );
    }
}

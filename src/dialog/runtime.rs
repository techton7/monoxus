use std::collections::HashMap;

use dioxus::{document::Eval, prelude::*};

pub use crate::foundation::compose::{
    compose_part_event_handlers, compose_part_refs, project_as_child,
};

use crate::foundation::{
    browser::{
        PresenceMonitorEvent, acquire_scroll_lock, focus_element_by_id,
        focus_first_focusable as foundation_focus_first_focusable, recv_presence_monitor_event,
        release_scroll_lock, restore_focus_element_by_id, start_presence_monitor,
        stop_presence_monitor,
    },
    overlay::{
        Presence, PresenceCloseCycleId, PresenceController, PresenceControllerUpdate, PresenceState,
    },
    state::DataState,
};

use super::{
    attrs::{
        DialogCloseAttributes, DialogContentAttributes, DialogDescriptionAttributes,
        DialogOverlayAttributes, DialogPortalAttributes, DialogRootAttributes,
        DialogTitleAttributes, DialogTriggerAttributes,
    },
    relationships::DialogRelationships,
    state::{Dialog, DialogLifecycle},
    types::{
        DIALOG_FOCUSABLE_SELECTOR, DialogCloseFocusPolicy, DialogMountedHandle,
        DialogOpenFocusPolicy,
    },
};

#[derive(Clone, Copy)]
struct DialogRuntimeState {
    trigger_handle: Signal<Option<DialogMountedHandle>>,
    content_handle: Signal<Option<DialogMountedHandle>>,
    focus_targets: Signal<HashMap<String, DialogMountedHandle>>,
    presence_lane: Signal<DialogPresenceLane>,
    overlay_presence_monitor: RetainedRootPresenceMonitorState,
    content_presence_monitor: RetainedRootPresenceMonitorState,
    last_open: Signal<bool>,
    pending_open_focus: Signal<bool>,
    scroll_lock_held: Signal<bool>,
}

#[derive(Clone, Copy)]
struct RetainedRootPresenceMonitorState {
    monitor: Signal<Option<Eval>>,
    cycle_id: Signal<Option<PresenceCloseCycleId>>,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
struct DialogPresenceLane {
    overlay: PresenceController,
    content: PresenceController,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
struct DialogPresenceLaneUpdate {
    overlay: PresenceControllerUpdate,
    content: PresenceControllerUpdate,
}

impl DialogPresenceLane {
    fn new(presence: &Presence) -> Self {
        let controller = PresenceController::new(presence.desired_present())
            .with_retained_mount(presence.retain_mount());

        Self {
            overlay: controller,
            content: controller,
        }
    }

    fn sync(&mut self, desired_present: bool) -> DialogPresenceLaneUpdate {
        DialogPresenceLaneUpdate {
            overlay: self.overlay.sync(desired_present),
            content: self.content.sync(desired_present),
        }
    }

    const fn should_render_portal(&self) -> bool {
        self.should_render_overlay() || self.should_render_content()
    }

    const fn should_render_overlay(&self) -> bool {
        self.overlay.should_render()
    }

    const fn should_render_content(&self) -> bool {
        self.content.should_render()
    }

    fn complete_overlay_close_cycle(&mut self, cycle_id: PresenceCloseCycleId) -> bool {
        self.overlay.complete_close_cycle(cycle_id)
    }

    fn complete_content_close_cycle(&mut self, cycle_id: PresenceCloseCycleId) -> bool {
        self.content.complete_close_cycle(cycle_id)
    }
}

#[derive(Clone)]
pub struct DialogRuntime {
    dialog: Dialog,
    state: DialogRuntimeState,
}

pub fn use_dialog_runtime(dialog: Dialog) -> DialogRuntime {
    let state = DialogRuntimeState {
        trigger_handle: use_signal(|| None),
        content_handle: use_signal(|| None),
        focus_targets: use_signal(HashMap::new),
        presence_lane: use_signal(|| DialogPresenceLane::new(dialog.lifecycle().presence())),
        overlay_presence_monitor: RetainedRootPresenceMonitorState {
            monitor: use_signal(|| Option::<Eval>::None),
            cycle_id: use_signal(|| None),
        },
        content_presence_monitor: RetainedRootPresenceMonitorState {
            monitor: use_signal(|| Option::<Eval>::None),
            cycle_id: use_signal(|| None),
        },
        last_open: use_signal(|| dialog.is_open()),
        pending_open_focus: use_signal(|| dialog.is_open()),
        scroll_lock_held: use_signal(|| false),
    };
    let effect_state = state;
    let cleanup_state = state;
    let cleanup_key = dialog.relationships().root_id().to_owned();
    let cleanup_overlay_id = dialog.relationships().overlay_id().to_owned();
    let cleanup_content_id = dialog.relationships().content_id().to_owned();

    use_effect(use_reactive((&dialog,), move |(dialog,)| {
        sync_dialog_runtime(&dialog, effect_state);
    }));

    dioxus::core::use_drop(move || {
        stop_dialog_presence_monitor(
            cleanup_state.overlay_presence_monitor,
            "overlay",
            cleanup_overlay_id.as_str(),
        );
        stop_dialog_presence_monitor(
            cleanup_state.content_presence_monitor,
            "content",
            cleanup_content_id.as_str(),
        );
        if *cleanup_state.scroll_lock_held.peek() {
            release_scroll_lock(&cleanup_key, None);
        }
    });

    DialogRuntime { dialog, state }
}

impl DialogRuntime {
    pub fn dialog(&self) -> &Dialog {
        &self.dialog
    }

    pub const fn is_open(&self) -> bool {
        self.dialog.is_open()
    }

    pub fn data_state(&self) -> DataState {
        self.dialog.data_state()
    }

    pub fn relationships(&self) -> &DialogRelationships {
        self.dialog.relationships()
    }

    pub fn lifecycle(&self) -> &DialogLifecycle {
        self.dialog.lifecycle()
    }

    pub fn root(&self) -> DialogRootAttributes {
        self.dialog.root()
    }

    pub fn trigger(&self) -> DialogTriggerAttributes {
        self.dialog.trigger()
    }

    pub fn portal(&self) -> DialogPortalAttributes {
        self.dialog.portal()
    }

    pub fn overlay(&self) -> DialogOverlayAttributes {
        self.dialog.overlay()
    }

    pub fn content(&self) -> DialogContentAttributes {
        self.dialog.content()
    }

    pub fn should_render_portal(&self) -> bool {
        self.state
            .presence_lane
            .with_peek(|lane| lane.should_render_portal())
    }

    pub fn should_render_overlay(&self) -> bool {
        self.state
            .presence_lane
            .with_peek(|lane| lane.should_render_overlay())
    }

    pub fn should_render_content(&self) -> bool {
        self.state
            .presence_lane
            .with_peek(|lane| lane.should_render_content())
    }

    pub fn title(&self) -> DialogTitleAttributes {
        self.dialog.title()
    }

    pub fn description(&self) -> DialogDescriptionAttributes {
        self.dialog.description()
    }

    pub fn close(&self) -> DialogCloseAttributes {
        self.dialog.close()
    }

    pub fn trigger_handle(&self) -> Option<DialogMountedHandle> {
        self.state.trigger_handle.cloned()
    }

    pub fn content_handle(&self) -> Option<DialogMountedHandle> {
        self.state.content_handle.cloned()
    }

    pub fn mounted_focus_target(&self, id: &str) -> Option<DialogMountedHandle> {
        self.state
            .focus_targets
            .with_peek(|targets| targets.get(id).cloned())
    }

    pub fn capture_trigger(&self, mounted: DialogMountedHandle) {
        let mut trigger_handle = self.state.trigger_handle;
        trigger_handle.set(Some(mounted.clone()));
        self.capture_focus_target(self.relationships().trigger_id().to_owned(), mounted);
    }

    pub fn capture_content(&self, mounted: DialogMountedHandle) {
        let mut content_handle = self.state.content_handle;
        content_handle.set(Some(mounted.clone()));
        self.capture_focus_target(self.relationships().content_id().to_owned(), mounted);
    }

    pub fn capture_close(&self, mounted: DialogMountedHandle) {
        self.capture_focus_target(self.relationships().close_id().to_owned(), mounted);
    }

    pub fn capture_focus_target(&self, id: impl Into<String>, mounted: DialogMountedHandle) {
        let id = id.into();
        let mut focus_targets = self.state.focus_targets;
        focus_targets.with_mut(|targets| {
            targets.insert(id, mounted);
        });
    }

    pub fn mount_trigger(&self) -> impl FnMut(MountedEvent) + 'static {
        let runtime = self.clone();
        move |event| runtime.capture_trigger(event.data())
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
}

fn sync_dialog_runtime(dialog: &Dialog, mut state: DialogRuntimeState) {
    let presence_update = state
        .presence_lane
        .with_mut(|lane| lane.sync(dialog.is_open()));
    sync_dialog_presence_root(
        dialog.relationships().overlay_id(),
        state.presence_lane,
        state.overlay_presence_monitor,
        presence_update.overlay,
        DialogPresenceLane::complete_overlay_close_cycle,
        "overlay",
    );
    sync_dialog_presence_root(
        dialog.relationships().content_id(),
        state.presence_lane,
        state.content_presence_monitor,
        presence_update.content,
        DialogPresenceLane::complete_content_close_cycle,
        "content",
    );

    let is_open = dialog.is_open();
    let was_open = *state.last_open.peek();
    let should_hold_scroll_lock = is_open
        && dialog.lifecycle().mode().is_modal()
        && dialog.lifecycle().scroll_lock_policy().is_enabled();
    let is_holding_scroll_lock = *state.scroll_lock_held.peek();

    if is_open && !was_open && !*state.pending_open_focus.peek() {
        let mut pending_open_focus = state.pending_open_focus;
        pending_open_focus.set(true);
    }

    if should_hold_scroll_lock && !is_holding_scroll_lock {
        acquire_scroll_lock(dialog.relationships().root_id());
        let mut scroll_lock_held = state.scroll_lock_held;
        scroll_lock_held.set(true);
    } else if !should_hold_scroll_lock && is_holding_scroll_lock {
        let restore_delay = if was_open && !is_open {
            dialog.lifecycle().scroll_lock_policy().restore_delay()
        } else {
            None
        };
        release_scroll_lock(dialog.relationships().root_id(), restore_delay);
        let mut scroll_lock_held = state.scroll_lock_held;
        scroll_lock_held.set(false);
    }

    if is_open {
        if *state.pending_open_focus.peek() && apply_open_focus(dialog) {
            let mut pending_open_focus = state.pending_open_focus;
            pending_open_focus.set(false);
        }
    } else {
        if was_open {
            restore_close_focus(dialog, state);
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

fn apply_open_focus(dialog: &Dialog) -> bool {
    if !dialog.lifecycle().focus_scope().autofocus_enabled() {
        return true;
    }

    match dialog.lifecycle().open_focus_policy() {
        DialogOpenFocusPolicy::FirstFocusable => {
            focus_first_focusable(dialog.relationships().content_id());
            true
        }
        DialogOpenFocusPolicy::Target(target) => {
            focus_element_by_id(target);
            true
        }
        DialogOpenFocusPolicy::Suppress => true,
    }
}

fn restore_close_focus(dialog: &Dialog, _state: DialogRuntimeState) {
    match dialog.lifecycle().close_focus_policy() {
        DialogCloseFocusPolicy::Trigger => {
            restore_focus_element_by_id(dialog.relationships().trigger_id());
        }
        DialogCloseFocusPolicy::Target(target) => {
            restore_focus_element_by_id(target);
        }
        DialogCloseFocusPolicy::None => {}
    }
}

pub(crate) fn focus_first_focusable(content_id: &str) {
    foundation_focus_first_focusable(content_id, Some(DIALOG_FOCUSABLE_SELECTOR));
}

fn sync_dialog_presence_root(
    root_id: &str,
    presence_lane: Signal<DialogPresenceLane>,
    monitor_state: RetainedRootPresenceMonitorState,
    update: PresenceControllerUpdate,
    complete_close_cycle: fn(&mut DialogPresenceLane, PresenceCloseCycleId) -> bool,
    root_label: &'static str,
) {
    if update.invalidated_close_cycle().is_some()
        || !matches!(update.state(), PresenceState::Suspended)
    {
        stop_dialog_presence_monitor(monitor_state, root_label, root_id);
    }

    if let Some(cycle_id) = update.started_close_cycle() {
        start_dialog_presence_monitor(
            root_id.to_owned(),
            presence_lane,
            monitor_state,
            cycle_id,
            complete_close_cycle,
            root_label,
        );
    }
}

fn start_dialog_presence_monitor(
    root_id: String,
    presence_lane: Signal<DialogPresenceLane>,
    monitor_state: RetainedRootPresenceMonitorState,
    cycle_id: PresenceCloseCycleId,
    complete_close_cycle: fn(&mut DialogPresenceLane, PresenceCloseCycleId) -> bool,
    root_label: &'static str,
) {
    stop_dialog_presence_monitor(monitor_state, root_label, root_id.as_str());

    let monitor = start_presence_monitor(root_id.as_str(), cycle_id);
    let mut active_monitor = monitor_state.monitor;
    active_monitor.set(Some(monitor));
    let mut active_cycle_id = monitor_state.cycle_id;
    active_cycle_id.set(Some(cycle_id));

    spawn(async move {
        let mut monitor = monitor;

        loop {
            if monitor_state
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
                    complete_dialog_presence_close_cycle(
                        presence_lane,
                        monitor_state,
                        event_cycle,
                        complete_close_cycle,
                    );
                    break;
                }
                Ok(PresenceMonitorEvent::Stopped {
                    cycle_id: event_cycle,
                }) => {
                    if monitor_state
                        .cycle_id
                        .with_peek(|current| *current == Some(event_cycle))
                    {
                        clear_dialog_presence_monitor_state(monitor_state);
                    }
                    break;
                }
                Err(error) => {
                    if monitor_state
                        .cycle_id
                        .with_peek(|current| *current == Some(cycle_id))
                    {
                        eprintln!(
                            "monoxus dialog runtime could not observe {root_label} presence for {root_id}: {error}",
                        );
                        clear_dialog_presence_monitor_state(monitor_state);
                    }
                    break;
                }
            }
        }
    });
}

fn complete_dialog_presence_close_cycle(
    mut presence_lane: Signal<DialogPresenceLane>,
    monitor_state: RetainedRootPresenceMonitorState,
    cycle_id: PresenceCloseCycleId,
    complete_close_cycle: fn(&mut DialogPresenceLane, PresenceCloseCycleId) -> bool,
) {
    clear_dialog_presence_monitor_state(monitor_state);
    let _ = presence_lane.with_mut(|lane| complete_close_cycle(lane, cycle_id));
}

fn stop_dialog_presence_monitor(
    monitor_state: RetainedRootPresenceMonitorState,
    root_label: &str,
    root_id: &str,
) {
    let Some(monitor) = monitor_state.monitor.with_peek(|monitor| *monitor) else {
        return;
    };

    clear_dialog_presence_monitor_state(monitor_state);

    if let Err(error) = stop_presence_monitor(monitor) {
        eprintln!(
            "monoxus dialog runtime could not stop {root_label} presence monitor for {root_id}: {error}",
        );
    }
}

fn clear_dialog_presence_monitor_state(monitor_state: RetainedRootPresenceMonitorState) {
    let mut active_monitor = monitor_state.monitor;
    active_monitor.set(None);
    let mut active_cycle_id = monitor_state.cycle_id;
    active_cycle_id.set(None);
}

#[cfg(test)]
mod tests {
    use super::DialogPresenceLane;
    use crate::foundation::overlay::{Presence, PresenceState};

    #[test]
    fn phase_3_7_step_4_dialog_presence_lane_keeps_portal_alive_until_both_roots_unmount() {
        let presence = Presence::new(true).with_retained_mount(true);
        let mut lane = DialogPresenceLane::new(&presence);

        let close = lane.sync(false);
        let overlay_cycle = close.overlay.started_close_cycle().unwrap();
        let content_cycle = close.content.started_close_cycle().unwrap();

        assert_eq!(close.overlay.state(), PresenceState::Suspended);
        assert_eq!(close.content.state(), PresenceState::Suspended);
        assert!(lane.should_render_portal());
        assert!(lane.should_render_overlay());
        assert!(lane.should_render_content());

        assert!(lane.complete_overlay_close_cycle(overlay_cycle));
        assert!(lane.should_render_portal());
        assert!(!lane.should_render_overlay());
        assert!(lane.should_render_content());

        assert!(lane.complete_content_close_cycle(content_cycle));
        assert!(!lane.should_render_portal());
        assert!(!lane.should_render_overlay());
        assert!(!lane.should_render_content());
    }
}

use dioxus::prelude::*;

use crate::foundation::{
    browser::{start_presence_monitor, PresenceMonitorEvent, WatcherGuard},
    overlay::{
        Presence, PresenceCloseCycleId, PresenceController, PresenceControllerUpdate,
    },
};

use super::runtime::SelectRuntimeState;

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct SelectContentPresenceLane {
    pub(crate) content: PresenceController,
}

impl SelectContentPresenceLane {
    pub fn new(presence: &Presence) -> Self {
        Self {
            content: PresenceController::new(presence.desired_present())
                .with_retained_mount(presence.retain_mount()),
        }
    }

    pub fn sync(&mut self, desired_present: bool) -> PresenceControllerUpdate {
        self.content.sync(desired_present)
    }

    pub const fn should_render_portal(&self) -> bool {
        self.should_render_content()
    }

    pub const fn should_render_content(&self) -> bool {
        self.content.should_render()
    }

    pub fn complete_close_cycle(&mut self, cycle_id: PresenceCloseCycleId) -> bool {
        self.content.complete_close_cycle(cycle_id)
    }
}

#[derive(Clone, Copy, PartialEq)]
pub struct RetainedRootPresenceMonitorState {
    pub monitor: Signal<Option<WatcherGuard>>,
    pub cycle_id: Signal<Option<PresenceCloseCycleId>>,
}

pub(crate) fn sync_select_presence(content_id: &str, mut state: SelectRuntimeState, open: bool) {
    let update = state
        .presence_lane
        .with_mut(|lane| lane.sync(open));

    if update.invalidated_close_cycle().is_some() || !update.should_render() {
        stop_select_presence_monitor(state, content_id);
    }

    if let Some(cycle_id) = update.started_close_cycle() {
        start_select_presence_monitor(content_id, state, cycle_id);
    }
}

pub(crate) fn start_select_presence_monitor(
    content_id: &str,
    state: SelectRuntimeState,
    cycle_id: PresenceCloseCycleId,
) {
    stop_select_presence_monitor(state, content_id);

    let watcher = start_presence_monitor(content_id, cycle_id, move |event| {
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
            } => {
                complete_select_presence_close_cycle(state, event_cycle);
            }
            PresenceMonitorEvent::Stopped {
                cycle_id: event_cycle,
            } => {
                if state
                    .presence_monitor
                    .cycle_id
                    .with_peek(|current| *current == Some(event_cycle))
                {
                    clear_select_presence_monitor_state(state);
                }
            }
        }
    });
    let mut active_monitor = state.presence_monitor.monitor;
    active_monitor.set(Some(watcher));
    let mut active_cycle_id = state.presence_monitor.cycle_id;
    active_cycle_id.set(Some(cycle_id));
}

pub(crate) fn stop_select_presence_monitor(state: SelectRuntimeState, _content_id: &str) {
    clear_select_presence_monitor_state(state);
}

pub(crate) fn clear_select_presence_monitor_state(state: SelectRuntimeState) {
    let mut active_monitor = state.presence_monitor.monitor;
    active_monitor.set(None);
    let mut active_cycle_id = state.presence_monitor.cycle_id;
    active_cycle_id.set(None);
}

pub(crate) fn complete_select_presence_close_cycle(
    mut state: SelectRuntimeState,
    cycle_id: PresenceCloseCycleId,
) {
    clear_select_presence_monitor_state(state);
    let _ = state
        .presence_lane
        .with_mut(|lane| lane.complete_close_cycle(cycle_id));
}

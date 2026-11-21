use std::collections::HashMap;

use dioxus::prelude::*;

pub use crate::foundation::compose::{
    compose_part_event_handlers, compose_part_refs, project_as_child,
};

use crate::foundation::{
    browser::{
        DocumentDismissEvent, FocusOptions, ModalShieldGuard, PresenceMonitorEvent, WatcherGuard,
        acquire_scroll_lock_with_options, focus_element_by_id,
        focus_first_focusable as foundation_focus_first_focusable, is_body_pointer_events_locked,
        release_scroll_lock, restore_focus_element_by_id, restore_focus_element_by_id_with_options,
        start_document_dismiss_monitor_with_boundaries, start_presence_monitor,
        suppress_external_dom,
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
    modal_shield: Signal<Option<ModalShieldGuard>>,
    dismiss_monitor: Signal<Option<WatcherGuard>>,
    prevent_body_pointer_events_override: Signal<Option<bool>>,
    last_dismiss_was_pointer: Signal<bool>,
    overlay_id: Signal<Option<String>>,
    content_id: Signal<Option<String>>,
    trigger_id: Signal<Option<String>>,
    branches: Signal<Vec<String>>,
    open_focus_target: Signal<Option<String>>,
}

#[derive(Clone, Copy)]
struct RetainedRootPresenceMonitorState {
    monitor: Signal<Option<WatcherGuard>>,
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
            monitor: use_signal(|| None),
            cycle_id: use_signal(|| None),
        },
        content_presence_monitor: RetainedRootPresenceMonitorState {
            monitor: use_signal(|| None),
            cycle_id: use_signal(|| None),
        },
        last_open: use_signal(|| dialog.is_open()),
        pending_open_focus: use_signal(|| dialog.is_open()),
        scroll_lock_held: use_signal(|| false),
        modal_shield: use_signal(|| None),
        dismiss_monitor: use_signal(|| None),
        prevent_body_pointer_events_override: use_signal(|| None),
        last_dismiss_was_pointer: use_signal(|| false),
        overlay_id: use_signal(|| None),
        content_id: use_signal(|| None),
        trigger_id: use_signal(|| None),
        branches: use_signal(Vec::new),
        open_focus_target: use_signal(|| None),
    };
    let effect_state = state;
    let mut cleanup_state = state;
    let cleanup_key = dialog.relationships().root_id().to_owned();
    let cleanup_overlay_id = dialog.relationships().overlay_id().to_owned();
    let cleanup_content_id = dialog.relationships().content_id().to_owned();

    use_effect(use_reactive((&dialog,), move |(dialog,)| {
        sync_dialog_runtime(&dialog, effect_state);
    }));

    dioxus::core::use_drop(move || {
        let overlay_id = cleanup_state
            .overlay_id
            .try_peek()
            .ok()
            .and_then(|v| v.clone())
            .unwrap_or(cleanup_overlay_id);
        let content_id = cleanup_state
            .content_id
            .try_peek()
            .ok()
            .and_then(|v| v.clone())
            .unwrap_or(cleanup_content_id);
        stop_dialog_presence_monitor(
            cleanup_state.overlay_presence_monitor,
            "overlay",
            overlay_id.as_str(),
        );
        stop_dialog_presence_monitor(
            cleanup_state.content_presence_monitor,
            "content",
            content_id.as_str(),
        );
        if let Ok(mut mon) = cleanup_state.dismiss_monitor.try_write() {
            *mon = None;
        }
        if let Ok(mut shield) = cleanup_state.modal_shield.try_write() {
            *shield = None;
        }
        if cleanup_state
            .scroll_lock_held
            .try_peek()
            .map(|held| *held)
            .unwrap_or(false)
        {
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
        let mut attrs = self.dialog.trigger();
        let content_id = self
            .state
            .content_id
            .peek()
            .clone()
            .unwrap_or_else(|| self.relationships().content_id().to_string());
        attrs.set_target_content_id(content_id.clone());
        attrs.set_aria_controls(if self.is_open() || self.should_render_content() {
            Some(content_id)
        } else {
            None
        });
        attrs
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
        self.state.presence_lane.read().should_render_portal()
    }

    pub fn should_render_overlay(&self) -> bool {
        self.state.presence_lane.read().should_render_overlay()
    }

    pub fn should_render_content(&self) -> bool {
        self.state.presence_lane.read().should_render_content()
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

    pub fn register_overlay_id(&self, id: impl Into<String>) {
        let mut overlay_id = self.state.overlay_id;
        let id = id.into();
        if overlay_id.peek().as_deref() != Some(id.as_str()) {
            overlay_id.set(Some(id));
        }
    }

    pub fn register_content_id(&self, id: impl Into<String>) {
        let mut content_id = self.state.content_id;
        let id = id.into();
        if content_id.peek().as_deref() != Some(id.as_str()) {
            content_id.set(Some(id));
        }
    }

    pub fn unregister_content_id(&self, id: &str) {
        let mut content_id = self.state.content_id;
        if let Ok(mut current) = content_id.try_write() {
            if current.as_deref() == Some(id) {
                *current = None;
            }
        }
    }

    pub fn set_open_focus_target(&self, target: impl Into<String>) {
        let target = target.into();
        let mut oft = self.state.open_focus_target;
        if oft.peek().as_deref() != Some(target.as_str()) {
            oft.set(Some(target));
        }
    }

    pub fn register_branch(&self, id: impl Into<String>) {
        let id = id.into();
        let mut branches = self.state.branches;
        let already_registered = branches.try_peek().is_ok_and(|b| b.contains(&id));
        if !already_registered {
            if let Ok(mut b) = branches.try_write() {
                if !b.contains(&id) {
                    b.push(id);
                }
            }
        }
    }

    pub fn unregister_branch(&self, id: &str) {
        let mut branches = self.state.branches;
        let present = branches
            .try_peek()
            .is_ok_and(|b| b.iter().any(|candidate| candidate == id));
        if present {
            if let Ok(mut b) = branches.try_write() {
                b.retain(|candidate| candidate != id);
            }
        }
    }

    pub fn branches(&self) -> Vec<String> {
        self.state.branches.cloned()
    }

    pub(crate) fn branches_signal(&self) -> Signal<Vec<String>> {
        self.state.branches
    }

    pub fn set_prevent_body_pointer_events_override(&self, override_value: Option<bool>) {
        let mut sig = self.state.prevent_body_pointer_events_override;
        if let Ok(mut current) = sig.try_write() {
            if *current != override_value {
                *current = override_value;
            }
        }
    }

    pub fn effective_prevent_body_pointer_events(&self) -> bool {
        self.state
            .prevent_body_pointer_events_override
            .peek()
            .unwrap_or_else(|| {
                self.dialog
                    .lifecycle()
                    .scroll_lock_policy()
                    .prevent_body_pointer_events()
            })
    }

    pub fn should_force_pointer_events_auto(&self) -> bool {
        (self.dialog.lifecycle().mode().is_modal()
            && self.dialog.lifecycle().scroll_lock_policy().is_enabled()
            && self.effective_prevent_body_pointer_events())
            || is_body_pointer_events_locked()
    }

    pub(crate) fn sync_scroll_lock(&self) {
        sync_dialog_scroll_lock(&self.dialog, self.state, self.dialog.is_open());
    }

    pub(crate) fn sync_modal_shield(&self) {
        sync_dialog_modal_shield(&self.dialog, self.state);
    }

    pub(crate) fn stop_content_dismiss_monitor(&self) {
        let mut mon = self.state.dismiss_monitor;
        if let Ok(mut guard) = mon.try_write() {
            *guard = None;
        }
    }

    pub(crate) fn start_content_dismiss_monitor(
        &self,
        is_alert_dialog: bool,
        mut on_request_close: impl FnMut(bool) + 'static,
    ) {
        self.stop_content_dismiss_monitor();
        if !self.dialog.is_open() {
            return;
        }

        let dialog = self.dialog.clone();
        let state = self.state;
        let content_id = self.content_element_id();
        let trigger_id = self.trigger_element_id();
        let mut initial_boundaries = vec![content_id, trigger_id];
        initial_boundaries.extend(dialog.lifecycle().dismiss_layer().branches().iter().cloned());
        initial_boundaries.extend(state.branches.peek().iter().cloned());
        let boundary_refs: Vec<&str> = initial_boundaries.iter().map(String::as_str).collect();

        let watcher = start_document_dismiss_monitor_with_boundaries(&boundary_refs, move |event| {
            let outside_policy = *dialog.lifecycle().outside_interaction_policy();
            let is_inside = |path_ids: &[String]| {
                let current_cid = state
                    .content_id
                    .try_peek()
                    .ok()
                    .and_then(|v| v.clone())
                    .unwrap_or_else(|| dialog.relationships().content_id().to_string());
                let current_tid = state
                    .trigger_id
                    .try_peek()
                    .ok()
                    .and_then(|v| v.clone())
                    .unwrap_or_else(|| dialog.relationships().trigger_id().to_string());
                let dyn_branches = state
                    .branches
                    .try_peek()
                    .map(|b| b.clone())
                    .unwrap_or_default();
                let focus_guards = dialog.lifecycle().focus_guards();
                let static_branches = dialog.lifecycle().dismiss_layer().branches();

                path_ids.iter().any(|id| {
                    id == &current_cid
                        || id == &current_tid
                        || id == focus_guards.before()
                        || id == focus_guards.after()
                        || static_branches.iter().any(|b| b == id)
                        || dyn_branches.iter().any(|b| b == id)
                })
            };

            match event {
                DocumentDismissEvent::PointerDown { path_ids }
                | DocumentDismissEvent::Click { path_ids } => {
                    if !is_inside(&path_ids)
                        && !is_alert_dialog
                        && outside_policy.pointer_down_outside().dismisses()
                    {
                        on_request_close(true);
                    }
                }
                DocumentDismissEvent::FocusIn { path_ids } => {
                    if !is_inside(&path_ids)
                        && !is_alert_dialog
                        && outside_policy.focus_outside().dismisses()
                    {
                        on_request_close(false);
                    }
                }
                DocumentDismissEvent::Escape => {
                    on_request_close(false);
                }
            }
        });

        let mut mon = self.state.dismiss_monitor;
        mon.set(Some(watcher));
    }

    pub fn apply_open_focus_for_content(&self, content_id: &str) {
        if !self.dialog.lifecycle().focus_scope().autofocus_enabled() {
            return;
        }

        let target_policy = if let Some(target) = self.state.open_focus_target.peek().clone() {
            DialogOpenFocusPolicy::Target(target)
        } else {
            self.dialog.lifecycle().open_focus_policy().clone()
        };

        let content_id = content_id.to_string();
        match target_policy {
            DialogOpenFocusPolicy::FirstFocusable => {
                crate::foundation::browser::spawn_root_task(async move {
                    for _ in 0..5 {
                        let _ = oxidase::frame::next_frame().await;
                        let doc = oxidase::dom::Document::current().unwrap_or_else(oxidase::dom::document);
                        if let Some(el) = doc.element_by_id(&content_id) {
                            if el.is_connected() {
                                focus_first_focusable(&content_id);
                                break;
                            }
                        }
                    }
                });
            }
            DialogOpenFocusPolicy::Target(target) => {
                crate::foundation::browser::spawn_root_task(async move {
                    for _ in 0..5 {
                        let _ = oxidase::frame::next_frame().await;
                        let doc = oxidase::dom::Document::current().unwrap_or_else(oxidase::dom::document);
                        if let Some(el) = doc.element_by_id(&target) {
                            if el.is_connected() {
                                focus_element_by_id(&target);
                                break;
                            }
                        }
                    }
                });
            }
            DialogOpenFocusPolicy::Suppress => {}
        }
    }

    pub fn register_trigger_id(&self, id: impl Into<String>) {
        let mut trigger_id = self.state.trigger_id;
        let id = id.into();
        if trigger_id.peek().as_deref() != Some(id.as_str()) {
            trigger_id.set(Some(id));
        }
    }

    pub fn overlay_element_id(&self) -> String {
        self.state
            .overlay_id
            .peek()
            .clone()
            .unwrap_or_else(|| self.dialog.relationships().overlay_id().to_string())
    }

    pub fn content_element_id(&self) -> String {
        self.state
            .content_id
            .peek()
            .clone()
            .unwrap_or_else(|| self.dialog.relationships().content_id().to_string())
    }

    pub fn trigger_element_id(&self) -> String {
        self.state
            .trigger_id
            .peek()
            .clone()
            .unwrap_or_else(|| self.dialog.relationships().trigger_id().to_string())
    }

    pub fn mount_trigger(&self) -> impl FnMut(MountedEvent) + 'static {
        let runtime = self.clone();
        move |event| runtime.capture_trigger(event.data())
    }

    pub fn mount_overlay(&self) -> impl FnMut(MountedEvent) + 'static {
        let runtime = self.clone();
        move |event| {
            runtime.capture_focus_target(runtime.overlay_element_id(), event.data());
        }
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

    pub fn mark_pointer_dismiss(&self) {
        let mut ptr = self.state.last_dismiss_was_pointer;
        ptr.set(true);
    }

    pub fn clear_pointer_dismiss(&self) {
        let mut ptr = self.state.last_dismiss_was_pointer;
        ptr.set(false);
    }

    pub fn last_dismiss_was_pointer(&self) -> bool {
        *self.state.last_dismiss_was_pointer.peek()
    }

    pub(crate) fn restore_close_focus_with_fallback(&self, fallback_trigger_id: Option<&str>) {
        restore_close_focus(&self.dialog, self.state, fallback_trigger_id);
    }
}

fn sync_dialog_runtime(dialog: &Dialog, mut state: DialogRuntimeState) {
    let presence_update = state
        .presence_lane
        .with_mut(|lane| lane.sync(dialog.is_open()));
    let overlay_id = state
        .overlay_id
        .peek()
        .clone()
        .unwrap_or_else(|| dialog.relationships().overlay_id().to_string());
    let content_id = state
        .content_id
        .peek()
        .clone()
        .unwrap_or_else(|| dialog.relationships().content_id().to_string());

    sync_dialog_presence_root(
        &overlay_id,
        state.presence_lane,
        state.overlay_presence_monitor,
        presence_update.overlay,
        DialogPresenceLane::complete_overlay_close_cycle,
        "overlay",
    );
    sync_dialog_presence_root(
        &content_id,
        state.presence_lane,
        state.content_presence_monitor,
        presence_update.content,
        DialogPresenceLane::complete_content_close_cycle,
        "content",
    );

    let is_open = dialog.is_open();
    let was_open = *state.last_open.peek();

    if is_open && !was_open {
        let mut ptr = state.last_dismiss_was_pointer;
        ptr.set(false);
        if !*state.pending_open_focus.peek() {
            let mut pending_open_focus = state.pending_open_focus;
            pending_open_focus.set(true);
        }
    }

    sync_dialog_scroll_lock(dialog, state, was_open);
    sync_dialog_modal_shield(dialog, state);

    if is_open {
        if *state.pending_open_focus.peek() && apply_open_focus(dialog, state) {
            let mut pending_open_focus = state.pending_open_focus;
            pending_open_focus.set(false);
        }
    } else {
        let mut dismiss_mon = state.dismiss_monitor;
        if let Ok(mut guard) = dismiss_mon.try_write() {
            *guard = None;
        }
        if was_open {
            restore_close_focus(dialog, state, None);
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

fn sync_dialog_scroll_lock(dialog: &Dialog, state: DialogRuntimeState, was_open: bool) {
    let is_open = dialog.is_open();
    let should_hold_scroll_lock = is_open
        && dialog.lifecycle().mode().is_modal()
        && dialog.lifecycle().scroll_lock_policy().is_enabled();
    let is_holding_scroll_lock = *state.scroll_lock_held.peek();
    let prevent_body_pointer_events = state
        .prevent_body_pointer_events_override
        .peek()
        .unwrap_or_else(|| {
            dialog
                .lifecycle()
                .scroll_lock_policy()
                .prevent_body_pointer_events()
        });

    if should_hold_scroll_lock {
        acquire_scroll_lock_with_options(
            dialog.relationships().root_id(),
            prevent_body_pointer_events,
        );
        if !is_holding_scroll_lock {
            let mut scroll_lock_held = state.scroll_lock_held;
            scroll_lock_held.set(true);
        }
    } else if is_holding_scroll_lock {
        let restore_delay = if was_open && !is_open {
            dialog.lifecycle().scroll_lock_policy().restore_delay()
        } else {
            None
        };
        release_scroll_lock(dialog.relationships().root_id(), restore_delay);
        let mut scroll_lock_held = state.scroll_lock_held;
        scroll_lock_held.set(false);
    }
}

fn sync_dialog_modal_shield(dialog: &Dialog, state: DialogRuntimeState) {
    let should_shield = dialog.is_open() && dialog.lifecycle().mode().is_modal();
    let mut modal_shield = state.modal_shield;

    if !should_shield {
        if let Ok(mut guard) = modal_shield.try_write() {
            *guard = None;
        }
        return;
    }

    let content_id = state
        .content_id
        .peek()
        .clone()
        .unwrap_or_else(|| dialog.relationships().content_id().to_string());
    let overlay_id = state
        .overlay_id
        .peek()
        .clone()
        .unwrap_or_else(|| dialog.relationships().overlay_id().to_string());

    let mut keep_ids = vec![content_id, overlay_id];
    for branch in dialog.lifecycle().dismiss_layer().branches() {
        if !keep_ids.contains(branch) {
            keep_ids.push(branch.clone());
        }
    }
    for branch in state.branches.peek().iter() {
        if !keep_ids.contains(branch) {
            keep_ids.push(branch.clone());
        }
    }

    let keep_refs: Vec<&str> = keep_ids.iter().map(String::as_str).collect();
    let next_guard = suppress_external_dom(&keep_refs);
    modal_shield.set(Some(next_guard));
}

fn apply_open_focus(dialog: &Dialog, state: DialogRuntimeState) -> bool {
    if !dialog.lifecycle().focus_scope().autofocus_enabled() {
        return true;
    }

    let policy = if let Some(target) = state.open_focus_target.peek().clone() {
        DialogOpenFocusPolicy::Target(target)
    } else {
        dialog.lifecycle().open_focus_policy().clone()
    };

    match policy {
        DialogOpenFocusPolicy::FirstFocusable => {
            let content_id = state
                .content_id
                .peek()
                .clone()
                .unwrap_or_else(|| dialog.relationships().content_id().to_string());
            focus_first_focusable(&content_id);
            true
        }
        DialogOpenFocusPolicy::Target(target) => {
            focus_element_by_id(&target);
            true
        }
        DialogOpenFocusPolicy::Suppress => true,
    }
}

fn restore_close_focus(
    dialog: &Dialog,
    state: DialogRuntimeState,
    fallback_trigger_id: Option<&str>,
) {
    let was_pointer = *state.last_dismiss_was_pointer.peek();
    let pointer_options = FocusOptions::with_focus_visible(true, Some(false));
    let trigger_id = state
        .trigger_id
        .peek()
        .clone()
        .or_else(|| fallback_trigger_id.map(str::to_owned))
        .unwrap_or_else(|| dialog.relationships().trigger_id().to_string());
    match dialog.lifecycle().close_focus_policy() {
        DialogCloseFocusPolicy::Trigger => {
            if was_pointer {
                restore_focus_element_by_id_with_options(&trigger_id, pointer_options);
            } else {
                restore_focus_element_by_id(&trigger_id);
            }
        }
        DialogCloseFocusPolicy::Target(target) => {
            if was_pointer {
                restore_focus_element_by_id_with_options(target, pointer_options);
            } else {
                restore_focus_element_by_id(target);
            }
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

    let watcher = start_presence_monitor(root_id.as_str(), cycle_id, move |event| {
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
                complete_dialog_presence_close_cycle(
                    presence_lane,
                    monitor_state,
                    event_cycle,
                    complete_close_cycle,
                );
            }
            PresenceMonitorEvent::Stopped {
                cycle_id: event_cycle,
            } => {
                if monitor_state
                    .cycle_id
                    .with_peek(|current| *current == Some(event_cycle))
                {
                    clear_dialog_presence_monitor_state(monitor_state);
                }
            }
        }
    });
    let mut active_monitor = monitor_state.monitor;
    active_monitor.set(Some(watcher));
    let mut active_cycle_id = monitor_state.cycle_id;
    active_cycle_id.set(Some(cycle_id));
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
    _root_label: &str,
    _root_id: &str,
) {
    clear_dialog_presence_monitor_state(monitor_state);
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

use std::collections::HashMap;

use dioxus::prelude::*;

pub use crate::foundation::compose::{
    compose_part_event_handlers, compose_part_refs, project_as_child,
};

use crate::foundation::{
    browser::{
        acquire_scroll_lock, focus_element_by_id,
        focus_first_focusable as foundation_focus_first_focusable, release_scroll_lock,
        restore_focus_element_by_id,
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
    last_open: Signal<bool>,
    pending_open_focus: Signal<bool>,
    scroll_lock_held: Signal<bool>,
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
        last_open: use_signal(|| dialog.is_open()),
        pending_open_focus: use_signal(|| dialog.is_open()),
        scroll_lock_held: use_signal(|| false),
    };
    let effect_state = state;
    let cleanup_state = state;
    let cleanup_key = dialog.relationships().root_id().to_owned();

    use_effect(use_reactive((&dialog,), move |(dialog,)| {
        sync_dialog_runtime(&dialog, effect_state);
    }));

    dioxus::core::use_drop(move || {
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

fn sync_dialog_runtime(dialog: &Dialog, state: DialogRuntimeState) {
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

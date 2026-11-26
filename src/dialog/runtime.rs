use std::collections::HashMap;

use dioxus::prelude::*;

pub use crate::foundation::compose::{
    compose_part_event_handlers, compose_part_refs, project_as_child,
};

use crate::foundation::{
    browser::is_body_pointer_events_locked,
    overlay::{
        DismissCallbacks, DismissConfig, DismissEvent, DismissLayerHandle, FocusScopeConfig,
        FocusScopeHandle, ModalLayerConfig, ModalLayerHandle, ModalShieldPolicy, PresenceHandle,
        use_dismissable_layer, use_focus_scope, use_modal_layer, use_presence,
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
    overlay_presence: PresenceHandle,
    content_presence: PresenceHandle,
    dismiss_layer: DismissLayerHandle,
    modal_layer: ModalLayerHandle,
    focus_scope: FocusScopeHandle,
    prevent_body_pointer_events_override: Signal<Option<bool>>,
    last_dismiss_was_pointer: CopyValue<bool>,
    overlay_id: Signal<Option<String>>,
    content_id: Signal<Option<String>>,
    trigger_id: Signal<Option<String>>,
    open_focus_target: Signal<Option<String>>,
    is_alert_override: Signal<bool>,
    on_dismiss_request: CopyValue<Option<EventHandler<bool>>>,
    on_open_change: CopyValue<Option<EventHandler<bool>>>,
    has_on_open_change_slot: CopyValue<bool>,
}

#[derive(Clone)]
pub struct DialogRuntime {
    dialog: Dialog,
    state: DialogRuntimeState,
}

pub fn use_dialog_runtime(dialog: Dialog) -> DialogRuntime {
    let trigger_handle = use_signal(|| None);
    let content_handle = use_signal(|| None);
    let focus_targets = use_signal(HashMap::new);
    let prevent_body_pointer_events_override = use_signal(|| None::<bool>);
    let last_dismiss_was_pointer = use_hook(|| CopyValue::new(false));
    let overlay_id_sig = use_signal(|| None::<String>);
    let content_id_sig = use_signal(|| None::<String>);
    let trigger_id_sig = use_signal(|| None::<String>);
    let open_focus_target_sig = use_signal(|| None::<String>);
    let is_alert_override = use_signal(|| false);
    let on_dismiss_request = use_hook(|| CopyValue::new(Option::<EventHandler<bool>>::None));
    let on_open_change_slot = use_hook(|| CopyValue::new(Option::<EventHandler<bool>>::None));
    let has_on_open_change_slot = use_hook(|| CopyValue::new(false));

    let is_open = dialog.is_open();
    let retain_mount = dialog.lifecycle().presence().retain_mount();
    let effective_overlay_id = overlay_id_sig
        .read()
        .clone()
        .unwrap_or_else(|| dialog.relationships().overlay_id().to_string());
    let effective_content_id = content_id_sig
        .read()
        .clone()
        .unwrap_or_else(|| dialog.relationships().content_id().to_string());
    let effective_trigger_id = trigger_id_sig
        .read()
        .clone()
        .unwrap_or_else(|| dialog.relationships().trigger_id().to_string());
    let effective_pbpe = prevent_body_pointer_events_override
        .read()
        .unwrap_or_else(|| {
            dialog
                .lifecycle()
                .scroll_lock_policy()
                .prevent_body_pointer_events()
        });

    let overlay_presence = use_presence(is_open, &effective_overlay_id, retain_mount, None);
    let content_presence = use_presence(is_open, &effective_content_id, retain_mount, None);

    let outside_policy = *dialog.lifecycle().outside_interaction_policy();
    let is_alert = *is_alert_override.read();
    let focus_guards = dialog.lifecycle().focus_guards();
    let mut dismiss_boundaries = vec![
        effective_content_id.clone(),
        effective_trigger_id.clone(),
        focus_guards.before().to_string(),
        focus_guards.after().to_string(),
    ];
    for b in dialog.lifecycle().dismiss_layer().branches() {
        if !dismiss_boundaries.contains(b) {
            dismiss_boundaries.push(b.clone());
        }
    }

    let dismiss_config = DismissConfig {
        layer_id: effective_content_id.clone(),
        boundaries: dismiss_boundaries,
        modal: dialog.lifecycle().mode().is_modal(),
        // Modal pointer lock is owned by `use_modal_layer` below so `scroll_lock` &
        // `prevent_body_pointer_events` overrides stay single-source-of-truth.
        disable_outside_pointer_events: false,
        listen_escape: true,
        listen_pointer_down_outside: !is_alert && outside_policy.pointer_down_outside().dismisses(),
        listen_focus_outside: !is_alert && outside_policy.focus_outside().dismisses(),
    };
    let on_dismiss = use_hook(move || {
        EventHandler::new(move |evt: DismissEvent| {
            let was_pointer = matches!(evt, DismissEvent::PointerDownOutside);
            if let Some(handler) = on_dismiss_request.try_read().ok().and_then(|g| *g) {
                handler.call(was_pointer);
            }
        })
    });
    let dismiss_layer = use_dismissable_layer(
        is_open,
        dismiss_config,
        DismissCallbacks {
            on_dismiss: Some(on_dismiss),
            ..Default::default()
        },
    );

    let portal_id = dialog.relationships().scope().qualify("portal");
    let mut keep_ids = vec![
        effective_content_id.clone(),
        effective_overlay_id.clone(),
        portal_id,
    ];
    for branch in dialog.lifecycle().dismiss_layer().branches() {
        if !keep_ids.contains(branch) {
            keep_ids.push(branch.clone());
        }
    }
    for branch in dismiss_layer.branches_signal().read().iter() {
        if !keep_ids.contains(branch) {
            keep_ids.push(branch.clone());
        }
    }
    let modal_layer = use_modal_layer(
        is_open && dialog.lifecycle().mode().is_modal(),
        ModalLayerConfig {
            lock_id: dialog.relationships().root_id().to_string(),
            keep_ids,
            shield_policy: ModalShieldPolicy::InertAndAriaHidden,
            prevent_scroll: dialog.lifecycle().scroll_lock_policy().is_enabled(),
            prevent_body_pointer_events: effective_pbpe,
            restore_delay: dialog.lifecycle().scroll_lock_policy().restore_delay(),
        },
    );

    let effective_open_policy = if let Some(target) = open_focus_target_sig.read().clone() {
        DialogOpenFocusPolicy::Target(target)
    } else {
        dialog.lifecycle().open_focus_policy().clone()
    };
    let (auto_focus_on_open, open_focus_target_id) =
        if !dialog.lifecycle().focus_scope().autofocus_enabled() {
            (false, None)
        } else {
            match effective_open_policy {
                DialogOpenFocusPolicy::FirstFocusable => (true, None),
                DialogOpenFocusPolicy::Target(target) => (true, Some(target)),
                DialogOpenFocusPolicy::Suppress => (false, None),
            }
        };
    let (restore_focus_on_close, restore_target_id) = match dialog.lifecycle().close_focus_policy()
    {
        DialogCloseFocusPolicy::Trigger => (true, Some(effective_trigger_id.clone())),
        DialogCloseFocusPolicy::Target(target) => (true, Some(target.clone())),
        DialogCloseFocusPolicy::None => (false, None),
    };
    let focus_scope = use_focus_scope(
        is_open,
        FocusScopeConfig {
            container_id: effective_content_id,
            restore_target_id,
            open_focus_target_id,
            focusable_selector: Some(DIALOG_FOCUSABLE_SELECTOR.to_string()),
            trap_focus: dialog.lifecycle().focus_scope().traps_focus(),
            loop_focus: dialog.lifecycle().focus_scope().loops_focus(),
            auto_focus_on_open,
            restore_focus_on_close,
            suppress_tab_order: false,
        },
        None,
        None,
    );

    let custom_open_target = open_focus_target_sig.read().clone();
    let mut last_custom_open_target = use_hook(|| CopyValue::new(Option::<String>::None));
    use_effect(use_reactive(
        (&is_open, &custom_open_target),
        move |(open_now, target_opt)| {
            let prev = last_custom_open_target
                .try_read()
                .ok()
                .and_then(|g| g.clone());
            if !open_now {
                if prev.is_some()
                    && let Ok(mut w) = last_custom_open_target.try_write()
                {
                    *w = None;
                }
                return;
            }
            if let Some(target) = target_opt
                && prev.as_deref() != Some(target.as_str())
            {
                if let Ok(mut w) = last_custom_open_target.try_write() {
                    *w = Some(target.clone());
                }
                if !crate::foundation::browser::is_element_focused_by_id(&target) {
                    focus_scope.apply_open_focus();
                }
            }
        },
    ));

    let mut was_open = use_hook(|| CopyValue::new(is_open));
    let mut ptr_dismiss_for_effect = last_dismiss_was_pointer;
    use_effect(use_reactive((&is_open,), move |(open_now,)| {
        let prev = was_open.try_read().map(|g| *g).unwrap_or(false);
        if open_now && !prev {
            if let Ok(mut w) = ptr_dismiss_for_effect.try_write() {
                *w = false;
            }
        } else if !open_now && prev {
            let was_pointer = ptr_dismiss_for_effect
                .try_read()
                .map(|g| *g)
                .unwrap_or(false);
            focus_scope.apply_close_focus(was_pointer);
        }
        if prev != open_now
            && let Ok(mut w) = was_open.try_write()
        {
            *w = open_now;
        }
    }));

    let state = DialogRuntimeState {
        trigger_handle,
        content_handle,
        focus_targets,
        overlay_presence,
        content_presence,
        dismiss_layer,
        modal_layer,
        focus_scope,
        prevent_body_pointer_events_override,
        last_dismiss_was_pointer,
        overlay_id: overlay_id_sig,
        content_id: content_id_sig,
        trigger_id: trigger_id_sig,
        open_focus_target: open_focus_target_sig,
        is_alert_override,
        on_dismiss_request,
        on_open_change: on_open_change_slot,
        has_on_open_change_slot,
    };

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
        self.should_render_overlay() || self.should_render_content()
    }

    pub fn should_render_overlay(&self) -> bool {
        self.state.overlay_presence.should_render()
    }

    pub fn should_render_content(&self) -> bool {
        self.state.content_presence.should_render()
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
            content_id.set(Some(id.clone()));
        }
        self.state.dismiss_layer.rename_layer_id(&id);
        self.state.focus_scope.rename_container_id(&id);
    }

    pub fn unregister_content_id(&self, id: &str) {
        let mut content_id = self.state.content_id;
        if let Ok(mut current) = content_id.try_write()
            && current.as_deref() == Some(id)
        {
            *current = None;
        }
    }

    pub fn set_open_focus_target(&self, target: impl Into<String>) {
        let target = target.into();
        let mut oft = self.state.open_focus_target;
        if oft.peek().as_deref() != Some(target.as_str()) {
            oft.set(Some(target.clone()));
        }
        self.state.focus_scope.set_open_focus_target_id(Some(target));
    }

    pub fn register_branch(&self, id: impl Into<String>) {
        self.state.dismiss_layer.register_branch(id);
    }

    pub fn unregister_branch(&self, id: &str) {
        self.state.dismiss_layer.unregister_branch(id);
    }

    pub fn branches(&self) -> Vec<String> {
        self.state.dismiss_layer.branches_signal().cloned()
    }

    pub fn set_prevent_body_pointer_events_override(&self, override_value: Option<bool>) {
        let mut sig = self.state.prevent_body_pointer_events_override;
        if let Ok(mut current) = sig.try_write()
            && *current != override_value
        {
            *current = override_value;
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
        self.state.modal_layer.should_force_pointer_events_auto()
            || is_body_pointer_events_locked()
    }

    pub(crate) fn set_dismiss_callback(
        &self,
        is_alert_dialog: bool,
        callback: Option<EventHandler<bool>>,
    ) {
        let mut alert_sig = self.state.is_alert_override;
        if alert_sig.try_peek().is_ok_and(|cur| *cur != is_alert_dialog)
            && let Ok(mut w) = alert_sig.try_write()
        {
            *w = is_alert_dialog;
        }
        let mut cb_slot = self.state.on_dismiss_request;
        if let Ok(mut w) = cb_slot.try_write() {
            *w = callback;
        }
    }

    pub(crate) fn handle_escape_keydown(&self, evt: &KeyboardEvent) -> bool {
        self.state.dismiss_layer.handle_keydown(evt)
    }

    pub(crate) fn handle_tab_keydown(&self, evt: &KeyboardEvent) -> bool {
        self.state.focus_scope.handle_tab_keydown(evt)
    }

    pub fn apply_open_focus_for_content(&self, _content_id: &str) {
        self.state.focus_scope.apply_open_focus();
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

    pub fn overlay_element_id_reactive(&self) -> String {
        self.state
            .overlay_id
            .read()
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
        if let Ok(mut w) = ptr.try_write() {
            *w = true;
        }
    }

    pub fn clear_pointer_dismiss(&self) {
        let mut ptr = self.state.last_dismiss_was_pointer;
        if let Ok(mut w) = ptr.try_write() {
            *w = false;
        }
    }

    pub fn last_dismiss_was_pointer(&self) -> bool {
        self.state
            .last_dismiss_was_pointer
            .try_read()
            .map(|g| *g)
            .unwrap_or(false)
    }

    pub(crate) fn set_on_open_change(&self, callback: Option<EventHandler<bool>>) {
        let mut slot = self.state.on_open_change;
        if let Ok(mut w) = slot.try_write() {
            *w = callback;
        }
        let mut has_slot = self.state.has_on_open_change_slot;
        if let Ok(mut w) = has_slot.try_write() {
            *w = true;
        }
    }

    pub(crate) fn notify_open_change(
        &self,
        next: bool,
        fallback: Option<EventHandler<bool>>,
    ) {
        let use_slot = self
            .state
            .has_on_open_change_slot
            .try_read()
            .map(|g| *g)
            .unwrap_or(false);
        let handler = if use_slot {
            self.state.on_open_change.try_read().ok().and_then(|g| *g)
        } else {
            fallback
        };
        if let Some(h) = handler {
            h.call(next);
        }
    }
}

#[cfg(test)]
mod tests {
    use crate::foundation::overlay::{Presence, PresenceController, PresenceState};

    #[test]
    fn phase_3_7_step_4_dialog_presence_lane_keeps_portal_alive_until_both_roots_unmount() {
        let presence = Presence::new(true).with_retained_mount(true);
        let mut overlay = PresenceController::new(presence.desired_present())
            .with_retained_mount(presence.retain_mount());
        let mut content = PresenceController::new(presence.desired_present())
            .with_retained_mount(presence.retain_mount());

        let overlay_close = overlay.sync(false);
        let content_close = content.sync(false);
        let overlay_cycle = overlay_close.started_close_cycle().unwrap();
        let content_cycle = content_close.started_close_cycle().unwrap();

        assert_eq!(overlay_close.state(), PresenceState::Suspended);
        assert_eq!(content_close.state(), PresenceState::Suspended);
        assert!(overlay.should_render() || content.should_render());
        assert!(overlay.should_render());
        assert!(content.should_render());

        assert!(overlay.complete_close_cycle(overlay_cycle));
        assert!(overlay.should_render() || content.should_render());
        assert!(!overlay.should_render());
        assert!(content.should_render());

        assert!(content.complete_close_cycle(content_cycle));
        assert!(!(overlay.should_render() || content.should_render()));
        assert!(!overlay.should_render());
        assert!(!content.should_render());
    }
}

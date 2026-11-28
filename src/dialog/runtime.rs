use std::collections::HashMap;

use dioxus::prelude::*;
use ui_sdk::kernel::ChangeEvent;

pub use crate::foundation::compose::{
    compose_part_event_handlers, compose_part_refs, project_as_child,
};

use crate::foundation::{
    browser::is_body_pointer_events_locked,
    overlay::{
        DismissBehavior, DismissCallbacks, DismissConfig, DismissEvent, DismissLayerHandle,
        FocusScopeConfig, FocusScopeHandle, ModalLayerConfig, ModalLayerHandle, ModalShieldPolicy,
        OverlayLayerSpec, PresenceHandle, sync_overlay_layer_spec,
        use_dismissable_layer_with_details, use_focus_scope, use_modal_layer,
        use_presence_with_keep_mounted,
    },
    state::{DataState, sync_signal},
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
        CloseAutoFocusEvent, DIALOG_FOCUSABLE_SELECTOR, DialogCloseFocusPolicy,
        DialogMountedHandle, DialogOpenFocusPolicy, DialogOutsideInteractionPolicy,
        DialogScrollLockPolicy, EscapeKeyDownEvent, InteractOutsideEvent, OpenAutoFocusEvent,
        PointerDownOutsideEvent,
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
    open_focus_override: Signal<Option<DialogOpenFocusPolicy>>,
    close_focus_override: Signal<Option<DialogCloseFocusPolicy>>,
    scroll_lock_override: Signal<Option<DialogScrollLockPolicy>>,
    outside_interaction_override: Signal<Option<DialogOutsideInteractionPolicy>>,
    prevent_body_pointer_events_override: Signal<Option<bool>>,
    escape_key_down_behavior: Signal<Option<DismissBehavior>>,
    interact_outside_behavior: Signal<Option<DismissBehavior>>,
    content_keep_mounted: Signal<bool>,
    overlay_keep_mounted: Signal<bool>,
    last_dismiss_was_pointer: CopyValue<bool>,
    overlay_id: Signal<Option<String>>,
    content_id: Signal<Option<String>>,
    trigger_id: Signal<Option<String>>,
    open_focus_target: Signal<Option<String>>,
    is_alert_override: Signal<bool>,
    is_nested: Signal<bool>,
    nested_open_count: Signal<usize>,
    on_dismiss_request: CopyValue<Option<EventHandler<bool>>>,
    on_dismiss_details_request: CopyValue<Option<EventHandler<(DismissEvent, ChangeEvent<bool>)>>>,
    on_open_change: CopyValue<Option<EventHandler<bool>>>,
    has_on_open_change_slot: CopyValue<bool>,
    on_open_change_complete: CopyValue<Option<EventHandler<bool>>>,
    on_open_auto_focus: CopyValue<Option<EventHandler<OpenAutoFocusEvent>>>,
    on_close_auto_focus: CopyValue<Option<EventHandler<CloseAutoFocusEvent>>>,
    on_escape_key_down: CopyValue<Option<EventHandler<EscapeKeyDownEvent>>>,
    on_pointer_down_outside: CopyValue<Option<EventHandler<PointerDownOutsideEvent>>>,
    on_interact_outside: CopyValue<Option<EventHandler<InteractOutsideEvent>>>,
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
    let open_focus_override = use_signal(|| None::<DialogOpenFocusPolicy>);
    let close_focus_override = use_signal(|| None::<DialogCloseFocusPolicy>);
    let scroll_lock_override = use_signal(|| None::<DialogScrollLockPolicy>);
    let outside_interaction_override = use_signal(|| None::<DialogOutsideInteractionPolicy>);
    let prevent_body_pointer_events_override = use_signal(|| None::<bool>);
    let escape_key_down_behavior = use_signal(|| None::<DismissBehavior>);
    let interact_outside_behavior = use_signal(|| None::<DismissBehavior>);
    let content_keep_mounted = use_signal(|| false);
    let overlay_keep_mounted = use_signal(|| false);
    let last_dismiss_was_pointer = use_hook(|| CopyValue::new(false));
    let overlay_id_sig = use_signal(|| None::<String>);
    let content_id_sig = use_signal(|| None::<String>);
    let trigger_id_sig = use_signal(|| None::<String>);
    let open_focus_target_sig = use_signal(|| None::<String>);
    let is_alert_override = use_signal(|| false);
    let mut is_nested = use_signal(|| dialog.nested_state().is_nested());
    sync_signal(&mut is_nested, dialog.nested_state().is_nested());
    let mut nested_open_count = use_signal(|| dialog.nested_state().nested_open_dialog_count());
    sync_signal(
        &mut nested_open_count,
        dialog.nested_state().nested_open_dialog_count(),
    );
    let on_dismiss_request = use_hook(|| CopyValue::new(Option::<EventHandler<bool>>::None));
    let on_dismiss_details_request = use_hook(|| {
        CopyValue::new(Option::<EventHandler<(DismissEvent, ChangeEvent<bool>)>>::None)
    });
    let on_open_change_slot = use_hook(|| CopyValue::new(Option::<EventHandler<bool>>::None));
    let has_on_open_change_slot = use_hook(|| CopyValue::new(false));
    let on_open_change_complete = use_hook(|| CopyValue::new(Option::<EventHandler<bool>>::None));
    let on_open_auto_focus =
        use_hook(|| CopyValue::new(Option::<EventHandler<OpenAutoFocusEvent>>::None));
    let on_close_auto_focus =
        use_hook(|| CopyValue::new(Option::<EventHandler<CloseAutoFocusEvent>>::None));
    let on_escape_key_down =
        use_hook(|| CopyValue::new(Option::<EventHandler<EscapeKeyDownEvent>>::None));
    let on_pointer_down_outside =
        use_hook(|| CopyValue::new(Option::<EventHandler<PointerDownOutsideEvent>>::None));
    let on_interact_outside =
        use_hook(|| CopyValue::new(Option::<EventHandler<InteractOutsideEvent>>::None));

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

    let effective_scroll_lock = scroll_lock_override
        .read()
        .clone()
        .unwrap_or_else(|| dialog.lifecycle().scroll_lock_policy().clone());
    let effective_pbpe = prevent_body_pointer_events_override
        .read()
        .unwrap_or_else(|| effective_scroll_lock.prevent_body_pointer_events());

    let on_presence_complete = use_hook(move || {
        EventHandler::new(move |open: bool| {
            if let Some(cb) = on_open_change_complete.try_read().ok().and_then(|s| *s) {
                cb.call(open);
            }
        })
    });

    let overlay_presence = use_presence_with_keep_mounted(
        is_open,
        &effective_overlay_id,
        retain_mount,
        *overlay_keep_mounted.read(),
        None,
    );
    let content_presence = use_presence_with_keep_mounted(
        is_open,
        &effective_content_id,
        retain_mount,
        *content_keep_mounted.read(),
        Some(on_presence_complete),
    );

    let outside_policy = outside_interaction_override
        .read()
        .unwrap_or_else(|| *dialog.lifecycle().outside_interaction_policy());
    let is_alert = *is_alert_override.read();
    let escape_behavior = escape_key_down_behavior
        .read()
        .unwrap_or(DismissBehavior::Close);
    let explicit_outside_behavior = *interact_outside_behavior.read();
    let outside_behavior = explicit_outside_behavior.unwrap_or_else(|| {
        if is_alert {
            DismissBehavior::Ignore
        } else if outside_policy.pointer_down_outside().dismisses() {
            DismissBehavior::Close
        } else {
            DismissBehavior::Ignore
        }
    });

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
        listen_escape: escape_behavior.dismisses(),
        listen_pointer_down_outside: outside_behavior.dismisses(),
        listen_focus_outside: !is_alert
            && outside_behavior.dismisses()
            && outside_policy.focus_outside().dismisses(),
    };
    let on_escape_bridge = use_hook(move || {
        EventHandler::new(move |evt: EscapeKeyDownEvent| {
            if let Some(handler) = on_escape_key_down.try_read().ok().and_then(|g| *g) {
                handler.call(evt);
            }
        })
    });
    let on_pointer_down_outside_bridge = use_hook(move || {
        EventHandler::new(move |evt: PointerDownOutsideEvent| {
            if let Some(handler) = on_pointer_down_outside.try_read().ok().and_then(|g| *g) {
                handler.call(evt);
            }
        })
    });
    let on_interact_outside_bridge = use_hook(move || {
        EventHandler::new(move |evt: InteractOutsideEvent| {
            if let Some(handler) = on_interact_outside.try_read().ok().and_then(|g| *g) {
                handler.call(evt);
            }
        })
    });
    let on_dismiss = use_hook(move || {
        EventHandler::new(move |evt: DismissEvent| {
            let was_pointer = matches!(evt, DismissEvent::PointerDownOutside);
            if let Some(handler) = on_dismiss_request.try_read().ok().and_then(|g| *g) {
                handler.call(was_pointer);
            }
        })
    });
    let on_dismiss_details = use_hook(move || {
        EventHandler::new(move |(evt, change_evt): (DismissEvent, ChangeEvent<bool>)| {
            if let Some(handler) = on_dismiss_details_request.try_read().ok().and_then(|g| *g) {
                handler.call((evt, change_evt));
            } else if let Some(handler) = on_dismiss_request.try_read().ok().and_then(|g| *g) {
                let was_pointer = matches!(evt, DismissEvent::PointerDownOutside);
                handler.call(was_pointer);
            }
        })
    });
    let dismiss_layer = use_dismissable_layer_with_details(
        is_open,
        dismiss_config,
        DismissCallbacks {
            on_escape_key_down: Some(on_escape_bridge),
            on_pointer_down_outside: Some(on_pointer_down_outside_bridge),
            on_focus_outside: None,
            on_interact_outside: Some(on_interact_outside_bridge),
            on_dismiss: Some(on_dismiss),
        },
        Some(on_dismiss_details),
    );

    if is_open && !effective_content_id.is_empty() {
        let mut layer_spec = OverlayLayerSpec::new(effective_content_id.clone())
            .with_modal(dialog.lifecycle().mode().is_modal())
            .with_escape_behavior(escape_behavior)
            .with_outside_press_behavior(outside_behavior);
        if is_alert && explicit_outside_behavior.is_none() {
            layer_spec = layer_spec.with_disable_pointer_dismissal(true);
        }
        sync_overlay_layer_spec(layer_spec);
    }

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
            prevent_scroll: effective_scroll_lock.is_enabled(),
            prevent_body_pointer_events: effective_pbpe,
            restore_delay: effective_scroll_lock.restore_delay(),
        },
    );

    let effective_open_policy = if let Some(target) = open_focus_target_sig.read().clone() {
        DialogOpenFocusPolicy::Target(target)
    } else if let Some(policy) = open_focus_override.read().clone() {
        policy
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
    let effective_close_policy = close_focus_override
        .read()
        .clone()
        .unwrap_or_else(|| dialog.lifecycle().close_focus_policy().clone());
    let (restore_focus_on_close, restore_target_id) = match effective_close_policy {
        DialogCloseFocusPolicy::Trigger => (true, Some(effective_trigger_id.clone())),
        DialogCloseFocusPolicy::Target(target) => (true, Some(target)),
        DialogCloseFocusPolicy::None => (false, None),
    };
    let on_open_focus_bridge = use_hook(move || {
        EventHandler::new(move |evt: OpenAutoFocusEvent| {
            if let Some(handler) = on_open_auto_focus.try_read().ok().and_then(|s| *s) {
                handler.call(evt);
            }
        })
    });
    let on_close_focus_bridge = use_hook(move || {
        EventHandler::new(move |evt: CloseAutoFocusEvent| {
            if let Some(handler) = on_close_auto_focus.try_read().ok().and_then(|s| *s) {
                handler.call(evt);
            }
        })
    });
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
        Some(on_open_focus_bridge),
        Some(on_close_focus_bridge),
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
        open_focus_override,
        close_focus_override,
        scroll_lock_override,
        outside_interaction_override,
        prevent_body_pointer_events_override,
        escape_key_down_behavior,
        interact_outside_behavior,
        content_keep_mounted,
        overlay_keep_mounted,
        last_dismiss_was_pointer,
        overlay_id: overlay_id_sig,
        content_id: content_id_sig,
        trigger_id: trigger_id_sig,
        open_focus_target: open_focus_target_sig,
        is_alert_override,
        is_nested,
        nested_open_count,
        on_dismiss_request,
        on_dismiss_details_request,
        on_open_change: on_open_change_slot,
        has_on_open_change_slot,
        on_open_change_complete,
        on_open_auto_focus,
        on_close_auto_focus,
        on_escape_key_down,
        on_pointer_down_outside,
        on_interact_outside,
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

    pub fn content_data_starting_style(&self) -> Option<&'static str> {
        self.state.content_presence.data_starting_style()
    }

    pub fn content_data_ending_style(&self) -> Option<&'static str> {
        self.state.content_presence.data_ending_style()
    }

    pub fn is_content_hidden(&self) -> bool {
        self.state.content_presence.is_hidden()
    }

    pub fn overlay_data_starting_style(&self) -> Option<&'static str> {
        self.state.overlay_presence.data_starting_style()
    }

    pub fn overlay_data_ending_style(&self) -> Option<&'static str> {
        self.state.overlay_presence.data_ending_style()
    }

    pub fn is_overlay_hidden(&self) -> bool {
        self.state.overlay_presence.is_hidden()
    }

    pub fn set_content_keep_mounted(&self, keep_mounted: bool) {
        let mut sig = self.state.content_keep_mounted;
        sync_signal(&mut sig, keep_mounted);
    }

    pub fn set_overlay_keep_mounted(&self, keep_mounted: bool) {
        let mut sig = self.state.overlay_keep_mounted;
        sync_signal(&mut sig, keep_mounted);
    }

    pub fn set_on_open_change_complete(&self, handler: Option<EventHandler<bool>>) {
        let mut slot = self.state.on_open_change_complete;
        let _ = slot.try_write().map(|mut w| *w = handler);
    }

    #[allow(clippy::too_many_arguments)]
    pub fn set_content_policy_overrides(
        &self,
        open_focus: Option<DialogOpenFocusPolicy>,
        close_focus: Option<DialogCloseFocusPolicy>,
        scroll_lock: Option<DialogScrollLockPolicy>,
        outside_interaction: Option<DialogOutsideInteractionPolicy>,
        prevent_body_pointer_events: Option<bool>,
        escape_key_down_behavior: Option<DismissBehavior>,
        interact_outside_behavior: Option<DismissBehavior>,
    ) {
        let mut of = self.state.open_focus_override;
        sync_signal(&mut of, open_focus);
        let mut cf = self.state.close_focus_override;
        sync_signal(&mut cf, close_focus);
        let mut sl = self.state.scroll_lock_override;
        sync_signal(&mut sl, scroll_lock);
        let mut oi = self.state.outside_interaction_override;
        sync_signal(&mut oi, outside_interaction);
        let mut pbpe = self.state.prevent_body_pointer_events_override;
        sync_signal(&mut pbpe, prevent_body_pointer_events);
        let mut eb = self.state.escape_key_down_behavior;
        sync_signal(&mut eb, escape_key_down_behavior);
        let mut ob = self.state.interact_outside_behavior;
        sync_signal(&mut ob, interact_outside_behavior);
    }

    pub fn set_content_event_handlers(
        &self,
        on_open_auto_focus: Option<EventHandler<OpenAutoFocusEvent>>,
        on_close_auto_focus: Option<EventHandler<CloseAutoFocusEvent>>,
        on_escape_key_down: Option<EventHandler<EscapeKeyDownEvent>>,
        on_pointer_down_outside: Option<EventHandler<PointerDownOutsideEvent>>,
        on_interact_outside: Option<EventHandler<InteractOutsideEvent>>,
    ) {
        let mut oa = self.state.on_open_auto_focus;
        let _ = oa.try_write().map(|mut w| *w = on_open_auto_focus);
        let mut ca = self.state.on_close_auto_focus;
        let _ = ca.try_write().map(|mut w| *w = on_close_auto_focus);
        let mut ek = self.state.on_escape_key_down;
        let _ = ek.try_write().map(|mut w| *w = on_escape_key_down);
        let mut pd = self.state.on_pointer_down_outside;
        let _ = pd.try_write().map(|mut w| *w = on_pointer_down_outside);
        let mut io = self.state.on_interact_outside;
        let _ = io.try_write().map(|mut w| *w = on_interact_outside);
    }

    pub fn should_dismiss_on_outside_pointer(&self, is_alert_dialog: bool) -> bool {
        if let Some(behavior) = self
            .state
            .interact_outside_behavior
            .try_peek()
            .ok()
            .and_then(|b| *b)
        {
            return behavior.dismisses();
        }
        if is_alert_dialog {
            return false;
        }
        let policy = self
            .state
            .outside_interaction_override
            .try_peek()
            .ok()
            .and_then(|p| *p)
            .unwrap_or_else(|| *self.dialog.lifecycle().outside_interaction_policy());
        policy.pointer_down_outside().dismisses()
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
        sync_signal(&mut sig, override_value);
    }

    pub fn effective_prevent_body_pointer_events(&self) -> bool {
        self.state
            .prevent_body_pointer_events_override
            .peek()
            .unwrap_or_else(|| {
                self.state
                    .scroll_lock_override
                    .peek()
                    .as_ref()
                    .map(|sl| sl.prevent_body_pointer_events())
                    .unwrap_or_else(|| {
                        self.dialog
                            .lifecycle()
                            .scroll_lock_policy()
                            .prevent_body_pointer_events()
                    })
            })
    }

    pub fn should_force_pointer_events_auto(&self) -> bool {
        self.state.modal_layer.should_force_pointer_events_auto()
            || is_body_pointer_events_locked()
    }

    pub fn is_nested(&self) -> bool {
        self.state
            .is_nested
            .try_peek()
            .map(|v| *v)
            .unwrap_or_else(|_| self.dialog.nested_state().is_nested())
    }

    pub fn is_nested_reactive(&self) -> bool {
        *self.state.is_nested.read()
    }

    pub fn nested_open_count(&self) -> usize {
        self.state
            .nested_open_count
            .try_peek()
            .map(|v| *v)
            .unwrap_or_else(|_| self.dialog.nested_state().nested_open_dialog_count())
    }

    pub fn nested_open_count_reactive(&self) -> usize {
        *self.state.nested_open_count.read()
    }

    pub fn is_topmost_dismiss_layer(&self) -> bool {
        self.state.dismiss_layer.is_topmost()
    }

    pub fn is_topmost_dialog(&self) -> bool {
        self.nested_open_count() == 0 && self.state.dismiss_layer.is_topmost()
    }

    pub fn set_restore_focus_target(&self, target_id: Option<String>) {
        self.state.focus_scope.set_restore_target(target_id);
    }

    pub(crate) fn set_dismiss_callback(
        &self,
        is_alert_dialog: bool,
        callback: Option<EventHandler<bool>>,
    ) {
        let mut alert_sig = self.state.is_alert_override;
        sync_signal(&mut alert_sig, is_alert_dialog);
        let mut cb_slot = self.state.on_dismiss_request;
        if let Ok(mut w) = cb_slot.try_write() {
            *w = callback;
        }
    }

    pub(crate) fn set_dismiss_details_callback(
        &self,
        is_alert_dialog: bool,
        callback: Option<EventHandler<(DismissEvent, ChangeEvent<bool>)>>,
    ) {
        let mut alert_sig = self.state.is_alert_override;
        sync_signal(&mut alert_sig, is_alert_dialog);
        let mut cb_slot = self.state.on_dismiss_details_request;
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

    #[allow(dead_code)]
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

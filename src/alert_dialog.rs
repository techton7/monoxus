use dioxus::prelude::*;

use crate::{
    dialog::{
        Dialog, DialogCloseAttributes, DialogContentAttributes, DialogDescriptionAttributes,
        DialogLifecycle, DialogMountedHandle, DialogOverlayAttributes, DialogPortalAttributes,
        DialogRelationships, DialogRootAttributes, DialogTitleAttributes, DialogTriggerAttributes,
        DialogRuntime, use_dialog_runtime,
    },
    foundation::{overlay::PortalHost, shared::ScopeHandle, state::DataState},
};

pub use monoxus_core::alert_dialog::*;

#[derive(Clone)]
pub struct AlertDialogRuntime {
    alert: AlertDialog,
    dialog_runtime: DialogRuntime,
}

pub fn use_alert_dialog_runtime(alert: AlertDialog) -> AlertDialogRuntime {
    let dialog_runtime = use_dialog_runtime(alert.dialog().clone());

    AlertDialogRuntime {
        alert,
        dialog_runtime,
    }
}

impl AlertDialogRuntime {
    pub fn alert(&self) -> &AlertDialog {
        &self.alert
    }

    pub fn dialog(&self) -> &Dialog {
        self.alert.dialog()
    }

    pub fn dialog_runtime(&self) -> &DialogRuntime {
        &self.dialog_runtime
    }

    pub const fn is_open(&self) -> bool {
        self.alert.is_open()
    }

    pub fn data_state(&self) -> DataState {
        self.alert.data_state()
    }

    pub fn relationships(&self) -> &DialogRelationships {
        self.alert.relationships()
    }

    pub fn lifecycle(&self) -> &DialogLifecycle {
        self.alert.lifecycle()
    }

    pub fn action_id(&self) -> &str {
        self.alert.action_id()
    }

    pub fn cancel_id(&self) -> &str {
        self.alert.cancel_id()
    }

    pub fn root(&self) -> DialogRootAttributes {
        self.alert.root()
    }

    pub fn trigger(&self) -> DialogTriggerAttributes {
        self.dialog_runtime.trigger()
    }

    pub fn portal(&self) -> DialogPortalAttributes {
        self.alert.portal()
    }

    pub fn overlay(&self) -> DialogOverlayAttributes {
        self.alert.overlay()
    }

    pub fn content(&self) -> DialogContentAttributes {
        self.alert.content()
    }

    pub fn should_render_portal(&self) -> bool {
        self.dialog_runtime.should_render_portal()
    }

    pub fn should_render_overlay(&self) -> bool {
        self.dialog_runtime.should_render_overlay()
    }

    pub fn should_render_content(&self) -> bool {
        self.dialog_runtime.should_render_content()
    }

    pub fn title(&self) -> DialogTitleAttributes {
        self.alert.title()
    }

    pub fn description(&self) -> DialogDescriptionAttributes {
        self.alert.description()
    }

    pub fn close(&self) -> DialogCloseAttributes {
        self.alert.close()
    }

    pub fn action(&self) -> AlertDialogActionAttributes {
        self.alert.action()
    }

    pub fn cancel(&self) -> AlertDialogCancelAttributes {
        self.alert.cancel()
    }

    pub fn trigger_handle(&self) -> Option<DialogMountedHandle> {
        self.dialog_runtime.trigger_handle()
    }

    pub fn content_handle(&self) -> Option<DialogMountedHandle> {
        self.dialog_runtime.content_handle()
    }

    pub fn mounted_focus_target(&self, id: &str) -> Option<DialogMountedHandle> {
        self.dialog_runtime.mounted_focus_target(id)
    }

    pub fn capture_trigger(&self, mounted: DialogMountedHandle) {
        self.dialog_runtime.capture_trigger(mounted);
    }

    pub fn capture_content(&self, mounted: DialogMountedHandle) {
        self.dialog_runtime.capture_content(mounted);
    }

    pub fn capture_close(&self, mounted: DialogMountedHandle) {
        self.dialog_runtime.capture_close(mounted);
    }

    pub fn capture_action(&self, mounted: DialogMountedHandle) {
        self.dialog_runtime
            .capture_focus_target(self.action_id().to_owned(), mounted);
    }

    pub fn capture_cancel(&self, mounted: DialogMountedHandle) {
        self.dialog_runtime
            .capture_focus_target(self.cancel_id().to_owned(), mounted);
    }

    pub fn mount_trigger(&self) -> impl FnMut(MountedEvent) + 'static {
        self.dialog_runtime.mount_trigger()
    }

    pub fn mount_content(&self) -> impl FnMut(MountedEvent) + 'static {
        self.dialog_runtime.mount_content()
    }

    pub fn mount_close(&self) -> impl FnMut(MountedEvent) + 'static {
        self.dialog_runtime.mount_close()
    }

    pub fn mount_action(&self) -> impl FnMut(MountedEvent) + 'static {
        let runtime = self.clone();
        move |event| runtime.capture_action(event.data())
    }

    pub fn mount_cancel(&self) -> impl FnMut(MountedEvent) + 'static {
        let runtime = self.clone();
        move |event| runtime.capture_cancel(event.data())
    }
}

pub use crate::dialog::{
    AlertDialogAction, AlertDialogCancel, CloseAutoFocusEvent, DialogClose, DialogContent,
    DialogDescription, DialogOverlay, DialogPortal, DialogTitle, DialogTrigger, EscapeKeyDownEvent,
    FocusOutsideEvent, InteractOutsideEvent, OpenAutoFocusEvent, PointerDownOutsideEvent,
    PreventableEvent,
};
pub type AlertDialogPreventableEvent = PreventableEvent<()>;

#[component]
pub fn AlertDialogRoot(
    #[props(default)] id: Option<String>,
    #[props(default)] open: Option<Signal<bool>>,
    #[props(default)] default_open: Option<bool>,
    #[props(default)] on_open_change: Option<EventHandler<bool>>,
    #[props(default)] portal_host: Option<PortalHost>,
    #[props(default)] prevent_body_pointer_events: Option<bool>,
    children: Element,
) -> Element {
    let fallback_open = use_signal(|| default_open.unwrap_or(false));
    let open_signal = open.unwrap_or(fallback_open);
    let scope_id =
        crate::foundation::shared::use_stable_id(id.clone().map(std::borrow::Cow::Owned));
    let scope = ScopeHandle::root("alert-dialog").child(scope_id);

    let mut alert = AlertDialog::new(scope, open_signal());
    if let Some(ph) = portal_host {
        alert = alert.with_portal_host(ph);
    }
    if let Some(pbpe) = prevent_body_pointer_events {
        alert
            .dialog_mut()
            .lifecycle_mut()
            .scroll_lock_policy_mut()
            .set_prevent_body_pointer_events(pbpe);
    }

    let alert_runtime = use_alert_dialog_runtime(alert);
    let mut runtime_sig = use_signal(|| alert_runtime.dialog_runtime().clone());
    if runtime_sig.peek().is_open() != alert_runtime.dialog_runtime().is_open()
        || runtime_sig.peek().root().id() != alert_runtime.dialog_runtime().root().id()
    {
        runtime_sig.set(alert_runtime.dialog_runtime().clone());
    }

    let trigger_id = use_signal(|| None);
    let title_id = use_signal(|| None);
    let description_id = use_signal(|| None);

    use_context_provider(|| crate::dialog::DialogContext {
        runtime: runtime_sig,
        open: open_signal,
        on_open_change,
        is_alert_dialog: true,
        trigger_id,
        title_id,
        description_id,
    });

    let mut was_open_sig = use_signal(|| open_signal());
    use_effect(move || {
        let is_curr = open_signal();
        let was_open = *was_open_sig.peek();
        if was_open && !is_curr {
            runtime_sig
                .peek()
                .restore_close_focus_with_fallback(trigger_id.peek().as_deref());
        }
        if was_open != is_curr {
            was_open_sig.set(is_curr);
        }
    });

    let attrs = alert_runtime.root();

    rsx! {
        div {
            id: "{attrs.id()}",
            style: "display: contents;",
            "data-state": attrs.data_state().as_str(),
            {children}
        }
    }
}

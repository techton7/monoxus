use dioxus::prelude::*;

use crate::{
    dialog::{
        Dialog, DialogCloseAttributes, DialogContentAttributes, DialogDescriptionAttributes,
        DialogLifecycle, DialogMountedHandle, DialogOverlayAttributes, DialogPortalAttributes,
        DialogRelationships, DialogRootAttributes, DialogRuntime, DialogTitleAttributes,
        DialogTriggerAttributes, use_dialog_runtime,
    },
    foundation::{
        shared::ScopeHandle,
        state::{DataState, use_controllable_signal},
    },
};

pub use ui_sdk::alert_dialog::*;

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
    AlertDialogAction, AlertDialogActionProps, AlertDialogCancel, AlertDialogCancelProps,
    CloseAutoFocusEvent, DialogClose, DialogCloseProps, DialogContent, DialogContentProps,
    DialogDescription, DialogDescriptionProps, DialogOverlay, DialogOverlayProps, DialogPortal,
    DialogPortalProps, DialogTitle, DialogTitleProps, DialogTrigger, DialogTriggerProps,
    EscapeKeyDownEvent, FocusOutsideEvent, InteractOutsideEvent, OpenAutoFocusEvent,
    PointerDownOutsideEvent, PreventableEvent,
};
pub type AlertDialogPreventableEvent = PreventableEvent<()>;

#[derive(Clone, PartialEq, Props)]
pub struct AlertDialogRootProps {
    #[props(default, into)]
    pub id: Option<String>,
    #[props(default)]
    pub open: Option<bool>,
    #[props(default = false)]
    pub default_open: bool,
    #[props(default)]
    pub on_open_change: Option<EventHandler<bool>>,
    #[props(default)]
    pub on_open_change_complete: Option<EventHandler<bool>>,
    pub children: Element,
}

#[component]
pub fn AlertDialogRoot(props: AlertDialogRootProps) -> Element {
    let AlertDialogRootProps {
        id,
        open,
        default_open,
        on_open_change,
        on_open_change_complete,
        children,
    } = props;
    let open_ctl = use_controllable_signal(open, default_open, on_open_change);
    let open_signal = open_ctl.signal();
    let scope_id =
        crate::foundation::shared::use_stable_id(id.clone().map(std::borrow::Cow::Owned));
    let scope = ScopeHandle::root("alert-dialog").child(scope_id);

    let alert = AlertDialog::new(scope, open_signal());

    let alert_runtime = use_alert_dialog_runtime(alert);
    alert_runtime
        .dialog_runtime()
        .set_on_open_change(on_open_change);
    alert_runtime
        .dialog_runtime()
        .set_on_open_change_complete(on_open_change_complete);
    let mut runtime_sig = use_signal(|| alert_runtime.dialog_runtime().clone());
    if runtime_sig.peek().dialog() != alert_runtime.dialog_runtime().dialog() {
        runtime_sig.set(alert_runtime.dialog_runtime().clone());
    }

    let trigger_id = use_signal(|| None);
    let title_id = use_signal(|| None);
    let description_id = use_signal(|| None);

    use_context_provider(|| crate::dialog::DialogContext {
        runtime: runtime_sig,
        open: open_signal,
        open_ctl,
        on_open_change,
        is_alert_dialog: true,
        trigger_id,
        title_id,
        description_id,
    });

    rsx! {
        {children}
    }
}

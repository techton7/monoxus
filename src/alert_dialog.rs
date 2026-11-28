use dioxus::prelude::*;
use ui_sdk::{agent::AgentHints, kernel::ChangeEvent};

use crate::{
    dialog::{
        Dialog, DialogCloseAttributes, DialogContentAttributes, DialogDescriptionAttributes,
        DialogLifecycle, DialogMountedHandle, DialogNestedState, DialogOverlayAttributes,
        DialogPortalAttributes, DialogRelationships, DialogRootAttributes, DialogRuntime,
        DialogTitleAttributes, DialogTriggerAttributes,
        components::{use_dialog_agent_surface, use_dialog_handle_attachment, use_dialog_nesting},
        use_dialog_runtime,
    },
    foundation::{
        overlay::AlertDialogHandleProp,
        shared::ScopeHandle,
        state::{ControllableConfig, DataState, sync_signal, use_controllable_signal_config},
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
    pub on_open_change_details: Option<EventHandler<ChangeEvent<bool>>>,
    #[props(default)]
    pub on_open_change_complete: Option<EventHandler<bool>>,
    #[props(default, into)]
    pub handle: AlertDialogHandleProp,
    #[props(default)]
    pub agent: Option<AgentHints>,
    pub children: Element,
}

#[component]
pub fn AlertDialogRoot(props: AlertDialogRootProps) -> Element {
    let AlertDialogRootProps {
        id,
        open,
        default_open,
        on_open_change,
        on_open_change_details,
        on_open_change_complete,
        handle,
        agent,
        children,
    } = props;
    let open_ctl = use_controllable_signal_config(
        ControllableConfig::new(open, default_open)
            .with_explicit_default(default_open)
            .with_on_change(on_open_change)
            .with_on_change_details(on_open_change_details)
            .with_component("AlertDialog")
            .with_prop("open"),
    );
    let open_signal = open_ctl.signal();
    let is_open = open_signal();
    let (is_nested, nested_open_count) = use_dialog_nesting(is_open);

    let scope_id =
        crate::foundation::shared::use_stable_id(id.clone().map(std::borrow::Cow::Owned));
    let scope = ScopeHandle::root("alert-dialog").child(scope_id);

    let mut alert = AlertDialog::new(scope, is_open);
    let mut nested_state = DialogNestedState::new(is_nested);
    nested_state.nested_open_dialog_count = nested_open_count;
    alert.dialog_mut().set_nested_state(nested_state);

    let any_handle = handle.as_any().cloned();
    if let Some(ref h) = any_handle {
        for tid in h.registered_trigger_ids() {
            alert
                .dialog_mut()
                .lifecycle_mut()
                .dismiss_layer_mut()
                .register_branch(tid);
        }
    }

    let root_id = alert.relationships().root_id().to_owned();
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

    use_dialog_handle_attachment(
        any_handle.clone(),
        root_id,
        is_open,
        open_signal,
        open_ctl,
        alert_runtime.dialog_runtime(),
        runtime_sig,
    );

    use_dialog_agent_surface(
        "alert-dialog",
        id.unwrap_or_else(|| String::from("alert-dialog")),
        agent.as_ref(),
        open_signal,
        open_ctl,
    );

    let trigger_id = use_signal(|| None);
    let title_id = use_signal(|| None);
    let description_id = use_signal(|| None);
    let mut handle_sig = use_signal(|| any_handle.clone());
    sync_signal(&mut handle_sig, any_handle);

    use_context_provider(|| crate::dialog::DialogContext {
        runtime: runtime_sig,
        open: open_signal,
        open_ctl,
        on_open_change,
        is_alert_dialog: true,
        trigger_id,
        title_id,
        description_id,
        handle: handle_sig,
    });

    rsx! {
        {children}
    }
}

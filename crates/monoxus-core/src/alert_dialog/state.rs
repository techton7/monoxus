use crate::dialog::{
    Dialog, DialogCloseAttributes, DialogContentAttributes, DialogDescriptionAttributes,
    DialogLifecycle, DialogOpenFocusPolicy, DialogOutsideInteractionPolicy,
    DialogOverlayAttributes, DialogPortalAttributes, DialogRelationships, DialogRootAttributes,
    DialogTitleAttributes, DialogTriggerAttributes,
};
use crate::foundation::{overlay::PortalHost, shared::ScopeHandle, state::DataState};

use super::{
    attrs::{AlertDialogActionAttributes, AlertDialogCancelAttributes},
    types::{ALERT_DIALOG_PARTS, AlertDialogPart},
};

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct AlertDialog {
    dialog: Dialog,
    action_id: String,
    cancel_id: String,
}

impl AlertDialog {
    pub fn new(scope: ScopeHandle, open: bool) -> Self {
        let action_id = scope.qualify("action");
        let cancel_id = scope.qualify("cancel");
        let mut dialog = Dialog::new(scope, open)
            .with_outside_interaction_policy(DialogOutsideInteractionPolicy::alert_default());
        {
            let lifecycle = dialog.lifecycle_mut();
            lifecycle.register_branch(action_id.clone());
            lifecycle.register_branch(cancel_id.clone());
            lifecycle.set_open_focus_policy(DialogOpenFocusPolicy::Target(cancel_id.clone()));
        }

        Self {
            dialog,
            action_id,
            cancel_id,
        }
    }

    pub const fn parts() -> &'static [AlertDialogPart] {
        &ALERT_DIALOG_PARTS
    }

    pub const fn dialog(&self) -> &Dialog {
        &self.dialog
    }

    pub fn dialog_mut(&mut self) -> &mut Dialog {
        &mut self.dialog
    }

    pub fn with_portal_host(mut self, portal_host: PortalHost) -> Self {
        self.dialog = self.dialog.with_portal_host(portal_host);
        self
    }

    pub const fn is_open(&self) -> bool {
        self.dialog.is_open()
    }

    pub fn data_state(&self) -> DataState {
        self.dialog.data_state()
    }

    pub const fn relationships(&self) -> &DialogRelationships {
        self.dialog.relationships()
    }

    pub const fn lifecycle(&self) -> &DialogLifecycle {
        self.dialog.lifecycle()
    }

    pub fn lifecycle_mut(&mut self) -> &mut DialogLifecycle {
        self.dialog.lifecycle_mut()
    }

    pub fn action_id(&self) -> &str {
        &self.action_id
    }

    pub fn cancel_id(&self) -> &str {
        &self.cancel_id
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
        self.dialog.content_with_role("alertdialog")
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

    pub fn action(&self) -> AlertDialogActionAttributes {
        AlertDialogActionAttributes::new(self.action_id.clone(), self.data_state())
    }

    pub fn cancel(&self) -> AlertDialogCancelAttributes {
        AlertDialogCancelAttributes::new(self.cancel_id.clone(), self.data_state())
    }
}

use crate::{
    dialog::{
        Dialog, DialogCloseAttributes, DialogContentAttributes, DialogDescriptionAttributes,
        DialogLifecycle, DialogOverlayAttributes, DialogPart, DialogPortalAttributes,
        DialogRelationships, DialogRootAttributes, DialogStateRequest, DialogTitleAttributes,
        DialogTriggerAttributes,
    },
    foundation::{overlay::PortalHost, shared::ScopeHandle, state::DataState},
};

pub const ALERT_DIALOG_PARTS: [AlertDialogPart; 10] = [
    AlertDialogPart::Root,
    AlertDialogPart::Trigger,
    AlertDialogPart::Portal,
    AlertDialogPart::Overlay,
    AlertDialogPart::Content,
    AlertDialogPart::Title,
    AlertDialogPart::Description,
    AlertDialogPart::Close,
    AlertDialogPart::Action,
    AlertDialogPart::Cancel,
];

#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub enum AlertDialogPart {
    Root,
    Trigger,
    Portal,
    Overlay,
    Content,
    Title,
    Description,
    Close,
    Action,
    Cancel,
}

impl AlertDialogPart {
    pub const fn as_str(&self) -> &'static str {
        match self {
            Self::Root => DialogPart::Root.as_str(),
            Self::Trigger => DialogPart::Trigger.as_str(),
            Self::Portal => DialogPart::Portal.as_str(),
            Self::Overlay => DialogPart::Overlay.as_str(),
            Self::Content => DialogPart::Content.as_str(),
            Self::Title => DialogPart::Title.as_str(),
            Self::Description => DialogPart::Description.as_str(),
            Self::Close => DialogPart::Close.as_str(),
            Self::Action => "action",
            Self::Cancel => "cancel",
        }
    }
}

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

        Self {
            dialog: Dialog::new(scope, open),
            action_id,
            cancel_id,
        }
    }

    pub const fn parts() -> &'static [AlertDialogPart] {
        &ALERT_DIALOG_PARTS
    }

    pub fn dialog(&self) -> &Dialog {
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

    pub fn relationships(&self) -> &DialogRelationships {
        self.dialog.relationships()
    }

    pub fn lifecycle(&self) -> &DialogLifecycle {
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
        AlertDialogActionAttributes {
            id: self.action_id.clone(),
            data_state: self.data_state(),
            close_request: DialogStateRequest::Close,
        }
    }

    pub fn cancel(&self) -> AlertDialogCancelAttributes {
        AlertDialogCancelAttributes {
            id: self.cancel_id.clone(),
            data_state: self.data_state(),
            close_request: DialogStateRequest::Close,
        }
    }
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct AlertDialogActionAttributes {
    id: String,
    data_state: DataState,
    close_request: DialogStateRequest,
}

impl AlertDialogActionAttributes {
    pub fn id(&self) -> &str {
        &self.id
    }

    pub fn data_state(&self) -> &DataState {
        &self.data_state
    }

    pub const fn close_request(&self) -> DialogStateRequest {
        self.close_request
    }
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct AlertDialogCancelAttributes {
    id: String,
    data_state: DataState,
    close_request: DialogStateRequest,
}

impl AlertDialogCancelAttributes {
    pub fn id(&self) -> &str {
        &self.id
    }

    pub fn data_state(&self) -> &DataState {
        &self.data_state
    }

    pub const fn close_request(&self) -> DialogStateRequest {
        self.close_request
    }
}

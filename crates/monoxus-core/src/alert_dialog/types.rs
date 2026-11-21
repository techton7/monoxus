use crate::dialog::DialogPart;

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

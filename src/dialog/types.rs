use crate::foundation::{compose::MountedHandle, state::DataState};

pub type DialogMountedHandle = MountedHandle;

pub(crate) const DIALOG_FOCUSABLE_SELECTOR: &str = concat!(
    "[data-monoxus-autofocus],",
    "[autofocus],",
    "button:not([disabled]),",
    "[href],",
    "input:not([disabled]):not([type=\"hidden\"]),",
    "select:not([disabled]),",
    "textarea:not([disabled]),",
    "[tabindex]:not([tabindex=\"-1\"])"
);

pub const DIALOG_PARTS: [DialogPart; 8] = [
    DialogPart::Root,
    DialogPart::Trigger,
    DialogPart::Portal,
    DialogPart::Overlay,
    DialogPart::Content,
    DialogPart::Title,
    DialogPart::Description,
    DialogPart::Close,
];

#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub enum DialogPart {
    Root,
    Trigger,
    Portal,
    Overlay,
    Content,
    Title,
    Description,
    Close,
}

impl DialogPart {
    pub const fn as_str(&self) -> &'static str {
        match self {
            Self::Root => "root",
            Self::Trigger => "trigger",
            Self::Portal => "portal",
            Self::Overlay => "overlay",
            Self::Content => "content",
            Self::Title => "title",
            Self::Description => "description",
            Self::Close => "close",
        }
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub enum DialogStateRequest {
    Open,
    Close,
}

impl DialogStateRequest {
    pub const fn next_open(self) -> bool {
        matches!(self, Self::Open)
    }

    pub fn data_state(self) -> DataState {
        if self.next_open() {
            DataState::Open
        } else {
            DataState::Closed
        }
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub enum DialogMode {
    Modal,
    NonModal,
}

impl DialogMode {
    pub const fn is_modal(self) -> bool {
        matches!(self, Self::Modal)
    }
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub enum DialogOpenFocusPolicy {
    FirstFocusable,
    Target(String),
    Suppress,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub enum DialogCloseFocusPolicy {
    Trigger,
    Target(String),
    None,
}

/// Default restore delay in milliseconds when closing a modal dialog with scroll lock enabled.
///
/// This duration (200ms) matches the default modal exit transition
/// (`monoxus-dialog-fade-out 200ms ease-in forwards`), ensuring the scrollbar
/// does not reappear during the fade-out animation. Premature scrollbar restoration
/// causes a visible horizontal layout jerk as the viewport width shrinks by the scrollbar width.
///
/// For custom exit animation durations, override via [`DialogScrollLockPolicy::with_restore_delay`].
pub const DEFAULT_DIALOG_RESTORE_DELAY_MS: u64 = 200;

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct DialogScrollLockPolicy {
    enabled: bool,
    restore_delay: Option<u64>,
}

impl DialogScrollLockPolicy {
    pub const fn new(enabled: bool) -> Self {
        Self {
            enabled,
            restore_delay: if enabled {
                Some(DEFAULT_DIALOG_RESTORE_DELAY_MS)
            } else {
                None
            },
        }
    }

    pub const fn enabled() -> Self {
        Self::new(true)
    }

    pub const fn disabled() -> Self {
        Self {
            enabled: false,
            restore_delay: None,
        }
    }

    pub const fn with_restore_delay(mut self, restore_delay: Option<u64>) -> Self {
        self.restore_delay = restore_delay;
        self
    }

    pub const fn is_enabled(&self) -> bool {
        self.enabled
    }

    pub const fn restore_delay(&self) -> Option<u64> {
        self.restore_delay
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum DialogOutsideDismissBehavior {
    Dismiss,
    Ignore,
}

impl DialogOutsideDismissBehavior {
    pub const fn dismisses(self) -> bool {
        matches!(self, Self::Dismiss)
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct DialogOutsideInteractionPolicy {
    pointer_down_outside: DialogOutsideDismissBehavior,
    focus_outside: DialogOutsideDismissBehavior,
}

impl DialogOutsideInteractionPolicy {
    pub const fn new(
        pointer_down_outside: DialogOutsideDismissBehavior,
        focus_outside: DialogOutsideDismissBehavior,
    ) -> Self {
        Self {
            pointer_down_outside,
            focus_outside,
        }
    }

    pub const fn modal_default() -> Self {
        Self::new(
            DialogOutsideDismissBehavior::Dismiss,
            DialogOutsideDismissBehavior::Ignore,
        )
    }

    pub const fn non_modal_default() -> Self {
        Self::new(
            DialogOutsideDismissBehavior::Dismiss,
            DialogOutsideDismissBehavior::Dismiss,
        )
    }

    pub const fn alert_default() -> Self {
        Self::new(
            DialogOutsideDismissBehavior::Ignore,
            DialogOutsideDismissBehavior::Ignore,
        )
    }

    pub const fn pointer_down_outside(&self) -> DialogOutsideDismissBehavior {
        self.pointer_down_outside
    }

    pub const fn focus_outside(&self) -> DialogOutsideDismissBehavior {
        self.focus_outside
    }
}

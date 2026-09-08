use crate::foundation::state::DataState;

pub const POPOVER_GEOMETRY_NAMESPACE: &str = "popover";

#[derive(Clone, Debug, PartialEq, Eq)]
pub enum PopoverOpenFocusPolicy {
    FirstFocusable,
    Target(String),
    Suppress,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub enum PopoverCloseFocusPolicy {
    Trigger,
    Target(String),
    None,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum PopoverOutsideDismissBehavior {
    Dismiss,
    Ignore,
}

impl PopoverOutsideDismissBehavior {
    pub const fn dismisses(self) -> bool {
        matches!(self, Self::Dismiss)
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct PopoverOutsideInteractionPolicy {
    pointer_down_outside: PopoverOutsideDismissBehavior,
    focus_outside: PopoverOutsideDismissBehavior,
}

impl PopoverOutsideInteractionPolicy {
    pub const fn new(
        pointer_down_outside: PopoverOutsideDismissBehavior,
        focus_outside: PopoverOutsideDismissBehavior,
    ) -> Self {
        Self {
            pointer_down_outside,
            focus_outside,
        }
    }

    pub const fn modal_default() -> Self {
        Self::new(
            PopoverOutsideDismissBehavior::Dismiss,
            PopoverOutsideDismissBehavior::Ignore,
        )
    }

    pub const fn non_modal_default() -> Self {
        Self::new(
            PopoverOutsideDismissBehavior::Dismiss,
            PopoverOutsideDismissBehavior::Dismiss,
        )
    }

    pub const fn pointer_down_outside(&self) -> PopoverOutsideDismissBehavior {
        self.pointer_down_outside
    }

    pub const fn focus_outside(&self) -> PopoverOutsideDismissBehavior {
        self.focus_outside
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct PopoverScrollLockPolicy {
    enabled: bool,
    restore_delay: Option<u64>,
}

impl PopoverScrollLockPolicy {
    pub const fn new(enabled: bool) -> Self {
        Self {
            enabled,
            restore_delay: None,
        }
    }

    pub const fn enabled() -> Self {
        Self::new(true)
    }

    pub const fn disabled() -> Self {
        Self::new(false)
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

pub const POPOVER_PARTS: [PopoverPart; 7] = [
    PopoverPart::Root,
    PopoverPart::Trigger,
    PopoverPart::Portal,
    PopoverPart::Content,
    PopoverPart::Arrow,
    PopoverPart::Anchor,
    PopoverPart::Close,
];

#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub enum PopoverPart {
    Root,
    Trigger,
    Portal,
    Content,
    Arrow,
    Anchor,
    Close,
}

impl PopoverPart {
    pub const fn as_str(&self) -> &'static str {
        match self {
            Self::Root => "root",
            Self::Trigger => "trigger",
            Self::Portal => "portal",
            Self::Content => "content",
            Self::Arrow => "arrow",
            Self::Anchor => "anchor",
            Self::Close => "close",
        }
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub enum PopoverStateRequest {
    Open,
    Close,
    Toggle,
}

impl PopoverStateRequest {
    pub const fn next_open(self, current_open: bool) -> bool {
        match self {
            Self::Open => true,
            Self::Close => false,
            Self::Toggle => !current_open,
        }
    }

    pub fn data_state(self, current_open: bool) -> DataState {
        if self.next_open(current_open) {
            DataState::Open
        } else {
            DataState::Closed
        }
    }
}

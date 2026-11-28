pub use ui_sdk::foundation::shared::{
    CloseAutoFocusEvent, EscapeKeyDownEvent, FocusOutsideEvent, InteractOutsideEvent,
    OpenAutoFocusEvent, PointerDownOutsideEvent, PreventableEvent,
};
pub use ui_sdk::popover::types::*;

/// Preventable event passed to `Popover` auto-focus, Escape, and outside-interaction callbacks.
pub type PopoverPreventableEvent = PreventableEvent<()>;

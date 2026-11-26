pub use monoxus_core::foundation::shared::{
    CloseAutoFocusEvent, EscapeKeyDownEvent, FocusOutsideEvent, InteractOutsideEvent,
    OpenAutoFocusEvent, PointerDownOutsideEvent, PreventableEvent,
};
pub use monoxus_core::popover::types::*;

/// Preventable event passed to `Popover` auto-focus, Escape, and outside-interaction callbacks.
pub type PopoverPreventableEvent = PreventableEvent<()>;

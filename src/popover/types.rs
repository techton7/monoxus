use std::cell::Cell;
use std::rc::Rc;

pub use monoxus_core::popover::types::*;

/// Preventable event passed to `on_open_auto_focus`, `on_close_auto_focus`, `on_escape_key_down`, and `on_interact_outside`.
#[derive(Clone, Debug, Default)]
pub struct PopoverPreventableEvent {
    default_prevented: Rc<Cell<bool>>,
}

impl PopoverPreventableEvent {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn prevent_default(&self) {
        self.default_prevented.set(true);
    }

    pub fn is_default_prevented(&self) -> bool {
        self.default_prevented.get()
    }
}

pub type OpenAutoFocusEvent = PopoverPreventableEvent;
pub type CloseAutoFocusEvent = PopoverPreventableEvent;
pub type EscapeKeyDownEvent = PopoverPreventableEvent;
pub type InteractOutsideEvent = PopoverPreventableEvent;

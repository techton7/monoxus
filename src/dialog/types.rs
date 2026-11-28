use crate::foundation::compose::MountedHandle;

pub use crate::foundation::shared::{
    CloseAutoFocusEvent, EscapeKeyDownEvent, FocusOutsideEvent, InteractOutsideEvent,
    OpenAutoFocusEvent, PointerDownOutsideEvent, PreventableEvent,
};
pub use crate::compat::dialog::types::*;

pub type DialogMountedHandle = MountedHandle;
pub type DialogPreventableEvent = PreventableEvent<()>;

// [KEEP-MONOXUS-BRAND] Re-exports `ui_sdk::dialog` with `DIALOG_FOCUSABLE_SELECTOR` compat override.
pub use ui_sdk::dialog::{attrs, relationships, state};
pub mod types;

pub use self::{attrs::*, relationships::*, state::*, types::*};

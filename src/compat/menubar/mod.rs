// [KEEP-MONOXUS-BRAND] Re-exports `ui_sdk::menubar` with Monoxus/Radix compat shims.
pub mod attrs;
pub use ui_sdk::menubar::relationships;
pub mod state;
pub mod types;

pub use self::{attrs::*, relationships::*, state::*, types::*};

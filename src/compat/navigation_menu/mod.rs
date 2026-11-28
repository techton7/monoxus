// [KEEP-MONOXUS-BRAND] Re-exports `ui_sdk::navigation_menu` with Monoxus/Radix compat shims.
pub mod attrs;
pub use ui_sdk::navigation_menu::relationships;
pub mod state;
pub mod types;

#[cfg(test)]
mod tests;

pub use self::{attrs::*, relationships::*, state::*, types::*};

// [KEEP-MONOXUS-BRAND] Re-exports `ui_sdk::hover_card` with Monoxus/Radix compat shims.
pub mod attrs;
pub use ui_sdk::hover_card::relationships;
pub mod state;
pub mod types;

#[cfg(test)]
mod tests;

pub use self::{attrs::*, relationships::*, state::*, types::*};

//! [KEEP-MONOXUS-BRAND] Monoxus/Radix collapsible CSS variable compatibility shim.

pub mod attrs;
pub use ui_sdk::collapsible::{relationships, state, types};

pub use self::attrs::*;
pub use ui_sdk::collapsible::{relationships::*, state::*, types::*};

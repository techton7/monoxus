// [KEEP-MONOXUS-BRAND] Re-exports `ui_sdk::command` with dual `--monoxus-command-list-height` / `--bits-command-list-height` compat shim.
pub mod attrs;
pub use ui_sdk::command::{relationships, score, state, types};

pub use self::score as scorer;
pub use self::{attrs::*, relationships::*, score::*, state::*, types::*};

// [STAGE1-DROP] Re-exports `ui_sdk::tabs` with legacy `TabsValueChangeHandler` typedef shim.
pub use ui_sdk::tabs::{attrs, relationships, state};
pub mod types;

pub use self::{attrs::*, relationships::*, state::*, types::*};

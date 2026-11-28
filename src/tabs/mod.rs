pub mod components;
pub mod runtime;

pub use ui_sdk::tabs::{attrs, relationships, state, types};
pub use ui_sdk::tabs::{attrs::*, relationships::*, state::*, types::*};

pub use self::{components::*, runtime::*};
pub use self::components::TabsRoot as Tabs;


pub mod components;
pub mod runtime;

pub use crate::compat::collapsible::{attrs, relationships, state, types};
pub use crate::compat::collapsible::{attrs::*, relationships::*, state::*, types::*};

pub use self::{components::*, runtime::*};
pub use self::components::CollapsibleRoot as Collapsible;

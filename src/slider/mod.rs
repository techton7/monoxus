pub mod components;
pub mod runtime;

pub use ui_sdk::slider::{attrs, relationships, state, types};
pub use ui_sdk::slider::{attrs::*, relationships::*, state::*, types::*};

#[cfg(test)]
mod tests;

pub use self::{components::*, runtime::*};
pub use SliderRoot as Slider;

use std::rc::Rc;

pub use ui_sdk::tabs::types::*;

// [STAGE1-DROP] Legacy callback typedef retained until Stage 1 removes Rc<dyn Fn> aliases.
pub type TabsValueChangeHandler = Rc<dyn Fn(String)>;

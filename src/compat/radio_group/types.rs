// [STAGE1-DROP]
use std::rc::Rc;

pub use ui_sdk::radio_group::types::*;

// [STAGE1-DROP]
pub type RadioGroupValueChangeHandler = Rc<dyn Fn(String)>;

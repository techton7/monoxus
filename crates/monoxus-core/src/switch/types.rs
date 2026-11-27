// [STAGE1-DROP]
use std::rc::Rc;

pub use ui_sdk::switch::types::*;

// [STAGE1-DROP]
pub type SwitchCheckedChangeHandler = Rc<dyn Fn(bool)>;

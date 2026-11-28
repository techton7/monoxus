// [STAGE1-DROP]
use std::rc::Rc;

pub use ui_sdk::checkbox::types::*;

// [STAGE1-DROP]
pub type CheckboxCheckedChangeHandler = Rc<dyn Fn(CheckedState)>;
// [STAGE1-DROP]
pub type CheckboxGroupValuesChangeHandler = Rc<dyn Fn(Vec<String>)>;

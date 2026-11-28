// [STAGE1-DROP]
use std::rc::Rc;

pub use ui_sdk::toggle_group::types::*;

// [STAGE1-DROP]
pub type ToggleGroupSingleChangeHandler = Rc<dyn Fn(String)>;
// [STAGE1-DROP]
pub type ToggleGroupMultipleChangeHandler = Rc<dyn Fn(Vec<String>)>;

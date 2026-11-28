// [STAGE1-DROP]
use std::rc::Rc;

pub use ui_sdk::accordion::types::*;

// [STAGE1-DROP]
pub type AccordionValueChangeHandler = Rc<dyn Fn(Vec<String>)>;

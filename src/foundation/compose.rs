use std::rc::Rc;

use dioxus::prelude::MountedData;

pub use monoxus_core::foundation::compose::*;

pub type MountedHandle = Rc<MountedData>;

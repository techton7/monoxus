pub mod dismiss;
pub mod floating;
pub mod focus;
pub mod grace;
pub mod portal;
pub mod presence;

pub use self::dismiss::*;
pub use self::floating::*;
pub use self::focus::*;
pub use self::grace::*;
pub use self::portal::*;
pub use self::presence::*;

use std::{cell::RefCell, rc::Rc};

#[derive(Clone)]
pub struct OverlayBranchContext {
    register: Rc<RefCell<dyn FnMut(String)>>,
    unregister: Rc<RefCell<dyn FnMut(String)>>,
}

impl PartialEq for OverlayBranchContext {
    fn eq(&self, other: &Self) -> bool {
        Rc::ptr_eq(&self.register, &other.register)
            && Rc::ptr_eq(&self.unregister, &other.unregister)
    }
}

impl OverlayBranchContext {
    pub fn new(
        register_branch: impl FnMut(String) + 'static,
        unregister_branch: impl FnMut(String) + 'static,
    ) -> Self {
        Self {
            register: Rc::new(RefCell::new(register_branch)),
            unregister: Rc::new(RefCell::new(unregister_branch)),
        }
    }

    pub fn from_callbacks(
        register: dioxus::prelude::Callback<String>,
        unregister: dioxus::prelude::Callback<String>,
    ) -> Self {
        Self::new(
            move |id| register.call(id),
            move |id| unregister.call(id),
        )
    }

    pub fn register_branch(&self, branch: impl Into<String>) {
        if let Ok(mut cb) = self.register.try_borrow_mut() {
            cb(branch.into());
        }
    }

    pub fn unregister_branch(&self, branch: impl Into<String>) {
        if let Ok(mut cb) = self.unregister.try_borrow_mut() {
            cb(branch.into());
        }
    }
}

#[cfg(test)]
mod tests;

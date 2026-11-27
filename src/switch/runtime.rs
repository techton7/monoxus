use dioxus::prelude::*;

pub use crate::foundation::state::use_controllable_state;

/// Reactive context exposed by [`SwitchRoot`](super::components::SwitchRoot) to descendant components
/// (e.g. [`SwitchThumb`](super::components::SwitchThumb)).
#[derive(Clone)]
pub struct SwitchContext {
    pub(crate) root_id: String,
    pub(crate) thumb_id: String,
    pub(crate) checked: Signal<bool>,
    pub(crate) disabled: Signal<bool>,
    pub(crate) required: Signal<bool>,
    pub(crate) read_only: Signal<bool>,
    pub(crate) invalid: Signal<bool>,
}

impl SwitchContext {
    pub fn is_checked(&self) -> bool {
        *self.checked.read()
    }

    pub fn is_disabled(&self) -> bool {
        *self.disabled.read()
    }

    pub fn is_required(&self) -> bool {
        *self.required.read()
    }

    pub fn is_read_only(&self) -> bool {
        *self.read_only.read()
    }

    pub fn is_invalid(&self) -> bool {
        *self.invalid.read()
    }

    pub fn root_id(&self) -> &str {
        &self.root_id
    }

    pub fn thumb_id(&self) -> &str {
        &self.thumb_id
    }
}

/// Retrieves the current [`SwitchContext`] from the nearest ancestor [`SwitchRoot`](super::components::SwitchRoot).
pub fn use_switch_context() -> SwitchContext {
    use_context::<SwitchContext>()
}

/// Retrieves the current [`SwitchContext`] if one is present in the current component tree.
pub fn try_use_switch_context() -> Option<SwitchContext> {
    try_consume_context::<SwitchContext>()
}

use dioxus::prelude::*;

pub use crate::foundation::state::use_controllable_state;
use crate::foundation::state::ControllableSignal;

use super::types::CheckedState;

/// Reactive context exposed by [`CheckboxRoot`](super::components::CheckboxRoot) to descendant components
/// (e.g. [`CheckboxIndicator`](super::components::CheckboxIndicator) or custom icon components).
#[derive(Clone)]
pub struct CheckboxContext {
    pub(crate) root_id: String,
    pub(crate) indicator_id: String,
    pub(crate) checked: Signal<CheckedState>,
    pub(crate) disabled: Signal<bool>,
    pub(crate) required: Signal<bool>,
    pub(crate) read_only: Signal<bool>,
}

impl CheckboxContext {
    pub fn checked_state(&self) -> CheckedState {
        *self.checked.read()
    }

    pub fn is_checked(&self) -> bool {
        self.checked_state().is_checked()
    }

    pub fn is_unchecked(&self) -> bool {
        self.checked_state().is_unchecked()
    }

    pub fn is_indeterminate(&self) -> bool {
        self.checked_state().is_indeterminate()
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

    pub fn root_id(&self) -> &str {
        &self.root_id
    }

    pub fn indicator_id(&self) -> &str {
        &self.indicator_id
    }
}

/// Retrieves the current [`CheckboxContext`] from the nearest ancestor [`CheckboxRoot`](super::components::CheckboxRoot).
pub fn use_checkbox_context() -> CheckboxContext {
    use_context::<CheckboxContext>()
}

/// Retrieves the current [`CheckboxContext`] if one is present in the current component tree.
pub fn try_use_checkbox_context() -> Option<CheckboxContext> {
    try_consume_context::<CheckboxContext>()
}

/// Reactive context exposed by [`CheckboxGroup`](super::components::CheckboxGroup) to descendant controls.
#[derive(Clone)]
pub struct CheckboxGroupContext {
    pub(crate) group_id: String,
    pub(crate) name: Option<String>,
    pub(crate) form: Option<String>,
    pub(crate) name_sig: Signal<Option<String>>,
    pub(crate) form_sig: Signal<Option<String>>,
    pub(crate) disabled: Signal<bool>,
    pub(crate) required: Signal<bool>,
    pub(crate) read_only: Signal<bool>,
    pub(crate) values: Signal<Vec<String>>,
    pub(crate) values_ctl: ControllableSignal<Vec<String>>,
    pub(crate) label_id: Signal<Option<String>>,
    pub(crate) reset_epoch: Signal<u64>,
}

impl CheckboxGroupContext {
    pub fn group_id(&self) -> &str {
        &self.group_id
    }

    pub fn name(&self) -> Option<&str> {
        self.name.as_deref()
    }

    pub fn form(&self) -> Option<&str> {
        self.form.as_deref()
    }

    pub(crate) fn reactive_name(&self) -> Option<String> {
        self.name_sig.read().clone()
    }

    pub(crate) fn reactive_form(&self) -> Option<String> {
        self.form_sig.read().clone()
    }

    pub(crate) fn reset_epoch(&self) -> u64 {
        *self.reset_epoch.read()
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

    pub fn values(&self) -> Vec<String> {
        self.values.read().clone()
    }

    pub fn is_item_checked(&self, value: &str) -> bool {
        self.values.read().iter().any(|v| v == value)
    }

    pub fn toggle_item(&self, value: impl Into<String>) {
        if self.is_disabled() || self.is_read_only() {
            return;
        }
        let val = value.into();
        self.values_ctl.update(|current| {
            let mut list = current.clone();
            if let Some(pos) = list.iter().position(|v| v == &val) {
                list.remove(pos);
            } else {
                list.push(val);
            }
            list
        });
    }

    pub fn label_id(&self) -> Option<String> {
        self.label_id.read().clone()
    }

    pub fn set_label_id(&self, id: Option<String>) {
        let mut sig = self.label_id;
        if let Ok(mut guard) = sig.try_write() {
            *guard = id;
        }
    }
}

/// Retrieves the current [`CheckboxGroupContext`] from the nearest ancestor [`CheckboxGroup`](super::components::CheckboxGroup).
pub fn use_checkbox_group_context() -> CheckboxGroupContext {
    use_context::<CheckboxGroupContext>()
}

/// Retrieves the current [`CheckboxGroupContext`] if one is present in the current component tree.
pub fn try_use_checkbox_group_context() -> Option<CheckboxGroupContext> {
    try_consume_context::<CheckboxGroupContext>()
}

use super::types::CheckedState;

/// Pure state machine for an individual checkbox control.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct CheckboxState {
    checked: CheckedState,
    disabled: bool,
    required: bool,
    read_only: bool,
}

impl Default for CheckboxState {
    fn default() -> Self {
        Self::new()
    }
}

impl CheckboxState {
    pub const fn new() -> Self {
        Self {
            checked: CheckedState::Unchecked,
            disabled: false,
            required: false,
            read_only: false,
        }
    }

    pub const fn with_checked(mut self, checked: CheckedState) -> Self {
        self.checked = checked;
        self
    }

    pub const fn with_disabled(mut self, disabled: bool) -> Self {
        self.disabled = disabled;
        self
    }

    pub const fn with_required(mut self, required: bool) -> Self {
        self.required = required;
        self
    }

    pub const fn with_read_only(mut self, read_only: bool) -> Self {
        self.read_only = read_only;
        self
    }

    pub const fn checked(&self) -> CheckedState {
        self.checked
    }

    pub const fn is_disabled(&self) -> bool {
        self.disabled
    }

    pub const fn is_required(&self) -> bool {
        self.required
    }

    pub const fn is_read_only(&self) -> bool {
        self.read_only
    }

    /// Toggles the checkbox state according to WAI-ARIA transition rules:
    /// `Indeterminate -> Checked`, `Unchecked -> Checked`, `Checked -> Unchecked`.
    ///
    /// If `disabled` or `read_only` is true, this is a no-op and returns `false`.
    pub fn toggle(&mut self) -> bool {
        if self.disabled || self.read_only {
            return false;
        }
        let next = self.checked.toggle();
        self.checked = next;
        true
    }

    /// Sets the checked state explicitly.
    ///
    /// If `disabled` or `read_only` is true, this is a no-op and returns `false`.
    pub fn set_checked(&mut self, next: CheckedState) -> bool {
        if self.disabled || self.read_only {
            return false;
        }
        if self.checked != next {
            self.checked = next;
            true
        } else {
            false
        }
    }
}

/// Pure state machine for a checkbox group.
#[derive(Clone, Debug, Default, PartialEq, Eq)]
pub struct CheckboxGroupState {
    values: Vec<String>,
    disabled: bool,
    required: bool,
    read_only: bool,
}

impl CheckboxGroupState {
    pub const fn new() -> Self {
        Self {
            values: Vec::new(),
            disabled: false,
            required: false,
            read_only: false,
        }
    }

    pub fn with_values(mut self, values: Vec<String>) -> Self {
        self.values = values;
        self
    }

    pub const fn with_disabled(mut self, disabled: bool) -> Self {
        self.disabled = disabled;
        self
    }

    pub const fn with_required(mut self, required: bool) -> Self {
        self.required = required;
        self
    }

    pub const fn with_read_only(mut self, read_only: bool) -> Self {
        self.read_only = read_only;
        self
    }

    pub fn values(&self) -> &[String] {
        &self.values
    }

    pub const fn is_disabled(&self) -> bool {
        self.disabled
    }

    pub const fn is_required(&self) -> bool {
        self.required
    }

    pub const fn is_read_only(&self) -> bool {
        self.read_only
    }

    pub fn contains(&self, value: &str) -> bool {
        self.values.iter().any(|v| v == value)
    }

    pub fn add_value(&mut self, value: &str) -> bool {
        if self.disabled || self.read_only {
            return false;
        }
        if !self.contains(value) {
            self.values.push(value.to_string());
            true
        } else {
            false
        }
    }

    pub fn remove_value(&mut self, value: &str) -> bool {
        if self.disabled || self.read_only {
            return false;
        }
        if let Some(pos) = self.values.iter().position(|v| v == value) {
            self.values.remove(pos);
            true
        } else {
            false
        }
    }

    pub fn toggle_value(&mut self, value: &str) -> bool {
        if self.disabled || self.read_only {
            return false;
        }
        if self.contains(value) {
            self.remove_value(value)
        } else {
            self.add_value(value)
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_checked_state_transitions() {
        assert_eq!(CheckedState::Unchecked.toggle(), CheckedState::Checked);
        assert_eq!(CheckedState::Checked.toggle(), CheckedState::Unchecked);
        assert_eq!(CheckedState::Indeterminate.toggle(), CheckedState::Checked);

        assert_eq!(CheckedState::from(true), CheckedState::Checked);
        assert_eq!(CheckedState::from(false), CheckedState::Unchecked);
    }

    #[test]
    fn test_checkbox_state_toggle_and_guard() {
        let mut state = CheckboxState::new();
        assert_eq!(state.checked(), CheckedState::Unchecked);

        assert!(state.toggle());
        assert_eq!(state.checked(), CheckedState::Checked);

        assert!(state.toggle());
        assert_eq!(state.checked(), CheckedState::Unchecked);

        // Disabled guard
        let mut disabled_state = CheckboxState::new().with_disabled(true);
        assert!(!disabled_state.toggle());
        assert_eq!(disabled_state.checked(), CheckedState::Unchecked);
        assert!(!disabled_state.set_checked(CheckedState::Checked));

        // Read-only guard
        let mut readonly_state = CheckboxState::new().with_read_only(true);
        assert!(!readonly_state.toggle());
        assert_eq!(readonly_state.checked(), CheckedState::Unchecked);
        assert!(!readonly_state.set_checked(CheckedState::Checked));
    }

    #[test]
    fn test_checkbox_group_state_manipulation() {
        let mut group = CheckboxGroupState::new();
        assert!(!group.contains("apple"));

        assert!(group.add_value("apple"));
        assert!(group.contains("apple"));
        assert!(!group.add_value("apple")); // duplicate no-op

        assert!(group.toggle_value("banana"));
        assert!(group.contains("banana"));

        assert!(group.toggle_value("apple"));
        assert!(!group.contains("apple"));

        assert_eq!(group.values(), &["banana".to_string()]);

        // Disabled group
        let mut disabled_group = group.clone().with_disabled(true);
        assert!(!disabled_group.add_value("cherry"));
        assert!(!disabled_group.remove_value("banana"));
        assert!(!disabled_group.toggle_value("banana"));
    }
}

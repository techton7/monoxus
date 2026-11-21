/// Pure binary state machine for an individual switch control.
#[derive(Clone, Debug, Default, PartialEq, Eq)]
pub struct SwitchState {
    checked: bool,
    disabled: bool,
    required: bool,
    read_only: bool,
}

impl SwitchState {
    pub const fn new() -> Self {
        Self {
            checked: false,
            disabled: false,
            required: false,
            read_only: false,
        }
    }

    pub const fn with_checked(mut self, checked: bool) -> Self {
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

    pub const fn is_checked(&self) -> bool {
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

    /// Toggles the switch binary state (`false <-> true`).
    ///
    /// If `disabled` or `read_only` is true, this is a no-op and returns `false`.
    pub fn toggle(&mut self) -> bool {
        if self.disabled || self.read_only {
            return false;
        }
        self.checked = !self.checked;
        true
    }

    /// Sets the checked state explicitly.
    ///
    /// If `disabled` or `read_only` is true, this is a no-op and returns `false`.
    pub fn set_checked(&mut self, next: bool) -> bool {
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

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_switch_state_toggle_and_guards() {
        let mut state = SwitchState::new();
        assert!(!state.is_checked());

        assert!(state.toggle());
        assert!(state.is_checked());

        assert!(state.toggle());
        assert!(!state.is_checked());

        // Disabled
        let mut disabled_state = SwitchState::new().with_disabled(true);
        assert!(!disabled_state.toggle());
        assert!(!disabled_state.is_checked());
        assert!(!disabled_state.set_checked(true));

        // Read-only
        let mut readonly_state = SwitchState::new().with_read_only(true);
        assert!(!readonly_state.toggle());
        assert!(!readonly_state.is_checked());
        assert!(!readonly_state.set_checked(true));
    }
}

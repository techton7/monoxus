use std::{borrow::Cow, fmt};

#[derive(Clone, Debug, PartialEq, Eq, Hash)]
pub enum DataState {
    Open,
    Closed,
    Active,
    Inactive,
    Checked,
    Unchecked,
    On,
    Off,
    Indeterminate,
    Custom(Cow<'static, str>),
}

impl DataState {
    pub fn custom(token: impl Into<Cow<'static, str>>) -> Self {
        Self::Custom(token.into())
    }

    pub fn as_str(&self) -> &str {
        match self {
            Self::Open => "open",
            Self::Closed => "closed",
            Self::Active => "active",
            Self::Inactive => "inactive",
            Self::Checked => "checked",
            Self::Unchecked => "unchecked",
            Self::On => "on",
            Self::Off => "off",
            Self::Indeterminate => "indeterminate",
            Self::Custom(token) => token.as_ref(),
        }
    }
}

impl AsRef<str> for DataState {
    fn as_ref(&self) -> &str {
        self.as_str()
    }
}

impl fmt::Display for DataState {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        formatter.write_str(self.as_str())
    }
}

#[derive(Clone, Debug)]
pub struct ControllableStateCell<T> {
    is_controlled: bool,
    owned: T,
}

impl<T> ControllableStateCell<T>
where
    T: Clone + PartialEq,
{
    pub fn new(is_controlled: bool, owned: T) -> Self {
        Self {
            is_controlled,
            owned,
        }
    }

    pub fn is_controlled(&self) -> bool {
        self.is_controlled
    }

    pub fn owned(&self) -> &T {
        &self.owned
    }

    pub fn resolved<'a>(&'a self, controlled: Option<&'a T>) -> &'a T {
        debug_assert_mode_stable(self.is_controlled, controlled.is_some());
        controlled.unwrap_or(&self.owned)
    }

    pub fn write(&mut self, controlled: Option<&T>, next: T) -> Option<T> {
        if self.resolved(controlled) == &next {
            return None;
        }

        if !self.is_controlled {
            self.owned = next.clone();
        }

        Some(next)
    }

    pub fn reduce<A>(
        &mut self,
        controlled: Option<&T>,
        action: A,
        reducer: impl FnOnce(&T, A) -> T,
    ) -> Option<T> {
        let next = reducer(self.resolved(controlled), action);
        self.write(controlled, next)
    }
}

pub fn debug_assert_mode_stable(initially_controlled: bool, currently_controlled: bool) {
    debug_assert_eq!(
        initially_controlled, currently_controlled,
        "controllable state switched ownership mode within one component lifetime",
    );
}

#[cfg(test)]
mod tests {
    use super::{ControllableStateCell, DataState};

    #[test]
    fn data_state_exposes_common_and_custom_tokens() {
        assert_eq!(DataState::Open.as_str(), "open");
        assert_eq!(DataState::Closed.as_str(), "closed");
        assert_eq!(DataState::Indeterminate.to_string(), "indeterminate");
        assert_eq!(DataState::custom("placeholder").as_ref(), "placeholder");
    }

    #[test]
    fn uncontrolled_state_owns_updates_and_suppresses_noops() {
        let mut state = ControllableStateCell::new(false, false);

        assert!(!state.is_controlled());
        assert_eq!(state.write(None, false), None);
        assert_eq!(state.write(None, true), Some(true));
        assert_eq!(state.owned(), &true);
        assert_eq!(state.write(None, true), None);
    }

    #[test]
    fn controlled_state_keeps_external_value_authoritative() {
        let controlled = String::from("open");
        let mut state = ControllableStateCell::new(true, String::from("closed"));

        assert!(state.is_controlled());
        assert_eq!(state.write(Some(&controlled), controlled.clone()), None);
        assert_eq!(
            state.write(Some(&controlled), String::from("closed")),
            Some(String::from("closed"))
        );
        assert_eq!(state.owned(), "closed");
        assert_eq!(state.resolved(Some(&controlled)), "open");
    }

    #[test]
    fn reducer_variant_preserves_meaningful_change_semantics() {
        let mut uncontrolled = ControllableStateCell::new(false, 1_u32);

        assert_eq!(
            uncontrolled.reduce(None, 0, |value, step| value + step),
            None
        );
        assert_eq!(
            uncontrolled.reduce(None, 2, |value, step| value + step),
            Some(3)
        );
        assert_eq!(uncontrolled.owned(), &3);

        let controlled = 10_u32;
        let mut controlled_state = ControllableStateCell::new(true, 1_u32);

        assert_eq!(
            controlled_state.reduce(Some(&controlled), 0, |value, step| value + step),
            None,
        );
        assert_eq!(
            controlled_state.reduce(Some(&controlled), 5, |value, step| value + step),
            Some(15),
        );
        assert_eq!(controlled_state.owned(), &1);
        assert_eq!(controlled_state.resolved(Some(&controlled)), &10);
    }

    #[cfg(debug_assertions)]
    #[test]
    #[should_panic(expected = "switched ownership mode")]
    fn mode_switches_panic_in_debug_builds() {
        let state = ControllableStateCell::new(false, 0_u8);

        let _ = state.resolved(Some(&1_u8));
    }
}

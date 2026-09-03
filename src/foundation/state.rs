use std::{borrow::Cow, fmt, marker::PhantomData, rc::Rc};

use dioxus::prelude::{ReadableExt, Signal, WritableExt, use_hook, use_signal};

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

#[derive(Clone)]
pub struct ControllableStateProps<T, F = fn(T)> {
    pub value: Option<T>,
    pub default_value: T,
    pub on_change: Option<F>,
}

#[derive(Clone)]
pub struct ControllableStateHandle<T> {
    value: T,
    is_controlled: bool,
    set_value: Rc<dyn Fn(T)>,
}

impl<T> ControllableStateHandle<T> {
    pub fn value(&self) -> &T {
        &self.value
    }

    pub fn is_controlled(&self) -> bool {
        self.is_controlled
    }

    pub fn set(&self, next: T) {
        (self.set_value)(next);
    }
}

#[derive(Clone)]
pub struct ControllableStateReducerProps<T, A, R, F = fn(T)> {
    pub value: Option<T>,
    pub default_value: T,
    pub reducer: R,
    pub on_change: Option<F>,
    _marker: PhantomData<fn(A)>,
}

impl<T, A, R, F> ControllableStateReducerProps<T, A, R, F> {
    pub fn new(value: Option<T>, default_value: T, reducer: R, on_change: Option<F>) -> Self {
        Self {
            value,
            default_value,
            reducer,
            on_change,
            _marker: PhantomData,
        }
    }
}

#[derive(Clone)]
pub struct ControllableStateReducerHandle<T, A> {
    value: T,
    is_controlled: bool,
    dispatch_action: Rc<dyn Fn(A)>,
}

impl<T, A> ControllableStateReducerHandle<T, A> {
    pub fn value(&self) -> &T {
        &self.value
    }

    pub fn is_controlled(&self) -> bool {
        self.is_controlled
    }

    pub fn dispatch(&self, action: A) {
        (self.dispatch_action)(action);
    }
}

pub fn use_controllable_state<T, F>(
    props: ControllableStateProps<T, F>,
) -> ControllableStateHandle<T>
where
    T: Clone + PartialEq + 'static,
    F: Fn(T) + Clone + 'static,
{
    let state = use_controllable_parts(props.value, props.default_value);
    let value = state.resolved();
    let set_value = bind_value_writer(
        state.owned,
        state.controlled,
        state.is_controlled,
        props.on_change,
    );

    ControllableStateHandle {
        value,
        is_controlled: state.is_controlled,
        set_value,
    }
}

pub fn use_controllable_state_reducer<T, A, R, F>(
    props: ControllableStateReducerProps<T, A, R, F>,
) -> ControllableStateReducerHandle<T, A>
where
    T: Clone + PartialEq + 'static,
    A: 'static,
    R: Fn(&T, A) -> T + Clone + 'static,
    F: Fn(T) + Clone + 'static,
{
    let state = use_controllable_parts(props.value, props.default_value);
    let value = state.resolved();
    let dispatch_action = bind_reducer_dispatch(
        state.owned,
        state.controlled,
        state.is_controlled,
        props.reducer,
        props.on_change,
    );

    ControllableStateReducerHandle {
        value,
        is_controlled: state.is_controlled,
        dispatch_action,
    }
}

#[derive(Clone)]
struct ControllableParts<T> {
    owned: Signal<T>,
    controlled: Option<T>,
    is_controlled: bool,
}

impl<T> ControllableParts<T>
where
    T: Clone + 'static,
{
    fn resolved(&self) -> T {
        self.controlled
            .clone()
            .unwrap_or_else(|| self.owned.read().clone())
    }
}

fn use_controllable_parts<T>(controlled: Option<T>, default_value: T) -> ControllableParts<T>
where
    T: Clone + 'static,
{
    let owned = use_signal({
        let default_value = default_value.clone();
        move || default_value.clone()
    });
    let is_controlled = use_hook({
        let is_controlled = controlled.is_some();
        move || is_controlled
    });

    debug_assert_mode_stable(is_controlled, controlled.is_some());

    ControllableParts {
        owned,
        controlled,
        is_controlled,
    }
}

fn bind_value_writer<T, F>(
    owned: Signal<T>,
    controlled: Option<T>,
    is_controlled: bool,
    on_change: Option<F>,
) -> Rc<dyn Fn(T)>
where
    T: Clone + PartialEq + 'static,
    F: Fn(T) + Clone + 'static,
{
    Rc::new(move |next| {
        let mut owned = owned;
        let current_owned = owned.read().clone();
        let mut cell = ControllableStateCell::new(is_controlled, current_owned);

        if let Some(published) = cell.write(controlled.as_ref(), next) {
            if !is_controlled {
                owned.set(cell.owned().clone());
            }

            if let Some(on_change) = &on_change {
                on_change(published);
            }
        }
    })
}

fn bind_reducer_dispatch<T, A, R, F>(
    owned: Signal<T>,
    controlled: Option<T>,
    is_controlled: bool,
    reducer: R,
    on_change: Option<F>,
) -> Rc<dyn Fn(A)>
where
    T: Clone + PartialEq + 'static,
    A: 'static,
    R: Fn(&T, A) -> T + Clone + 'static,
    F: Fn(T) + Clone + 'static,
{
    Rc::new(move |action| {
        let mut owned = owned;
        let current_owned = owned.read().clone();
        let mut cell = ControllableStateCell::new(is_controlled, current_owned);

        if let Some(published) = cell.reduce(controlled.as_ref(), action, |value, action| {
            reducer(value, action)
        }) {
            if !is_controlled {
                owned.set(cell.owned().clone());
            }

            if let Some(on_change) = &on_change {
                on_change(published);
            }
        }
    })
}

#[derive(Clone, Debug)]
struct ControllableStateCell<T> {
    is_controlled: bool,
    owned: T,
}

impl<T> ControllableStateCell<T>
where
    T: Clone + PartialEq,
{
    fn new(is_controlled: bool, owned: T) -> Self {
        Self {
            is_controlled,
            owned,
        }
    }

    fn owned(&self) -> &T {
        &self.owned
    }

    fn resolved<'a>(&'a self, controlled: Option<&'a T>) -> &'a T {
        debug_assert_mode_stable(self.is_controlled, controlled.is_some());
        controlled.unwrap_or(&self.owned)
    }

    fn write(&mut self, controlled: Option<&T>, next: T) -> Option<T> {
        if self.resolved(controlled) == &next {
            return None;
        }

        if !self.is_controlled {
            self.owned = next.clone();
        }

        Some(next)
    }

    fn reduce<A>(
        &mut self,
        controlled: Option<&T>,
        action: A,
        reducer: impl FnOnce(&T, A) -> T,
    ) -> Option<T> {
        let next = reducer(self.resolved(controlled), action);
        self.write(controlled, next)
    }
}

fn debug_assert_mode_stable(initially_controlled: bool, currently_controlled: bool) {
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
        assert_eq!(DataState::Indeterminate.to_string(), "indeterminate");
        assert_eq!(DataState::custom("placeholder").as_ref(), "placeholder");
    }

    #[test]
    fn uncontrolled_state_owns_updates_and_suppresses_noops() {
        let mut state = ControllableStateCell::new(false, false);

        assert_eq!(state.write(None, false), None);
        assert_eq!(state.write(None, true), Some(true));
        assert_eq!(state.owned(), &true);
        assert_eq!(state.write(None, true), None);
    }

    #[test]
    fn controlled_state_keeps_external_value_authoritative() {
        let controlled = String::from("open");
        let mut state = ControllableStateCell::new(true, String::from("closed"));

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

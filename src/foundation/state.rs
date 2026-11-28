use std::{marker::PhantomData, rc::Rc};

use dioxus::prelude::{
    CopyValue, EventHandler, ReadSignal, ReadableExt, Signal, WritableExt, use_drop, use_effect,
    use_hook, use_signal,
};

pub use ui_sdk::foundation::state::*;

/// Synchronizes a [`Signal`] to `next` without subscribing to it (render-time / SSR frame-1 safe).
///
/// Returns `true` if the signal value changed and was updated.
pub fn sync_signal<T: PartialEq + 'static>(sig: &mut Signal<T>, next: T) -> bool {
    if sig.try_peek().is_ok_and(|cur| *cur != next)
        && let Ok(mut guard) = sig.try_write()
    {
        *guard = next;
        true
    } else {
        false
    }
}

/// Synchronizes a [`Signal`] to `next` using a custom equality comparator `eq` without subscribing.
///
/// Returns `true` if `!eq(current, &next)` and the signal was updated.
pub fn sync_signal_by<T: 'static>(
    sig: &mut Signal<T>,
    next: T,
    eq: impl Fn(&T, &T) -> bool,
) -> bool {
    if sig.try_peek().is_ok_and(|cur| !eq(&*cur, &next))
        && let Ok(mut guard) = sig.try_write()
    {
        *guard = next;
        true
    } else {
        false
    }
}

/// Synchronizes an `f64` [`Signal`] using bitwise equality (`f64::to_bits`), preventing infinite
/// re-render loops when `f64::NAN` is passed (`NaN != NaN` under `PartialEq`).
pub fn sync_f64_signal(sig: &mut Signal<f64>, next: f64) -> bool {
    sync_signal_by(sig, next, |a, b| a.to_bits() == b.to_bits())
}

/// Unified controlled-value binding for Pattern B widgets that support both legacy two-way
/// `Option<Signal<T>>` (`value`) and unidirectional `Option<ReadSignal<T>>` (`controlled_value`).
///
/// Under `ReadOnly`, widgets never mutate caller state directly and rely on `on_*_change`
/// callbacks to propose updates (`DEC-MFL-03`).
#[allow(dead_code)]
pub(crate) enum ControlledBinding<T: 'static> {
    Uncontrolled,
    TwoWay(Signal<T>),
    ReadOnly(ReadSignal<T>),
}

impl<T: 'static> std::fmt::Debug for ControlledBinding<T> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Uncontrolled => f.write_str("ControlledBinding::Uncontrolled"),
            Self::TwoWay(_) => f.write_str("ControlledBinding::TwoWay(..)"),
            Self::ReadOnly(_) => f.write_str("ControlledBinding::ReadOnly(..)"),
        }
    }
}

impl<T: 'static> Clone for ControlledBinding<T> {
    fn clone(&self) -> Self {
        *self
    }
}

impl<T: 'static> Copy for ControlledBinding<T> {}

impl<T: 'static> PartialEq for ControlledBinding<T> {
    fn eq(&self, other: &Self) -> bool {
        match (self, other) {
            (Self::Uncontrolled, Self::Uncontrolled) => true,
            (Self::TwoWay(a), Self::TwoWay(b)) => a == b,
            (Self::ReadOnly(a), Self::ReadOnly(b)) => a == b,
            _ => false,
        }
    }
}

impl<T: 'static> Eq for ControlledBinding<T> {}

#[allow(dead_code)]
impl<T: 'static> ControlledBinding<T> {
    /// Resolves a binding from legacy `two_way` (`value`) and additive `read_only`
    /// (`controlled_value`) props.
    ///
    /// Asserts in debug builds that both are not provided simultaneously; in release builds,
    /// `read_only` (`controlled_value`) takes precedence.
    pub(crate) fn from_props(
        two_way: Option<Signal<T>>,
        read_only: Option<ReadSignal<T>>,
    ) -> Self {
        debug_assert!(
            !(two_way.is_some() && read_only.is_some()),
            "monoxus: two-way `value` and unidirectional `controlled_value` cannot both be provided simultaneously"
        );
        match (read_only, two_way) {
            (Some(ro), _) => Self::ReadOnly(ro),
            (None, Some(sig)) => Self::TwoWay(sig),
            (None, None) => Self::Uncontrolled,
        }
    }

    #[inline]
    pub(crate) const fn is_controlled(&self) -> bool {
        !matches!(self, Self::Uncontrolled)
    }

    #[inline]
    pub(crate) const fn is_uncontrolled(&self) -> bool {
        matches!(self, Self::Uncontrolled)
    }

    #[inline]
    pub(crate) const fn is_two_way(&self) -> bool {
        matches!(self, Self::TwoWay(_))
    }

    #[inline]
    pub(crate) const fn is_read_only(&self) -> bool {
        matches!(self, Self::ReadOnly(_))
    }

    /// Subscribing read of the controlled value, returning `None` when uncontrolled.
    pub(crate) fn read_cloned(&self) -> Option<T>
    where
        T: Clone,
    {
        match self {
            Self::Uncontrolled => None,
            Self::TwoWay(sig) => Some(sig.read().clone()),
            Self::ReadOnly(sig) => Some(sig.read().clone()),
        }
    }

    /// Non-subscribing, drop-safe peek of the controlled value, returning `None` when uncontrolled.
    pub(crate) fn peek_cloned(&self) -> Option<T>
    where
        T: Clone,
    {
        match self {
            Self::Uncontrolled => None,
            Self::TwoWay(sig) => sig.try_peek().ok().map(|g| g.clone()),
            Self::ReadOnly(sig) => sig.try_peek().ok().map(|g| g.clone()),
        }
    }

    /// Returns the controlled peeked value if controlled, otherwise `default_value`.
    pub(crate) fn initial_or(&self, default_value: T) -> T
    where
        T: Clone,
    {
        self.peek_cloned().unwrap_or(default_value)
    }

    /// Writes `next` to the underlying signal ONLY when bound in [`Self::TwoWay`] mode and the
    /// current value differs from `next`. Strictly a no-op under [`Self::ReadOnly`] and
    /// [`Self::Uncontrolled`].
    pub(crate) fn try_write_two_way(&self, next: T) -> bool
    where
        T: PartialEq,
    {
        match self {
            Self::TwoWay(sig) => {
                let mut sig = *sig;
                sync_signal(&mut sig, next)
            }
            Self::ReadOnly(_) | Self::Uncontrolled => false,
        }
    }

    /// Unconditionally writes `next` ONLY when bound in [`Self::TwoWay`] mode.
    /// Strictly a no-op under [`Self::ReadOnly`] and [`Self::Uncontrolled`].
    pub(crate) fn write_if_two_way(&self, next: T) -> bool {
        match self {
            Self::TwoWay(sig) => {
                let mut sig = *sig;
                if let Ok(mut guard) = sig.try_write() {
                    *guard = next;
                    true
                } else {
                    false
                }
            }
            Self::ReadOnly(_) | Self::Uncontrolled => false,
        }
    }
}

/// Zero-overhead `Copy` controllable state handle backed by a [`Signal`] for reactive value reads
/// and non-reactive [`CopyValue`] slots for ownership mode and `on_change` callback updates.
pub struct ControllableSignal<T: 'static> {
    value: Signal<T>,
    controlled: CopyValue<bool>,
    on_change: CopyValue<Option<EventHandler<T>>>,
}

impl<T: 'static> Clone for ControllableSignal<T> {
    fn clone(&self) -> Self {
        *self
    }
}

impl<T: 'static> Copy for ControllableSignal<T> {}

impl<T: 'static> PartialEq for ControllableSignal<T> {
    fn eq(&self, other: &Self) -> bool {
        self.value == other.value
            && self.controlled == other.controlled
            && self.on_change == other.on_change
    }
}

impl<T: 'static> Eq for ControllableSignal<T> {}

impl<T: 'static> ControllableSignal<T> {
    /// Returns `true` when the value is externally controlled by props.
    pub fn is_controlled(&self) -> bool {
        self.controlled.try_read().map(|g| *g).unwrap_or(false)
    }

    /// Returns the underlying reactive [`Signal<T>`].
    pub fn signal(&self) -> Signal<T> {
        self.value
    }

    /// Reads and clones the current value, subscribing the calling reactive scope.
    pub fn get(&self) -> T
    where
        T: Clone,
    {
        self.value.read().clone()
    }

    /// Peeks and clones the current value without subscribing.
    pub fn peek(&self) -> T
    where
        T: Clone,
    {
        self.value.peek().clone()
    }

    /// Fallibly peeks and clones the current value without subscribing (drop-safe).
    pub fn try_peek_cloned(&self) -> Option<T>
    where
        T: Clone,
    {
        self.value.try_peek().ok().map(|g| g.clone())
    }

    /// Proposes a new value `next`.
    ///
    /// - Suppresses no-op updates when `*current == next`.
    /// - In uncontrolled mode, writes `next` to the internal signal immediately before invoking
    ///   `on_change`.
    /// - In controlled mode, keeps the internal signal untouched (avoiding 1-frame rejected flashes)
    ///   and invokes `on_change(next)`.
    ///
    /// Returns `true` if `next` differed from the current value.
    pub fn set(&self, next: T) -> bool
    where
        T: Clone + PartialEq,
    {
        self.set_by(next, PartialEq::eq)
    }

    /// Proposes a new value `next` using a custom equality comparator `eq` (e.g. `f64::to_bits`).
    pub fn set_by(&self, next: T, eq: impl Fn(&T, &T) -> bool) -> bool
    where
        T: Clone,
    {
        let changed = self
            .value
            .try_peek()
            .is_ok_and(|cur| !eq(&*cur, &next));
        if !changed {
            return false;
        }

        if !self.is_controlled() {
            let mut sig = self.value;
            if let Ok(mut guard) = sig.try_write() {
                *guard = next.clone();
            }
        }

        let cb = self
            .on_change
            .try_read()
            .ok()
            .and_then(|slot| *slot);
        if let Some(handler) = cb {
            handler.call(next);
        }

        true
    }

    /// Computes a next value from the current value via `f` and proposes it with [`Self::set`].
    pub fn update(&self, f: impl FnOnce(&T) -> T) -> bool
    where
        T: Clone + PartialEq,
    {
        let Ok(cur) = self.value.try_peek() else {
            return false;
        };
        let next = f(&*cur);
        drop(cur);
        self.set(next)
    }

    /// Computes a next value from the current value via `f` and proposes it with [`Self::set_by`].
    pub fn update_by(&self, f: impl FnOnce(&T) -> T, eq: impl Fn(&T, &T) -> bool) -> bool
    where
        T: Clone,
    {
        let Ok(cur) = self.value.try_peek() else {
            return false;
        };
        let next = f(&*cur);
        drop(cur);
        self.set_by(next, eq)
    }
}

/// Zero-overhead hook returning a `Copy` [`ControllableSignal<T>`] without `Rc<dyn Fn>` boxing.
///
/// Panics in debug builds if `controlled` switches between `Some` and `None` across renders.
pub fn use_controllable_signal<T>(
    controlled: Option<T>,
    default_value: T,
    on_change: Option<EventHandler<T>>,
) -> ControllableSignal<T>
where
    T: Clone + PartialEq + 'static,
{
    use_controllable_signal_by(controlled, default_value, on_change, |v| v, PartialEq::eq)
}

/// Like [`use_controllable_signal`], applying `sanitize` to the initial default and every
/// incoming controlled prop update.
pub fn use_controllable_signal_with<T>(
    controlled: Option<T>,
    default_value: T,
    on_change: Option<EventHandler<T>>,
    sanitize: impl Fn(T) -> T,
) -> ControllableSignal<T>
where
    T: Clone + PartialEq + 'static,
{
    use_controllable_signal_by(controlled, default_value, on_change, sanitize, PartialEq::eq)
}

/// Like [`use_controllable_signal_with`], using a custom equality comparator `eq` during
/// render-time synchronization (e.g. `f64::to_bits` for floating-point controls).
pub fn use_controllable_signal_by<T>(
    controlled: Option<T>,
    default_value: T,
    on_change: Option<EventHandler<T>>,
    sanitize: impl Fn(T) -> T,
    eq: impl Fn(&T, &T) -> bool,
) -> ControllableSignal<T>
where
    T: Clone + 'static,
{
    let is_controlled_now = controlled.is_some();
    let controlled_slot = use_hook(|| CopyValue::new(is_controlled_now));
    debug_assert_mode_stable(*controlled_slot.read(), is_controlled_now);

    let sanitized_controlled = controlled.map(&sanitize);
    let mut value = use_signal(|| {
        sanitized_controlled
            .clone()
            .unwrap_or_else(|| sanitize(default_value))
    });

    if let Some(next) = sanitized_controlled {
        sync_signal_by(&mut value, next, eq);
    }

    let mut on_change_slot = use_hook(|| CopyValue::new(on_change));
    on_change_slot.set(on_change);

    ControllableSignal {
        value,
        controlled: controlled_slot,
        on_change: on_change_slot,
    }
}

/// Runs a reactive `effect` returning a cleanup closure executed before each subsequent re-run
/// and on component unmount (`use_drop`), matching the `DioxusLabs/dioxus-components`
/// `CopyValue<Option<C>>` pattern with borrow-safe cleanup invocation.
pub fn use_effect_with_cleanup<F, C>(mut effect: F)
where
    F: FnMut() -> C + 'static,
    C: FnOnce() + 'static,
{
    let mut cleanup: CopyValue<Option<C>> = use_hook(|| CopyValue::new(None));

    use_drop(move || {
        let mut cleanup = cleanup;
        let prev = cleanup.try_write().ok().and_then(|mut slot| slot.take());
        if let Some(cb) = prev {
            cb();
        }
    });

    use_effect(move || {
        let prev = cleanup.try_write().ok().and_then(|mut slot| slot.take());
        if let Some(cb) = prev {
            cb();
        }
        let next_cleanup = effect();
        if let Ok(mut slot) = cleanup.try_write() {
            *slot = Some(next_cleanup);
        }
    });
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

#[cfg(test)]
mod tests {
    use std::cell::{Cell, RefCell};
    use std::rc::Rc;

    use dioxus::prelude::*;

    use super::{
        ControllableSignal, sync_f64_signal, sync_signal, use_controllable_signal,
        use_controllable_signal_with, use_effect_with_cleanup,
    };

    #[component]
    fn SyncAndControllableHarness(
        controlled_val: Signal<i32>,
        out_uncontrolled: Rc<RefCell<Option<ControllableSignal<String>>>>,
        out_controlled: Rc<RefCell<Option<ControllableSignal<i32>>>>,
        published: Rc<RefCell<Vec<i32>>>,
        nan_sync_changes: Rc<Cell<usize>>,
        cleanup_runs: Rc<Cell<usize>>,
        mount_child: Signal<bool>,
    ) -> Element {
        let mut f64_sig = use_signal(|| f64::NAN);
        if sync_f64_signal(&mut f64_sig, f64::NAN) {
            nan_sync_changes.set(nan_sync_changes.get() + 1);
        }

        let mut int_sig = use_signal(|| 1_i32);
        assert!(!sync_signal(&mut int_sig, 1));
        assert!(sync_signal(&mut int_sig, 2));
        assert!(!sync_signal(&mut int_sig, 2));

        let unc = use_controllable_signal(None, String::from("init"), None);
        *out_uncontrolled.borrow_mut() = Some(unc);

        let pub_clone = Rc::clone(&published);
        let ctl = use_controllable_signal_with(
            Some(controlled_val()),
            0,
            Some(EventHandler::new(move |next| {
                pub_clone.borrow_mut().push(next);
            })),
            |v| v.clamp(0, 100),
        );
        *out_controlled.borrow_mut() = Some(ctl);

        rsx! {
            if mount_child() {
                CleanupChild { cleanup_runs: Rc::clone(&cleanup_runs) }
            }
        }
    }

    #[component]
    fn CleanupChild(cleanup_runs: Rc<Cell<usize>>) -> Element {
        let runs = Rc::clone(&cleanup_runs);
        use_effect_with_cleanup(move || {
            let r = Rc::clone(&runs);
            move || {
                r.set(r.get() + 1);
            }
        });
        rsx! { div {} }
    }

    #[test]
    fn controllable_signal_handles_uncontrolled_controlled_rejection_nan_sync_and_cleanup() {
        let out_unc = Rc::new(RefCell::new(None));
        let out_ctl = Rc::new(RefCell::new(None));
        let published = Rc::new(RefCell::new(Vec::new()));
        let nan_changes = Rc::new(Cell::new(0));
        let cleanup_runs = Rc::new(Cell::new(0));
        let ctl_sig_holder: Rc<RefCell<Option<Signal<i32>>>> = Rc::new(RefCell::new(None));
        let mount_sig_holder: Rc<RefCell<Option<Signal<bool>>>> = Rc::new(RefCell::new(None));

        #[component]
        fn Wrapper(
            out_unc: Rc<RefCell<Option<ControllableSignal<String>>>>,
            out_ctl: Rc<RefCell<Option<ControllableSignal<i32>>>>,
            published: Rc<RefCell<Vec<i32>>>,
            nan_changes: Rc<Cell<usize>>,
            cleanup_runs: Rc<Cell<usize>>,
            ctl_sig_holder: Rc<RefCell<Option<Signal<i32>>>>,
            mount_sig_holder: Rc<RefCell<Option<Signal<bool>>>>,
        ) -> Element {
            let ctl_sig = use_signal(|| 10_i32);
            let mount_sig = use_signal(|| true);
            *ctl_sig_holder.borrow_mut() = Some(ctl_sig);
            *mount_sig_holder.borrow_mut() = Some(mount_sig);
            rsx! {
                SyncAndControllableHarness {
                    controlled_val: ctl_sig,
                    out_uncontrolled: out_unc,
                    out_controlled: out_ctl,
                    published,
                    nan_sync_changes: nan_changes,
                    cleanup_runs,
                    mount_child: mount_sig,
                }
            }
        }

        let mut dom = VirtualDom::new_with_props(
            Wrapper,
            WrapperProps {
                out_unc: Rc::clone(&out_unc),
                out_ctl: Rc::clone(&out_ctl),
                published: Rc::clone(&published),
                nan_changes: Rc::clone(&nan_changes),
                cleanup_runs: Rc::clone(&cleanup_runs),
                ctl_sig_holder: Rc::clone(&ctl_sig_holder),
                mount_sig_holder: Rc::clone(&mount_sig_holder),
            },
        );
        dom.rebuild_in_place();
        dom.render_immediate(&mut dioxus::core::NoOpMutations);

        assert_eq!(nan_changes.get(), 0);

        // Uncontrolled updates in place and suppresses no-ops
        let unc = out_unc.borrow().unwrap();
        assert!(!unc.is_controlled());
        assert_eq!(unc.peek(), "init");
        assert!(!unc.set(String::from("init")));
        assert!(unc.set(String::from("next")));
        assert_eq!(unc.peek(), "next");

        // Controlled rejection keeps signal at controlled prop (zero 1-frame flash)
        let ctl = out_ctl.borrow().unwrap();
        assert!(ctl.is_controlled());
        assert_eq!(ctl.peek(), 10);
        assert!(ctl.set(42));
        assert_eq!(ctl.peek(), 10);
        assert_eq!(*published.borrow(), vec![42]);

        // Unmounting CleanupChild invokes cleanup closure once
        let mut mount_sig = mount_sig_holder.borrow().unwrap();
        mount_sig.set(false);
        dom.render_immediate(&mut dioxus::core::NoOpMutations);
        assert_eq!(cleanup_runs.get(), 1);
    }

    #[test]
    fn controlled_binding_distinguishes_uncontrolled_two_way_and_read_only() {
        use super::ControlledBinding;

        let results: Rc<RefCell<Vec<String>>> = Rc::new(RefCell::new(Vec::new()));

        #[component]
        fn BindingHarness(results: Rc<RefCell<Vec<String>>>) -> Element {
            let mut two_way_sig = use_signal(|| String::from("a"));
            let ro_source = use_signal(|| String::from("locked"));
            let ro_sig: ReadSignal<String> = ro_source.into();

            let unc: ControlledBinding<String> = ControlledBinding::from_props(None, None);
            assert!(unc.is_uncontrolled());
            assert!(!unc.is_controlled());
            assert_eq!(unc.read_cloned(), None);
            assert_eq!(unc.peek_cloned(), None);
            assert_eq!(unc.initial_or(String::from("fallback")), "fallback");
            assert!(!unc.try_write_two_way(String::from("ignored")));

            let tw = ControlledBinding::from_props(Some(two_way_sig), None);
            assert!(tw.is_controlled());
            assert!(tw.is_two_way());
            assert!(!tw.is_read_only());
            assert_eq!(tw.peek_cloned().as_deref(), Some("a"));
            assert!(!tw.try_write_two_way(String::from("a")));
            assert!(tw.try_write_two_way(String::from("b")));
            assert_eq!(two_way_sig.peek().as_str(), "b");

            let ro = ControlledBinding::from_props(None, Some(ro_sig));
            assert!(ro.is_controlled());
            assert!(ro.is_read_only());
            assert!(!ro.is_two_way());
            assert_eq!(ro.read_cloned().as_deref(), Some("locked"));
            assert_eq!(ro.initial_or(String::from("fallback")), "locked");
            assert!(!ro.try_write_two_way(String::from("mutated")));
            assert!(!ro.write_if_two_way(String::from("mutated")));
            assert_eq!(ro_source.peek().as_str(), "locked");

            // Equality check across variants
            assert_ne!(tw, ro);
            assert_eq!(unc, ControlledBinding::Uncontrolled);
            assert_eq!(tw, ControlledBinding::TwoWay(two_way_sig));
            assert_eq!(ro, ControlledBinding::ReadOnly(ro_sig));

            // Suppress unused mut warning
            two_way_sig.set(String::from("b"));
            results.borrow_mut().push(String::from("ok"));
            rsx! { div {} }
        }

        let mut dom = VirtualDom::new_with_props(
            BindingHarness,
            BindingHarnessProps {
                results: Rc::clone(&results),
            },
        );
        dom.rebuild_in_place();
        assert_eq!(*results.borrow(), vec![String::from("ok")]);
    }

    #[cfg(debug_assertions)]
    #[test]
    #[should_panic(expected = "cannot both be provided simultaneously")]
    fn controlled_binding_panics_in_debug_when_both_two_way_and_read_only_provided() {
        use super::ControlledBinding;

        let captured: Rc<RefCell<Option<(Signal<i32>, ReadSignal<i32>)>>> =
            Rc::new(RefCell::new(None));

        #[component]
        fn DualBindingHarness(
            captured: Rc<RefCell<Option<(Signal<i32>, ReadSignal<i32>)>>>,
        ) -> Element {
            let sig = use_signal(|| 1_i32);
            let ro: ReadSignal<i32> = sig.into();
            *captured.borrow_mut() = Some((sig, ro));
            rsx! { div {} }
        }

        let mut dom = VirtualDom::new_with_props(
            DualBindingHarness,
            DualBindingHarnessProps {
                captured: Rc::clone(&captured),
            },
        );
        dom.rebuild_in_place();
        let (sig, ro) = captured.borrow().expect("signals captured");
        let _ = ControlledBinding::from_props(Some(sig), Some(ro));
    }
}

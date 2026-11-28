use std::{marker::PhantomData, rc::Rc};

use dioxus::prelude::{
    CopyValue, EventHandler, ReadSignal, ReadableExt, Signal, WritableExt, use_drop, use_effect,
    use_hook, use_signal,
};
#[cfg(debug_assertions)]
use ui_sdk::kernel::AskToken;
use ui_sdk::kernel::{
    ChangeContext, ChangeEvent, ChangeReason, ChangeSource, CommitDecision, TransitionGate,
};

pub use ui_sdk::foundation::state::*;

#[cfg(debug_assertions)]
use super::diagnostics::{
    DevDiagnostic, DiagnosticKind, cancel_pending_controlled_check, emit_diagnostic,
    next_diagnostic_instance_id, register_pending_controlled_check,
};

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
        #[cfg(debug_assertions)]
        if two_way.is_some() && read_only.is_some() {
            emit_diagnostic(DevDiagnostic {
                kind: DiagnosticKind::TwoWayAndReadOnlyConflict,
                component: "ControlledBinding",
                instance_id: String::new(),
                prop: "value",
                message: String::from(
                    "two-way `value` and unidirectional `controlled_value` cannot both be provided simultaneously",
                ),
            });
        }
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

/// Result of proposing a state transition through [`ControllableSignal`].
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum ChangeOutcome {
    /// Uncontrolled transition committed into the internal [`Signal`].
    Committed(ChangeContext),
    /// Controlled transition accepted by listeners and forwarded to `on_change`;
    /// the parent owns the authoritative value.
    ControlledPending(ChangeContext),
    /// A listener called `event.cancel()`; state remained untouched and legacy
    /// `on_change` was skipped.
    Canceled(ChangeContext),
    /// A re-entrant transition superseded this outer proposal; state and legacy
    /// `on_change` were skipped for the outer call.
    Superseded(ChangeContext),
    /// Proposed value matched the current (or top in-flight) value; no-op.
    Unchanged,
}

impl ChangeOutcome {
    /// Returns `true` when the transition committed into uncontrolled state.
    #[inline]
    pub const fn is_committed(&self) -> bool {
        matches!(self, Self::Committed(_))
    }

    /// Returns `true` when the transition was accepted (`Committed` or
    /// `ControlledPending`).
    #[inline]
    pub const fn is_accepted(&self) -> bool {
        matches!(self, Self::Committed(_) | Self::ControlledPending(_))
    }

    /// Returns `true` when a listener explicitly canceled the transition.
    #[inline]
    pub const fn is_canceled(&self) -> bool {
        matches!(self, Self::Canceled(_))
    }

    /// Returns the finalized [`ChangeContext`] if a transition was proposed.
    #[inline]
    pub const fn context(&self) -> Option<ChangeContext> {
        match *self {
            Self::Committed(ctx)
            | Self::ControlledPending(ctx)
            | Self::Canceled(ctx)
            | Self::Superseded(ctx) => Some(ctx),
            Self::Unchanged => None,
        }
    }
}

/// Configuration for [`use_controllable_signal_config`].
pub struct ControllableConfig<T: 'static, S = fn(T) -> T, E = fn(&T, &T) -> bool> {
    pub controlled: Option<T>,
    pub default_value: T,
    pub has_explicit_default: bool,
    pub on_change: Option<EventHandler<T>>,
    pub on_change_details: Option<EventHandler<ChangeEvent<T>>>,
    pub sanitize: S,
    pub eq: E,
    pub component: &'static str,
    pub prop: &'static str,
}

#[inline]
fn identity_sanitize<T>(value: T) -> T {
    value
}

impl<T: PartialEq + 'static> ControllableConfig<T> {
    /// Creates a [`ControllableConfig`] with identity sanitization and
    /// [`PartialEq::eq`] equality.
    pub fn new(controlled: Option<T>, default_value: T) -> Self {
        Self {
            controlled,
            default_value,
            has_explicit_default: false,
            on_change: None,
            on_change_details: None,
            sanitize: identity_sanitize::<T>,
            eq: PartialEq::eq,
            component: "ControllableSignal",
            prop: "value",
        }
    }
}

impl<T: 'static, S, E> ControllableConfig<T, S, E> {
    pub fn with_explicit_default(mut self, has_explicit_default: bool) -> Self {
        self.has_explicit_default = has_explicit_default;
        self
    }

    pub fn with_on_change(mut self, on_change: Option<EventHandler<T>>) -> Self {
        self.on_change = on_change;
        self
    }

    pub fn with_on_change_details(
        mut self,
        on_change_details: Option<EventHandler<ChangeEvent<T>>>,
    ) -> Self {
        self.on_change_details = on_change_details;
        self
    }

    pub fn with_component(mut self, component: &'static str) -> Self {
        self.component = component;
        self
    }

    pub fn with_prop(mut self, prop: &'static str) -> Self {
        self.prop = prop;
        self
    }

    pub fn with_sanitize<S2>(self, sanitize: S2) -> ControllableConfig<T, S2, E> {
        ControllableConfig {
            controlled: self.controlled,
            default_value: self.default_value,
            has_explicit_default: self.has_explicit_default,
            on_change: self.on_change,
            on_change_details: self.on_change_details,
            sanitize,
            eq: self.eq,
            component: self.component,
            prop: self.prop,
        }
    }

    pub fn with_eq<E2>(self, eq: E2) -> ControllableConfig<T, S, E2> {
        ControllableConfig {
            controlled: self.controlled,
            default_value: self.default_value,
            has_explicit_default: self.has_explicit_default,
            on_change: self.on_change,
            on_change_details: self.on_change_details,
            sanitize: self.sanitize,
            eq,
            component: self.component,
            prop: self.prop,
        }
    }
}

#[cfg(debug_assertions)]
#[derive(Clone, Debug)]
struct DiagMeta {
    instance_key: u64,
    instance_id: String,
    component: &'static str,
    prop: &'static str,
    initially_controlled: bool,
    mode_switch_warned: bool,
    refusal_warned: bool,
    alive: bool,
}

/// Zero-overhead `Copy` controllable state handle backed by a [`Signal`] for reactive value reads,
/// a [`TransitionGate`] for borrow-safe 3-phase `Propose -> Veto -> Commit` transitions, and
/// non-reactive [`CopyValue`] slots for callback updates.
pub struct ControllableSignal<T: 'static> {
    value: Signal<T>,
    gate: CopyValue<TransitionGate<T>>,
    on_change: CopyValue<Option<EventHandler<T>>>,
    on_change_details: CopyValue<Option<EventHandler<ChangeEvent<T>>>>,
    #[cfg(debug_assertions)]
    diag: CopyValue<DiagMeta>,
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
            && self.gate == other.gate
            && self.on_change == other.on_change
            && self.on_change_details == other.on_change_details
    }
}

impl<T: 'static> Eq for ControllableSignal<T> {}

impl<T: 'static> ControllableSignal<T> {
    /// Returns `true` when the value is externally controlled by props.
    pub fn is_controlled(&self) -> bool {
        self.gate
            .try_read()
            .map(|g| g.is_controlled())
            .unwrap_or(false)
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

    /// Updates the `on_change_details` callback slot without triggering reactivity.
    pub fn set_on_change_details(&self, handler: Option<EventHandler<ChangeEvent<T>>>) {
        let mut slot = self.on_change_details;
        if let Ok(mut guard) = slot.try_write() {
            *guard = handler;
        }
    }

    /// Proposes a new value `next` with programmatic context.
    ///
    /// Returns `true` when the transition was accepted (`Committed` or `ControlledPending`).
    pub fn set(&self, next: T) -> bool
    where
        T: Clone + PartialEq,
    {
        self.set_by(next, PartialEq::eq)
    }

    /// Proposes a new value `next` using a custom equality comparator `eq` (e.g. `f64::to_bits`).
    ///
    /// Returns `true` when the transition was accepted (`Committed` or `ControlledPending`).
    pub fn set_by(&self, next: T, eq: impl Fn(&T, &T) -> bool) -> bool
    where
        T: Clone,
    {
        self.set_by_with_context(
            next,
            ChangeContext::new(ChangeReason::Programmatic, ChangeSource::Programmatic),
            eq,
        )
        .is_accepted()
    }

    /// Proposes a new value `next` with explicit [`ChangeContext`] via the 3-phase
    /// `Propose -> Veto -> Commit` protocol.
    pub fn set_with_context(&self, next: T, ctx: ChangeContext) -> ChangeOutcome
    where
        T: Clone + PartialEq,
    {
        self.set_by_with_context(next, ctx, PartialEq::eq)
    }

    /// Proposes a new value `next` with explicit [`ChangeContext`] and custom equality `eq`.
    pub fn set_by_with_context(
        &self,
        next: T,
        ctx: ChangeContext,
        eq: impl Fn(&T, &T) -> bool,
    ) -> ChangeOutcome
    where
        T: Clone,
    {
        let event = ChangeEvent::new(next.clone(), ctx);
        self.set_by_with_event(next, &event, eq)
    }

    /// Proposes a new value `next` using a caller-supplied [`ChangeEvent<T>`] so shared
    /// `cancel()` and `allow_propagation()` flags remain observable by the caller.
    pub fn set_with_event(&self, next: T, event: &ChangeEvent<T>) -> ChangeOutcome
    where
        T: Clone + PartialEq,
    {
        self.set_by_with_event(next, event, PartialEq::eq)
    }

    /// Proposes a new value `next` using a caller-supplied [`ChangeEvent<T>`] and custom
    /// equality comparator `eq`.
    pub fn set_by_with_event(
        &self,
        next: T,
        event: &ChangeEvent<T>,
        eq: impl Fn(&T, &T) -> bool,
    ) -> ChangeOutcome
    where
        T: Clone,
    {
        self.propose_and_commit(next, event, eq, true)
    }

    /// Proposes a non-cancelable transition: `on_change_details` is still notified (and may
    /// set `allow_propagation()`), but `event.cancel()` is ignored during the Commit phase.
    pub fn set_non_cancelable(&self, next: T, ctx: ChangeContext) -> ChangeOutcome
    where
        T: Clone + PartialEq,
    {
        let event = ChangeEvent::new(next.clone(), ctx);
        self.propose_and_commit(next, &event, PartialEq::eq, false)
    }

    fn propose_and_commit(
        &self,
        next: T,
        event: &ChangeEvent<T>,
        eq: impl Fn(&T, &T) -> bool,
        cancelable: bool,
    ) -> ChangeOutcome
    where
        T: Clone,
    {
        // Phase 1: Propose (short write borrow on TransitionGate, dropped immediately).
        let initial_ctx = event.to_context();
        let token = {
            let mut gate_slot = self.gate;
            let Ok(mut gate) = gate_slot.try_write() else {
                return ChangeOutcome::Unchanged;
            };
            gate.begin_ask_by(next.clone(), &initial_ctx, eq)
        };
        let Some(token) = token else {
            return ChangeOutcome::Unchanged;
        };

        // Phase 2: Veto / Notify (zero CopyValue borrows held across user callback).
        let details_cb = self
            .on_change_details
            .try_read()
            .ok()
            .and_then(|slot| *slot);
        if let Some(handler) = details_cb {
            handler.call(event.clone());
        }

        let final_ctx = if cancelable {
            event.to_context()
        } else {
            let mut ctx = ChangeContext::new(event.reason(), event.source());
            if event.is_propagation_allowed() {
                ctx.allow_propagation();
            }
            ctx
        };

        // Phase 3: Commit (short write borrow on TransitionGate, dropped before side effects).
        let decision = {
            let mut gate_slot = self.gate;
            let Ok(mut gate) = gate_slot.try_write() else {
                return ChangeOutcome::Superseded(final_ctx);
            };
            gate.finish_ask(token, next, &final_ctx)
        };

        match decision {
            CommitDecision::Canceled => {
                #[cfg(debug_assertions)]
                self.cancel_pending_refusal();
                ChangeOutcome::Canceled(final_ctx)
            }
            CommitDecision::Superseded => ChangeOutcome::Superseded(final_ctx),
            CommitDecision::Commit { value, .. } => {
                let mut sig = self.value;
                if let Ok(mut guard) = sig.try_write() {
                    *guard = value.clone();
                }
                let cb = self.on_change.try_read().ok().and_then(|slot| *slot);
                if let Some(handler) = cb {
                    handler.call(value);
                }
                ChangeOutcome::Committed(final_ctx)
            }
            CommitDecision::ControlledPending { pending } => {
                let cb = self.on_change.try_read().ok().and_then(|slot| *slot);
                if let Some(handler) = cb {
                    handler.call(pending.value);
                }
                #[cfg(debug_assertions)]
                self.arm_pending_refusal(token);
                ChangeOutcome::ControlledPending(final_ctx)
            }
        }
    }

    #[cfg(debug_assertions)]
    fn cancel_pending_refusal(&self) {
        if let Ok(meta) = self.diag.try_read() {
            cancel_pending_controlled_check(meta.instance_key);
        }
    }

    #[cfg(debug_assertions)]
    fn arm_pending_refusal(&self, token: AskToken) {
        let still_pending = self
            .gate
            .try_read()
            .is_ok_and(|g| g.pending_controlled_for(token).is_some());
        if !still_pending {
            return;
        }
        let Ok(meta_read) = self.diag.try_read() else {
            return;
        };
        if !meta_read.alive || meta_read.refusal_warned {
            return;
        }
        let key = meta_read.instance_key;
        drop(meta_read);

        let gate = self.gate;
        let mut diag_slot = self.diag;
        register_pending_controlled_check(
            key,
            token.epoch(),
            Box::new(move || {
                let still_refused = gate
                    .try_read()
                    .is_ok_and(|g| g.pending_controlled_for(token).is_some());
                if !still_refused {
                    return;
                }
                let diag = diag_slot.try_write().ok().and_then(|mut meta| {
                    if !meta.alive || meta.refusal_warned {
                        return None;
                    }
                    meta.refusal_warned = true;
                    Some(DevDiagnostic {
                        kind: DiagnosticKind::ControlledRefusal,
                        component: meta.component,
                        instance_id: meta.instance_id.clone(),
                        prop: meta.prop,
                        message: format!(
                            "{} requested a controlled `{}` update that was not applied within the grace window. Update the controlled prop in `on_{}_change`, or call `event.cancel()` in `on_{}_change_details` if the refusal is intentional.",
                            meta.component, meta.prop, meta.prop, meta.prop
                        ),
                    })
                });
                if let Some(diag) = diag {
                    emit_diagnostic(diag);
                }
            }),
        );
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

/// Full-featured hook returning a `Copy` [`ControllableSignal<T>`] backed by
/// [`TransitionGate<T>`] with `on_change_details` veto support and dev-only
/// diagnostics.
pub fn use_controllable_signal_config<T, S, E>(
    config: ControllableConfig<T, S, E>,
) -> ControllableSignal<T>
where
    T: Clone + 'static,
    S: Fn(T) -> T,
    E: Fn(&T, &T) -> bool,
{
    let is_controlled_now = config.controlled.is_some();
    let sanitized_controlled = config.controlled.map(&config.sanitize);
    #[cfg(debug_assertions)]
    let (component, prop, has_explicit_default) = (
        config.component,
        config.prop,
        config.has_explicit_default,
    );

    let mut value = use_signal(|| {
        sanitized_controlled
            .clone()
            .unwrap_or_else(|| (config.sanitize)(config.default_value))
    });

    let mut gate = use_hook(|| {
        let initial = value.peek().clone();
        let initial_gate = if is_controlled_now {
            TransitionGate::new_controlled(initial)
        } else {
            TransitionGate::new_uncontrolled(initial)
        };
        CopyValue::new(initial_gate)
    });

    let mut on_change_slot = use_hook(|| CopyValue::new(config.on_change));
    on_change_slot.set(config.on_change);

    let mut on_change_details_slot = use_hook(|| CopyValue::new(config.on_change_details));
    on_change_details_slot.set(config.on_change_details);

    #[cfg(debug_assertions)]
    let diag = {
        let mut diag = use_hook(|| {
            let key = next_diagnostic_instance_id();
            let instance_id = format!("{component}-{key}");
            if is_controlled_now && has_explicit_default {
                emit_diagnostic(DevDiagnostic {
                    kind: DiagnosticKind::ControlledAndDefaultProvided,
                    component,
                    instance_id: instance_id.clone(),
                    prop,
                    message: format!(
                        "{component} received both a controlled `{prop}` prop and an explicit default value. Choose either controlled or uncontrolled mode.",
                    ),
                });
            }
            CopyValue::new(DiagMeta {
                instance_key: key,
                instance_id,
                component,
                prop,
                initially_controlled: is_controlled_now,
                mode_switch_warned: false,
                refusal_warned: false,
                alive: true,
            })
        });

        let mode_switch_diag = diag.try_write().ok().and_then(|mut meta| {
            if meta.initially_controlled != is_controlled_now && !meta.mode_switch_warned {
                meta.mode_switch_warned = true;
                let from = if meta.initially_controlled {
                    "controlled"
                } else {
                    "uncontrolled"
                };
                let to = if is_controlled_now {
                    "controlled"
                } else {
                    "uncontrolled"
                };
                Some(DevDiagnostic {
                    kind: DiagnosticKind::ControlledModeSwitch,
                    component: meta.component,
                    instance_id: meta.instance_id.clone(),
                    prop: meta.prop,
                    message: format!(
                        "{} switched `{}` from {} to {} mode within a single component lifetime.",
                        meta.component, meta.prop, from, to
                    ),
                })
            } else {
                None
            }
        });
        if let Some(d) = mode_switch_diag {
            emit_diagnostic(d);
        }

        use_drop(move || {
            let key = diag.try_write().ok().map(|mut meta| {
                meta.alive = false;
                meta.instance_key
            });
            if let Some(key) = key {
                cancel_pending_controlled_check(key);
            }
        });

        diag
    };

    if let Some(next) = sanitized_controlled {
        let sync_changed = gate
            .try_write()
            .ok()
            .map(|mut g| {
                if !g.is_controlled() {
                    g.set_controlled(true);
                }
                g.sync_controlled_outcome_by(next.clone(), &config.eq)
                    .changed()
            })
            .unwrap_or(false);
        if sync_changed {
            #[cfg(debug_assertions)]
            if let Ok(meta) = diag.try_read() {
                cancel_pending_controlled_check(meta.instance_key);
            }
        }
        sync_signal_by(&mut value, next, &config.eq);
    } else if let Ok(mut g) = gate.try_write()
        && g.is_controlled()
    {
        g.set_controlled(false);
        #[cfg(debug_assertions)]
        if let Ok(meta) = diag.try_read() {
            cancel_pending_controlled_check(meta.instance_key);
        }
    }

    ControllableSignal {
        value,
        gate,
        on_change: on_change_slot,
        on_change_details: on_change_details_slot,
        #[cfg(debug_assertions)]
        diag,
    }
}

/// Zero-overhead hook returning a `Copy` [`ControllableSignal<T>`] without `Rc<dyn Fn>` boxing.
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
    use_controllable_signal_config(ControllableConfig {
        controlled,
        default_value,
        has_explicit_default: false,
        on_change,
        on_change_details: None,
        sanitize,
        eq,
        component: "ControllableSignal",
        prop: "value",
    })
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
    use ui_sdk::kernel::{ChangeContext, ChangeEvent, ChangeReason, ChangeSource};

    use super::{
        ChangeOutcome, ControllableConfig, ControllableSignal, sync_f64_signal, sync_signal,
        use_controllable_signal, use_controllable_signal_config, use_controllable_signal_with,
        use_effect_with_cleanup,
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

    #[test]
    fn transition_gate_cancel_reentrant_clamp_back_and_shared_event() {
        let handle_slot: Rc<RefCell<Option<ControllableSignal<i32>>>> =
            Rc::new(RefCell::new(None));
        let legacy_calls: Rc<RefCell<Vec<i32>>> = Rc::new(RefCell::new(Vec::new()));
        let cancel_next = Rc::new(Cell::new(false));
        let clamp_back = Rc::new(Cell::new(false));
        let allow_prop = Rc::new(Cell::new(false));

        #[component]
        fn GateHarness(
            handle_slot: Rc<RefCell<Option<ControllableSignal<i32>>>>,
            legacy_calls: Rc<RefCell<Vec<i32>>>,
            cancel_next: Rc<Cell<bool>>,
            clamp_back: Rc<Cell<bool>>,
            allow_prop: Rc<Cell<bool>>,
        ) -> Element {
            let legacy = Rc::clone(&legacy_calls);
            let handle_for_cb = Rc::clone(&handle_slot);
            let cancel_flag = Rc::clone(&cancel_next);
            let clamp_flag = Rc::clone(&clamp_back);
            let prop_flag = Rc::clone(&allow_prop);

            let sig = use_controllable_signal_config(
                ControllableConfig::new(None, 10_i32)
                    .with_component("Slider")
                    .with_prop("value")
                    .with_on_change(Some(EventHandler::new(move |v| {
                        legacy.borrow_mut().push(v);
                    })))
                    .with_on_change_details(Some(EventHandler::new(move |ev: ChangeEvent<i32>| {
                        if prop_flag.get() {
                            ev.allow_propagation();
                        }
                        if cancel_flag.get() {
                            ev.cancel();
                        } else if clamp_flag.get() && *ev.value() == 50 {
                            if let Some(h) = *handle_for_cb.borrow() {
                                let _ = h.set_with_context(
                                    10,
                                    ChangeContext::new(
                                        ChangeReason::Programmatic,
                                        ChangeSource::Programmatic,
                                    ),
                                );
                            }
                        }
                    }))),
            );
            *handle_slot.borrow_mut() = Some(sig);
            rsx! { div {} }
        }

        let mut dom = VirtualDom::new_with_props(
            GateHarness,
            GateHarnessProps {
                handle_slot: Rc::clone(&handle_slot),
                legacy_calls: Rc::clone(&legacy_calls),
                cancel_next: Rc::clone(&cancel_next),
                clamp_back: Rc::clone(&clamp_back),
                allow_prop: Rc::clone(&allow_prop),
            },
        );
        dom.rebuild_in_place();

        let h = handle_slot.borrow().expect("handle");

        // 1. Cancel prevents mutation and skips legacy on_change
        cancel_next.set(true);
        let outcome = h.set_with_context(
            20,
            ChangeContext::new(ChangeReason::ValueChange, ChangeSource::Keyboard),
        );
        assert!(outcome.is_canceled());
        assert!(!outcome.is_committed());
        assert_eq!(h.peek(), 10);
        assert!(legacy_calls.borrow().is_empty());

        // 2. set_non_cancelable ignores ev.cancel() and still commits
        let non_cancel = h.set_non_cancelable(
            15,
            ChangeContext::new(ChangeReason::Reset, ChangeSource::Programmatic),
        );
        assert!(non_cancel.is_committed());
        assert_eq!(h.peek(), 15);
        assert_eq!(*legacy_calls.borrow(), vec![15]);

        // Reset to 10
        cancel_next.set(false);
        assert!(h.set(10));
        legacy_calls.borrow_mut().clear();

        // 3. Re-entrant clamp-back (10 -> 50 -> 10) keeps value at 10 (outer Superseded)
        clamp_back.set(true);
        let clamped = h.set_with_context(
            50,
            ChangeContext::new(ChangeReason::Drag, ChangeSource::Mouse),
        );
        assert!(matches!(clamped, ChangeOutcome::Superseded(_)));
        assert_eq!(h.peek(), 10);
        assert!(legacy_calls.borrow().is_empty());
        clamp_back.set(false);

        // 4. set_with_event shares allow_propagation() with caller's ChangeEvent
        allow_prop.set(true);
        let shared_ev = ChangeEvent::new(
            25,
            ChangeContext::new(ChangeReason::EscapeKey, ChangeSource::Keyboard),
        );
        let res = h.set_with_event(25, &shared_ev);
        assert!(res.is_committed());
        assert!(shared_ev.is_propagation_allowed());
        assert!(res.context().is_some_and(|c| c.is_propagation_allowed()));
    }

    #[cfg(debug_assertions)]
    #[test]
    fn diagnostics_controlled_refusal_mode_switch_and_explicit_default() {
        use crate::foundation::diagnostics::{
            DiagnosticKind, capture_diagnostics, flush_pending_controlled_checks,
        };

        let guard = capture_diagnostics();

        let ctl_prop_holder: Rc<RefCell<Option<Signal<Option<bool>>>>> =
            Rc::new(RefCell::new(None));
        let mount_holder: Rc<RefCell<Option<Signal<bool>>>> = Rc::new(RefCell::new(None));
        let sig_holder: Rc<RefCell<Option<ControllableSignal<bool>>>> =
            Rc::new(RefCell::new(None));
        let cancel_flag = Rc::new(Cell::new(false));

        #[component]
        fn Child(
            open: Option<bool>,
            has_explicit_default: bool,
            sig_holder: Rc<RefCell<Option<ControllableSignal<bool>>>>,
            cancel_flag: Rc<Cell<bool>>,
        ) -> Element {
            let cancel = Rc::clone(&cancel_flag);
            let s = use_controllable_signal_config(
                ControllableConfig::new(open, false)
                    .with_explicit_default(has_explicit_default)
                    .with_component("Dialog")
                    .with_prop("open")
                    .with_on_change_details(Some(EventHandler::new(move |ev: ChangeEvent<bool>| {
                        if cancel.get() {
                            ev.cancel();
                        }
                    }))),
            );
            *sig_holder.borrow_mut() = Some(s);
            rsx! { div {} }
        }

        #[component]
        fn Root(
            ctl_prop_holder: Rc<RefCell<Option<Signal<Option<bool>>>>>,
            mount_holder: Rc<RefCell<Option<Signal<bool>>>>,
            sig_holder: Rc<RefCell<Option<ControllableSignal<bool>>>>,
            cancel_flag: Rc<Cell<bool>>,
        ) -> Element {
            let open = use_signal(|| Some(false));
            let mounted = use_signal(|| true);
            *ctl_prop_holder.borrow_mut() = Some(open);
            *mount_holder.borrow_mut() = Some(mounted);
            rsx! {
                if mounted() {
                    Child {
                        open: open(),
                        has_explicit_default: true,
                        sig_holder: Rc::clone(&sig_holder),
                        cancel_flag: Rc::clone(&cancel_flag),
                    }
                }
            }
        }

        let mut dom = VirtualDom::new_with_props(
            Root,
            RootProps {
                ctl_prop_holder: Rc::clone(&ctl_prop_holder),
                mount_holder: Rc::clone(&mount_holder),
                sig_holder: Rc::clone(&sig_holder),
                cancel_flag: Rc::clone(&cancel_flag),
            },
        );
        dom.rebuild_in_place();

        // Explicit default + controlled triggers ControlledAndDefaultProvided on mount
        let initial_diags = guard.take();
        assert_eq!(initial_diags.len(), 1);
        assert_eq!(
            initial_diags[0].kind,
            DiagnosticKind::ControlledAndDefaultProvided
        );

        let h = sig_holder.borrow().unwrap();

        // Guard 1: Intentional cancel -> zero refusal diagnostic
        cancel_flag.set(true);
        assert!(
            h.set_with_context(
                true,
                ChangeContext::new(ChangeReason::Open, ChangeSource::Mouse)
            )
            .is_canceled()
        );
        flush_pending_controlled_checks();
        assert!(guard.take().is_empty());
        cancel_flag.set(false);

        // Guard 2: Parent applies controlled update before flush -> zero refusal diagnostic
        assert!(
            h.set_with_context(
                true,
                ChangeContext::new(ChangeReason::Open, ChangeSource::Mouse)
            )
            .is_accepted()
        );
        ctl_prop_holder.borrow().unwrap().set(Some(true));
        dom.render_immediate(&mut dioxus::core::NoOpMutations);
        flush_pending_controlled_checks();
        assert!(guard.take().is_empty());

        // Guard 3: Parent ignores controlled update -> ControlledRefusal emitted once on flush
        assert!(
            h.set_with_context(
                false,
                ChangeContext::new(ChangeReason::EscapeKey, ChangeSource::Keyboard)
            )
            .is_accepted()
        );
        flush_pending_controlled_checks();
        let refusal_diags = guard.take();
        assert_eq!(refusal_diags.len(), 1);
        assert_eq!(refusal_diags[0].kind, DiagnosticKind::ControlledRefusal);

        // Mode switch emits ControlledModeSwitch once (without panicking when strict mode is off)
        ctl_prop_holder.borrow().unwrap().set(None);
        dom.render_immediate(&mut dioxus::core::NoOpMutations);
        let switch_diags = guard.take();
        assert_eq!(switch_diags.len(), 1);
        assert_eq!(switch_diags[0].kind, DiagnosticKind::ControlledModeSwitch);

        // Unmount cancels any pending checks cleanly
        mount_holder.borrow().unwrap().set(false);
        dom.render_immediate(&mut dioxus::core::NoOpMutations);
        flush_pending_controlled_checks();
        assert!(guard.take().is_empty());
    }
}

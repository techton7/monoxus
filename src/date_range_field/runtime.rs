use std::{cell::RefCell, rc::Rc};

use dioxus::prelude::*;
use ui_sdk::foundation::date_time::field_policy::{
    DateFieldPolicy, FieldRangeEndpoint, RangeFieldPolicy, RangeFieldRelationships,
    RangeFieldSnapshot, RangeFieldState,
};
use ui_sdk::kernel::{ChangeContext, ChangeEvent, ChangeReason, ChangeSource};

use crate::{
    agent::{cancel_current_agent_action, current_change_context},
    date_field::runtime::{SegmentFieldRuntime, SegmentFieldUiPolicy},
    foundation::{
        browser::focus_element_by_id,
        date_time::{CalendarDate, ChronoRange, FieldInvalidEvent, FieldInvalidReason},
        state::{ControllableSignal, sync_signal},
    },
};

use super::attrs::{RangeFieldLabelAttrs, RangeFieldRootAttrs};

/// Zero-overhead policy-parameterized reactive runtime shared by `DateRangeField` and `TimeRangeField`.
pub struct RangeFieldRuntime<P: RangeFieldPolicy + SegmentFieldUiPolicy> {
    pub state: Signal<RangeFieldState<P>>,
    pub rels: RangeFieldRelationships,
    pub root_dom_id: Signal<String>,
    pub start_value: Signal<Option<P::Value>>,
    pub end_value: Signal<Option<P::Value>>,
    pub placeholder: Signal<P::Value>,
    pub custom_label_id: Signal<Option<String>>,
    pub last_invalid_reason: Signal<Option<FieldInvalidReason>>,
    pub trailing_trigger_id: Signal<Option<String>>,
    pub start_runtime: Rc<RefCell<Option<SegmentFieldRuntime<P>>>>,
    pub end_runtime: Rc<RefCell<Option<SegmentFieldRuntime<P>>>>,
    pub default_value: CopyValue<ChronoRange<P::Value>>,
    pub(crate) value_ctl: ControllableSignal<ChronoRange<P::Value>>,
    pub(crate) start_ctl: ControllableSignal<Option<P::Value>>,
    pub(crate) end_ctl: ControllableSignal<Option<P::Value>>,
    pub(crate) placeholder_ctl: ControllableSignal<P::Value>,
    pub on_value_change: CopyValue<Option<EventHandler<ChronoRange<P::Value>>>>,
    pub on_value_change_details: CopyValue<Option<EventHandler<ChangeEvent<ChronoRange<P::Value>>>>>,
    pub on_start_value_change: CopyValue<Option<EventHandler<Option<P::Value>>>>,
    pub on_start_value_change_details:
        CopyValue<Option<EventHandler<ChangeEvent<Option<P::Value>>>>>,
    pub on_end_value_change: CopyValue<Option<EventHandler<Option<P::Value>>>>,
    pub on_end_value_change_details:
        CopyValue<Option<EventHandler<ChangeEvent<Option<P::Value>>>>>,
    pub is_date_unavailable: CopyValue<Option<Callback<CalendarDate, bool>>>,
    pub validate: CopyValue<Option<Callback<ChronoRange<P::Value>, Option<Vec<String>>>>>,
    pub on_invalid: CopyValue<Option<EventHandler<FieldInvalidEvent>>>,
    pub parent_focus_in: CopyValue<Option<EventHandler<FocusEvent>>>,
    pub parent_focus_out: CopyValue<Option<EventHandler<FocusEvent>>>,
}

impl<P: RangeFieldPolicy + SegmentFieldUiPolicy> Clone for RangeFieldRuntime<P> {
    fn clone(&self) -> Self {
        Self {
            state: self.state,
            rels: self.rels.clone(),
            root_dom_id: self.root_dom_id,
            start_value: self.start_value,
            end_value: self.end_value,
            placeholder: self.placeholder,
            custom_label_id: self.custom_label_id,
            last_invalid_reason: self.last_invalid_reason,
            trailing_trigger_id: self.trailing_trigger_id,
            start_runtime: Rc::clone(&self.start_runtime),
            end_runtime: Rc::clone(&self.end_runtime),
            default_value: self.default_value,
            value_ctl: self.value_ctl,
            start_ctl: self.start_ctl,
            end_ctl: self.end_ctl,
            placeholder_ctl: self.placeholder_ctl,
            on_value_change: self.on_value_change,
            on_value_change_details: self.on_value_change_details,
            on_start_value_change: self.on_start_value_change,
            on_start_value_change_details: self.on_start_value_change_details,
            on_end_value_change: self.on_end_value_change,
            on_end_value_change_details: self.on_end_value_change_details,
            is_date_unavailable: self.is_date_unavailable,
            validate: self.validate,
            on_invalid: self.on_invalid,
            parent_focus_in: self.parent_focus_in,
            parent_focus_out: self.parent_focus_out,
        }
    }
}

impl<P: RangeFieldPolicy + SegmentFieldUiPolicy> PartialEq for RangeFieldRuntime<P> {
    fn eq(&self, other: &Self) -> bool {
        self.rels == other.rels && self.state == other.state
    }
}

impl<P: RangeFieldPolicy + SegmentFieldUiPolicy> RangeFieldRuntime<P> {
    pub fn snapshot(&self) -> RangeFieldSnapshot<P> {
        self.state.read().snapshot()
    }

    pub fn current_root_dom_id(&self) -> String {
        self.root_dom_id
            .try_read()
            .map(|g| g.clone())
            .unwrap_or_else(|_| self.rels.root_id().to_string())
    }

    pub fn set_root_dom_id(&self, id: String) {
        let mut sig = self.root_dom_id;
        sync_signal(&mut sig, id);
    }

    pub(crate) fn set_parent_focus_handlers(
        &self,
        focus_in: Option<EventHandler<FocusEvent>>,
        focus_out: Option<EventHandler<FocusEvent>>,
    ) {
        let mut fi = self.parent_focus_in;
        fi.set(focus_in);
        let mut fo = self.parent_focus_out;
        fo.set(focus_out);
    }

    pub(crate) fn notify_parent_focus_in(&self, evt: FocusEvent) {
        if let Some(cb) = self.parent_focus_in.try_read().ok().and_then(|s| *s) {
            cb.call(evt);
        }
    }

    pub(crate) fn notify_parent_focus_out(&self, evt: FocusEvent) {
        if let Some(cb) = self.parent_focus_out.try_read().ok().and_then(|s| *s) {
            cb.call(evt);
        }
    }

    pub fn effective_label_id(&self) -> String {
        self.custom_label_id
            .read()
            .clone()
            .filter(|s| !s.is_empty())
            .unwrap_or_else(|| self.rels.label_id().to_string())
    }

    /// Registers a custom `id` from `*RangeFieldLabel` so `aria-labelledby` stays synced (`DEF-5C-07`).
    pub fn register_custom_label_id(&mut self, id: Option<String>) {
        let normalized = id.filter(|s| !s.is_empty());
        if self.custom_label_id.try_peek().is_ok_and(|cur| *cur != normalized)
            && let Ok(mut guard) = self.custom_label_id.try_write()
        {
            *guard = normalized;
        }
    }

    /// Clears `custom_label_id` only if it still matches `expected`, preventing keyed label swaps
    /// from clobbering the replacement label's ID.
    pub fn unregister_custom_label_id(&mut self, expected: &str) {
        if self
            .custom_label_id
            .try_peek()
            .ok()
            .and_then(|cur| cur.as_deref().map(|s| s == expected))
            .unwrap_or(false)
            && let Ok(mut guard) = self.custom_label_id.try_write()
        {
            *guard = None;
        }
    }

    pub fn default_part_value(&self, kind: FieldRangeEndpoint) -> Option<P::Value> {
        let dv = self.default_value.try_read().map(|v| *v).unwrap_or_default();
        match kind {
            FieldRangeEndpoint::Start => dv.start,
            FieldRangeEndpoint::End => dv.end,
        }
    }

    pub fn is_part_invalid(&self, kind: FieldRangeEndpoint) -> bool {
        self.state.read().is_part_invalid(kind)
    }

    pub fn root_attrs(&self, id_override: Option<String>) -> RangeFieldRootAttrs {
        let st = self.state.read();
        let id = id_override
            .filter(|s| !s.is_empty())
            .unwrap_or_else(|| self.rels.root_id().to_string());
        RangeFieldRootAttrs::new(
            id,
            self.effective_label_id(),
            st.def().dir,
            st.is_invalid(),
            st.def().disabled,
        )
    }

    pub fn label_attrs(&self, id_override: Option<String>) -> RangeFieldLabelAttrs {
        let st = self.state.read();
        let id = id_override
            .filter(|s| !s.is_empty())
            .unwrap_or_else(|| self.effective_label_id());
        RangeFieldLabelAttrs::new(id, st.is_invalid(), st.def().disabled)
    }

    pub fn register_part_runtime(&self, kind: FieldRangeEndpoint, rt: SegmentFieldRuntime<P>) {
        match kind {
            FieldRangeEndpoint::Start => *self.start_runtime.borrow_mut() = Some(rt),
            FieldRangeEndpoint::End => *self.end_runtime.borrow_mut() = Some(rt),
        }
    }

    /// Clears `start_runtime` or `end_runtime` only when the registered runtime still matches
    /// `expected`, preventing keyed endpoint replacements from being clobbered on drop.
    pub fn unregister_part_runtime(
        &self,
        kind: FieldRangeEndpoint,
        expected: &SegmentFieldRuntime<P>,
    ) {
        let slot = match kind {
            FieldRangeEndpoint::Start => &self.start_runtime,
            FieldRangeEndpoint::End => &self.end_runtime,
        };
        let mut guard = slot.borrow_mut();
        if guard.as_ref().is_some_and(|cur| cur == expected) {
            *guard = None;
        }
    }

    /// Registers an optional trailing picker trigger ID (`DateRangePickerTrigger`)
    /// so navigating past the last `End` segment focuses the trigger.
    pub fn register_trigger_id(&mut self, id: Option<String>) {
        if self
            .trailing_trigger_id
            .try_peek()
            .is_ok_and(|cur| *cur != id)
            && let Ok(mut guard) = self.trailing_trigger_id.try_write()
        {
            *guard = id;
        }
    }

    /// Clears `trailing_trigger_id` only if it still matches `expected`, preventing keyed
    /// trigger swaps from clobbering the replacement trigger's ID.
    pub fn unregister_trigger_id(&mut self, expected: &str) {
        if self
            .trailing_trigger_id
            .try_peek()
            .ok()
            .and_then(|cur| cur.as_deref().map(|s| s == expected))
            .unwrap_or(false)
            && let Ok(mut guard) = self.trailing_trigger_id.try_write()
        {
            *guard = None;
        }
    }

    pub fn focus_first_segment(&self) {
        let target = self
            .start_runtime
            .borrow()
            .as_ref()
            .and_then(|rt| rt.first_interactive_segment_id())
            .or_else(|| {
                self.end_runtime
                    .borrow()
                    .as_ref()
                    .and_then(|rt| rt.first_interactive_segment_id())
            });
        if let Some(id) = target {
            focus_element_by_id(&id);
        }
    }

    pub fn focus_last_segment(&self) {
        let target = self
            .end_runtime
            .borrow()
            .as_ref()
            .and_then(|rt| rt.last_interactive_segment_id())
            .or_else(|| {
                self.start_runtime
                    .borrow()
                    .as_ref()
                    .and_then(|rt| rt.last_interactive_segment_id())
            });
        if let Some(id) = target {
            focus_element_by_id(&id);
        }
    }

    pub fn handle_part_next_boundary(&self, kind: FieldRangeEndpoint) {
        match kind {
            FieldRangeEndpoint::Start => {
                let target = self
                    .end_runtime
                    .borrow()
                    .as_ref()
                    .and_then(|rt| rt.first_interactive_segment_id());
                if let Some(id) = target {
                    focus_element_by_id(&id);
                } else if let Ok(guard) = self.trailing_trigger_id.try_peek()
                    && let Some(trigger_id) = guard.as_deref()
                {
                    focus_element_by_id(trigger_id);
                }
            }
            FieldRangeEndpoint::End => {
                if let Ok(guard) = self.trailing_trigger_id.try_peek()
                    && let Some(trigger_id) = guard.as_deref()
                {
                    focus_element_by_id(trigger_id);
                }
            }
        }
    }

    pub fn handle_part_prev_boundary(&self, kind: FieldRangeEndpoint) {
        if kind == FieldRangeEndpoint::End {
            let target = self
                .start_runtime
                .borrow()
                .as_ref()
                .and_then(|rt| rt.last_interactive_segment_id());
            if let Some(id) = target {
                focus_element_by_id(&id);
            }
        }
    }

    pub fn sync_validation_and_notify(&mut self) {
        let (range, min_val, max_val) = {
            let st = self.state.peek();
            (st.value(), st.def().min_value, st.def().max_value)
        };
        let (start_unavailable, end_unavailable) = if P::SUPPORTS_UNAVAILABLE {
            if let Some(cb) = self
                .is_date_unavailable
                .try_read()
                .ok()
                .and_then(|slot| *slot)
            {
                (
                    range
                        .start
                        .and_then(P::extract_calendar_date)
                        .is_some_and(|d| cb.call(d)),
                    range
                        .end
                        .and_then(P::extract_calendar_date)
                        .is_some_and(|d| cb.call(d)),
                )
            } else {
                (false, false)
            }
        } else {
            (false, false)
        };
        let is_unavailable = start_unavailable || end_unavailable;
        let custom_errors = if !is_unavailable && range.is_complete() {
            self.validate
                .try_read()
                .ok()
                .and_then(|slot| *slot)
                .and_then(|cb| cb.call(range))
        } else {
            None
        };

        let next_reason =
            P::validate_range(range, min_val, max_val, is_unavailable, custom_errors).err();
        let needs_state_update = {
            let st = self.state.peek();
            st.invalid_reason() != next_reason.as_ref()
                || st.unavailable_endpoints() != (start_unavailable, end_unavailable)
        };
        if needs_state_update {
            let mut st = self.state.write();
            st.set_unavailable_endpoints(start_unavailable, end_unavailable);
            st.set_local_invalid_reason(next_reason.clone());
        }
        let prev_reason = self.last_invalid_reason.peek().clone();
        if next_reason != prev_reason {
            self.last_invalid_reason.set(next_reason.clone());
            if let Some(reason) = next_reason {
                let messages = match &reason {
                    FieldInvalidReason::Custom(msgs) => msgs.clone(),
                    _ => Vec::new(),
                };
                if let Some(handler) = self.on_invalid.try_read().ok().and_then(|slot| *slot) {
                    handler.call(FieldInvalidEvent { reason, messages });
                }
            }
        }
    }

    fn notify_controlled_fallback<T: Clone + 'static>(
        next: T,
        ctx: ChangeContext,
        on_details: CopyValue<Option<EventHandler<ChangeEvent<T>>>>,
        on_legacy: CopyValue<Option<EventHandler<T>>>,
    ) -> bool {
        let event = ChangeEvent::new(next.clone(), ctx);
        if let Some(handler) = on_details.try_read().ok().and_then(|slot| *slot) {
            handler.call(event.clone());
        }
        if event.is_canceled() {
            return true;
        }
        if let Some(handler) = on_legacy.try_read().ok().and_then(|slot| *slot) {
            handler.call(next);
        }
        false
    }

    pub(crate) fn set_range_from_agent(
        &mut self,
        next: ChronoRange<P::Value>,
        ctx: ChangeContext,
    ) -> bool {
        let prev = self.state.peek().value();
        if prev == next {
            return false;
        }

        let start_controlled = self.value_ctl.is_controlled() || self.start_ctl.is_controlled();
        let end_controlled = self.value_ctl.is_controlled() || self.end_ctl.is_controlled();
        let any_controlled = start_controlled || end_controlled;

        if !any_controlled || self.value_ctl.is_controlled() {
            let outcome = self.value_ctl.set_with_context(next, ctx);
            if outcome.is_canceled() {
                cancel_current_agent_action();
                return true;
            }
        } else if Self::notify_controlled_fallback(
            next,
            ctx,
            self.on_value_change_details,
            self.on_value_change,
        ) {
            cancel_current_agent_action();
            return true;
        }

        if prev.start != next.start {
            if !start_controlled || self.start_ctl.is_controlled() {
                let _ = self.start_ctl.set_with_context(next.start, ctx);
            } else {
                let _ = Self::notify_controlled_fallback(
                    next.start,
                    ctx,
                    self.on_start_value_change_details,
                    self.on_start_value_change,
                );
            }
        }
        if prev.end != next.end {
            if !end_controlled || self.end_ctl.is_controlled() {
                let _ = self.end_ctl.set_with_context(next.end, ctx);
            } else {
                let _ = Self::notify_controlled_fallback(
                    next.end,
                    ctx,
                    self.on_end_value_change_details,
                    self.on_end_value_change,
                );
            }
        }

        if !any_controlled {
            self.state.write().set_value(next);
        }
        self.sync_validation_and_notify();
        false
    }

    pub fn update_part_value(&mut self, kind: FieldRangeEndpoint, next: Option<P::Value>) {
        let prev_range = self.state.peek().value();
        let prev_part = match kind {
            FieldRangeEndpoint::Start => prev_range.start,
            FieldRangeEndpoint::End => prev_range.end,
        };
        if prev_part == next {
            return;
        }

        let mut next_range = prev_range;
        match kind {
            FieldRangeEndpoint::Start => next_range.start = next,
            FieldRangeEndpoint::End => next_range.end = next,
        }

        let start_controlled = self.value_ctl.is_controlled() || self.start_ctl.is_controlled();
        let end_controlled = self.value_ctl.is_controlled() || self.end_ctl.is_controlled();
        let part_controlled = match kind {
            FieldRangeEndpoint::Start => start_controlled,
            FieldRangeEndpoint::End => end_controlled,
        };
        let ctx = current_change_context()
            .unwrap_or_else(|| ChangeContext::new(ChangeReason::Input, ChangeSource::Keyboard));

        if !part_controlled {
            let value_canceled = if !self.value_ctl.is_controlled() {
                self.value_ctl.set_with_context(next_range, ctx).is_canceled()
            } else {
                Self::notify_controlled_fallback(
                    next_range,
                    ctx,
                    self.on_value_change_details,
                    self.on_value_change,
                )
            };
            if value_canceled {
                cancel_current_agent_action();
                return;
            }
            let part_outcome = match kind {
                FieldRangeEndpoint::Start => self.start_ctl.set_with_context(next, ctx),
                FieldRangeEndpoint::End => self.end_ctl.set_with_context(next, ctx),
            };
            if part_outcome.is_canceled() {
                cancel_current_agent_action();
                return;
            }
            self.state.write().set_part_value(kind, next);
        } else {
            let value_canceled = if self.value_ctl.is_controlled() {
                self.value_ctl.set_with_context(next_range, ctx).is_canceled()
            } else {
                Self::notify_controlled_fallback(
                    next_range,
                    ctx,
                    self.on_value_change_details,
                    self.on_value_change,
                )
            };
            if value_canceled {
                cancel_current_agent_action();
                return;
            }
            match kind {
                FieldRangeEndpoint::Start => {
                    if self.start_ctl.is_controlled() {
                        let _ = self.start_ctl.set_with_context(next, ctx);
                    } else {
                        let _ = Self::notify_controlled_fallback(
                            next,
                            ctx,
                            self.on_start_value_change_details,
                            self.on_start_value_change,
                        );
                    }
                }
                FieldRangeEndpoint::End => {
                    if self.end_ctl.is_controlled() {
                        let _ = self.end_ctl.set_with_context(next, ctx);
                    } else {
                        let _ = Self::notify_controlled_fallback(
                            next,
                            ctx,
                            self.on_end_value_change_details,
                            self.on_end_value_change,
                        );
                    }
                }
            }
        }

        self.sync_validation_and_notify();
    }

    pub fn update_placeholder(&mut self, next: P::Value) {
        if *self.placeholder.peek() == next {
            return;
        }
        let ctx = current_change_context()
            .unwrap_or_else(|| ChangeContext::new(ChangeReason::Input, ChangeSource::Keyboard));
        let outcome = self.placeholder_ctl.set_with_context(next, ctx);
        if outcome.is_canceled() {
            cancel_current_agent_action();
            return;
        }
        if outcome.is_committed() {
            self.state.write().set_placeholder(next);
        }
    }

    pub fn reconcile_external_state(
        &mut self,
        external_value: Option<ChronoRange<P::Value>>,
        external_start: Option<Option<P::Value>>,
        external_end: Option<Option<P::Value>>,
        external_placeholder: Option<P::Value>,
        is_invalid_override: Option<bool>,
    ) {
        let mut target_val = external_value.unwrap_or_else(|| self.state.peek().value());
        if let Some(s) = external_start {
            target_val.start = s;
        }
        if let Some(e) = external_end {
            target_val.end = e;
        }
        if self.state.peek().value() != target_val {
            self.state.write().set_value(target_val);
            if *self.start_value.peek() != target_val.start {
                self.start_value.set(target_val.start);
            }
            if *self.end_value.peek() != target_val.end {
                self.end_value.set(target_val.end);
            }
        }
        if let Some(ph) = external_placeholder
            && self.state.peek().placeholder() != ph
        {
            self.state.write().set_placeholder(ph);
        }
        if let Some(inv) = is_invalid_override
            && self.state.peek().external_invalid() != inv
        {
            self.state.write().set_external_invalid(inv);
        }
        self.sync_validation_and_notify();
    }
}

/// Dioxus context wrapping [`RangeFieldRuntime`].
pub struct RangeFieldContext<P: RangeFieldPolicy + SegmentFieldUiPolicy>(
    pub RangeFieldRuntime<P>,
);

impl<P: RangeFieldPolicy + SegmentFieldUiPolicy> Clone for RangeFieldContext<P> {
    fn clone(&self) -> Self {
        Self(self.0.clone())
    }
}

impl<P: RangeFieldPolicy + SegmentFieldUiPolicy> PartialEq for RangeFieldContext<P> {
    fn eq(&self, other: &Self) -> bool {
        self.0 == other.0
    }
}

/// Consumes the nearest ancestor [`RangeFieldRuntime<P>`] context.
pub fn use_range_field_context<P: RangeFieldPolicy + SegmentFieldUiPolicy>()
-> RangeFieldRuntime<P> {
    use_context::<RangeFieldContext<P>>().0
}

/// Attempts to consume the nearest ancestor [`RangeFieldRuntime<P>`] context.
pub fn try_use_range_field_context<P: RangeFieldPolicy + SegmentFieldUiPolicy>()
-> Option<RangeFieldRuntime<P>> {
    try_consume_context::<RangeFieldContext<P>>().map(|ctx| ctx.0)
}

/// Reactive runtime for `DateRangeField`.
pub type DateRangeFieldRuntime = RangeFieldRuntime<DateFieldPolicy>;

/// Dioxus context wrapping [`DateRangeFieldRuntime`].
pub type DateRangeFieldContext = RangeFieldContext<DateFieldPolicy>;

/// Consumes the nearest ancestor [`DateRangeFieldRuntime`] context (panics if outside `DateRangeFieldRoot`).
pub fn use_date_range_field_context() -> DateRangeFieldRuntime {
    use_range_field_context::<DateFieldPolicy>()
}

/// Attempts to consume the nearest ancestor [`DateRangeFieldRuntime`] context.
pub fn try_use_date_range_field_context() -> Option<DateRangeFieldRuntime> {
    try_use_range_field_context::<DateFieldPolicy>()
}

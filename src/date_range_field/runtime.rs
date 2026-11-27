use std::{cell::RefCell, rc::Rc};

use dioxus::prelude::*;
use monoxus_core::foundation::date_time::field_policy::{
    DateFieldPolicy, FieldRangeEndpoint, RangeFieldPolicy, RangeFieldRelationships,
    RangeFieldSnapshot, RangeFieldState,
};

use crate::{
    date_field::runtime::{SegmentFieldRuntime, SegmentFieldUiPolicy},
    foundation::{
        browser::focus_element_by_id,
        date_time::{CalendarDate, ChronoRange, FieldInvalidEvent, FieldInvalidReason},
        state::{ControlledBinding, sync_signal},
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
    pub(crate) controlled_value: CopyValue<ControlledBinding<ChronoRange<P::Value>>>,
    pub(crate) controlled_placeholder: CopyValue<ControlledBinding<P::Value>>,
    pub on_value_change: CopyValue<Option<EventHandler<ChronoRange<P::Value>>>>,
    pub on_start_value_change: CopyValue<Option<EventHandler<Option<P::Value>>>>,
    pub on_end_value_change: CopyValue<Option<EventHandler<Option<P::Value>>>>,
    pub on_placeholder_change: CopyValue<Option<EventHandler<P::Value>>>,
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
            controlled_value: self.controlled_value,
            controlled_placeholder: self.controlled_placeholder,
            on_value_change: self.on_value_change,
            on_start_value_change: self.on_start_value_change,
            on_end_value_change: self.on_end_value_change,
            on_placeholder_change: self.on_placeholder_change,
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

    pub fn update_part_value(&mut self, kind: FieldRangeEndpoint, next: Option<P::Value>) {
        let prev_range = self.state.peek().value();
        let prev_part = match kind {
            FieldRangeEndpoint::Start => prev_range.start,
            FieldRangeEndpoint::End => prev_range.end,
        };
        if prev_part == next {
            return;
        }

        let val_binding = self
            .controlled_value
            .try_read()
            .map(|slot| *slot)
            .unwrap_or(ControlledBinding::Uncontrolled);

        let mut next_range = prev_range;
        match kind {
            FieldRangeEndpoint::Start => next_range.start = next,
            FieldRangeEndpoint::End => next_range.end = next,
        }

        if val_binding.is_read_only() {
            match kind {
                FieldRangeEndpoint::Start => {
                    if let Some(cb) = self
                        .on_start_value_change
                        .try_read()
                        .ok()
                        .and_then(|slot| *slot)
                    {
                        cb.call(next);
                    }
                }
                FieldRangeEndpoint::End => {
                    if let Some(cb) = self
                        .on_end_value_change
                        .try_read()
                        .ok()
                        .and_then(|slot| *slot)
                    {
                        cb.call(next);
                    }
                }
            }
            if let Some(cb) = self.on_value_change.try_read().ok().and_then(|slot| *slot) {
                cb.call(next_range);
            }
            if let Some(accepted) = val_binding.peek_cloned()
                && self.state.peek().value() != accepted
            {
                self.state.write().set_value(accepted);
                if *self.start_value.peek() != accepted.start {
                    self.start_value.set(accepted.start);
                }
                if *self.end_value.peek() != accepted.end {
                    self.end_value.set(accepted.end);
                }
            }
            self.sync_validation_and_notify();
            return;
        }

        self.state.write().set_part_value(kind, next);
        let next_range = self.state.peek().value();

        match kind {
            FieldRangeEndpoint::Start => {
                if *self.start_value.peek() != next {
                    self.start_value.set(next);
                }
                if let Some(cb) = self
                    .on_start_value_change
                    .try_read()
                    .ok()
                    .and_then(|slot| *slot)
                {
                    cb.call(next);
                }
            }
            FieldRangeEndpoint::End => {
                if *self.end_value.peek() != next {
                    self.end_value.set(next);
                }
                if let Some(cb) = self
                    .on_end_value_change
                    .try_read()
                    .ok()
                    .and_then(|slot| *slot)
                {
                    cb.call(next);
                }
            }
        }

        val_binding.try_write_two_way(next_range);
        if let Some(cb) = self.on_value_change.try_read().ok().and_then(|slot| *slot) {
            cb.call(next_range);
        }

        self.sync_validation_and_notify();
    }

    pub fn update_placeholder(&mut self, next: P::Value) {
        if *self.placeholder.peek() == next {
            return;
        }
        let ph_binding = self
            .controlled_placeholder
            .try_read()
            .map(|slot| *slot)
            .unwrap_or(ControlledBinding::Uncontrolled);
        if ph_binding.is_read_only() {
            if let Some(cb) = self
                .on_placeholder_change
                .try_read()
                .ok()
                .and_then(|slot| *slot)
            {
                cb.call(next);
            }
            if let Some(accepted_ph) = ph_binding.peek_cloned()
                && *self.placeholder.peek() != accepted_ph
            {
                self.placeholder.set(accepted_ph);
                self.state.write().set_placeholder(accepted_ph);
            }
            return;
        }
        self.placeholder.set(next);
        self.state.write().set_placeholder(next);
        ph_binding.try_write_two_way(next);
        if let Some(cb) = self
            .on_placeholder_change
            .try_read()
            .ok()
            .and_then(|slot| *slot)
        {
            cb.call(next);
        }
    }

    pub fn reconcile_external_state(
        &mut self,
        external_value: Option<ChronoRange<P::Value>>,
        external_placeholder: Option<P::Value>,
        is_invalid_override: Option<bool>,
    ) {
        if let Some(val) = external_value
            && self.state.peek().value() != val
        {
            self.state.write().set_value(val);
            if *self.start_value.peek() != val.start {
                self.start_value.set(val.start);
            }
            if *self.end_value.peek() != val.end {
                self.end_value.set(val.end);
            }
        }
        if let Some(ph) = external_placeholder
            && *self.placeholder.peek() != ph
        {
            self.placeholder.set(ph);
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

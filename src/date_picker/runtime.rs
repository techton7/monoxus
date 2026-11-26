use std::ops::{Deref, DerefMut};

use dioxus::prelude::*;
use monoxus_core::date_picker::{PickerPolicy, PickerRelationships, PickerState};

use crate::foundation::date_time::{CalendarConfig, CalendarDate, DateValue};

use super::state::DatePickerPolicy;

/// Shared calendar configuration forwarded from `DatePickerRoot` to `DatePickerCalendar`.
#[derive(Clone, Debug, PartialEq)]
pub struct DatePickerCalendarConfig {
    pub config: CalendarConfig,
    pub initial_focus: bool,
}

impl Deref for DatePickerCalendarConfig {
    type Target = CalendarConfig;

    #[inline]
    fn deref(&self) -> &Self::Target {
        &self.config
    }
}

/// Zero-overhead policy-parameterized runtime core shared by `DatePicker` and `DateRangePicker`.
pub struct PickerRuntimeCore<P: PickerPolicy> {
    pub state: Signal<PickerState<P>>,
    pub rels: PickerRelationships<P>,
    pub value: Signal<P::Value>,
    pub calendar_value: Signal<P::CalendarValue>,
    pub placeholder: Signal<DateValue>,
    pub calendar_placeholder: Signal<CalendarDate>,
    pub open: Signal<bool>,
    pub controlled_value: CopyValue<Option<Signal<P::Value>>>,
    pub controlled_placeholder: CopyValue<Option<Signal<DateValue>>>,
    pub controlled_open: CopyValue<Option<Signal<bool>>>,
    pub on_value_change: CopyValue<Option<EventHandler<P::Value>>>,
    pub on_placeholder_change: CopyValue<Option<EventHandler<DateValue>>>,
    pub on_open_change: CopyValue<Option<EventHandler<bool>>>,
    pub is_date_disabled: Signal<Option<Callback<CalendarDate, bool>>>,
    pub is_date_unavailable: Signal<Option<Callback<CalendarDate, bool>>>,
}

impl<P: PickerPolicy> Clone for PickerRuntimeCore<P> {
    fn clone(&self) -> Self {
        Self {
            state: self.state,
            rels: self.rels.clone(),
            value: self.value,
            calendar_value: self.calendar_value,
            placeholder: self.placeholder,
            calendar_placeholder: self.calendar_placeholder,
            open: self.open,
            controlled_value: self.controlled_value,
            controlled_placeholder: self.controlled_placeholder,
            controlled_open: self.controlled_open,
            on_value_change: self.on_value_change,
            on_placeholder_change: self.on_placeholder_change,
            on_open_change: self.on_open_change,
            is_date_disabled: self.is_date_disabled,
            is_date_unavailable: self.is_date_unavailable,
        }
    }
}

impl<P: PickerPolicy> PartialEq for PickerRuntimeCore<P> {
    fn eq(&self, other: &Self) -> bool {
        self.rels == other.rels && self.state == other.state
    }
}

impl<P: PickerPolicy> PickerRuntimeCore<P> {
    pub fn snapshot(&self) -> P::Snapshot {
        self.state.read().snapshot()
    }

    pub(crate) fn sync_signals_from_state(&mut self) {
        let (val, cal_val, ph, cal_ph, open) = {
            let st = self.state.peek();
            (
                st.value(),
                st.calendar_value(),
                st.placeholder(),
                st.calendar_placeholder(),
                st.is_open(),
            )
        };
        if *self.value.peek() != val {
            self.value.set(val);
        }
        if *self.calendar_value.peek() != cal_val {
            self.calendar_value.set(cal_val);
        }
        if *self.placeholder.peek() != ph {
            self.placeholder.set(ph);
        }
        if *self.calendar_placeholder.peek() != cal_ph {
            self.calendar_placeholder.set(cal_ph);
        }
        if *self.open.peek() != open {
            self.open.set(open);
        }
    }

    pub(crate) fn notify_value_change(&self, next: P::Value) {
        if let Some(mut sig) = self.controlled_value.try_read().ok().and_then(|s| *s)
            && *sig.peek() != next
        {
            sig.set(next);
        }
        if let Some(cb) = self.on_value_change.try_read().ok().and_then(|s| *s) {
            cb.call(next);
        }
    }

    pub(crate) fn notify_placeholder_change(&self, next: DateValue) {
        if let Some(mut sig) = self.controlled_placeholder.try_read().ok().and_then(|s| *s)
            && *sig.peek() != next
        {
            sig.set(next);
        }
        if let Some(cb) = self.on_placeholder_change.try_read().ok().and_then(|s| *s) {
            cb.call(next);
        }
    }

    pub(crate) fn notify_open_change(&self, next: bool) {
        if let Some(mut sig) = self.controlled_open.try_read().ok().and_then(|s| *s)
            && *sig.peek() != next
        {
            sig.set(next);
        }
        if let Some(cb) = self.on_open_change.try_read().ok().and_then(|s| *s) {
            cb.call(next);
        }
    }

    pub fn set_open(&mut self, next: bool) {
        let prev = self.state.peek().is_open();
        let effective = self.state.write().set_open(next);
        self.sync_signals_from_state();
        if effective != prev {
            self.notify_open_change(effective);
        }
    }

    pub fn handle_popover_open_change(&mut self, next: bool) {
        self.set_open(next);
    }

    pub fn handle_field_placeholder_change(&mut self, next: DateValue) {
        if self.state.peek().placeholder() == next {
            return;
        }
        self.state.write().set_placeholder(next);
        self.sync_signals_from_state();
        self.notify_placeholder_change(next);
    }

    pub fn handle_calendar_placeholder_change(&mut self, next_date: CalendarDate) {
        let prev_ph = self.state.peek().placeholder();
        let next_ph = self.state.write().set_calendar_placeholder(next_date);
        self.sync_signals_from_state();
        if next_ph != prev_ph {
            self.notify_placeholder_change(next_ph);
        }
    }
}

/// Reactive runtime coordinating `DateField`, `Popover`, and `Calendar` for `DatePicker`.
#[derive(Clone)]
pub struct DatePickerRuntime {
    pub core: PickerRuntimeCore<DatePickerPolicy>,
    pub calendar_config: Signal<DatePickerCalendarConfig>,
}

impl Deref for DatePickerRuntime {
    type Target = PickerRuntimeCore<DatePickerPolicy>;

    #[inline]
    fn deref(&self) -> &Self::Target {
        &self.core
    }
}

impl DerefMut for DatePickerRuntime {
    #[inline]
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.core
    }
}

impl PartialEq for DatePickerRuntime {
    fn eq(&self, other: &Self) -> bool {
        self.core == other.core
    }
}

impl DatePickerRuntime {
    pub fn handle_field_value_change(&mut self, next: Option<DateValue>) {
        let (prev_val, prev_ph) = {
            let st = self.state.peek();
            (st.value(), st.placeholder())
        };
        if prev_val == next {
            return;
        }
        self.state.write().set_field_value(next);
        let next_ph = self.state.peek().placeholder();
        self.sync_signals_from_state();
        self.notify_value_change(next);
        if next_ph != prev_ph {
            self.notify_placeholder_change(next_ph);
        }
    }

    pub fn handle_calendar_value_change(&mut self, next: Option<CalendarDate>) {
        let (prev_val, prev_ph) = {
            let st = self.state.peek();
            (st.value(), st.placeholder())
        };
        let outcome = self.state.write().apply_calendar_value(next);
        self.sync_signals_from_state();
        if outcome.next_value != prev_val {
            self.notify_value_change(outcome.next_value);
        }
        if outcome.next_placeholder != prev_ph {
            self.notify_placeholder_change(outcome.next_placeholder);
        }
    }

    pub fn handle_calendar_date_select(&mut self, selected: CalendarDate) {
        let (prev_val, prev_ph, prev_open) = {
            let st = self.state.peek();
            (st.value(), st.placeholder(), st.is_open())
        };
        let outcome = self.state.write().commit_calendar_date_select(selected);
        let next_open = self.state.peek().is_open();
        self.sync_signals_from_state();
        if outcome.next_value != prev_val {
            self.notify_value_change(outcome.next_value);
        }
        if outcome.next_placeholder != prev_ph {
            self.notify_placeholder_change(outcome.next_placeholder);
        }
        if next_open != prev_open {
            self.notify_open_change(next_open);
        }
    }

    pub fn reconcile_external_state(
        &mut self,
        external_value: Option<Option<DateValue>>,
        external_placeholder: Option<DateValue>,
        external_open: Option<bool>,
    ) {
        if let Some(val) = external_value
            && self.state.peek().value() != val
        {
            self.state.write().set_field_value(val);
        }
        if let Some(ph) = external_placeholder
            && self.state.peek().placeholder() != ph
        {
            self.state.write().set_placeholder(ph);
        }
        if let Some(open) = external_open
            && self.state.peek().is_open() != open
        {
            self.state.write().set_open(open);
        }
        self.sync_signals_from_state();
    }
}

/// Dioxus context wrapping [`DatePickerRuntime`].
#[derive(Clone, PartialEq)]
pub struct DatePickerContext(pub DatePickerRuntime);

/// Consumes the nearest ancestor [`DatePickerRuntime`] context (panics if outside `DatePickerRoot`).
pub fn use_date_picker_context() -> DatePickerRuntime {
    use_context::<DatePickerContext>().0
}

/// Attempts to consume the nearest ancestor [`DatePickerRuntime`] context.
pub fn try_use_date_picker_context() -> Option<DatePickerRuntime> {
    try_consume_context::<DatePickerContext>().map(|ctx| ctx.0)
}

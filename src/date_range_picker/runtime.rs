use std::ops::{Deref, DerefMut};

use dioxus::prelude::*;

use crate::{
    date_picker::runtime::PickerRuntimeCore,
    foundation::{
        date_time::{CalendarConfig, CalendarDate, DateRange, DateValue, DateValueRange},
        state::ControllableSignal,
    },
};

use super::state::DateRangePickerPolicy;

/// Shared calendar configuration forwarded from `DateRangePickerRoot` to `DateRangePickerCalendar`.
#[derive(Clone, Debug, PartialEq)]
pub struct DateRangePickerCalendarConfig {
    pub config: CalendarConfig,
    pub min_days: Option<usize>,
    pub max_days: Option<usize>,
    pub exclude_disabled: bool,
    pub initial_focus: bool,
}

impl Deref for DateRangePickerCalendarConfig {
    type Target = CalendarConfig;

    #[inline]
    fn deref(&self) -> &Self::Target {
        &self.config
    }
}

/// Reactive runtime coordinating `DateRangeField`, `Popover`, and `RangeCalendar`.
#[derive(Clone)]
pub struct DateRangePickerRuntime {
    pub core: PickerRuntimeCore<DateRangePickerPolicy>,
    pub calendar_start: Signal<Option<CalendarDate>>,
    pub calendar_end: Signal<Option<CalendarDate>>,
    pub calendar_config: Signal<DateRangePickerCalendarConfig>,
    pub(crate) start_ctl: ControllableSignal<Option<DateValue>>,
    pub(crate) end_ctl: ControllableSignal<Option<DateValue>>,
    pub on_value_change: CopyValue<Option<EventHandler<DateValueRange>>>,
    pub on_start_value_change: CopyValue<Option<EventHandler<Option<DateValue>>>>,
    pub on_end_value_change: CopyValue<Option<EventHandler<Option<DateValue>>>>,
}

impl Deref for DateRangePickerRuntime {
    type Target = PickerRuntimeCore<DateRangePickerPolicy>;

    #[inline]
    fn deref(&self) -> &Self::Target {
        &self.core
    }
}

impl DerefMut for DateRangePickerRuntime {
    #[inline]
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.core
    }
}

impl PartialEq for DateRangePickerRuntime {
    fn eq(&self, other: &Self) -> bool {
        self.core == other.core
    }
}

impl DateRangePickerRuntime {
    fn sync_all_signals(&mut self) {
        self.core.sync_signals_from_state();
        let cal_val = self.state.peek().calendar_value();
        if *self.calendar_start.peek() != cal_val.start {
            self.calendar_start.set(cal_val.start);
        }
        if *self.calendar_end.peek() != cal_val.end {
            self.calendar_end.set(cal_val.end);
        }
    }

    fn notify_range_value_change(&self, prev: DateValueRange, next: DateValueRange) {
        let start_controlled = self.value_ctl.is_controlled() || self.start_ctl.is_controlled();
        let end_controlled = self.value_ctl.is_controlled() || self.end_ctl.is_controlled();
        let any_controlled = start_controlled || end_controlled;

        if prev.start != next.start {
            if !start_controlled {
                self.start_ctl.set(next.start);
            } else if let Some(cb) = self.on_start_value_change.try_read().ok().and_then(|s| *s) {
                cb.call(next.start);
            }
        }
        if prev.end != next.end {
            if !end_controlled {
                self.end_ctl.set(next.end);
            } else if let Some(cb) = self.on_end_value_change.try_read().ok().and_then(|s| *s) {
                cb.call(next.end);
            }
        }
        if prev != next {
            if !any_controlled {
                self.value_ctl.set(next);
            } else if let Some(cb) = self.on_value_change.try_read().ok().and_then(|s| *s) {
                cb.call(next);
            }
        }
    }

    pub fn handle_field_value_change(&mut self, next: DateValueRange) {
        let (prev_val, prev_ph) = {
            let st = self.state.peek();
            (st.value(), st.placeholder())
        };
        if prev_val == next {
            return;
        }

        let start_controlled = self.value_ctl.is_controlled() || self.start_ctl.is_controlled();
        let end_controlled = self.value_ctl.is_controlled() || self.end_ctl.is_controlled();

        let mut immediate = prev_val;
        if !start_controlled {
            immediate.start = next.start;
        }
        if !end_controlled {
            immediate.end = next.end;
        }
        if immediate != prev_val {
            self.state.write().set_field_value(immediate);
        }
        let next_ph = self.state.peek().placeholder();
        if self.placeholder_ctl.is_controlled() && next_ph != prev_ph {
            self.state.write().set_placeholder(prev_ph);
        }

        self.sync_all_signals();
        self.notify_range_value_change(prev_val, next);

        if next_ph != prev_ph && !(start_controlled || end_controlled) {
            self.placeholder_ctl.set(next_ph);
        }
    }

    pub fn handle_calendar_value_change(&mut self, next: DateRange) {
        let (prev_val, prev_ph) = {
            let st = self.state.peek();
            (st.value(), st.placeholder())
        };

        let mut candidate = self.state.peek().clone();
        let outcome = candidate.apply_calendar_range(next);

        let start_controlled = self.value_ctl.is_controlled() || self.start_ctl.is_controlled();
        let end_controlled = self.value_ctl.is_controlled() || self.end_ctl.is_controlled();

        let mut immediate = prev_val;
        if !start_controlled {
            immediate.start = outcome.next_value.start;
        }
        if !end_controlled {
            immediate.end = outcome.next_value.end;
        }
        if immediate != prev_val {
            self.state.write().set_field_value(immediate);
        }
        if !self.placeholder_ctl.is_controlled() && outcome.next_placeholder != prev_ph {
            self.state.write().set_placeholder(outcome.next_placeholder);
        } else if self.placeholder_ctl.is_controlled() && self.state.peek().placeholder() != prev_ph
        {
            self.state.write().set_placeholder(prev_ph);
        }

        self.sync_all_signals();
        if outcome.next_value != prev_val {
            self.notify_range_value_change(prev_val, outcome.next_value);
        }
        if outcome.next_placeholder != prev_ph && !(start_controlled || end_controlled) {
            self.placeholder_ctl.set(outcome.next_placeholder);
        }
    }

    pub fn handle_calendar_range_select(&mut self, completed: DateRange) {
        let (prev_val, prev_ph, prev_open) = {
            let st = self.state.peek();
            (st.value(), st.placeholder(), st.is_open())
        };

        let mut candidate = self.state.peek().clone();
        let outcome = candidate.commit_calendar_range_select(completed);
        let next_open = candidate.is_open();

        let start_controlled = self.value_ctl.is_controlled() || self.start_ctl.is_controlled();
        let end_controlled = self.value_ctl.is_controlled() || self.end_ctl.is_controlled();

        let mut immediate = prev_val;
        if !start_controlled {
            immediate.start = outcome.next_value.start;
        }
        if !end_controlled {
            immediate.end = outcome.next_value.end;
        }
        if immediate != prev_val {
            self.state.write().set_field_value(immediate);
        }
        if !self.placeholder_ctl.is_controlled() && outcome.next_placeholder != prev_ph {
            self.state.write().set_placeholder(outcome.next_placeholder);
        } else if self.placeholder_ctl.is_controlled() && self.state.peek().placeholder() != prev_ph
        {
            self.state.write().set_placeholder(prev_ph);
        }
        if !self.open_ctl.is_controlled() && next_open != prev_open {
            self.state.write().set_open(next_open);
        }

        self.sync_all_signals();
        if outcome.next_value != prev_val {
            self.notify_range_value_change(prev_val, outcome.next_value);
        }
        if outcome.next_placeholder != prev_ph && !(start_controlled || end_controlled) {
            self.placeholder_ctl.set(outcome.next_placeholder);
        }
        if next_open != prev_open {
            self.open_ctl.set(next_open);
        }
    }

    pub fn reconcile_external_state(
        &mut self,
        external_value: Option<DateValueRange>,
        external_start: Option<Option<DateValue>>,
        external_end: Option<Option<DateValue>>,
        external_placeholder: Option<DateValue>,
        external_open: Option<bool>,
    ) {
        let mut target_val = external_value.unwrap_or_else(|| self.state.peek().value());
        if let Some(s) = external_start {
            target_val.start = s;
        }
        if let Some(e) = external_end {
            target_val.end = e;
        }
        if self.state.peek().value() != target_val {
            self.state.write().set_field_value(target_val);
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
        self.sync_all_signals();
    }
}

/// Dioxus context wrapping [`DateRangePickerRuntime`].
#[derive(Clone, PartialEq)]
pub struct DateRangePickerContext(pub DateRangePickerRuntime);

/// Consumes the nearest ancestor [`DateRangePickerRuntime`] context (panics if outside `DateRangePickerRoot`).
pub fn use_date_range_picker_context() -> DateRangePickerRuntime {
    use_context::<DateRangePickerContext>().0
}

/// Attempts to consume the nearest ancestor [`DateRangePickerRuntime`] context.
pub fn try_use_date_range_picker_context() -> Option<DateRangePickerRuntime> {
    try_consume_context::<DateRangePickerContext>().map(|ctx| ctx.0)
}

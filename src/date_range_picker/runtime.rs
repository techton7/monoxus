use std::ops::{Deref, DerefMut};

use dioxus::prelude::*;

use crate::{
    date_picker::runtime::PickerRuntimeCore,
    foundation::date_time::{CalendarConfig, CalendarDate, DateRange, DateValue, DateValueRange},
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
    pub controlled_start_value: CopyValue<Option<Signal<Option<DateValue>>>>,
    pub controlled_end_value: CopyValue<Option<Signal<Option<DateValue>>>>,
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
        if prev.start != next.start {
            if let Some(mut sig) = self.controlled_start_value.try_read().ok().and_then(|s| *s)
                && *sig.peek() != next.start
            {
                sig.set(next.start);
            }
            if let Some(cb) = self.on_start_value_change.try_read().ok().and_then(|s| *s) {
                cb.call(next.start);
            }
        }
        if prev.end != next.end {
            if let Some(mut sig) = self.controlled_end_value.try_read().ok().and_then(|s| *s)
                && *sig.peek() != next.end
            {
                sig.set(next.end);
            }
            if let Some(cb) = self.on_end_value_change.try_read().ok().and_then(|s| *s) {
                cb.call(next.end);
            }
        }
        if prev != next {
            self.core.notify_value_change(next);
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
        self.state.write().set_field_value(next);
        let next_ph = self.state.peek().placeholder();
        self.sync_all_signals();
        self.notify_range_value_change(prev_val, next);
        if next_ph != prev_ph {
            self.notify_placeholder_change(next_ph);
        }
    }

    pub fn handle_calendar_value_change(&mut self, next: DateRange) {
        let (prev_val, prev_ph) = {
            let st = self.state.peek();
            (st.value(), st.placeholder())
        };
        let outcome = self.state.write().apply_calendar_range(next);
        self.sync_all_signals();
        if outcome.next_value != prev_val {
            self.notify_range_value_change(prev_val, outcome.next_value);
        }
        if outcome.next_placeholder != prev_ph {
            self.notify_placeholder_change(outcome.next_placeholder);
        }
    }

    pub fn handle_calendar_range_select(&mut self, completed: DateRange) {
        let (prev_val, prev_ph, prev_open) = {
            let st = self.state.peek();
            (st.value(), st.placeholder(), st.is_open())
        };
        let outcome = self.state.write().commit_calendar_range_select(completed);
        let next_open = self.state.peek().is_open();
        self.sync_all_signals();
        if outcome.next_value != prev_val {
            self.notify_range_value_change(prev_val, outcome.next_value);
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

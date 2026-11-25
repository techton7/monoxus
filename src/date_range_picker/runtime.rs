use std::{cell::RefCell, rc::Rc};

use dioxus::prelude::*;

use crate::foundation::{
    date_time::{
        CalendarDate, CalendarLocale, DateRange, DateValue, DateValueRange, MonthFormat, Weekday,
        WeekdayFormat, YearFormat,
    },
    shared::Direction,
};

use super::{
    relationships::DateRangePickerRelationships,
    state::DateRangePickerState,
    types::DateRangePickerSnapshot,
};

/// Shared calendar configuration forwarded from `DateRangePickerRoot` to `DateRangePickerCalendar`.
#[derive(Clone, Debug, PartialEq)]
pub struct DateRangePickerCalendarConfig {
    pub min_value: Option<CalendarDate>,
    pub max_value: Option<CalendarDate>,
    pub min_days: Option<usize>,
    pub max_days: Option<usize>,
    pub exclude_disabled: bool,
    pub disabled: bool,
    pub read_only: bool,
    pub prevent_deselect: bool,
    pub paged_navigation: bool,
    pub fixed_weeks: bool,
    pub number_of_months: usize,
    pub week_starts_on: Option<Weekday>,
    pub weekday_format: WeekdayFormat,
    pub month_format: MonthFormat,
    pub year_format: YearFormat,
    pub locale: CalendarLocale,
    pub calendar_label: String,
    pub dir: Direction,
    pub disable_days_outside_month: bool,
    pub initial_focus: bool,
}

/// Reactive runtime coordinating `DateRangeField`, `Popover`, and `RangeCalendar`.
#[derive(Clone)]
pub struct DateRangePickerRuntime {
    pub state: Signal<DateRangePickerState>,
    pub rels: DateRangePickerRelationships,
    pub value: Signal<DateValueRange>,
    pub calendar_value: Signal<DateRange>,
    pub calendar_start: Signal<Option<CalendarDate>>,
    pub calendar_end: Signal<Option<CalendarDate>>,
    pub placeholder: Signal<DateValue>,
    pub calendar_placeholder: Signal<CalendarDate>,
    pub open: Signal<bool>,
    pub calendar_config: Signal<DateRangePickerCalendarConfig>,
    pub controlled_value: Rc<RefCell<Option<Signal<DateValueRange>>>>,
    pub controlled_start_value: Rc<RefCell<Option<Signal<Option<DateValue>>>>>,
    pub controlled_end_value: Rc<RefCell<Option<Signal<Option<DateValue>>>>>,
    pub controlled_placeholder: Rc<RefCell<Option<Signal<DateValue>>>>,
    pub controlled_open: Rc<RefCell<Option<Signal<bool>>>>,
    pub on_value_change: Rc<RefCell<Option<EventHandler<DateValueRange>>>>,
    pub on_start_value_change: Rc<RefCell<Option<EventHandler<Option<DateValue>>>>>,
    pub on_end_value_change: Rc<RefCell<Option<EventHandler<Option<DateValue>>>>>,
    pub on_placeholder_change: Rc<RefCell<Option<EventHandler<DateValue>>>>,
    pub on_open_change: Rc<RefCell<Option<EventHandler<bool>>>>,
    pub is_date_disabled: Signal<Option<Callback<CalendarDate, bool>>>,
    pub is_date_unavailable: Signal<Option<Callback<CalendarDate, bool>>>,
}

impl PartialEq for DateRangePickerRuntime {
    fn eq(&self, other: &Self) -> bool {
        self.rels == other.rels && self.state == other.state
    }
}

impl DateRangePickerRuntime {
    pub fn snapshot(&self) -> DateRangePickerSnapshot {
        self.state.read().snapshot()
    }

    fn sync_signals_from_state(&mut self) {
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
        if *self.calendar_start.peek() != cal_val.start {
            self.calendar_start.set(cal_val.start);
        }
        if *self.calendar_end.peek() != cal_val.end {
            self.calendar_end.set(cal_val.end);
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

    fn notify_value_change(&self, prev: DateValueRange, next: DateValueRange) {
        if prev.start != next.start {
            if let Some(mut sig) = *self.controlled_start_value.borrow()
                && *sig.peek() != next.start
            {
                sig.set(next.start);
            }
            if let Some(cb) = self.on_start_value_change.borrow().as_ref() {
                cb.call(next.start);
            }
        }
        if prev.end != next.end {
            if let Some(mut sig) = *self.controlled_end_value.borrow()
                && *sig.peek() != next.end
            {
                sig.set(next.end);
            }
            if let Some(cb) = self.on_end_value_change.borrow().as_ref() {
                cb.call(next.end);
            }
        }
        if prev != next {
            if let Some(mut sig) = *self.controlled_value.borrow()
                && *sig.peek() != next
            {
                sig.set(next);
            }
            if let Some(cb) = self.on_value_change.borrow().as_ref() {
                cb.call(next);
            }
        }
    }

    fn notify_placeholder_change(&self, next: DateValue) {
        if let Some(mut sig) = *self.controlled_placeholder.borrow()
            && *sig.peek() != next
        {
            sig.set(next);
        }
        if let Some(cb) = self.on_placeholder_change.borrow().as_ref() {
            cb.call(next);
        }
    }

    fn notify_open_change(&self, next: bool) {
        if let Some(mut sig) = *self.controlled_open.borrow()
            && *sig.peek() != next
        {
            sig.set(next);
        }
        if let Some(cb) = self.on_open_change.borrow().as_ref() {
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
        self.sync_signals_from_state();
        self.notify_value_change(prev_val, next);
        if next_ph != prev_ph {
            self.notify_placeholder_change(next_ph);
        }
    }

    pub fn handle_field_placeholder_change(&mut self, next: DateValue) {
        if self.state.peek().placeholder() == next {
            return;
        }
        self.state.write().set_placeholder(next);
        self.sync_signals_from_state();
        self.notify_placeholder_change(next);
    }

    pub fn handle_calendar_value_change(&mut self, next: DateRange) {
        let (prev_val, prev_ph) = {
            let st = self.state.peek();
            (st.value(), st.placeholder())
        };
        let outcome = self.state.write().apply_calendar_range(next);
        self.sync_signals_from_state();
        if outcome.next_value != prev_val {
            self.notify_value_change(prev_val, outcome.next_value);
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
        self.sync_signals_from_state();
        if outcome.next_value != prev_val {
            self.notify_value_change(prev_val, outcome.next_value);
        }
        if outcome.next_placeholder != prev_ph {
            self.notify_placeholder_change(outcome.next_placeholder);
        }
        if next_open != prev_open {
            self.notify_open_change(next_open);
        }
    }

    pub fn handle_calendar_placeholder_change(&mut self, next_date: CalendarDate) {
        let prev_ph = self.state.peek().placeholder();
        let next_ph = self.state.write().set_calendar_placeholder(next_date);
        self.sync_signals_from_state();
        if next_ph != prev_ph {
            self.notify_placeholder_change(next_ph);
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
        self.sync_signals_from_state();
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

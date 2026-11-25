use std::{cell::RefCell, rc::Rc};

use dioxus::prelude::*;

use crate::foundation::{
    date_time::{
        CalendarDate, CalendarLocale, DateValue, MonthFormat, Weekday, WeekdayFormat, YearFormat,
    },
    shared::Direction,
};

use super::{
    relationships::DatePickerRelationships,
    state::DatePickerState,
    types::DatePickerSnapshot,
};

/// Shared calendar configuration forwarded from `DatePickerRoot` to `DatePickerCalendar`.
#[derive(Clone, Debug, PartialEq)]
pub struct DatePickerCalendarConfig {
    pub min_value: Option<CalendarDate>,
    pub max_value: Option<CalendarDate>,
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

/// Reactive runtime coordinating `DateField`, `Popover`, and `Calendar` for `DatePicker`.
#[derive(Clone)]
pub struct DatePickerRuntime {
    pub state: Signal<DatePickerState>,
    pub rels: DatePickerRelationships,
    pub value: Signal<Option<DateValue>>,
    pub calendar_value: Signal<Option<CalendarDate>>,
    pub placeholder: Signal<DateValue>,
    pub calendar_placeholder: Signal<CalendarDate>,
    pub open: Signal<bool>,
    pub calendar_config: Signal<DatePickerCalendarConfig>,
    pub controlled_value: Rc<RefCell<Option<Signal<Option<DateValue>>>>>,
    pub controlled_placeholder: Rc<RefCell<Option<Signal<DateValue>>>>,
    pub controlled_open: Rc<RefCell<Option<Signal<bool>>>>,
    pub on_value_change: Rc<RefCell<Option<EventHandler<Option<DateValue>>>>>,
    pub on_placeholder_change: Rc<RefCell<Option<EventHandler<DateValue>>>>,
    pub on_open_change: Rc<RefCell<Option<EventHandler<bool>>>>,
    pub is_date_disabled: Signal<Option<Callback<CalendarDate, bool>>>,
    pub is_date_unavailable: Signal<Option<Callback<CalendarDate, bool>>>,
}

impl PartialEq for DatePickerRuntime {
    fn eq(&self, other: &Self) -> bool {
        self.rels == other.rels && self.state == other.state
    }
}

impl DatePickerRuntime {
    pub fn snapshot(&self) -> DatePickerSnapshot {
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

    fn notify_value_change(&self, next: Option<DateValue>) {
        if let Some(mut sig) = *self.controlled_value.borrow()
            && *sig.peek() != next
        {
            sig.set(next);
        }
        if let Some(cb) = self.on_value_change.borrow().as_ref() {
            cb.call(next);
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

    pub fn handle_field_placeholder_change(&mut self, next: DateValue) {
        if self.state.peek().placeholder() == next {
            return;
        }
        self.state.write().set_placeholder(next);
        self.sync_signals_from_state();
        self.notify_placeholder_change(next);
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

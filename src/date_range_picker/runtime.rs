use std::ops::{Deref, DerefMut};

use dioxus::prelude::*;
use ui_sdk::kernel::{ChangeContext, ChangeEvent, ChangeReason, ChangeSource};

use crate::{
    agent::{cancel_current_agent_action, current_change_context},
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
    pub on_value_change_details: CopyValue<Option<EventHandler<ChangeEvent<DateValueRange>>>>,
    pub on_start_value_change: CopyValue<Option<EventHandler<Option<DateValue>>>>,
    pub on_start_value_change_details:
        CopyValue<Option<EventHandler<ChangeEvent<Option<DateValue>>>>>,
    pub on_end_value_change: CopyValue<Option<EventHandler<Option<DateValue>>>>,
    pub on_end_value_change_details:
        CopyValue<Option<EventHandler<ChangeEvent<Option<DateValue>>>>>,
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

    fn propose_range_value_change(
        &self,
        prev: DateValueRange,
        next: DateValueRange,
        ctx: ChangeContext,
    ) -> bool {
        let start_controlled = self.value_ctl.is_controlled() || self.start_ctl.is_controlled();
        let end_controlled = self.value_ctl.is_controlled() || self.end_ctl.is_controlled();
        let any_controlled = start_controlled || end_controlled;

        if prev != next {
            if !any_controlled || self.value_ctl.is_controlled() {
                if self.value_ctl.set_with_context(next, ctx).is_canceled() {
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
        false
    }

    pub fn handle_field_value_change(&mut self, next: DateValueRange) {
        let ctx = current_change_context()
            .unwrap_or_else(|| ChangeContext::new(ChangeReason::ValueChange, ChangeSource::Mouse));
        let _ = self.handle_field_value_change_with_context(next, ctx);
    }

    pub(crate) fn handle_field_value_change_with_context(
        &mut self,
        next: DateValueRange,
        ctx: ChangeContext,
    ) -> bool {
        let (prev_val, prev_ph) = {
            let st = self.state.peek();
            (st.value(), st.placeholder())
        };
        if prev_val == next {
            return false;
        }

        if self.propose_range_value_change(prev_val, next, ctx) {
            return true;
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
        if next_ph != prev_ph {
            if !(start_controlled || end_controlled) {
                let ph_outcome = self.placeholder_ctl.set_with_context(next_ph, ctx);
                if !ph_outcome.is_committed() {
                    self.state.write().set_placeholder(prev_ph);
                }
            } else if self.placeholder_ctl.is_controlled() {
                self.state.write().set_placeholder(prev_ph);
            }
        }

        self.sync_all_signals();
        false
    }

    pub fn handle_calendar_value_change(&mut self, next: DateRange) {
        let (prev_val, prev_ph) = {
            let st = self.state.peek();
            (st.value(), st.placeholder())
        };

        let mut candidate = self.state.peek().clone();
        let outcome = candidate.apply_calendar_range(next);
        let ctx = current_change_context()
            .unwrap_or_else(|| ChangeContext::new(ChangeReason::ValueChange, ChangeSource::Mouse));

        if outcome.next_value != prev_val
            && self.propose_range_value_change(prev_val, outcome.next_value, ctx)
        {
            return;
        }

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
        if outcome.next_placeholder != prev_ph && !(start_controlled || end_controlled) {
            let ph_outcome = self
                .placeholder_ctl
                .set_with_context(outcome.next_placeholder, ctx);
            if ph_outcome.is_committed() {
                self.state.write().set_placeholder(outcome.next_placeholder);
            } else {
                self.state.write().set_placeholder(prev_ph);
            }
        } else if self.placeholder_ctl.is_controlled() && self.state.peek().placeholder() != prev_ph
        {
            self.state.write().set_placeholder(prev_ph);
        }

        self.sync_all_signals();
    }

    pub fn handle_calendar_range_select(&mut self, completed: DateRange) {
        let (prev_val, prev_ph, prev_open) = {
            let st = self.state.peek();
            (st.value(), st.placeholder(), st.is_open())
        };

        let mut candidate = self.state.peek().clone();
        let outcome = candidate.commit_calendar_range_select(completed);
        let next_open = candidate.is_open();
        let ctx = current_change_context()
            .unwrap_or_else(|| ChangeContext::new(ChangeReason::ValueChange, ChangeSource::Mouse));

        if outcome.next_value != prev_val
            && self.propose_range_value_change(prev_val, outcome.next_value, ctx)
        {
            return;
        }

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
        if outcome.next_placeholder != prev_ph && !(start_controlled || end_controlled) {
            let ph_outcome = self
                .placeholder_ctl
                .set_with_context(outcome.next_placeholder, ctx);
            if ph_outcome.is_committed() {
                self.state.write().set_placeholder(outcome.next_placeholder);
            } else {
                self.state.write().set_placeholder(prev_ph);
            }
        } else if self.placeholder_ctl.is_controlled() && self.state.peek().placeholder() != prev_ph
        {
            self.state.write().set_placeholder(prev_ph);
        }
        if next_open != prev_open {
            let open_outcome = self.open_ctl.set_with_context(next_open, ctx);
            if open_outcome.is_committed() {
                self.state.write().set_open(next_open);
            }
        }

        self.sync_all_signals();
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

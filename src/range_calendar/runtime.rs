use std::ops::{Deref, DerefMut};

use dioxus::prelude::*;
use ui_sdk::kernel::{ChangeContext, ChangeEvent, ChangeReason, ChangeSource};

use crate::{
    agent::{cancel_current_agent_action, current_change_context},
    calendar::{
        attrs::CalendarRootAttrs,
        runtime::CalendarBaseRuntime,
        types::CalendarSelectOption,
    },
    foundation::{
        date_time::{
            CalendarDate, CalendarMonth, CalendarNavKey, DateRange, MonthFormat, YearFormat,
            compute_highlighted_range,
        },
        state::ControllableSignal,
    },
};

use super::{
    attrs::{RangeCalendarCellAttrs, RangeCalendarDayAttrs},
    state::{
        RangeCalendarDefinition, RangeCalendarState, RangeCalendarUpdate, apply_range_click,
        apply_range_escape, is_date_selected_in_range, is_range_selection_invalid,
        reconcile_exclude_disabled,
    },
    types::{RangeCalendarCellSnapshot, RangeCalendarSnapshot},
};

/// Reactive runtime for `RangeCalendar`.
#[derive(Clone)]
pub struct RangeCalendarRuntime {
    pub base: CalendarBaseRuntime,
    pub def: Signal<RangeCalendarDefinition>,
    pub value: Signal<DateRange>,
    pub(crate) value_ctl: ControllableSignal<DateRange>,
    pub start_value: Signal<Option<CalendarDate>>,
    pub(crate) start_ctl: ControllableSignal<Option<CalendarDate>>,
    pub end_value: Signal<Option<CalendarDate>>,
    pub(crate) end_ctl: ControllableSignal<Option<CalendarDate>>,
    pub last_pressed: Signal<Option<CalendarDate>>,
    pub focused_value: Signal<Option<CalendarDate>>,
    pub on_value_change: CopyValue<Option<EventHandler<DateRange>>>,
    pub on_value_change_details: CopyValue<Option<EventHandler<ChangeEvent<DateRange>>>>,
    pub on_start_value_change: CopyValue<Option<EventHandler<Option<CalendarDate>>>>,
    pub on_start_value_change_details:
        CopyValue<Option<EventHandler<ChangeEvent<Option<CalendarDate>>>>>,
    pub on_end_value_change: CopyValue<Option<EventHandler<Option<CalendarDate>>>>,
    pub on_end_value_change_details:
        CopyValue<Option<EventHandler<ChangeEvent<Option<CalendarDate>>>>>,
    pub on_range_select: CopyValue<Option<EventHandler<DateRange>>>,
}

impl Deref for RangeCalendarRuntime {
    type Target = CalendarBaseRuntime;

    #[inline]
    fn deref(&self) -> &Self::Target {
        &self.base
    }
}

impl DerefMut for RangeCalendarRuntime {
    #[inline]
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.base
    }
}

impl RangeCalendarRuntime {
    pub fn is_date_selected(&self, date: CalendarDate) -> bool {
        is_date_selected_in_range(date, *self.start_value.read(), *self.end_value.read())
    }

    pub fn highlighted_range(&self) -> Option<(CalendarDate, CalendarDate)> {
        compute_highlighted_range(
            *self.start_value.read(),
            *self.end_value.read(),
            *self.focused_value.read(),
            |d| self.is_date_disabled(d),
            |d| self.is_date_unavailable(d),
        )
    }

    pub fn is_invalid(&self) -> bool {
        is_range_selection_invalid(
            *self.start_value.read(),
            *self.end_value.read(),
            |d| self.is_date_disabled(d),
            |d| self.is_date_unavailable(d),
        )
    }

    pub fn snapshot(&self) -> RangeCalendarSnapshot {
        RangeCalendarSnapshot {
            value: *self.value.read(),
            start_value: *self.start_value.read(),
            end_value: *self.end_value.read(),
            highlighted_range: self.highlighted_range(),
            placeholder: self.current_placeholder(),
            months: self.current_months(),
            weekdays: self.weekdays(),
            heading: self.heading_value(),
            full_label: self.full_calendar_label(),
            is_invalid: self.is_invalid(),
            is_prev_disabled: self.is_prev_disabled(),
            is_next_disabled: self.is_next_disabled(),
            tabbable_date: self.tabbable_date(),
        }
    }

    pub fn set_focused_value(&mut self, date: Option<CalendarDate>) {
        if *self.focused_value.peek() != date {
            self.focused_value.set(date);
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

    pub(crate) fn set_range_from_agent(&mut self, next: DateRange, ctx: ChangeContext) -> bool {
        let update = RangeCalendarUpdate {
            value: next,
            start_value: next.start,
            end_value: next.end,
            last_pressed: next.end.or(next.start),
            placeholder: next.start.or(next.end),
            announcement: None,
            range_completed: (next.start.is_some() && next.end.is_some()).then_some(next),
        };
        self.commit_update_with_context(update, ctx)
    }

    fn commit_update(&mut self, update: RangeCalendarUpdate) {
        let ctx = current_change_context()
            .unwrap_or_else(|| ChangeContext::new(ChangeReason::Selection, ChangeSource::Mouse));
        let _ = self.commit_update_with_context(update, ctx);
    }

    fn commit_update_with_context(
        &mut self,
        update: RangeCalendarUpdate,
        ctx: ChangeContext,
    ) -> bool {
        let prev_start = *self.start_value.peek();
        let prev_end = *self.end_value.peek();
        let prev_val = *self.value.peek();

        let start_controlled = self.value_ctl.is_controlled() || self.start_ctl.is_controlled();
        let end_controlled = self.value_ctl.is_controlled() || self.end_ctl.is_controlled();
        let any_controlled = start_controlled || end_controlled;

        if prev_val != update.value {
            if !any_controlled || self.value_ctl.is_controlled() {
                let outcome = self.value_ctl.set_with_context(update.value, ctx);
                if outcome.is_canceled() {
                    cancel_current_agent_action();
                    return true;
                }
            } else if Self::notify_controlled_fallback(
                update.value,
                ctx,
                self.on_value_change_details,
                self.on_value_change,
            ) {
                cancel_current_agent_action();
                return true;
            }
        }

        if prev_start != update.start_value {
            if !start_controlled || self.start_ctl.is_controlled() {
                let outcome = self.start_ctl.set_with_context(update.start_value, ctx);
                if outcome.is_canceled() {
                    cancel_current_agent_action();
                }
            } else {
                let _ = Self::notify_controlled_fallback(
                    update.start_value,
                    ctx,
                    self.on_start_value_change_details,
                    self.on_start_value_change,
                );
            }
        }

        if prev_end != update.end_value {
            if !end_controlled || self.end_ctl.is_controlled() {
                let outcome = self.end_ctl.set_with_context(update.end_value, ctx);
                if outcome.is_canceled() {
                    cancel_current_agent_action();
                }
            } else {
                let _ = Self::notify_controlled_fallback(
                    update.end_value,
                    ctx,
                    self.on_end_value_change_details,
                    self.on_end_value_change,
                );
            }
        }

        if !any_controlled {
            if *self.last_pressed.peek() != update.last_pressed {
                self.last_pressed.set(update.last_pressed);
            }
            if let Some(next_ph) = update.placeholder {
                self.set_placeholder(next_ph);
            }
            if let Some(text) = update.announcement {
                self.announcement.set(text);
            }
        }
        if let Some(completed) = update.range_completed
            && let Some(handler) = self
                .on_range_select
                .try_read()
                .ok()
                .and_then(|slot| *slot)
        {
            handler.call(completed);
        }
        false
    }

    pub fn activate_date(&mut self, date: CalendarDate, is_outside_month: bool) {
        let def = self.def.read().clone();
        let is_cell_disabled =
            self.is_date_disabled(date) || (is_outside_month && def.disable_days_outside_month);
        let is_cell_unavailable = self.is_date_unavailable(date);

        let state = RangeCalendarState {
            value: *self.value.peek(),
            start_value: *self.start_value.peek(),
            end_value: *self.end_value.peek(),
            last_pressed: *self.last_pressed.peek(),
            focused_value: *self.focused_value.peek(),
        };

        let Some(update) = apply_range_click(
            &state,
            date,
            def.read_only,
            def.prevent_deselect,
            is_cell_disabled,
            is_cell_unavailable,
            def.min_days,
            def.max_days,
            def.exclude_disabled,
            &def.locale,
            |d| self.is_date_disabled(d),
            |d| self.is_date_unavailable(d),
        ) else {
            return;
        };

        self.commit_update(update);
    }

    pub fn handle_escape(&mut self) -> bool {
        let def = self.def.read().clone();
        let state = RangeCalendarState {
            value: *self.value.peek(),
            start_value: *self.start_value.peek(),
            end_value: *self.end_value.peek(),
            last_pressed: *self.last_pressed.peek(),
            focused_value: *self.focused_value.peek(),
        };
        let Some(update) = apply_range_escape(&state, def.read_only, def.disabled, &def.locale)
        else {
            return false;
        };
        self.commit_update(update);
        true
    }

    pub fn reconcile_disabled_in_range(&mut self) {
        let def = self.def.read().clone();
        let start = *self.start_value.peek();
        let end = *self.end_value.peek();
        let update = reconcile_exclude_disabled(
            start,
            end,
            def.exclude_disabled,
            &def.locale,
            |d| self.is_date_disabled(d),
        );
        if let Some(update) = update {
            self.commit_update(update);
        }
    }

    pub fn handle_nav_key(&mut self, current_date: CalendarDate, key: CalendarNavKey) -> bool {
        let Some(target) = self.base.handle_nav_key_target(current_date, key) else {
            return false;
        };
        self.set_focused_value(Some(target));
        true
    }

    pub fn root_attrs(&self, custom_id: Option<String>) -> CalendarRootAttrs {
        self.base.root_attrs_with_invalid(self.is_invalid(), custom_id)
    }

    pub fn cell_attrs(
        &self,
        date: CalendarDate,
        month: CalendarDate,
        custom_id: Option<String>,
    ) -> RangeCalendarCellAttrs {
        let cfg = self.config.read();
        let month_idx = self.month_index_for(month);
        let id = custom_id.unwrap_or_else(|| self.rels.cell_id(month_idx, date));
        let is_outside_month = !date.is_same_month(month);
        let start = *self.start_value.read();
        let end = *self.end_value.read();
        let is_selected = self.is_date_selected(date);
        let is_selection_start = start.is_some_and(|s| s == date);
        let is_selection_end = end.is_some_and(|e| e == date);
        let is_range_start = is_selection_start;
        let is_range_end = if end.is_none() {
            is_selection_start
        } else {
            is_selection_end
        };
        let is_range_middle = is_selected && !is_selection_start && !is_selection_end;
        let is_highlighted = self
            .highlighted_range()
            .is_some_and(|(lo, hi)| date.is_between_inclusive(lo, hi));

        RangeCalendarCellAttrs::new(
            id,
            date,
            is_selected,
            self.is_date_disabled(date),
            self.is_date_unavailable(date),
            date == *self.today_date.read(),
            is_outside_month,
            self.is_outside_visible_months(date),
            date == self.current_placeholder(),
            is_selection_start,
            is_selection_end,
            is_range_start,
            is_range_end,
            is_range_middle,
            is_highlighted,
            cfg.disable_days_outside_month,
        )
    }

    pub fn day_attrs(
        &self,
        date: CalendarDate,
        month: CalendarDate,
        cell_attrs: &RangeCalendarCellAttrs,
        custom_id: Option<String>,
    ) -> RangeCalendarDayAttrs {
        let cfg = self.config.read();
        let month_idx = self.month_index_for(month);
        let id = custom_id.unwrap_or_else(|| self.rels.day_id(month_idx, date));
        let is_tabbable = date == self.tabbable_date();
        RangeCalendarDayAttrs::from_cell(id, date, &cfg.locale, cell_attrs, is_tabbable)
    }

    pub fn cell_snapshot(
        &self,
        date: CalendarDate,
        month: CalendarDate,
        cell_attrs: &RangeCalendarCellAttrs,
    ) -> RangeCalendarCellSnapshot {
        RangeCalendarCellSnapshot {
            date,
            month,
            day_text: date.day.to_string(),
            is_disabled: cell_attrs.data_disabled.is_some(),
            is_unavailable: cell_attrs.data_unavailable.is_some(),
            is_selected: cell_attrs.data_selected.is_some(),
            is_today: cell_attrs.data_today.is_some(),
            is_outside_month: cell_attrs.data_outside_month.is_some(),
            is_outside_visible_months: cell_attrs.data_outside_visible_months.is_some(),
            is_focused: cell_attrs.data_focused.is_some(),
            is_selection_start: cell_attrs.data_selection_start.is_some(),
            is_selection_end: cell_attrs.data_selection_end.is_some(),
            is_range_start: cell_attrs.data_range_start.is_some(),
            is_range_end: cell_attrs.data_range_end.is_some(),
            is_range_middle: cell_attrs.data_range_middle.is_some(),
            is_highlighted: cell_attrs.data_highlighted.is_some(),
        }
    }
}

#[derive(Clone)]
pub struct RangeCalendarContext {
    pub runtime: RangeCalendarRuntime,
}

impl RangeCalendarContext {
    pub fn snapshot(&self) -> RangeCalendarSnapshot {
        self.runtime.snapshot()
    }

    pub fn months(&self) -> Vec<CalendarMonth> {
        self.runtime.current_months()
    }

    pub fn weekdays(&self) -> Vec<String> {
        self.runtime.weekdays()
    }

    pub fn heading(&self) -> String {
        self.runtime.heading_value()
    }

    pub fn prev_page(&mut self) {
        self.runtime.prev_page();
    }

    pub fn next_page(&mut self) {
        self.runtime.next_page();
    }

    pub fn current_month(&self) -> u8 {
        self.runtime.current_month()
    }

    pub fn current_year(&self) -> i32 {
        self.runtime.current_year()
    }

    pub fn set_month(&mut self, month: u8) {
        self.runtime.set_month(month);
    }

    pub fn set_year(&mut self, year: i32) {
        self.runtime.set_year(year);
    }

    pub fn default_years(&self) -> Vec<i32> {
        self.runtime.default_years()
    }

    pub fn month_options(&self, format: MonthFormat) -> Vec<CalendarSelectOption> {
        self.runtime.month_options(format)
    }

    pub fn year_options(&self, format: YearFormat) -> Vec<CalendarSelectOption> {
        self.runtime.year_options(format)
    }
}

pub fn use_range_calendar_context() -> RangeCalendarContext {
    use_context::<RangeCalendarContext>()
}

pub fn try_use_range_calendar_context() -> Option<RangeCalendarContext> {
    try_consume_context::<RangeCalendarContext>()
}

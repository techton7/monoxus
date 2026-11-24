use std::{cell::RefCell, collections::HashMap, rc::Rc};

use dioxus::prelude::*;

use crate::{
    calendar::{
        attrs::CalendarRootAttrs,
        runtime::{CalendarViewConfig, local_today},
        types::{
            CalendarKind, CalendarSelectOption, build_calendar_month_options,
            build_calendar_year_options,
        },
    },
    foundation::{
        browser::{FocusOptions, focus_element_sync_or_defer},
        date_time::{
            CalendarDate, CalendarMonth, CalendarNavKey, DateRange, MonthFormat, YearFormat,
            build_visible_months, compute_highlighted_range, default_year_range,
            ensure_non_disabled_placeholder, is_next_page_disabled, is_prev_page_disabled,
            resolve_grid_navigation, resolve_tabbable_date, visible_weekday_headers,
        },
    },
};

use super::{
    attrs::{RangeCalendarCellAttrs, RangeCalendarDayAttrs},
    relationships::RangeCalendarRelationships,
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
    pub def: Signal<RangeCalendarDefinition>,
    pub rels: RangeCalendarRelationships,
    pub root_dom_id: Signal<String>,
    pub value: Signal<DateRange>,
    pub start_value: Signal<Option<CalendarDate>>,
    pub end_value: Signal<Option<CalendarDate>>,
    pub last_pressed: Signal<Option<CalendarDate>>,
    pub focused_value: Signal<Option<CalendarDate>>,
    pub placeholder: Signal<CalendarDate>,
    pub months: Signal<Vec<CalendarMonth>>,
    pub announcement: Signal<String>,
    pub today_date: Signal<CalendarDate>,
    pub initial_placeholder_year: i32,
    pub is_date_disabled_cb: Rc<RefCell<Option<Callback<CalendarDate, bool>>>>,
    pub is_date_unavailable_cb: Rc<RefCell<Option<Callback<CalendarDate, bool>>>>,
    pub on_value_change: Rc<RefCell<Option<EventHandler<DateRange>>>>,
    pub on_start_value_change: Rc<RefCell<Option<EventHandler<Option<CalendarDate>>>>>,
    pub on_end_value_change: Rc<RefCell<Option<EventHandler<Option<CalendarDate>>>>>,
    pub on_placeholder_change: Rc<RefCell<Option<EventHandler<CalendarDate>>>>,
    pub on_range_select: Rc<RefCell<Option<EventHandler<DateRange>>>>,
    pub day_id_overrides: Rc<RefCell<HashMap<(usize, CalendarDate), String>>>,
    pub(crate) pending_focus_target: Rc<RefCell<Option<(usize, CalendarDate)>>>,
    pub(crate) pending_focus_rev: Signal<u64>,
    pub(crate) last_view_config: Rc<RefCell<CalendarViewConfig>>,
}

impl RangeCalendarRuntime {
    pub fn current_placeholder(&self) -> CalendarDate {
        (self.placeholder)()
    }

    pub fn current_months(&self) -> Vec<CalendarMonth> {
        (self.months)()
    }

    pub fn month_index_for(&self, month: CalendarDate) -> usize {
        self.months
            .read()
            .iter()
            .position(|m| m.value.is_same_month(month))
            .unwrap_or(0)
    }

    pub fn register_day_id(&self, month_idx: usize, date: CalendarDate, id: String) {
        self.day_id_overrides
            .borrow_mut()
            .insert((month_idx, date), id);
    }

    pub fn unregister_day_id(&self, month_idx: usize, date: CalendarDate) {
        self.day_id_overrides.borrow_mut().remove(&(month_idx, date));
    }

    pub fn resolve_day_id(&self, month_idx: usize, date: CalendarDate) -> String {
        self.day_id_overrides
            .borrow()
            .get(&(month_idx, date))
            .cloned()
            .unwrap_or_else(|| self.rels.day_id(month_idx, date))
    }

    pub fn is_date_disabled(&self, date: CalendarDate) -> bool {
        let def = self.def.read();
        let custom = self
            .is_date_disabled_cb
            .borrow()
            .as_ref()
            .is_some_and(|cb| cb.call(date));
        def.is_date_disabled_with(date, custom)
    }

    pub fn is_date_unavailable(&self, date: CalendarDate) -> bool {
        self.is_date_unavailable_cb
            .borrow()
            .as_ref()
            .is_some_and(|cb| cb.call(date))
    }

    pub fn is_outside_visible_months(&self, date: CalendarDate) -> bool {
        !self
            .months
            .read()
            .iter()
            .any(|month| month.value.is_same_month(date))
    }

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

    pub fn tabbable_date(&self) -> CalendarDate {
        let placeholder = self.current_placeholder();
        let months = self.current_months();
        resolve_tabbable_date(placeholder, &months, |d| self.is_date_disabled(d))
    }

    pub fn ensure_valid_initial_placeholder(&mut self) {
        let placeholder = *self.placeholder.peek();
        let months = self.months.peek().clone();
        let next = ensure_non_disabled_placeholder(placeholder, &months, |d| {
            self.is_date_disabled(d)
        });
        if next != placeholder {
            self.set_placeholder_internal(next, false);
        }
    }

    pub fn heading_value(&self) -> String {
        let def = self.def.read();
        let month_dates: Vec<CalendarDate> = self.months.read().iter().map(|m| m.value).collect();
        def.locale
            .heading_label(&month_dates, def.month_format, def.year_format)
    }

    pub fn full_calendar_label(&self) -> String {
        let label = self.def.read().calendar_label.clone();
        format!("{label} {}", self.heading_value())
    }

    pub fn weekdays(&self) -> Vec<String> {
        let def = self.def.read();
        let months = self.months.read();
        if let Some(first) = months.first() {
            visible_weekday_headers(first, &def.locale, def.weekday_format)
        } else {
            Vec::new()
        }
    }

    pub fn default_years(&self) -> Vec<i32> {
        let def = self.def.read();
        let today = *self.today_date.read();
        default_year_range(
            self.initial_placeholder_year,
            today.year,
            def.min_value,
            def.max_value,
        )
    }

    pub fn is_prev_disabled(&self) -> bool {
        let def = self.def.read();
        let months = self.months.read();
        is_prev_page_disabled(&months, def.min_value, def.disabled)
    }

    pub fn is_next_disabled(&self) -> bool {
        let def = self.def.read();
        let months = self.months.read();
        is_next_page_disabled(&months, def.max_value, def.disabled)
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

    pub fn rebuild_months_from(&mut self, start_month: CalendarDate) {
        let def = self.def.read().clone();
        let next = build_visible_months(
            start_month,
            def.number_of_months,
            def.effective_week_starts_on(),
            def.fixed_weeks,
        );
        self.months.set(next);
    }

    pub fn set_placeholder(&mut self, next: CalendarDate) {
        self.set_placeholder_internal(next, true);
    }

    fn set_placeholder_internal(&mut self, next: CalendarDate, rebuild_if_outside: bool) {
        let prev = *self.placeholder.peek();
        if prev != next {
            self.placeholder.set(next);
            if let Some(handler) = *self.on_placeholder_change.borrow() {
                handler.call(next);
            }
        }
        if rebuild_if_outside {
            let in_view = self
                .months
                .peek()
                .iter()
                .any(|m| m.value.is_same_month(next));
            if !in_view {
                self.rebuild_months_from(next);
            }
        }
    }

    pub fn prev_page(&mut self) {
        if self.is_prev_disabled() {
            return;
        }
        let def = self.def.read().clone();
        let step = if def.paged_navigation {
            def.number_of_months.max(1) as i32
        } else {
            1
        };
        let first = self
            .months
            .peek()
            .first()
            .map_or(*self.placeholder.peek(), |m| m.value);
        let new_first = first.add_months_clamped(-step);
        self.rebuild_months_from(new_first);
        self.set_placeholder_internal(new_first, false);
    }

    pub fn next_page(&mut self) {
        if self.is_next_disabled() {
            return;
        }
        let def = self.def.read().clone();
        let step = if def.paged_navigation {
            def.number_of_months.max(1) as i32
        } else {
            1
        };
        let first = self
            .months
            .peek()
            .first()
            .map_or(*self.placeholder.peek(), |m| m.value);
        let new_first = first.add_months_clamped(step);
        self.rebuild_months_from(new_first);
        self.set_placeholder_internal(new_first, false);
    }

    pub fn set_month(&mut self, month: u8) {
        if self.def.read().disabled {
            return;
        }
        let next = self.placeholder.peek().with_month_clamped(month);
        self.set_placeholder(next);
    }

    pub fn set_year(&mut self, year: i32) {
        if self.def.read().disabled {
            return;
        }
        let next = self.placeholder.peek().with_year_clamped(year);
        self.set_placeholder(next);
    }

    pub fn set_focused_value(&mut self, date: Option<CalendarDate>) {
        if *self.focused_value.peek() != date {
            self.focused_value.set(date);
        }
    }

    fn commit_update(&mut self, update: RangeCalendarUpdate) {
        let prev_start = *self.start_value.peek();
        let prev_end = *self.end_value.peek();
        let prev_val = *self.value.peek();

        if *self.last_pressed.peek() != update.last_pressed {
            self.last_pressed.set(update.last_pressed);
        }
        if prev_start != update.start_value {
            self.start_value.set(update.start_value);
            if let Some(handler) = *self.on_start_value_change.borrow() {
                handler.call(update.start_value);
            }
        }
        if prev_end != update.end_value {
            self.end_value.set(update.end_value);
            if let Some(handler) = *self.on_end_value_change.borrow() {
                handler.call(update.end_value);
            }
        }
        if prev_val != update.value {
            self.value.set(update.value);
            if let Some(handler) = *self.on_value_change.borrow() {
                handler.call(update.value);
            }
        }
        if let Some(next_ph) = update.placeholder {
            self.set_placeholder(next_ph);
        }
        if let Some(text) = update.announcement {
            self.announcement.set(text);
        }
        if let Some(completed) = update.range_completed
            && let Some(handler) = *self.on_range_select.borrow()
        {
            handler.call(completed);
        }
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
        let def = self.def.read().clone();
        let months = self.months.peek().clone();
        let result = resolve_grid_navigation(
            current_date,
            &months,
            def.number_of_months,
            def.effective_week_starts_on(),
            def.dir,
            key,
            def.min_value,
            def.max_value,
            def.disabled,
            def.disable_days_outside_month,
            |d| self.is_date_disabled(d),
        );
        let Some(nav) = result else {
            return false;
        };
        if nav.paged
            && let Some(view_start) = nav.next_view_start
        {
            self.rebuild_months_from(view_start);
        }
        self.set_placeholder_internal(nav.next_placeholder, !nav.paged);
        self.set_focused_value(Some(nav.target_date));

        let target_month_idx = self.month_index_for(nav.target_date);
        if nav.paged {
            *self.pending_focus_target.borrow_mut() = Some((target_month_idx, nav.target_date));
            let next_rev = self.pending_focus_rev.peek().wrapping_add(1);
            self.pending_focus_rev.set(next_rev);
        } else {
            let target_id = self.resolve_day_id(target_month_idx, nav.target_date);
            focus_element_sync_or_defer(&target_id, FocusOptions::new(true));
        }
        true
    }

    pub fn root_attrs(&self, custom_id: Option<String>) -> CalendarRootAttrs {
        let def = self.def.read();
        let id = custom_id.unwrap_or_else(|| (self.root_dom_id)());
        CalendarRootAttrs::new(
            id,
            CalendarKind::RangeCalendar,
            self.full_calendar_label(),
            def.dir,
            self.is_invalid(),
            def.disabled,
            def.read_only,
        )
    }

    pub fn cell_attrs(
        &self,
        date: CalendarDate,
        month: CalendarDate,
        custom_id: Option<String>,
    ) -> RangeCalendarCellAttrs {
        let def = self.def.read();
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
            def.disable_days_outside_month,
        )
    }

    pub fn day_attrs(
        &self,
        date: CalendarDate,
        month: CalendarDate,
        cell_attrs: &RangeCalendarCellAttrs,
        custom_id: Option<String>,
    ) -> RangeCalendarDayAttrs {
        let def = self.def.read();
        let month_idx = self.month_index_for(month);
        let id = custom_id.unwrap_or_else(|| self.rels.day_id(month_idx, date));
        let is_tabbable = date == self.tabbable_date();
        RangeCalendarDayAttrs::from_cell(id, date, &def.locale, cell_attrs, is_tabbable)
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
        self.runtime.current_placeholder().month
    }

    pub fn current_year(&self) -> i32 {
        self.runtime.current_placeholder().year
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
        let locale = &self.runtime.def.read().locale;
        build_calendar_month_options(None, locale, format)
    }

    pub fn year_options(&self, format: YearFormat) -> Vec<CalendarSelectOption> {
        let years = self.runtime.default_years();
        let locale = &self.runtime.def.read().locale;
        build_calendar_year_options(&years, locale, format)
    }
}

pub fn use_range_calendar_context() -> RangeCalendarContext {
    use_context::<RangeCalendarContext>()
}

pub fn try_use_range_calendar_context() -> Option<RangeCalendarContext> {
    try_consume_context::<RangeCalendarContext>()
}

pub fn default_range_today() -> CalendarDate {
    local_today()
}

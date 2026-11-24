use std::{cell::RefCell, collections::HashMap, rc::Rc};

use dioxus::prelude::keyboard_types::{Key, NamedKey};
use dioxus::prelude::*;

use crate::foundation::{
    browser::{FocusOptions, focus_element_sync_or_defer, local_today_ymd},
    date_time::{
        CalendarDate, CalendarLocale, CalendarMonth, CalendarNavKey, MonthFormat, YearFormat,
        build_visible_months, default_year_range, ensure_non_disabled_placeholder,
        is_next_page_disabled, is_prev_page_disabled, resolve_grid_navigation,
        resolve_tabbable_date, visible_weekday_headers,
    },
};

use super::{
    attrs::{
        CalendarCellAttrs, CalendarDayAttrs, CalendarGridAttrs, CalendarHeaderAttrs,
        CalendarHeadingAttrs, CalendarNavButtonAttrs, CalendarRootAttrs, CalendarSelectAttrs,
        CalendarSubpartAttrs,
    },
    relationships::CalendarRelationships,
    state::{CalendarDefinition, apply_multiple_click, apply_single_click},
    types::{
        CalendarCellSnapshot, CalendarKind, CalendarMode, CalendarPart, CalendarSelectOption,
        CalendarSnapshot, build_calendar_month_options, build_calendar_year_options,
    },
};

/// Returns the host local calendar date via `oxidase::time::local_today_ymd()`.
pub fn local_today() -> CalendarDate {
    let (year, month, day) = local_today_ymd();
    CalendarDate::from_ymd_clamped(year, month, day)
}

/// Translates Dioxus `keyboard_types::Key` + Shift modifier into host-neutral [`CalendarNavKey`].
pub fn translate_calendar_nav_key(key: &Key, shift: bool) -> Option<CalendarNavKey> {
    match key {
        Key::Named(NamedKey::ArrowLeft) => Some(CalendarNavKey::Left),
        Key::Named(NamedKey::ArrowRight) => Some(CalendarNavKey::Right),
        Key::Named(NamedKey::ArrowUp) => Some(CalendarNavKey::Up),
        Key::Named(NamedKey::ArrowDown) => Some(CalendarNavKey::Down),
        Key::Named(NamedKey::PageUp) => Some(if shift {
            CalendarNavKey::ShiftPageUp
        } else {
            CalendarNavKey::PageUp
        }),
        Key::Named(NamedKey::PageDown) => Some(if shift {
            CalendarNavKey::ShiftPageDown
        } else {
            CalendarNavKey::PageDown
        }),
        Key::Named(NamedKey::Home) => Some(CalendarNavKey::Home),
        Key::Named(NamedKey::End) => Some(CalendarNavKey::End),
        _ => None,
    }
}

/// Returns `true` when `key` activates a focused calendar day cell (`Enter` or `Space`).
pub fn is_calendar_activate_key(key: &Key) -> bool {
    matches!(key, Key::Named(NamedKey::Enter))
        || matches!(key, Key::Character(ch) if ch == " ")
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub(crate) struct CalendarViewConfig {
    pub week_starts_on: crate::foundation::date_time::Weekday,
    pub fixed_weeks: bool,
    pub number_of_months: usize,
}

/// Reactive runtime for `Calendar`.
#[derive(Clone)]
pub struct CalendarRuntime {
    pub def: Signal<CalendarDefinition>,
    pub rels: CalendarRelationships,
    pub root_dom_id: Signal<String>,
    pub value: Signal<Option<CalendarDate>>,
    pub values: Signal<Vec<CalendarDate>>,
    pub placeholder: Signal<CalendarDate>,
    pub months: Signal<Vec<CalendarMonth>>,
    pub announcement: Signal<String>,
    pub today_date: Signal<CalendarDate>,
    pub initial_placeholder_year: i32,
    pub is_date_disabled_cb: Rc<RefCell<Option<Callback<CalendarDate, bool>>>>,
    pub is_date_unavailable_cb: Rc<RefCell<Option<Callback<CalendarDate, bool>>>>,
    pub on_value_change: Rc<RefCell<Option<EventHandler<Option<CalendarDate>>>>>,
    pub on_values_change: Rc<RefCell<Option<EventHandler<Vec<CalendarDate>>>>>,
    pub on_placeholder_change: Rc<RefCell<Option<EventHandler<CalendarDate>>>>,
    pub on_date_select: Rc<RefCell<Option<EventHandler<CalendarDate>>>>,
    pub day_id_overrides: Rc<RefCell<HashMap<(usize, CalendarDate), String>>>,
    pub(crate) pending_focus_target: Rc<RefCell<Option<(usize, CalendarDate)>>>,
    pub(crate) pending_focus_rev: Signal<u64>,
    pub(crate) last_view_config: Rc<RefCell<CalendarViewConfig>>,
}

impl CalendarRuntime {
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
        let mode = self.def.read().mode;
        match mode {
            CalendarMode::Single => self.value.read().is_some_and(|d| d == date),
            CalendarMode::Multiple => self.values.read().contains(&date),
        }
    }

    pub fn is_invalid(&self) -> bool {
        let mode = self.def.read().mode;
        match mode {
            CalendarMode::Single => self
                .value
                .read()
                .is_some_and(|d| self.is_date_disabled(d) || self.is_date_unavailable(d)),
            CalendarMode::Multiple => self
                .values
                .read()
                .iter()
                .any(|d| self.is_date_disabled(*d) || self.is_date_unavailable(*d)),
        }
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

    pub fn snapshot(&self) -> CalendarSnapshot {
        CalendarSnapshot {
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

    pub fn activate_date(&mut self, date: CalendarDate) {
        let def = self.def.read().clone();
        let is_disabled = self.is_date_disabled(date);
        let is_unavailable = self.is_date_unavailable(date);

        match def.mode {
            CalendarMode::Single => {
                let prev = *self.value.peek();
                let Some(update) = apply_single_click(
                    prev,
                    date,
                    def.prevent_deselect,
                    def.read_only,
                    is_disabled,
                    is_unavailable,
                    &def.locale,
                ) else {
                    return;
                };
                if *self.value.peek() != update.next_value {
                    self.value.set(update.next_value);
                    if let Some(handler) = *self.on_value_change.borrow() {
                        handler.call(update.next_value);
                    }
                }
                if let Some(next_ph) = update.next_placeholder {
                    self.set_placeholder(next_ph);
                }
                self.announcement.set(update.announcement);
                if let Some(selected) = update.date_selected
                    && let Some(handler) = *self.on_date_select.borrow()
                {
                    handler.call(selected);
                }
            }
            CalendarMode::Multiple => {
                let prev = self.values.peek().clone();
                let Some(update) = apply_multiple_click(
                    &prev,
                    date,
                    def.prevent_deselect,
                    def.max_days,
                    def.read_only,
                    is_disabled,
                    is_unavailable,
                ) else {
                    return;
                };
                if *self.values.peek() != update.next_values {
                    self.values.set(update.next_values.clone());
                    if let Some(handler) = *self.on_values_change.borrow() {
                        handler.call(update.next_values);
                    }
                }
                if let Some(next_ph) = update.next_placeholder {
                    self.set_placeholder(next_ph);
                }
            }
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
            CalendarKind::Calendar,
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
    ) -> CalendarCellAttrs {
        let def = self.def.read();
        let month_idx = self.month_index_for(month);
        let id = custom_id.unwrap_or_else(|| self.rels.cell_id(month_idx, date));
        let is_outside_month = !date.is_same_month(month);
        CalendarCellAttrs::new(
            id,
            date,
            self.is_date_selected(date),
            self.is_date_disabled(date),
            self.is_date_unavailable(date),
            date == *self.today_date.read(),
            is_outside_month,
            self.is_outside_visible_months(date),
            date == self.current_placeholder(),
            def.disable_days_outside_month,
        )
    }

    pub fn day_attrs(
        &self,
        date: CalendarDate,
        month: CalendarDate,
        cell_attrs: &CalendarCellAttrs,
        custom_id: Option<String>,
    ) -> CalendarDayAttrs {
        let def = self.def.read();
        let month_idx = self.month_index_for(month);
        let id = custom_id.unwrap_or_else(|| self.rels.day_id(month_idx, date));
        let is_tabbable = date == self.tabbable_date();
        CalendarDayAttrs::from_cell(id, date, &def.locale, cell_attrs, is_tabbable)
    }

    pub fn cell_snapshot(
        &self,
        date: CalendarDate,
        month: CalendarDate,
        cell_attrs: &CalendarCellAttrs,
    ) -> CalendarCellSnapshot {
        CalendarCellSnapshot {
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
        }
    }
}

/// Shared runtime abstraction consumed by the 11 parts shared between `Calendar` and `RangeCalendar`.
#[derive(Clone)]
pub enum CalendarSharedRuntime {
    Calendar(CalendarRuntime),
    Range(crate::range_calendar::runtime::RangeCalendarRuntime),
}

impl CalendarSharedRuntime {
    pub fn kind(&self) -> CalendarKind {
        match self {
            Self::Calendar(_) => CalendarKind::Calendar,
            Self::Range(_) => CalendarKind::RangeCalendar,
        }
    }

    pub fn disabled(&self) -> bool {
        match self {
            Self::Calendar(rt) => rt.def.read().disabled,
            Self::Range(rt) => rt.def.read().disabled,
        }
    }

    pub fn read_only(&self) -> bool {
        match self {
            Self::Calendar(rt) => rt.def.read().read_only,
            Self::Range(rt) => rt.def.read().read_only,
        }
    }

    pub fn heading_value(&self) -> String {
        match self {
            Self::Calendar(rt) => rt.heading_value(),
            Self::Range(rt) => rt.heading_value(),
        }
    }

    pub fn is_prev_disabled(&self) -> bool {
        match self {
            Self::Calendar(rt) => rt.is_prev_disabled(),
            Self::Range(rt) => rt.is_prev_disabled(),
        }
    }

    pub fn is_next_disabled(&self) -> bool {
        match self {
            Self::Calendar(rt) => rt.is_next_disabled(),
            Self::Range(rt) => rt.is_next_disabled(),
        }
    }

    pub fn prev_page(&mut self) {
        match self {
            Self::Calendar(rt) => rt.prev_page(),
            Self::Range(rt) => rt.prev_page(),
        }
    }

    pub fn next_page(&mut self) {
        match self {
            Self::Calendar(rt) => rt.next_page(),
            Self::Range(rt) => rt.next_page(),
        }
    }

    pub fn set_month(&mut self, month: u8) {
        match self {
            Self::Calendar(rt) => rt.set_month(month),
            Self::Range(rt) => rt.set_month(month),
        }
    }

    pub fn set_year(&mut self, year: i32) {
        match self {
            Self::Calendar(rt) => rt.set_year(year),
            Self::Range(rt) => rt.set_year(year),
        }
    }

    pub fn current_month(&self) -> u8 {
        match self {
            Self::Calendar(rt) => rt.current_placeholder().month,
            Self::Range(rt) => rt.current_placeholder().month,
        }
    }

    pub fn current_year(&self) -> i32 {
        match self {
            Self::Calendar(rt) => rt.current_placeholder().year,
            Self::Range(rt) => rt.current_placeholder().year,
        }
    }

    pub fn locale(&self) -> CalendarLocale {
        match self {
            Self::Calendar(rt) => rt.def.read().locale.clone(),
            Self::Range(rt) => rt.def.read().locale.clone(),
        }
    }

    pub fn default_years(&self) -> Vec<i32> {
        match self {
            Self::Calendar(rt) => rt.default_years(),
            Self::Range(rt) => rt.default_years(),
        }
    }

    pub fn header_attrs(&self, custom_id: Option<String>) -> CalendarHeaderAttrs {
        let id = custom_id.unwrap_or_else(|| match self {
            Self::Calendar(rt) => rt.rels.header_id().to_string(),
            Self::Range(rt) => rt.rels.header_id().to_string(),
        });
        CalendarHeaderAttrs::new(id, self.kind(), self.disabled(), self.read_only())
    }

    pub fn heading_attrs(&self, custom_id: Option<String>) -> CalendarHeadingAttrs {
        let id = custom_id.unwrap_or_else(|| match self {
            Self::Calendar(rt) => rt.rels.heading_id().to_string(),
            Self::Range(rt) => rt.rels.heading_id().to_string(),
        });
        CalendarHeadingAttrs::new(id, self.kind(), self.disabled(), self.read_only())
    }

    pub fn prev_button_attrs(&self, custom_id: Option<String>) -> CalendarNavButtonAttrs {
        let id = custom_id.unwrap_or_else(|| match self {
            Self::Calendar(rt) => rt.rels.prev_button_id().to_string(),
            Self::Range(rt) => rt.rels.prev_button_id().to_string(),
        });
        CalendarNavButtonAttrs::prev(id, self.kind(), self.is_prev_disabled())
    }

    pub fn next_button_attrs(&self, custom_id: Option<String>) -> CalendarNavButtonAttrs {
        let id = custom_id.unwrap_or_else(|| match self {
            Self::Calendar(rt) => rt.rels.next_button_id().to_string(),
            Self::Range(rt) => rt.rels.next_button_id().to_string(),
        });
        CalendarNavButtonAttrs::next(id, self.kind(), self.is_next_disabled())
    }

    pub fn grid_attrs(&self, month_idx: usize, custom_id: Option<String>) -> CalendarGridAttrs {
        let id = custom_id.unwrap_or_else(|| match self {
            Self::Calendar(rt) => rt.rels.grid_id(month_idx),
            Self::Range(rt) => rt.rels.grid_id(month_idx),
        });
        CalendarGridAttrs::new(id, self.kind(), self.disabled(), self.read_only())
    }

    pub fn subpart_attrs(
        &self,
        part: CalendarPart,
        default_id: String,
        custom_id: Option<String>,
    ) -> CalendarSubpartAttrs {
        CalendarSubpartAttrs::new(
            custom_id.unwrap_or(default_id),
            self.kind(),
            part,
            self.disabled(),
            self.read_only(),
        )
    }

    pub fn month_select_attrs(
        &self,
        custom_id: Option<String>,
        aria_label: Option<String>,
        disabled: bool,
    ) -> CalendarSelectAttrs {
        let id = custom_id.unwrap_or_else(|| match self {
            Self::Calendar(rt) => rt.rels.month_select_id().to_string(),
            Self::Range(rt) => rt.rels.month_select_id().to_string(),
        });
        CalendarSelectAttrs::month(id, self.kind(), aria_label, self.disabled() || disabled)
    }

    pub fn year_select_attrs(
        &self,
        custom_id: Option<String>,
        aria_label: Option<String>,
        disabled: bool,
    ) -> CalendarSelectAttrs {
        let id = custom_id.unwrap_or_else(|| match self {
            Self::Calendar(rt) => rt.rels.year_select_id().to_string(),
            Self::Range(rt) => rt.rels.year_select_id().to_string(),
        });
        CalendarSelectAttrs::year(id, self.kind(), aria_label, self.disabled() || disabled)
    }
}

#[derive(Clone)]
pub struct CalendarContext {
    pub runtime: CalendarRuntime,
}

impl CalendarContext {
    pub fn snapshot(&self) -> CalendarSnapshot {
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

pub fn use_calendar_context() -> CalendarContext {
    use_context::<CalendarContext>()
}

pub fn try_use_calendar_context() -> Option<CalendarContext> {
    try_consume_context::<CalendarContext>()
}

#[derive(Clone)]
pub struct CalendarSharedContext {
    pub runtime: CalendarSharedRuntime,
}

pub fn use_calendar_shared_context() -> CalendarSharedContext {
    use_context::<CalendarSharedContext>()
}

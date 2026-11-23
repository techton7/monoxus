use crate::foundation::{
    date_time::{
        CalendarDate, CalendarLocale, MonthFormat, Weekday, WeekdayFormat, YearFormat,
    },
    shared::{Direction, ScopeHandle},
};

use super::types::CalendarMode;

/// Immutable configuration descriptor for a `Calendar` instance.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct CalendarDefinition {
    pub scope: ScopeHandle,
    pub mode: CalendarMode,
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
    pub max_days: Option<usize>,
}

impl CalendarDefinition {
    pub fn new(scope: ScopeHandle) -> Self {
        Self {
            scope,
            mode: CalendarMode::Single,
            min_value: None,
            max_value: None,
            disabled: false,
            read_only: false,
            prevent_deselect: false,
            paged_navigation: false,
            fixed_weeks: false,
            number_of_months: 1,
            week_starts_on: None,
            weekday_format: WeekdayFormat::Narrow,
            month_format: MonthFormat::Long,
            year_format: YearFormat::Numeric,
            locale: CalendarLocale::EnUs,
            calendar_label: String::from("Event"),
            dir: Direction::Ltr,
            disable_days_outside_month: true,
            max_days: None,
        }
    }

    pub fn with_mode(mut self, mode: CalendarMode) -> Self {
        self.mode = mode;
        self
    }

    pub fn with_min_value(mut self, min_value: Option<CalendarDate>) -> Self {
        self.min_value = min_value;
        self
    }

    pub fn with_max_value(mut self, max_value: Option<CalendarDate>) -> Self {
        self.max_value = max_value;
        self
    }

    pub fn with_disabled(mut self, disabled: bool) -> Self {
        self.disabled = disabled;
        self
    }

    pub fn with_read_only(mut self, read_only: bool) -> Self {
        self.read_only = read_only;
        self
    }

    pub fn with_prevent_deselect(mut self, prevent_deselect: bool) -> Self {
        self.prevent_deselect = prevent_deselect;
        self
    }

    pub fn with_paged_navigation(mut self, paged_navigation: bool) -> Self {
        self.paged_navigation = paged_navigation;
        self
    }

    pub fn with_fixed_weeks(mut self, fixed_weeks: bool) -> Self {
        self.fixed_weeks = fixed_weeks;
        self
    }

    pub fn with_number_of_months(mut self, number_of_months: usize) -> Self {
        self.number_of_months = number_of_months.max(1);
        self
    }

    pub fn with_week_starts_on(mut self, week_starts_on: Option<Weekday>) -> Self {
        self.week_starts_on = week_starts_on;
        self
    }

    pub fn with_weekday_format(mut self, weekday_format: WeekdayFormat) -> Self {
        self.weekday_format = weekday_format;
        self
    }

    pub fn with_month_format(mut self, month_format: MonthFormat) -> Self {
        self.month_format = month_format;
        self
    }

    pub fn with_year_format(mut self, year_format: YearFormat) -> Self {
        self.year_format = year_format;
        self
    }

    pub fn with_locale(mut self, locale: CalendarLocale) -> Self {
        self.locale = locale;
        self
    }

    pub fn with_calendar_label(mut self, calendar_label: impl Into<String>) -> Self {
        self.calendar_label = calendar_label.into();
        self
    }

    pub fn with_direction(mut self, dir: Direction) -> Self {
        self.dir = dir;
        self
    }

    pub fn with_disable_days_outside_month(mut self, disable_days_outside_month: bool) -> Self {
        self.disable_days_outside_month = disable_days_outside_month;
        self
    }

    pub fn with_max_days(mut self, max_days: Option<usize>) -> Self {
        self.max_days = max_days;
        self
    }

    /// Resolves the effective first day of the week (explicit `week_starts_on` or locale default).
    pub fn effective_week_starts_on(&self) -> Weekday {
        self.week_starts_on
            .unwrap_or_else(|| self.locale.default_week_starts_on())
    }

    /// Checks whether `date` is disabled by `root.disabled`, `[min_value, max_value]`, or custom matcher.
    pub fn is_date_disabled_with(
        &self,
        date: CalendarDate,
        custom_disabled: bool,
    ) -> bool {
        if self.disabled || custom_disabled {
            return true;
        }
        if self.min_value.is_some_and(|min| date < min) {
            return true;
        }
        if self.max_value.is_some_and(|max| date > max) {
            return true;
        }
        false
    }
}

/// Output of activating a day cell in [`CalendarMode::Single`].
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct CalendarSingleUpdate {
    pub next_value: Option<CalendarDate>,
    pub next_placeholder: Option<CalendarDate>,
    pub announcement: String,
    pub date_selected: Option<CalendarDate>,
}

/// Pure reducer for clicking or keyboard-activating a day in [`CalendarMode::Single`].
pub fn apply_single_click(
    prev: Option<CalendarDate>,
    clicked: CalendarDate,
    prevent_deselect: bool,
    read_only: bool,
    is_disabled: bool,
    is_unavailable: bool,
    locale: &CalendarLocale,
) -> Option<CalendarSingleUpdate> {
    if read_only || is_disabled || is_unavailable {
        return None;
    }

    if prev == Some(clicked) && !prevent_deselect {
        return Some(CalendarSingleUpdate {
            next_value: None,
            next_placeholder: Some(clicked),
            announcement: locale.empty_announcement(),
            date_selected: None,
        });
    }

    Some(CalendarSingleUpdate {
        next_value: Some(clicked),
        next_placeholder: Some(clicked),
        announcement: locale.selected_announcement(clicked),
        date_selected: Some(clicked),
    })
}

/// Output of activating a day cell in [`CalendarMode::Multiple`].
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct CalendarMultipleUpdate {
    pub next_values: Vec<CalendarDate>,
    pub next_placeholder: Option<CalendarDate>,
}

/// Pure reducer for clicking or keyboard-activating a day in [`CalendarMode::Multiple`].
pub fn apply_multiple_click(
    prev: &[CalendarDate],
    clicked: CalendarDate,
    prevent_deselect: bool,
    max_days: Option<usize>,
    read_only: bool,
    is_disabled: bool,
    is_unavailable: bool,
) -> Option<CalendarMultipleUpdate> {
    if read_only || is_disabled || is_unavailable {
        return None;
    }

    if let Some(pos) = prev.iter().position(|d| *d == clicked) {
        if prevent_deselect {
            return Some(CalendarMultipleUpdate {
                next_values: prev.to_vec(),
                next_placeholder: None,
            });
        }
        let mut next = prev.to_vec();
        next.remove(pos);
        let next_placeholder = if next.is_empty() {
            Some(clicked)
        } else {
            next.last().copied()
        };
        return Some(CalendarMultipleUpdate {
            next_values: next,
            next_placeholder,
        });
    }

    let mut next = prev.to_vec();
    next.push(clicked);
    if max_days.is_some_and(|max| next.len() > max) {
        next = vec![clicked];
    }
    let next_placeholder = next.last().copied();
    Some(CalendarMultipleUpdate {
        next_values: next,
        next_placeholder,
    })
}

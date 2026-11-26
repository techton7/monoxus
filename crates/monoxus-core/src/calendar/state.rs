use std::ops::Deref;

use crate::foundation::{
    date_time::{
        CalendarConfig, CalendarDate, CalendarLocale, MonthFormat, Weekday, WeekdayFormat,
        YearFormat,
    },
    shared::{Direction, ScopeHandle},
};

use super::types::CalendarMode;

/// Immutable configuration descriptor for a `Calendar` instance.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct CalendarDefinition {
    pub scope: ScopeHandle,
    pub mode: CalendarMode,
    pub config: CalendarConfig,
    pub max_days: Option<usize>,
}

impl Deref for CalendarDefinition {
    type Target = CalendarConfig;

    #[inline]
    fn deref(&self) -> &Self::Target {
        &self.config
    }
}

impl CalendarDefinition {
    pub fn new(scope: ScopeHandle) -> Self {
        Self {
            scope,
            mode: CalendarMode::Single,
            config: CalendarConfig::default(),
            max_days: None,
        }
    }

    pub fn with_mode(mut self, mode: CalendarMode) -> Self {
        self.mode = mode;
        self
    }

    pub fn with_min_value(mut self, min_value: Option<CalendarDate>) -> Self {
        self.config = self.config.with_min_value(min_value);
        self
    }

    pub fn with_max_value(mut self, max_value: Option<CalendarDate>) -> Self {
        self.config = self.config.with_max_value(max_value);
        self
    }

    pub fn with_disabled(mut self, disabled: bool) -> Self {
        self.config = self.config.with_disabled(disabled);
        self
    }

    pub fn with_read_only(mut self, read_only: bool) -> Self {
        self.config = self.config.with_read_only(read_only);
        self
    }

    pub fn with_prevent_deselect(mut self, prevent_deselect: bool) -> Self {
        self.config = self.config.with_prevent_deselect(prevent_deselect);
        self
    }

    pub fn with_paged_navigation(mut self, paged_navigation: bool) -> Self {
        self.config = self.config.with_paged_navigation(paged_navigation);
        self
    }

    pub fn with_fixed_weeks(mut self, fixed_weeks: bool) -> Self {
        self.config = self.config.with_fixed_weeks(fixed_weeks);
        self
    }

    pub fn with_number_of_months(mut self, number_of_months: usize) -> Self {
        self.config = self.config.with_number_of_months(number_of_months);
        self
    }

    pub fn with_week_starts_on(mut self, week_starts_on: Option<Weekday>) -> Self {
        self.config = self.config.with_week_starts_on(week_starts_on);
        self
    }

    pub fn with_weekday_format(mut self, weekday_format: WeekdayFormat) -> Self {
        self.config = self.config.with_weekday_format(weekday_format);
        self
    }

    pub fn with_month_format(mut self, month_format: MonthFormat) -> Self {
        self.config = self.config.with_month_format(month_format);
        self
    }

    pub fn with_year_format(mut self, year_format: YearFormat) -> Self {
        self.config = self.config.with_year_format(year_format);
        self
    }

    pub fn with_locale(mut self, locale: CalendarLocale) -> Self {
        self.config = self.config.with_locale(locale);
        self
    }

    pub fn with_calendar_label(mut self, calendar_label: impl Into<String>) -> Self {
        self.config = self.config.with_calendar_label(calendar_label);
        self
    }

    pub fn with_direction(mut self, dir: Direction) -> Self {
        self.config = self.config.with_direction(dir);
        self
    }

    pub fn with_disable_days_outside_month(mut self, disable_days_outside_month: bool) -> Self {
        self.config = self
            .config
            .with_disable_days_outside_month(disable_days_outside_month);
        self
    }

    pub fn with_max_days(mut self, max_days: Option<usize>) -> Self {
        self.max_days = max_days;
        self
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

use crate::foundation::{
    date_time::{
        CalendarDate, CalendarLocale, DateRange, MonthFormat, Weekday, WeekdayFormat, YearFormat,
        compute_highlighted_range, has_disabled_dates_in_range, is_range_valid,
    },
    shared::{Direction, ScopeHandle},
};

/// Immutable configuration descriptor for a `RangeCalendar` instance.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct RangeCalendarDefinition {
    pub scope: ScopeHandle,
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
    pub min_days: Option<usize>,
    pub max_days: Option<usize>,
    pub exclude_disabled: bool,
}

impl RangeCalendarDefinition {
    pub fn new(scope: ScopeHandle) -> Self {
        Self {
            scope,
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
            min_days: None,
            max_days: None,
            exclude_disabled: false,
        }
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

    pub fn with_min_days(mut self, min_days: Option<usize>) -> Self {
        self.min_days = min_days;
        self
    }

    pub fn with_max_days(mut self, max_days: Option<usize>) -> Self {
        self.max_days = max_days;
        self
    }

    pub fn with_exclude_disabled(mut self, exclude_disabled: bool) -> Self {
        self.exclude_disabled = exclude_disabled;
        self
    }

    pub fn effective_week_starts_on(&self) -> Weekday {
        self.week_starts_on
            .unwrap_or_else(|| self.locale.default_week_starts_on())
    }

    pub fn is_date_disabled_with(&self, date: CalendarDate, custom_disabled: bool) -> bool {
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

/// Interactive range selection state tracked by `RangeCalendar`.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub struct RangeCalendarState {
    pub value: DateRange,
    pub start_value: Option<CalendarDate>,
    pub end_value: Option<CalendarDate>,
    pub last_pressed: Option<CalendarDate>,
    pub focused_value: Option<CalendarDate>,
}

impl RangeCalendarState {
    pub fn from_value(value: DateRange) -> Self {
        Self {
            value,
            start_value: value.start,
            end_value: value.end,
            last_pressed: None,
            focused_value: None,
        }
    }
}

/// Output of a `RangeCalendar` state transition.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct RangeCalendarUpdate {
    pub value: DateRange,
    pub start_value: Option<CalendarDate>,
    pub end_value: Option<CalendarDate>,
    pub placeholder: Option<CalendarDate>,
    pub last_pressed: Option<CalendarDate>,
    pub announcement: Option<String>,
    pub range_completed: Option<DateRange>,
}

/// Returns `true` when `date` falls within the current `start_value`/`end_value` selection.
pub fn is_date_selected_in_range(
    date: CalendarDate,
    start_value: Option<CalendarDate>,
    end_value: Option<CalendarDate>,
) -> bool {
    if start_value == Some(date) || end_value == Some(date) {
        return true;
    }
    if let (Some(start), Some(end)) = (start_value, end_value) {
        return date.is_between_inclusive(start, end);
    }
    false
}

/// Computes `is_invalid` for a `RangeCalendar` (either endpoint invalid/disabled/unavailable or `end < start`).
pub fn is_range_selection_invalid<FD, FU>(
    start_value: Option<CalendarDate>,
    end_value: Option<CalendarDate>,
    is_disabled: FD,
    is_unavailable: FU,
) -> bool
where
    FD: Fn(CalendarDate) -> bool,
    FU: Fn(CalendarDate) -> bool,
{
    if let Some(start) = start_value
        && (is_disabled(start) || is_unavailable(start))
    {
        return true;
    }
    if let Some(end) = end_value
        && (is_disabled(end) || is_unavailable(end))
    {
        return true;
    }
    if let (Some(start), Some(end)) = (start_value, end_value)
        && end < start
    {
        return true;
    }
    false
}

/// Pure reducer for clicking or keyboard-activating a day in `RangeCalendar`,
/// enforcing the `5B D-3` `read_only` guard and `bits-ui` range selection state machine.
#[allow(clippy::too_many_arguments)]
pub fn apply_range_click<FD, FU>(
    state: &RangeCalendarState,
    clicked: CalendarDate,
    read_only: bool,
    prevent_deselect: bool,
    is_cell_disabled: bool,
    is_cell_unavailable: bool,
    min_days: Option<usize>,
    max_days: Option<usize>,
    exclude_disabled: bool,
    locale: &CalendarLocale,
    is_date_disabled: FD,
    is_date_unavailable: FU,
) -> Option<RangeCalendarUpdate>
where
    FD: Fn(CalendarDate) -> bool,
    FU: Fn(CalendarDate) -> bool,
{
    if read_only || is_cell_disabled || is_cell_unavailable {
        return None;
    }

    let prev_last_pressed = state.last_pressed;
    let next_last_pressed = Some(clicked);
    let highlighted = compute_highlighted_range(
        state.start_value,
        state.end_value,
        state.focused_value,
        &is_date_disabled,
        &is_date_unavailable,
    );

    let mut cur_start = state.start_value;
    let mut cur_end = state.end_value;
    let mut cur_val = state.value;
    let mut announcement = None;

    if let Some(start) = cur_start
        && highlighted.is_none()
    {
        if start == clicked && !prevent_deselect && cur_end.is_none() {
            return Some(RangeCalendarUpdate {
                value: DateRange::default(),
                start_value: None,
                end_value: None,
                placeholder: Some(clicked),
                last_pressed: next_last_pressed,
                announcement: Some(locale.empty_announcement()),
                range_completed: None,
            });
        } else if cur_end.is_none() && prev_last_pressed == Some(clicked) {
            cur_start = Some(clicked);
            cur_val.start = Some(clicked);
            announcement = Some(locale.selected_announcement(clicked));
        }
    }

    if let (Some(_start), Some(end)) = (cur_start, cur_end)
        && end == clicked
        && !prevent_deselect
    {
        return Some(RangeCalendarUpdate {
            value: DateRange::default(),
            start_value: None,
            end_value: None,
            placeholder: Some(clicked),
            last_pressed: next_last_pressed,
            announcement: Some(locale.empty_announcement()),
            range_completed: None,
        });
    }

    let Some(start_date) = cur_start else {
        let next_val = DateRange {
            start: Some(clicked),
            end: cur_val.end,
        };
        return Some(RangeCalendarUpdate {
            value: next_val,
            start_value: Some(clicked),
            end_value: cur_end,
            placeholder: None,
            last_pressed: next_last_pressed,
            announcement: Some(locale.selected_announcement(clicked)),
            range_completed: next_val.is_complete().then_some(next_val),
        });
    };

    if cur_end.is_none() {
        let (ordered_start, ordered_end) = if clicked < start_date {
            (clicked, start_date)
        } else {
            (start_date, clicked)
        };

        if !is_range_valid(
            ordered_start,
            ordered_end,
            min_days,
            max_days,
            exclude_disabled,
            &is_date_disabled,
        ) {
            let next_val = DateRange {
                start: Some(clicked),
                end: None,
            };
            return Some(RangeCalendarUpdate {
                value: next_val,
                start_value: Some(clicked),
                end_value: None,
                placeholder: None,
                last_pressed: next_last_pressed,
                announcement: Some(locale.selected_announcement(clicked)),
                range_completed: None,
            });
        }

        let committed = DateRange {
            start: Some(ordered_start),
            end: Some(ordered_end),
        };
        return Some(RangeCalendarUpdate {
            value: committed,
            start_value: Some(ordered_start),
            end_value: Some(ordered_end),
            placeholder: None,
            last_pressed: next_last_pressed,
            announcement: Some(locale.range_announcement(ordered_start, ordered_end)),
            range_completed: Some(committed),
        });
    }

    // Both start and end were already selected: start a new range from `clicked`.
    let _ = cur_end.take();
    let next_val = DateRange {
        start: Some(clicked),
        end: None,
    };
    Some(RangeCalendarUpdate {
        value: next_val,
        start_value: Some(clicked),
        end_value: None,
        placeholder: None,
        last_pressed: next_last_pressed,
        announcement: Some(announcement.unwrap_or_else(|| locale.selected_announcement(clicked))),
        range_completed: None,
    })
}

/// Pure reducer for `Escape` in `RangeCalendar` (`5B D-1`):
/// clears a partial range (`start.is_some() && end.is_none()`) and announces empty.
pub fn apply_range_escape(
    state: &RangeCalendarState,
    read_only: bool,
    disabled: bool,
    locale: &CalendarLocale,
) -> Option<RangeCalendarUpdate> {
    if disabled || read_only {
        return None;
    }
    if state.start_value.is_some() && state.end_value.is_none() {
        return Some(RangeCalendarUpdate {
            value: DateRange::default(),
            start_value: None,
            end_value: None,
            placeholder: None,
            last_pressed: None,
            announcement: Some(locale.empty_announcement()),
            range_completed: None,
        });
    }
    None
}

/// Clears an active completed range when `exclude_disabled` is `true` and the range spans a disabled date.
pub fn reconcile_exclude_disabled<FD>(
    start_value: Option<CalendarDate>,
    end_value: Option<CalendarDate>,
    exclude_disabled: bool,
    locale: &CalendarLocale,
    is_date_disabled: FD,
) -> Option<RangeCalendarUpdate>
where
    FD: Fn(CalendarDate) -> bool,
{
    if !exclude_disabled {
        return None;
    }
    let (Some(start), Some(end)) = (start_value, end_value) else {
        return None;
    };
    if has_disabled_dates_in_range(start, end, is_date_disabled) {
        return Some(RangeCalendarUpdate {
            value: DateRange::default(),
            start_value: None,
            end_value: None,
            placeholder: None,
            last_pressed: None,
            announcement: Some(locale.empty_announcement()),
            range_completed: None,
        });
    }
    None
}

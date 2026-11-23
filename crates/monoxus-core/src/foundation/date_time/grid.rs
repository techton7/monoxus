use crate::foundation::shared::Direction;

use super::{
    date::{CalendarDate, Weekday},
    locale::{CalendarLocale, WeekdayFormat},
};

/// A single rendered month grid containing leading/trailing outside-month days grouped into 7-day weeks.
#[derive(Clone, Debug, PartialEq, Eq)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub struct CalendarMonth {
    /// Representative date within this month (matches `placeholder + month_offset`).
    pub value: CalendarDate,
    /// Flat list of every cell date in row-major order.
    pub dates: Vec<CalendarDate>,
    /// Cell dates chunked into 7-day rows (`weeks`).
    pub weeks: Vec<Vec<CalendarDate>>,
}

/// Builds a 2D month grid for `month` aligned to `week_starts_on`.
///
/// When `fixed_weeks` is `true`, pads trailing days up to 6 weeks (42 cells).
pub fn build_month_grid(
    month: CalendarDate,
    week_starts_on: Weekday,
    fixed_weeks: bool,
) -> CalendarMonth {
    let first_day_of_month = month.start_of_month();
    let last_day_of_month = month.end_of_month();
    let last_weekday_of_week = week_starts_on.add_days(6);

    let leading_days = first_day_of_month.weekday().offset_from(week_starts_on) as i64;
    let trailing_days = last_weekday_of_week.offset_from(last_day_of_month.weekday()) as i64;

    let grid_start = first_day_of_month.add_days(-leading_days);
    let grid_end = last_day_of_month.add_days(trailing_days);

    let mut total_days = (grid_end.days_since_epoch() - grid_start.days_since_epoch() + 1) as usize;
    if fixed_weeks && total_days < 42 {
        total_days = 42;
    }

    let dates: Vec<CalendarDate> = (0..total_days as i64)
        .map(|offset| grid_start.add_days(offset))
        .collect();
    let weeks: Vec<Vec<CalendarDate>> = dates.chunks(7).map(|chunk| chunk.to_vec()).collect();

    CalendarMonth {
        value: month,
        dates,
        weeks,
    }
}

/// Builds `number_of_months` consecutive month grids starting at `placeholder`.
pub fn build_visible_months(
    placeholder: CalendarDate,
    number_of_months: usize,
    week_starts_on: Weekday,
    fixed_weeks: bool,
) -> Vec<CalendarMonth> {
    let count = number_of_months.max(1);
    (0..count)
        .map(|offset| {
            let month_date = if offset == 0 {
                placeholder
            } else {
                placeholder.add_months_clamped(offset as i32)
            };
            build_month_grid(month_date, week_starts_on, fixed_weeks)
        })
        .collect()
}

/// Formats the 7 weekday column headers matching the column order of `first_month`.
pub fn visible_weekday_headers(
    first_month: &CalendarMonth,
    locale: &CalendarLocale,
    format: WeekdayFormat,
) -> Vec<String> {
    if let Some(first_week) = first_month.weeks.first()
        && first_week.len() == 7
    {
        return first_week
            .iter()
            .map(|date| locale.weekday_label(date.weekday(), format))
            .collect();
    }

    let start = locale.default_week_starts_on();
    (0..7)
        .map(|offset| locale.weekday_label(start.add_days(offset), format))
        .collect()
}

/// Computes the default selectable year list for `YearSelect`, honoring `min_value` and `max_value`
/// and matching `bits-ui`'s 111-year window (`[latest - 100 ..= latest + 10]`).
pub fn default_year_range(
    placeholder_year: i32,
    current_year: i32,
    min_value: Option<CalendarDate>,
    max_value: Option<CalendarDate>,
) -> Vec<i32> {
    let latest_year = placeholder_year.max(current_year);

    let min_year = if let Some(min) = min_value {
        min.year
    } else {
        let initial_min = latest_year - 100;
        if placeholder_year < initial_min {
            placeholder_year - 10
        } else {
            initial_min
        }
    };

    let max_year = max_value.map_or(latest_year + 10, |max| max.year);
    let clamped_min = min_year.min(max_year);

    (clamped_min..=max_year).collect()
}

/// Resolves which date in the visible months should hold `tabindex="0"`:
/// `placeholder` when enabled, otherwise the first enabled in-month date, or `placeholder` if none exist.
pub fn resolve_tabbable_date<F>(
    placeholder: CalendarDate,
    months: &[CalendarMonth],
    is_disabled: F,
) -> CalendarDate
where
    F: Fn(CalendarDate) -> bool,
{
    if !is_disabled(placeholder) {
        return placeholder;
    }
    for month in months {
        if let Some(found) = month
            .dates
            .iter()
            .copied()
            .find(|d| d.is_same_month(month.value) && !is_disabled(*d))
        {
            return found;
        }
    }
    placeholder
}

/// Ensures an initial placeholder is not stuck on a disabled date when an enabled date is visible.
pub fn ensure_non_disabled_placeholder<F>(
    placeholder: CalendarDate,
    months: &[CalendarMonth],
    is_disabled: F,
) -> CalendarDate
where
    F: Fn(CalendarDate) -> bool,
{
    resolve_tabbable_date(placeholder, months, is_disabled)
}

/// Returns `true` when paging backward would land entirely before `min_value` (or root is `disabled`).
pub fn is_prev_page_disabled(
    months: &[CalendarMonth],
    min_value: Option<CalendarDate>,
    disabled: bool,
) -> bool {
    let Some(min) = min_value else {
        return false;
    };
    if disabled {
        return true;
    }
    let Some(first_month) = months.first() else {
        return false;
    };
    let prev_month_end = first_month.value.add_months_clamped(-1).end_of_month();
    prev_month_end < min
}

/// Returns `true` when paging forward would land entirely after `max_value` (or root is `disabled`).
pub fn is_next_page_disabled(
    months: &[CalendarMonth],
    max_value: Option<CalendarDate>,
    disabled: bool,
) -> bool {
    let Some(max) = max_value else {
        return false;
    };
    if disabled {
        return true;
    }
    let Some(last_month) = months.last() else {
        return false;
    };
    let next_month_start = last_month.value.add_months_clamped(1).start_of_month();
    next_month_start > max
}

/// Host-neutral 2D calendar grid keyboard navigation intents (`5B D-1`).
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub enum CalendarNavKey {
    Left,
    Right,
    Up,
    Down,
    PageUp,
    PageDown,
    ShiftPageUp,
    ShiftPageDown,
    Home,
    End,
}

/// Result of resolving a 2D keyboard navigation step.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct CalendarNavResult {
    /// Target cell date that should receive DOM focus.
    pub target_date: CalendarDate,
    /// Updated `placeholder` date.
    pub next_placeholder: CalendarDate,
    /// New first visible month start when `paged == true` (to preserve multi-month alignment).
    pub next_view_start: Option<CalendarDate>,
    /// Whether the visible month window changed.
    pub paged: bool,
}

/// Computes the next focused cell and optional page transition for WAI-ARIA APG 2D calendar grid navigation (`5B D-1`).
#[allow(clippy::too_many_arguments)]
pub fn resolve_grid_navigation<F>(
    current_date: CalendarDate,
    months: &[CalendarMonth],
    number_of_months: usize,
    week_starts_on: Weekday,
    dir: Direction,
    key: CalendarNavKey,
    min_value: Option<CalendarDate>,
    max_value: Option<CalendarDate>,
    disabled: bool,
    disable_days_outside_month: bool,
    is_disabled: F,
) -> Option<CalendarNavResult>
where
    F: Fn(CalendarDate) -> bool,
{
    if disabled || months.is_empty() {
        return None;
    }

    let step_months = number_of_months.max(1) as i32;
    let first_visible_month = months.first()?.value.start_of_month();
    let prev_blocked = is_prev_page_disabled(months, min_value, disabled);
    let next_blocked = is_next_page_disabled(months, max_value, disabled);

    let is_in_visible_months = |date: CalendarDate| -> bool {
        months.iter().any(|month| date.is_same_month(month.value))
    };

    let paged_view_start_for = |target: CalendarDate, forward: bool| -> CalendarDate {
        let mut view_start = first_visible_month;
        let delta = if forward { step_months } else { -step_months };
        for _ in 0..240 {
            view_start = view_start.add_months_clamped(delta);
            let in_window = (0..step_months)
                .any(|offset| target.is_same_month(view_start.add_months_clamped(offset)));
            if in_window {
                return view_start;
            }
        }
        target.start_of_month()
    };

    match key {
        CalendarNavKey::Left | CalendarNavKey::Right => {
            let step: i64 = match (key, dir) {
                (CalendarNavKey::Left, Direction::Ltr)
                | (CalendarNavKey::Right, Direction::Rtl) => -1,
                (CalendarNavKey::Right, Direction::Ltr)
                | (CalendarNavKey::Left, Direction::Rtl) => 1,
                _ => unreachable!(),
            };
            let target = find_enabled_by_step(
                current_date,
                step,
                3660,
                min_value,
                max_value,
                &is_disabled,
            )?;
            let in_view = is_in_visible_months(target);
            if in_view {
                Some(CalendarNavResult {
                    target_date: target,
                    next_placeholder: target,
                    next_view_start: None,
                    paged: false,
                })
            } else {
                if (step < 0 && prev_blocked) || (step > 0 && next_blocked) {
                    return None;
                }
                let view_start = paged_view_start_for(target, step > 0);
                Some(CalendarNavResult {
                    target_date: target,
                    next_placeholder: target,
                    next_view_start: Some(view_start),
                    paged: true,
                })
            }
        }
        CalendarNavKey::Up | CalendarNavKey::Down => {
            let step: i64 = match key {
                CalendarNavKey::Up => -7,
                CalendarNavKey::Down => 7,
                _ => unreachable!(),
            };
            let target = find_enabled_by_step(
                current_date,
                step,
                520,
                min_value,
                max_value,
                &is_disabled,
            )?;
            let in_view = is_in_visible_months(target);
            if in_view {
                Some(CalendarNavResult {
                    target_date: target,
                    next_placeholder: target,
                    next_view_start: None,
                    paged: false,
                })
            } else {
                if (step < 0 && prev_blocked) || (step > 0 && next_blocked) {
                    return None;
                }
                let view_start = paged_view_start_for(target, step > 0);
                Some(CalendarNavResult {
                    target_date: target,
                    next_placeholder: target,
                    next_view_start: Some(view_start),
                    paged: true,
                })
            }
        }
        CalendarNavKey::PageUp
        | CalendarNavKey::PageDown
        | CalendarNavKey::ShiftPageUp
        | CalendarNavKey::ShiftPageDown => {
            let raw_target = match key {
                CalendarNavKey::PageUp => current_date.add_months_clamped(-1),
                CalendarNavKey::PageDown => current_date.add_months_clamped(1),
                CalendarNavKey::ShiftPageUp => current_date.add_years_clamped(-1),
                CalendarNavKey::ShiftPageDown => current_date.add_years_clamped(1),
                _ => unreachable!(),
            };
            let clamped = raw_target.clamp(min_value, max_value);
            let target = find_nearest_enabled_in_month_or_bounds(
                clamped,
                min_value,
                max_value,
                &is_disabled,
            )?;
            let in_view = is_in_visible_months(target);
            if in_view {
                Some(CalendarNavResult {
                    target_date: target,
                    next_placeholder: target,
                    next_view_start: None,
                    paged: false,
                })
            } else {
                let forward = target > first_visible_month;
                let view_start = if step_months == 1 {
                    target.start_of_month()
                } else if forward {
                    let candidate = first_visible_month.add_months_clamped(1);
                    let in_candidate = (0..step_months)
                        .any(|offset| target.is_same_month(candidate.add_months_clamped(offset)));
                    if in_candidate {
                        candidate
                    } else {
                        target.start_of_month()
                    }
                } else {
                    target.start_of_month()
                };
                Some(CalendarNavResult {
                    target_date: target,
                    next_placeholder: target,
                    next_view_start: Some(view_start),
                    paged: true,
                })
            }
        }
        CalendarNavKey::Home | CalendarNavKey::End => {
            let week_start = current_date.start_of_week(week_starts_on);
            let indices: Vec<i64> = match key {
                CalendarNavKey::Home => (0..7).collect(),
                CalendarNavKey::End => (0..7).rev().collect(),
                _ => unreachable!(),
            };
            let mut found = None;
            for offset in indices {
                let candidate = week_start.add_days(offset);
                if min_value.is_some_and(|min| candidate < min)
                    || max_value.is_some_and(|max| candidate > max)
                {
                    continue;
                }
                if disable_days_outside_month && !is_in_visible_months(candidate) {
                    continue;
                }
                if !is_disabled(candidate) {
                    found = Some(candidate);
                    break;
                }
            }
            let target = found?;
            let in_view = is_in_visible_months(target);
            Some(CalendarNavResult {
                target_date: target,
                next_placeholder: target,
                next_view_start: (!in_view).then(|| target.start_of_month()),
                paged: !in_view,
            })
        }
    }
}

fn find_enabled_by_step<F>(
    start: CalendarDate,
    step_days: i64,
    max_iters: usize,
    min_value: Option<CalendarDate>,
    max_value: Option<CalendarDate>,
    is_disabled: &F,
) -> Option<CalendarDate>
where
    F: Fn(CalendarDate) -> bool,
{
    let mut candidate = start.add_days(step_days);
    for _ in 0..max_iters {
        if step_days < 0 && min_value.is_some_and(|min| candidate < min) {
            return None;
        }
        if step_days > 0 && max_value.is_some_and(|max| candidate > max) {
            return None;
        }
        if !is_disabled(candidate) {
            return Some(candidate);
        }
        candidate = candidate.add_days(step_days);
    }
    None
}

fn find_nearest_enabled_in_month_or_bounds<F>(
    target: CalendarDate,
    min_value: Option<CalendarDate>,
    max_value: Option<CalendarDate>,
    is_disabled: &F,
) -> Option<CalendarDate>
where
    F: Fn(CalendarDate) -> bool,
{
    if !is_disabled(target) {
        return Some(target);
    }
    let month_start = target.start_of_month();
    let month_end = target.end_of_month();
    for delta in 1..=31_i64 {
        let forward = target.add_days(delta);
        if forward <= month_end
            && min_value.is_none_or(|min| forward >= min)
            && max_value.is_none_or(|max| forward <= max)
            && !is_disabled(forward)
        {
            return Some(forward);
        }
        let backward = target.add_days(-delta);
        if backward >= month_start
            && min_value.is_none_or(|min| backward >= min)
            && max_value.is_none_or(|max| backward <= max)
            && !is_disabled(backward)
        {
            return Some(backward);
        }
    }
    None
}

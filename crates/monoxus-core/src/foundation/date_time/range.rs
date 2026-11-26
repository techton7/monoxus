use std::cmp::Ordering;

use super::{
    date::{CalendarDate, DateValue, TimeValue, days_between_inclusive},
    field::FieldInvalidReason,
};

/// Chronological comparison contract for range endpoints (`CalendarDate`, `DateValue`, `TimeValue`).
pub trait ChronoComparable: Copy {
    fn cmp_chronological(&self, other: &Self) -> Ordering;
}

impl ChronoComparable for CalendarDate {
    #[inline]
    fn cmp_chronological(&self, other: &Self) -> Ordering {
        self.days_since_epoch().cmp(&other.days_since_epoch())
    }
}

impl ChronoComparable for DateValue {
    #[inline]
    fn cmp_chronological(&self, other: &Self) -> Ordering {
        self.compare_chronological(other)
    }
}

impl ChronoComparable for TimeValue {
    #[inline]
    fn cmp_chronological(&self, other: &Self) -> Ordering {
        self.time().cmp(&other.time())
    }
}

/// Generic contiguous inclusive chronological range (`{ start, end }`).
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub struct ChronoRange<T> {
    pub start: Option<T>,
    pub end: Option<T>,
}

impl<T> Default for ChronoRange<T> {
    fn default() -> Self {
        Self {
            start: None,
            end: None,
        }
    }
}

impl<T> ChronoRange<T> {
    /// Creates a range from optional `start` and `end` endpoints.
    pub const fn new(start: Option<T>, end: Option<T>) -> Self {
        Self { start, end }
    }

    /// Returns `true` when neither `start` nor `end` is set.
    pub const fn is_empty(&self) -> bool {
        self.start.is_none() && self.end.is_none()
    }

    /// Returns `true` when both `start` and `end` are set.
    pub const fn is_complete(&self) -> bool {
        self.start.is_some() && self.end.is_some()
    }
}

impl<T: ChronoComparable> ChronoRange<T> {
    /// Returns `true` when both endpoints are set and `end < start` chronologically.
    ///
    /// For [`TimeRange`], this is an opt-in helper for consumers enforcing same-day ordering via
    /// the `validate` callback; overnight ranges (e.g. `20:00..05:00`) remain valid by default in
    /// [`validate_time_range`].
    pub fn is_inverted(&self) -> bool {
        match (self.start, self.end) {
            (Some(s), Some(e)) => e.cmp_chronological(&s).is_lt(),
            _ => false,
        }
    }

    /// Validates that a complete range is not inverted (`end < start` -> [`FieldInvalidReason::RangeOrder`]).
    pub fn validate_range_order(&self) -> Result<(), FieldInvalidReason> {
        if self.is_inverted() {
            Err(FieldInvalidReason::RangeOrder)
        } else {
            Ok(())
        }
    }

    /// Returns the chronologically ordered `(lo, hi)` pair when both endpoints are set.
    pub fn ordered(&self) -> Option<(T, T)> {
        match (self.start, self.end) {
            (Some(s), Some(e)) => {
                if s.cmp_chronological(&e).is_le() {
                    Some((s, e))
                } else {
                    Some((e, s))
                }
            }
            _ => None,
        }
    }
}

/// Contiguous inclusive date range (`{ start, end }`).
pub type DateRange = ChronoRange<CalendarDate>;

/// Contiguous inclusive date or date-time range (`{ start, end }`) for `DateRangeField` and `DateRangePicker`.
pub type DateValueRange = ChronoRange<DateValue>;

/// Alias for [`DateValueRange`] matching Sub-wave 5C specification naming.
pub type DateTimeRange = DateValueRange;

/// Contiguous inclusive time range (`{ start, end }`) for `TimeRangeField`.
pub type TimeRange = ChronoRange<TimeValue>;

impl ChronoRange<CalendarDate> {
    /// Returns `true` when `date` equals either endpoint or lies inclusively between `start` and `end`.
    pub fn contains_date(&self, date: CalendarDate) -> bool {
        if self.start == Some(date) || self.end == Some(date) {
            return true;
        }
        if let (Some(start), Some(end)) = (self.start, self.end) {
            return date.is_between_inclusive(start, end);
        }
        false
    }
}

impl ChronoRange<DateValue> {
    /// Extracts the date-only [`DateRange`] endpoints.
    pub const fn to_date_range(&self) -> DateRange {
        DateRange {
            start: match self.start {
                Some(v) => Some(v.date()),
                None => None,
            },
            end: match self.end {
                Some(v) => Some(v.date()),
                None => None,
            },
        }
    }
}

impl From<DateRange> for DateValueRange {
    #[inline]
    fn from(range: DateRange) -> Self {
        Self {
            start: range.start.map(DateValue::Date),
            end: range.end.map(DateValue::Date),
        }
    }
}

impl From<DateValueRange> for DateRange {
    #[inline]
    fn from(range: DateValueRange) -> Self {
        range.to_date_range()
    }
}

/// Validates a [`DateValueRange`] following Sub-wave 5C precedence:
/// `Unavailable` (any present endpoint, including partial ranges) -> `Custom` (when both endpoints set) -> `RangeOrder` (`end < start`) -> `Min` -> `Max`.
pub fn validate_date_range_with_unavailable(
    range: DateValueRange,
    min: Option<DateValue>,
    max: Option<DateValue>,
    is_unavailable: bool,
    custom_errors: Option<Vec<String>>,
) -> Result<(), FieldInvalidReason> {
    if !range.is_empty() && is_unavailable {
        return Err(FieldInvalidReason::Unavailable);
    }
    if range.is_complete()
        && let Some(msgs) = custom_errors
        && !msgs.is_empty()
    {
        return Err(FieldInvalidReason::Custom(msgs));
    }
    range.validate_range_order()?;
    if let Some(min_val) = min {
        if range
            .start
            .is_some_and(|s| s.compare_chronological(&min_val).is_lt())
            || range
                .end
                .is_some_and(|e| e.compare_chronological(&min_val).is_lt())
        {
            return Err(FieldInvalidReason::Min);
        }
    }
    if let Some(max_val) = max {
        if range
            .start
            .is_some_and(|s| s.compare_chronological(&max_val).is_gt())
            || range
                .end
                .is_some_and(|e| e.compare_chronological(&max_val).is_gt())
        {
            return Err(FieldInvalidReason::Max);
        }
    }
    Ok(())
}

/// Validates a [`DateValueRange`] following Sub-wave 5C precedence:
/// `Custom` (when both endpoints set) -> `RangeOrder` (`end < start`) -> `Min` -> `Max`.
pub fn validate_date_value_range(
    range: DateValueRange,
    min: Option<DateValue>,
    max: Option<DateValue>,
    custom_errors: Option<Vec<String>>,
) -> Result<(), FieldInvalidReason> {
    validate_date_range_with_unavailable(range, min, max, false, custom_errors)
}

/// Validates a [`TimeRange`] matching `bits-ui` (`time-range-field.svelte.ts:186-224`) precedence:
/// `Custom` (when both endpoints set) -> `Min` (`start < min` or `end < min`) -> `Max` (`start > max` or `end > max`).
///
/// Overnight time ranges where `end < start` (e.g. `20:00..05:00`) are valid by default; consumers
/// enforcing same-day ordering can opt in via the `validate` callback (e.g. using
/// [`TimeRange::is_inverted`] or [`TimeRange::validate_range_order`]).
pub fn validate_time_range(
    range: TimeRange,
    min: Option<TimeValue>,
    max: Option<TimeValue>,
    custom_errors: Option<Vec<String>>,
) -> Result<(), FieldInvalidReason> {
    if range.is_complete()
        && let Some(msgs) = custom_errors
        && !msgs.is_empty()
    {
        return Err(FieldInvalidReason::Custom(msgs));
    }
    if let Some(min_val) = min {
        if range.start.is_some_and(|s| s.time() < min_val.time())
            || range.end.is_some_and(|e| e.time() < min_val.time())
        {
            return Err(FieldInvalidReason::Min);
        }
    }
    if let Some(max_val) = max {
        if range.start.is_some_and(|s| s.time() > max_val.time())
            || range.end.is_some_and(|e| e.time() > max_val.time())
        {
            return Err(FieldInvalidReason::Max);
        }
    }
    Ok(())
}

/// Checks that every strictly intermediate day between `start` and `end` is neither disabled nor unavailable.
///
/// Adjacent or equal endpoints always return `true` (matching `bits-ui`'s `areAllDaysBetweenValid`).
pub fn are_all_days_between_valid<FD, FU>(
    start: CalendarDate,
    end: CalendarDate,
    is_disabled: FD,
    is_unavailable: FU,
) -> bool
where
    FD: Fn(CalendarDate) -> bool,
    FU: Fn(CalendarDate) -> bool,
{
    let (lo, hi) = if start <= end {
        (start, end)
    } else {
        (end, start)
    };
    let lo_days = lo.days_since_epoch();
    let hi_days = hi.days_since_epoch();
    if hi_days - lo_days <= 1 {
        return true;
    }

    for day in (lo_days + 1)..hi_days {
        let date = CalendarDate::from_days_since_epoch(day);
        if is_disabled(date) || is_unavailable(date) {
            return false;
        }
    }
    true
}

/// Returns `true` if any day in the inclusive range `[start ..= end]` satisfies `is_disabled`.
pub fn has_disabled_dates_in_range<FD>(
    start: CalendarDate,
    end: CalendarDate,
    is_disabled: FD,
) -> bool
where
    FD: Fn(CalendarDate) -> bool,
{
    let (lo, hi) = if start <= end {
        (start, end)
    } else {
        (end, start)
    };
    let lo_days = lo.days_since_epoch();
    let hi_days = hi.days_since_epoch();
    for day in lo_days..=hi_days {
        let date = CalendarDate::from_days_since_epoch(day);
        if is_disabled(date) {
            return true;
        }
    }
    false
}

/// Validates a candidate range `[start ..= end]` against `min_days`, `max_days`, and `exclude_disabled`.
pub fn is_range_valid<FD>(
    start: CalendarDate,
    end: CalendarDate,
    min_days: Option<usize>,
    max_days: Option<usize>,
    exclude_disabled: bool,
    is_disabled: FD,
) -> bool
where
    FD: Fn(CalendarDate) -> bool,
{
    let (lo, hi) = if start <= end {
        (start, end)
    } else {
        (end, start)
    };
    let days_in_range = days_between_inclusive(lo, hi);

    if min_days.is_some_and(|min| days_in_range < min) {
        return false;
    }
    if max_days.is_some_and(|max| days_in_range > max) {
        return false;
    }
    if exclude_disabled && has_disabled_dates_in_range(lo, hi, is_disabled) {
        return false;
    }
    true
}

/// Computes the hover/focus preview range (`data-highlighted`) when `start` is set, `end` is `None`,
/// and `focused` is hovered/focused without invalid intermediate dates.
pub fn compute_highlighted_range<FD, FU>(
    start: Option<CalendarDate>,
    end: Option<CalendarDate>,
    focused: Option<CalendarDate>,
    is_disabled: FD,
    is_unavailable: FU,
) -> Option<(CalendarDate, CalendarDate)>
where
    FD: Fn(CalendarDate) -> bool,
    FU: Fn(CalendarDate) -> bool,
{
    if start.is_some() && end.is_some() {
        return None;
    }
    let (Some(start), Some(focused)) = (start, focused) else {
        return None;
    };

    let (lo, hi) = if start <= focused {
        (start, focused)
    } else {
        (focused, start)
    };

    if lo == hi || lo.add_days(1) == hi {
        return Some((lo, hi));
    }

    are_all_days_between_valid(lo, hi, is_disabled, is_unavailable).then_some((lo, hi))
}

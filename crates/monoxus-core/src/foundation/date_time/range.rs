use super::{
    date::{CalendarDate, DateValue, TimeValue},
    field::FieldInvalidReason,
};

/// Contiguous inclusive date range (`{ start, end }`).
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, Hash)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub struct DateRange {
    pub start: Option<CalendarDate>,
    pub end: Option<CalendarDate>,
}

impl DateRange {
    /// Creates a [`DateRange`] from optional `start` and `end` endpoints.
    pub const fn new(start: Option<CalendarDate>, end: Option<CalendarDate>) -> Self {
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

    /// Returns `true` when both endpoints are set and `end < start`.
    pub const fn is_inverted(&self) -> bool {
        match (self.start, self.end) {
            (Some(s), Some(e)) => e.days_since_epoch() < s.days_since_epoch(),
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

    /// Returns the ordered `(lo, hi)` pair when both endpoints are set.
    pub const fn ordered(&self) -> Option<(CalendarDate, CalendarDate)> {
        match (self.start, self.end) {
            (Some(s), Some(e)) => {
                if s.days_since_epoch() <= e.days_since_epoch() {
                    Some((s, e))
                } else {
                    Some((e, s))
                }
            }
            _ => None,
        }
    }
}

/// Contiguous inclusive date or date-time range (`{ start, end }`) for `DateRangeField` and `DateRangePicker`.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, Hash)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub struct DateValueRange {
    pub start: Option<DateValue>,
    pub end: Option<DateValue>,
}

/// Alias for [`DateValueRange`] matching Sub-wave 5C specification naming.
pub type DateTimeRange = DateValueRange;

impl DateValueRange {
    /// Creates a [`DateValueRange`] from optional `start` and `end` endpoints.
    pub const fn new(start: Option<DateValue>, end: Option<DateValue>) -> Self {
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

    /// Returns `true` when both endpoints are set and `end < start` chronologically.
    pub fn is_inverted(&self) -> bool {
        match (self.start, self.end) {
            (Some(s), Some(e)) => e.compare_chronological(&s).is_lt(),
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
    pub fn ordered(&self) -> Option<(DateValue, DateValue)> {
        match (self.start, self.end) {
            (Some(s), Some(e)) => {
                if s.compare_chronological(&e).is_le() {
                    Some((s, e))
                } else {
                    Some((e, s))
                }
            }
            _ => None,
        }
    }

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

/// Contiguous inclusive time range (`{ start, end }`) for `TimeRangeField`.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, Hash)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub struct TimeRange {
    pub start: Option<TimeValue>,
    pub end: Option<TimeValue>,
}

impl TimeRange {
    /// Creates a [`TimeRange`] from optional `start` and `end` endpoints.
    pub const fn new(start: Option<TimeValue>, end: Option<TimeValue>) -> Self {
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

    /// Returns `true` when both endpoints are set and `end < start`.
    pub fn is_inverted(&self) -> bool {
        match (self.start, self.end) {
            (Some(s), Some(e)) => e.time() < s.time(),
            _ => false,
        }
    }

    /// Validates that a complete time range is not inverted (`end < start` -> [`FieldInvalidReason::RangeOrder`]).
    pub fn validate_range_order(&self) -> Result<(), FieldInvalidReason> {
        if self.is_inverted() {
            Err(FieldInvalidReason::RangeOrder)
        } else {
            Ok(())
        }
    }

    /// Returns the ordered `(lo, hi)` pair when both endpoints are set.
    pub fn ordered(&self) -> Option<(TimeValue, TimeValue)> {
        match (self.start, self.end) {
            (Some(s), Some(e)) => {
                if s.time() <= e.time() {
                    Some((s, e))
                } else {
                    Some((e, s))
                }
            }
            _ => None,
        }
    }
}

/// Validates a [`DateValueRange`] following Sub-wave 5C precedence:
/// `Custom` (when both endpoints set) -> `RangeOrder` (`end < start`) -> `Min` -> `Max`.
pub fn validate_date_value_range(
    range: DateValueRange,
    min: Option<DateValue>,
    max: Option<DateValue>,
    custom_errors: Option<Vec<String>>,
) -> Result<(), FieldInvalidReason> {
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

/// Validates a [`TimeRange`] following Sub-wave 5C precedence:
/// `Custom` (when both endpoints set) -> `RangeOrder` (`end < start`) -> `Min` -> `Max`.
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
    range.validate_range_order()?;
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
    let days_in_range = (hi.days_since_epoch() - lo.days_since_epoch() + 1) as usize;

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

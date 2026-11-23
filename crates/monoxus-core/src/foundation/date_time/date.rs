use std::{cmp::Ordering, fmt, str::FromStr};

/// Day of the week (`0 = Sunday .. 6 = Saturday`), matching JavaScript / WAI-ARIA `weekStartsOn` numbering.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, PartialOrd, Ord, Hash)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
#[repr(u8)]
pub enum Weekday {
    #[default]
    Sunday = 0,
    Monday = 1,
    Tuesday = 2,
    Wednesday = 3,
    Thursday = 4,
    Friday = 5,
    Saturday = 6,
}

impl Weekday {
    /// Converts a `0..=6` integer (`0 = Sunday .. 6 = Saturday`) into [`Weekday`].
    pub const fn from_u8(value: u8) -> Option<Self> {
        match value {
            0 => Some(Self::Sunday),
            1 => Some(Self::Monday),
            2 => Some(Self::Tuesday),
            3 => Some(Self::Wednesday),
            4 => Some(Self::Thursday),
            5 => Some(Self::Friday),
            6 => Some(Self::Saturday),
            _ => None,
        }
    }

    /// Returns the numeric `0..=6` representation (`0 = Sunday .. 6 = Saturday`).
    pub const fn to_u8(self) -> u8 {
        self as u8
    }

    /// Returns the zero-based column offset of `self` relative to `first_day` (`0..=6`).
    pub const fn offset_from(self, first_day: Self) -> u8 {
        (self.to_u8() + 7 - first_day.to_u8()) % 7
    }

    /// Shifts this weekday forward or backward by `delta` days modulo 7.
    pub const fn add_days(self, delta: i64) -> Self {
        let shifted = (self.to_u8() as i64 + delta).rem_euclid(7) as u8;
        match Self::from_u8(shifted) {
            Some(day) => day,
            None => Self::Sunday,
        }
    }
}

/// Returns `true` if `year` is a leap year in the proleptic Gregorian calendar.
#[inline]
pub const fn is_leap_year(year: i32) -> bool {
    (year % 4 == 0 && year % 100 != 0) || (year % 400 == 0)
}

/// Returns the number of days in `(year, month)` (`month` in `1..=12`), or `0` if `month` is out of range.
#[inline]
pub const fn days_in_month(year: i32, month: u8) -> u8 {
    match month {
        1 | 3 | 5 | 7 | 8 | 10 | 12 => 31,
        4 | 6 | 9 | 11 => 30,
        2 => {
            if is_leap_year(year) {
                29
            } else {
                28
            }
        }
        _ => 0,
    }
}

/// Converts a proleptic Gregorian `(year, month, day)` triple to days since `1970-01-01`.
///
/// Uses Howard Hinnant's O(1) civil-from-days / days-from-civil algorithm.
#[inline]
pub const fn days_from_civil(year: i32, month: u8, day: u8) -> i64 {
    let y = year as i64 - if month <= 2 { 1 } else { 0 };
    let era = if y >= 0 { y } else { y - 399 } / 400;
    let yoe = y - era * 400;
    let m = month as i64;
    let doy = (153 * (m + if m > 2 { -3 } else { 9 }) + 2) / 5 + day as i64 - 1;
    let doe = yoe * 365 + yoe / 4 - yoe / 100 + doy;
    era * 146_097 + doe - 719_468
}

/// Converts days since `1970-01-01` back into a proleptic Gregorian `(year, month, day)` triple.
#[inline]
pub const fn civil_from_days(days_since_epoch: i64) -> (i32, u8, u8) {
    let z = days_since_epoch + 719_468;
    let era = if z >= 0 { z } else { z - 146_096 } / 146_097;
    let doe = z - era * 146_097;
    let yoe = (doe - doe / 1460 + doe / 36_524 - doe / 146_096) / 365;
    let y = yoe + era * 400;
    let doy = doe - (365 * yoe + yoe / 4 - yoe / 100);
    let mp = (5 * doy + 2) / 153;
    let d = doy - (153 * mp + 2) / 5 + 1;
    let m = mp + if mp < 10 { 3 } else { -9 };
    let year = y + if m <= 2 { 1 } else { 0 };
    (year as i32, m as u8, d as u8)
}

/// Proleptic Gregorian calendar date without time zone or wall-clock time.
#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord, Hash)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub struct CalendarDate {
    pub year: i32,
    pub month: u8,
    pub day: u8,
}

impl Default for CalendarDate {
    fn default() -> Self {
        Self {
            year: 1970,
            month: 1,
            day: 1,
        }
    }
}

impl fmt::Display for CalendarDate {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        if self.year < 0 {
            write!(f, "-{:04}-{:02}-{:02}", -self.year, self.month, self.day)
        } else {
            write!(f, "{:04}-{:02}-{:02}", self.year, self.month, self.day)
        }
    }
}

impl FromStr for CalendarDate {
    type Err = ();

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        Self::from_iso_str(s).ok_or(())
    }
}

impl CalendarDate {
    /// Constructs a validated proleptic Gregorian [`CalendarDate`], returning `None` if invalid.
    pub const fn new(year: i32, month: u8, day: u8) -> Option<Self> {
        let dim = days_in_month(year, month);
        if dim == 0 || day < 1 || day > dim {
            None
        } else {
            Some(Self { year, month, day })
        }
    }

    /// Constructs a [`CalendarDate`], clamping `month` into `1..=12` and `day` into `1..=days_in_month`.
    pub const fn from_ymd_clamped(year: i32, month: u8, day: u8) -> Self {
        let clamped_month = if month < 1 {
            1
        } else if month > 12 {
            12
        } else {
            month
        };
        let dim = days_in_month(year, clamped_month);
        let clamped_day = if day < 1 {
            1
        } else if day > dim {
            dim
        } else {
            day
        };
        Self {
            year,
            month: clamped_month,
            day: clamped_day,
        }
    }

    /// Parses an ISO-8601 `YYYY-MM-DD` string into a validated [`CalendarDate`].
    pub fn from_iso_str(raw: &str) -> Option<Self> {
        let trimmed = raw.trim();
        let (negative, rest) = if let Some(stripped) = trimmed.strip_prefix('-') {
            (true, stripped)
        } else {
            (false, trimmed)
        };
        let mut parts = rest.split('-');
        let year_str = parts.next()?;
        let month_str = parts.next()?;
        let day_str = parts.next()?;
        if parts.next().is_some()
            || year_str.is_empty()
            || month_str.len() != 2
            || day_str.len() != 2
        {
            return None;
        }
        let abs_year: i32 = year_str.parse().ok()?;
        let year = if negative { -abs_year } else { abs_year };
        let month: u8 = month_str.parse().ok()?;
        let day: u8 = day_str.parse().ok()?;
        Self::new(year, month, day)
    }

    /// Formats the date in ISO-8601 `YYYY-MM-DD` form.
    pub fn to_iso_string(&self) -> String {
        self.to_string()
    }

    /// Returns days elapsed since `1970-01-01` (negative for earlier dates).
    #[inline]
    pub const fn days_since_epoch(&self) -> i64 {
        days_from_civil(self.year, self.month, self.day)
    }

    /// Reconstructs a [`CalendarDate`] from days elapsed since `1970-01-01`.
    #[inline]
    pub const fn from_days_since_epoch(days: i64) -> Self {
        let (year, month, day) = civil_from_days(days);
        Self { year, month, day }
    }

    /// Returns the weekday of this date (`1970-01-01` was Thursday).
    #[inline]
    pub const fn weekday(&self) -> Weekday {
        let dow = (self.days_since_epoch() + 4).rem_euclid(7) as u8;
        match Weekday::from_u8(dow) {
            Some(day) => day,
            None => Weekday::Sunday,
        }
    }

    /// Returns the first day of this date's month.
    #[inline]
    pub const fn start_of_month(&self) -> Self {
        Self {
            year: self.year,
            month: self.month,
            day: 1,
        }
    }

    /// Returns the last day of this date's month.
    #[inline]
    pub const fn end_of_month(&self) -> Self {
        Self {
            year: self.year,
            month: self.month,
            day: days_in_month(self.year, self.month),
        }
    }

    /// Returns the date shifted by `delta` days.
    #[inline]
    pub const fn add_days(&self, delta: i64) -> Self {
        Self::from_days_since_epoch(self.days_since_epoch() + delta)
    }

    /// Returns the date shifted by `delta` months, clamping `day` to the target month's length.
    pub const fn add_months_clamped(&self, delta: i32) -> Self {
        let total_months = self.year as i64 * 12 + (self.month as i64 - 1) + delta as i64;
        let new_year = total_months.div_euclid(12) as i32;
        let new_month = (total_months.rem_euclid(12) + 1) as u8;
        Self::from_ymd_clamped(new_year, new_month, self.day)
    }

    /// Returns the date shifted by `delta` years, clamping `day` (e.g., Feb 29 -> Feb 28).
    #[inline]
    pub const fn add_years_clamped(&self, delta: i32) -> Self {
        self.with_year_clamped(self.year.saturating_add(delta))
    }

    /// Returns a copy with `year` updated and `day` clamped to the target month's length.
    #[inline]
    pub const fn with_year_clamped(&self, year: i32) -> Self {
        Self::from_ymd_clamped(year, self.month, self.day)
    }

    /// Returns a copy with `month` clamped to `1..=12` and `day` clamped to the month's length.
    #[inline]
    pub const fn with_month_clamped(&self, month: u8) -> Self {
        Self::from_ymd_clamped(self.year, month, self.day)
    }

    /// Returns a copy with `day` clamped to the current month's length.
    #[inline]
    pub const fn with_day_clamped(&self, day: u8) -> Self {
        Self::from_ymd_clamped(self.year, self.month, day)
    }

    /// Returns the first day of the week containing `self`, starting at `first_day`.
    #[inline]
    pub const fn start_of_week(&self, first_day: Weekday) -> Self {
        let offset = self.weekday().offset_from(first_day) as i64;
        self.add_days(-offset)
    }

    /// Returns the last day of the week containing `self`, starting at `first_day`.
    #[inline]
    pub const fn end_of_week(&self, first_day: Weekday) -> Self {
        self.start_of_week(first_day).add_days(6)
    }

    /// Clamps `self` into the optional `[min, max]` inclusive interval.
    #[inline]
    pub fn clamp(self, min: Option<Self>, max: Option<Self>) -> Self {
        let mut out = self;
        if let Some(min_date) = min
            && out < min_date
        {
            out = min_date;
        }
        if let Some(max_date) = max
            && out > max_date
        {
            out = max_date;
        }
        out
    }

    /// Returns `true` when `self` and `other` share the same year and month.
    #[inline]
    pub const fn is_same_month(&self, other: Self) -> bool {
        self.year == other.year && self.month == other.month
    }

    /// Returns `true` when `self` and `other` represent the exact same calendar day.
    #[inline]
    pub const fn is_same_day(&self, other: Self) -> bool {
        self.year == other.year && self.month == other.month && self.day == other.day
    }

    /// Returns `true` when `self` lies inclusively between `a` and `b` (order-independent).
    #[inline]
    pub const fn is_between_inclusive(&self, a: Self, b: Self) -> bool {
        let d = self.days_since_epoch();
        let da = a.days_since_epoch();
        let db = b.days_since_epoch();
        let (lo, hi) = if da <= db { (da, db) } else { (db, da) };
        d >= lo && d <= hi
    }
}

/// Returns the inclusive day count between `start` and `end` (order-independent).
#[inline]
pub fn days_between_inclusive(start: CalendarDate, end: CalendarDate) -> usize {
    let diff = (end.days_since_epoch() - start.days_since_epoch()).abs();
    (diff + 1) as usize
}

/// Compares two [`CalendarDate`] values chronologically.
#[inline]
pub fn compare_dates(a: CalendarDate, b: CalendarDate) -> Ordering {
    a.cmp(&b)
}

/// Resolves the initial placeholder date from optional `default_value` or `today`,
/// clamped into `[min_value, max_value]`.
#[inline]
pub fn default_placeholder_date(
    default_value: Option<CalendarDate>,
    min_value: Option<CalendarDate>,
    max_value: Option<CalendarDate>,
    today: CalendarDate,
) -> CalendarDate {
    default_value.unwrap_or(today).clamp(min_value, max_value)
}

/// Wall-clock time of day (`00:00:00.000`..=`23:59:59.999`), prepared for Sub-wave 5C time/date fields.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, PartialOrd, Ord, Hash)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub struct CalendarTime {
    pub hour: u8,
    pub minute: u8,
    pub second: u8,
    pub millisecond: u16,
}

impl CalendarTime {
    /// Constructs a validated [`CalendarTime`], returning `None` if any component is out of range.
    pub const fn new(hour: u8, minute: u8, second: u8, millisecond: u16) -> Option<Self> {
        if hour >= 24 || minute >= 60 || second >= 60 || millisecond >= 1000 {
            None
        } else {
            Some(Self {
                hour,
                minute,
                second,
                millisecond,
            })
        }
    }

    /// Returns midnight (`00:00:00.000`).
    pub const fn midnight() -> Self {
        Self {
            hour: 0,
            minute: 0,
            second: 0,
            millisecond: 0,
        }
    }

    /// Formats the time as `HH:MM:SS` or `HH:MM:SS.mmm` when milliseconds are non-zero.
    pub fn to_iso_string(&self) -> String {
        if self.millisecond == 0 {
            format!("{:02}:{:02}:{:02}", self.hour, self.minute, self.second)
        } else {
            format!(
                "{:02}:{:02}:{:02}.{:03}",
                self.hour, self.minute, self.second, self.millisecond
            )
        }
    }
}

impl fmt::Display for CalendarTime {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(&self.to_iso_string())
    }
}

/// Combined proleptic Gregorian date and wall-clock time without time zone, prepared for Sub-wave 5C.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, PartialOrd, Ord, Hash)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub struct CalendarDateTime {
    pub date: CalendarDate,
    pub time: CalendarTime,
}

impl CalendarDateTime {
    /// Combines a [`CalendarDate`] and [`CalendarTime`].
    pub const fn new(date: CalendarDate, time: CalendarTime) -> Self {
        Self { date, time }
    }

    /// Wraps a [`CalendarDate`] at midnight (`00:00:00.000`).
    pub const fn from_date(date: CalendarDate) -> Self {
        Self {
            date,
            time: CalendarTime::midnight(),
        }
    }

    /// Replaces the date component while preserving the existing wall-clock time.
    pub const fn with_date(&self, date: CalendarDate) -> Self {
        Self {
            date,
            time: self.time,
        }
    }

    /// Replaces the time component while preserving the existing date.
    pub const fn with_time(&self, time: CalendarTime) -> Self {
        Self {
            date: self.date,
            time,
        }
    }

    /// Formats as ISO-8601 `YYYY-MM-DDTHH:MM:SS(.mmm)`.
    pub fn to_iso_string(&self) -> String {
        format!("{}T{}", self.date, self.time)
    }
}

impl fmt::Display for CalendarDateTime {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}T{}", self.date, self.time)
    }
}

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

    /// Cycles `year` by `delta` clamped into `1..=9999`, clamping `day` to the target month length.
    #[inline]
    pub const fn cycle_year(&self, delta: i32) -> Self {
        let next_year = self.year.saturating_add(delta);
        let clamped_year = if next_year < 1 {
            1
        } else if next_year > 9999 {
            9999
        } else {
            next_year
        };
        Self::from_ymd_clamped(clamped_year, self.month, self.day)
    }

    /// Cycles `month` by `delta` within `1..=12` without carrying into `year`, clamping `day`.
    #[inline]
    pub const fn cycle_month(&self, delta: i32) -> Self {
        let next_month = ((self.month as i32 - 1 + delta).rem_euclid(12) + 1) as u8;
        Self::from_ymd_clamped(self.year, next_month, self.day)
    }

    /// Cycles `day` by `delta` within `1..=days_in_month(year, month)` without carrying into `month`.
    #[inline]
    pub const fn cycle_day(&self, delta: i32) -> Self {
        let dim = days_in_month(self.year, self.month) as i32;
        let next_day = ((self.day as i32 - 1 + delta).rem_euclid(dim) + 1) as u8;
        Self {
            year: self.year,
            month: self.month,
            day: next_day,
        }
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

/// 12-hour vs 24-hour clock display and cycling mode.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, Hash)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub enum HourCycle {
    #[default]
    H12,
    H24,
}

/// Meridiem day period (`AM` / `PM`) for 12-hour wall-clock times.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, PartialOrd, Ord, Hash)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub enum DayPeriod {
    #[default]
    Am,
    Pm,
}

impl DayPeriod {
    /// Returns the opposite day period (`Am <-> Pm`).
    #[inline]
    pub const fn toggled(self) -> Self {
        match self {
            Self::Am => Self::Pm,
            Self::Pm => Self::Am,
        }
    }

    /// Returns the canonical ASCII label (`"AM"` or `"PM"`).
    #[inline]
    pub const fn as_str(self) -> &'static str {
        match self {
            Self::Am => "AM",
            Self::Pm => "PM",
        }
    }
}

/// Precision granularity for `DateField` and `DatePicker` values.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, PartialOrd, Ord, Hash)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub enum Granularity {
    #[default]
    Day,
    Hour,
    Minute,
    Second,
}

impl Granularity {
    /// Returns `true` when time segments are rendered (`Hour`, `Minute`, or `Second`).
    #[inline]
    pub const fn includes_time(self) -> bool {
        !matches!(self, Self::Day)
    }

    /// Returns `true` when the `hour` segment is rendered.
    #[inline]
    pub const fn includes_hour(self) -> bool {
        matches!(self, Self::Hour | Self::Minute | Self::Second)
    }

    /// Returns `true` when the `minute` segment is rendered.
    #[inline]
    pub const fn includes_minute(self) -> bool {
        matches!(self, Self::Minute | Self::Second)
    }

    /// Returns `true` when the `second` segment is rendered.
    #[inline]
    pub const fn includes_second(self) -> bool {
        matches!(self, Self::Second)
    }

    /// Converts a time-bearing [`Granularity`] into [`TimeGranularity`], returning `None` for [`Granularity::Day`].
    #[inline]
    pub const fn to_time_granularity(self) -> Option<TimeGranularity> {
        match self {
            Self::Day => None,
            Self::Hour => Some(TimeGranularity::Hour),
            Self::Minute => Some(TimeGranularity::Minute),
            Self::Second => Some(TimeGranularity::Second),
        }
    }

    /// Infers the default granularity (`Minute` if `value` or `placeholder` has time, else `Day`).
    #[inline]
    pub const fn infer_from_date_value(value: Option<DateValue>, placeholder: DateValue) -> Self {
        let has_time = match value {
            Some(v) => v.has_time(),
            None => false,
        };
        if has_time || placeholder.has_time() {
            Self::Minute
        } else {
            Self::Day
        }
    }
}

/// Precision granularity for `TimeField` and `TimeRangeField` values.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, PartialOrd, Ord, Hash)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub enum TimeGranularity {
    Hour,
    #[default]
    Minute,
    Second,
}

impl TimeGranularity {
    /// Returns `true` when the `minute` segment is rendered.
    #[inline]
    pub const fn includes_minute(self) -> bool {
        matches!(self, Self::Minute | Self::Second)
    }

    /// Returns `true` when the `second` segment is rendered.
    #[inline]
    pub const fn includes_second(self) -> bool {
        matches!(self, Self::Second)
    }
}

impl From<TimeGranularity> for Granularity {
    #[inline]
    fn from(value: TimeGranularity) -> Self {
        match value {
            TimeGranularity::Hour => Self::Hour,
            TimeGranularity::Minute => Self::Minute,
            TimeGranularity::Second => Self::Second,
        }
    }
}

/// Converts a 24-hour clock hour (`0..=23`) to its 12-hour display hour (`1..=12`).
#[inline]
pub const fn hour_12_display(hour_24: u8) -> u8 {
    let rem = hour_24 % 12;
    if rem == 0 { 12 } else { rem }
}

/// Converts a 12-hour display hour (`1..=12`) and [`DayPeriod`] to a 24-hour clock hour (`0..=23`).
#[inline]
pub const fn hour_12_to_24(hour_12: u8, period: DayPeriod) -> Option<u8> {
    if hour_12 < 1 || hour_12 > 12 {
        return None;
    }
    Some(match (hour_12, period) {
        (12, DayPeriod::Am) => 0,
        (h, DayPeriod::Am) => h,
        (12, DayPeriod::Pm) => 12,
        (h, DayPeriod::Pm) => h + 12,
    })
}

const MILLIS_PER_SECOND: i64 = 1_000;
const MILLIS_PER_MINUTE: i64 = 60 * MILLIS_PER_SECOND;
const MILLIS_PER_HOUR: i64 = 60 * MILLIS_PER_MINUTE;
const MILLIS_PER_DAY: i64 = 24 * MILLIS_PER_HOUR;

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

    /// Constructs a validated [`CalendarTime`] with zero milliseconds.
    pub const fn from_hms(hour: u8, minute: u8, second: u8) -> Option<Self> {
        Self::new(hour, minute, second, 0)
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

    /// Parses an ISO-8601 wall-clock time string (`HH:MM`, `HH:MM:SS`, or `HH:MM:SS.mmm`).
    pub fn from_iso_str(raw: &str) -> Option<Self> {
        let trimmed = raw.trim();
        let mut parts = trimmed.split(':');
        let hour_str = parts.next()?;
        let minute_str = parts.next()?;
        let sec_part = parts.next();
        if parts.next().is_some()
            || hour_str.len() != 2
            || minute_str.len() != 2
            || !hour_str.bytes().all(|b| b.is_ascii_digit())
            || !minute_str.bytes().all(|b| b.is_ascii_digit())
        {
            return None;
        }
        let hour: u8 = hour_str.parse().ok()?;
        let minute: u8 = minute_str.parse().ok()?;
        let (second, millisecond) = match sec_part {
            None => (0, 0),
            Some(sec_raw) => {
                if let Some((sec_str, frac_str)) = sec_raw.split_once('.') {
                    if sec_str.len() != 2
                        || !sec_str.bytes().all(|b| b.is_ascii_digit())
                        || frac_str.is_empty()
                        || frac_str.len() > 9
                        || !frac_str.bytes().all(|b| b.is_ascii_digit())
                    {
                        return None;
                    }
                    let sec: u8 = sec_str.parse().ok()?;
                    let mut ms_bytes = [b'0'; 3];
                    for (dst, src) in ms_bytes.iter_mut().zip(frac_str.bytes().take(3)) {
                        *dst = src;
                    }
                    let ms_str = std::str::from_utf8(&ms_bytes).ok()?;
                    let ms: u16 = ms_str.parse().ok()?;
                    (sec, ms)
                } else {
                    if sec_raw.len() != 2 || !sec_raw.bytes().all(|b| b.is_ascii_digit()) {
                        return None;
                    }
                    (sec_raw.parse().ok()?, 0)
                }
            }
        };
        Self::new(hour, minute, second, millisecond)
    }

    /// Returns total milliseconds since midnight (`0..86_400_000`).
    #[inline]
    pub const fn total_milliseconds(&self) -> u32 {
        (self.hour as u32) * 3_600_000
            + (self.minute as u32) * 60_000
            + (self.second as u32) * 1_000
            + (self.millisecond as u32)
    }

    /// Constructs a [`CalendarTime`] from total milliseconds wrapped modulo 24 hours.
    #[inline]
    pub const fn from_total_milliseconds_wrapped(total_ms: i64) -> Self {
        let wrapped = total_ms.rem_euclid(MILLIS_PER_DAY);
        let hour = (wrapped / MILLIS_PER_HOUR) as u8;
        let rem_after_hour = wrapped % MILLIS_PER_HOUR;
        let minute = (rem_after_hour / MILLIS_PER_MINUTE) as u8;
        let rem_after_minute = rem_after_hour % MILLIS_PER_MINUTE;
        let second = (rem_after_minute / MILLIS_PER_SECOND) as u8;
        let millisecond = (rem_after_minute % MILLIS_PER_SECOND) as u16;
        Self {
            hour,
            minute,
            second,
            millisecond,
        }
    }

    /// Constructs a [`CalendarTime`] from total milliseconds clamped into `00:00:00.000..=23:59:59.999`.
    #[inline]
    pub const fn from_total_milliseconds_clamped(total_ms: i64) -> Self {
        let clamped = if total_ms < 0 {
            0
        } else if total_ms >= MILLIS_PER_DAY {
            MILLIS_PER_DAY - 1
        } else {
            total_ms
        };
        Self::from_total_milliseconds_wrapped(clamped)
    }

    /// Clamps `self` into the optional `[min, max]` inclusive interval.
    #[inline]
    pub fn clamp(self, min: Option<Self>, max: Option<Self>) -> Self {
        let mut out = self;
        if let Some(min_time) = min
            && out < min_time
        {
            out = min_time;
        }
        if let Some(max_time) = max
            && out > max_time
        {
            out = max_time;
        }
        out
    }

    /// Shifts `self` by `delta` hours, wrapping modulo 24 hours.
    #[inline]
    pub const fn add_hours(&self, delta: i64) -> Self {
        let next_hour = (self.hour as i64 + delta).rem_euclid(24) as u8;
        Self {
            hour: next_hour,
            minute: self.minute,
            second: self.second,
            millisecond: self.millisecond,
        }
    }

    /// Shifts `self` by `delta` minutes, wrapping modulo 24 hours.
    #[inline]
    pub const fn add_minutes(&self, delta: i64) -> Self {
        let total = self.total_milliseconds() as i64 + delta.saturating_mul(MILLIS_PER_MINUTE);
        Self::from_total_milliseconds_wrapped(total)
    }

    /// Shifts `self` by `delta` seconds, wrapping modulo 24 hours.
    #[inline]
    pub const fn add_seconds(&self, delta: i64) -> Self {
        let total = self.total_milliseconds() as i64 + delta.saturating_mul(MILLIS_PER_SECOND);
        Self::from_total_milliseconds_wrapped(total)
    }

    /// Shifts `self` by `delta` hours, clamping into `00:00:00.000..=23:59:59.999`.
    #[inline]
    pub const fn add_hours_clamped(&self, delta: i64) -> Self {
        let total = self.total_milliseconds() as i64 + delta.saturating_mul(MILLIS_PER_HOUR);
        Self::from_total_milliseconds_clamped(total)
    }

    /// Shifts `self` by `delta` minutes, clamping into `00:00:00.000..=23:59:59.999`.
    #[inline]
    pub const fn add_minutes_clamped(&self, delta: i64) -> Self {
        let total = self.total_milliseconds() as i64 + delta.saturating_mul(MILLIS_PER_MINUTE);
        Self::from_total_milliseconds_clamped(total)
    }

    /// Shifts `self` by `delta` seconds, clamping into `00:00:00.000..=23:59:59.999`.
    #[inline]
    pub const fn add_seconds_clamped(&self, delta: i64) -> Self {
        let total = self.total_milliseconds() as i64 + delta.saturating_mul(MILLIS_PER_SECOND);
        Self::from_total_milliseconds_clamped(total)
    }

    /// Cycles `hour` by `delta` without carrying across days (or across `AM`/`PM` in 12-hour mode).
    #[inline]
    pub const fn cycle_hour(&self, delta: i32, hour_cycle: HourCycle) -> Self {
        let next_hour = match hour_cycle {
            HourCycle::H24 => (self.hour as i32 + delta).rem_euclid(24) as u8,
            HourCycle::H12 => {
                let base = if self.hour >= 12 { 12 } else { 0 };
                let offset = (self.hour - base) as i32;
                base + (offset + delta).rem_euclid(12) as u8
            }
        };
        Self {
            hour: next_hour,
            minute: self.minute,
            second: self.second,
            millisecond: self.millisecond,
        }
    }

    /// Cycles `minute` by `delta` within `0..=59` without carrying into `hour`.
    #[inline]
    pub const fn cycle_minute(&self, delta: i32) -> Self {
        let next_minute = (self.minute as i32 + delta).rem_euclid(60) as u8;
        Self {
            hour: self.hour,
            minute: next_minute,
            second: self.second,
            millisecond: self.millisecond,
        }
    }

    /// Cycles `second` by `delta` within `0..=59` without carrying into `minute`.
    #[inline]
    pub const fn cycle_second(&self, delta: i32) -> Self {
        let next_second = (self.second as i32 + delta).rem_euclid(60) as u8;
        Self {
            hour: self.hour,
            minute: self.minute,
            second: next_second,
            millisecond: self.millisecond,
        }
    }

    /// Returns the meridiem [`DayPeriod`] (`Am` for `0..=11`, `Pm` for `12..=23`).
    #[inline]
    pub const fn day_period(&self) -> DayPeriod {
        if self.hour < 12 {
            DayPeriod::Am
        } else {
            DayPeriod::Pm
        }
    }

    /// Toggles between `AM` and `PM` by shifting `hour` by 12 modulo 24.
    #[inline]
    pub const fn toggle_day_period(&self) -> Self {
        Self {
            hour: (self.hour + 12) % 24,
            minute: self.minute,
            second: self.second,
            millisecond: self.millisecond,
        }
    }

    /// Sets the meridiem [`DayPeriod`] while preserving the 12-hour offset.
    #[inline]
    pub const fn with_day_period(&self, period: DayPeriod) -> Self {
        let offset = self.hour % 12;
        let next_hour = match period {
            DayPeriod::Am => offset,
            DayPeriod::Pm => offset + 12,
        };
        Self {
            hour: next_hour,
            minute: self.minute,
            second: self.second,
            millisecond: self.millisecond,
        }
    }

    /// Returns the 12-hour display hour (`1..=12`) and [`DayPeriod`].
    #[inline]
    pub const fn hour_12(&self) -> (u8, DayPeriod) {
        (hour_12_display(self.hour), self.day_period())
    }

    /// Returns the 12-hour display hour (`1..=12`).
    #[inline]
    pub const fn hour_12_display(&self) -> u8 {
        hour_12_display(self.hour)
    }

    /// Converts a 12-hour display hour (`1..=12`) and [`DayPeriod`] into a 24-hour value (`0..=23`).
    #[inline]
    pub const fn from_12h(hour_12: u8, period: DayPeriod) -> Option<u8> {
        hour_12_to_24(hour_12, period)
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

impl FromStr for CalendarTime {
    type Err = ();

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        Self::from_iso_str(s).ok_or(())
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

    /// Parses an ISO-8601 `YYYY-MM-DDTHH:MM[:SS[.mmm]]` (or space-separated) string.
    pub fn from_iso_str(raw: &str) -> Option<Self> {
        let trimmed = raw.trim();
        let (date_str, time_str) = trimmed
            .split_once('T')
            .or_else(|| trimmed.split_once('t'))
            .or_else(|| trimmed.split_once(' '))?;
        let date = CalendarDate::from_iso_str(date_str)?;
        let time = CalendarTime::from_iso_str(time_str)?;
        Some(Self { date, time })
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

    /// Clamps `self` into the optional `[min, max]` inclusive interval.
    #[inline]
    pub fn clamp(self, min: Option<Self>, max: Option<Self>) -> Self {
        let mut out = self;
        if let Some(min_dt) = min
            && out < min_dt
        {
            out = min_dt;
        }
        if let Some(max_dt) = max
            && out > max_dt
        {
            out = max_dt;
        }
        out
    }

    /// Shifts the date component by `delta` days while preserving wall-clock time.
    #[inline]
    pub const fn add_days(&self, delta: i64) -> Self {
        Self {
            date: self.date.add_days(delta),
            time: self.time,
        }
    }

    /// Shifts the date component by `delta` months (clamping day) while preserving wall-clock time.
    #[inline]
    pub const fn add_months_clamped(&self, delta: i32) -> Self {
        Self {
            date: self.date.add_months_clamped(delta),
            time: self.time,
        }
    }

    /// Shifts the date component by `delta` years (clamping day) while preserving wall-clock time.
    #[inline]
    pub const fn add_years_clamped(&self, delta: i32) -> Self {
        Self {
            date: self.date.add_years_clamped(delta),
            time: self.time,
        }
    }

    /// Shifts `self` by `delta` hours, carrying across day boundaries.
    #[inline]
    pub const fn add_hours(&self, delta: i64) -> Self {
        self.add_milliseconds(delta.saturating_mul(MILLIS_PER_HOUR))
    }

    /// Shifts `self` by `delta` minutes, carrying across day boundaries.
    #[inline]
    pub const fn add_minutes(&self, delta: i64) -> Self {
        self.add_milliseconds(delta.saturating_mul(MILLIS_PER_MINUTE))
    }

    /// Shifts `self` by `delta` seconds, carrying across day boundaries.
    #[inline]
    pub const fn add_seconds(&self, delta: i64) -> Self {
        self.add_milliseconds(delta.saturating_mul(MILLIS_PER_SECOND))
    }

    #[inline]
    const fn add_milliseconds(&self, delta_ms: i64) -> Self {
        let total = self.time.total_milliseconds() as i64 + delta_ms;
        let day_carry = total.div_euclid(MILLIS_PER_DAY);
        let time = CalendarTime::from_total_milliseconds_wrapped(total);
        Self {
            date: self.date.add_days(day_carry),
            time,
        }
    }

    /// Cycles `year` without carrying, clamping into `1..=9999` and clamping `day`.
    #[inline]
    pub const fn cycle_year(&self, delta: i32) -> Self {
        Self {
            date: self.date.cycle_year(delta),
            time: self.time,
        }
    }

    /// Cycles `month` within `1..=12` without carrying into `year`.
    #[inline]
    pub const fn cycle_month(&self, delta: i32) -> Self {
        Self {
            date: self.date.cycle_month(delta),
            time: self.time,
        }
    }

    /// Cycles `day` within `1..=days_in_month` without carrying into `month`.
    #[inline]
    pub const fn cycle_day(&self, delta: i32) -> Self {
        Self {
            date: self.date.cycle_day(delta),
            time: self.time,
        }
    }

    /// Cycles `hour` without carrying into `date` (or across `AM`/`PM` in 12-hour mode).
    #[inline]
    pub const fn cycle_hour(&self, delta: i32, hour_cycle: HourCycle) -> Self {
        Self {
            date: self.date,
            time: self.time.cycle_hour(delta, hour_cycle),
        }
    }

    /// Cycles `minute` within `0..=59` without carrying into `hour`.
    #[inline]
    pub const fn cycle_minute(&self, delta: i32) -> Self {
        Self {
            date: self.date,
            time: self.time.cycle_minute(delta),
        }
    }

    /// Cycles `second` within `0..=59` without carrying into `minute`.
    #[inline]
    pub const fn cycle_second(&self, delta: i32) -> Self {
        Self {
            date: self.date,
            time: self.time.cycle_second(delta),
        }
    }

    /// Toggles the wall-clock meridiem (`AM <-> PM`) without changing the date.
    #[inline]
    pub const fn toggle_day_period(&self) -> Self {
        Self {
            date: self.date,
            time: self.time.toggle_day_period(),
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

impl FromStr for CalendarDateTime {
    type Err = ();

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        Self::from_iso_str(s).ok_or(())
    }
}

/// Unified date or date-time value for segmented date fields and date pickers.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub enum DateValue {
    Date(CalendarDate),
    DateTime(CalendarDateTime),
}

impl Default for DateValue {
    fn default() -> Self {
        Self::Date(CalendarDate::default())
    }
}

impl DateValue {
    /// Returns the underlying [`CalendarDate`].
    #[inline]
    pub const fn date(&self) -> CalendarDate {
        match self {
            Self::Date(d) => *d,
            Self::DateTime(dt) => dt.date,
        }
    }

    /// Alias for [`Self::date`].
    #[inline]
    pub const fn as_calendar_date(&self) -> CalendarDate {
        self.date()
    }

    /// Returns the wall-clock [`CalendarTime`] when this is a [`DateValue::DateTime`].
    #[inline]
    pub const fn time(&self) -> Option<CalendarTime> {
        match self {
            Self::Date(_) => None,
            Self::DateTime(dt) => Some(dt.time),
        }
    }

    /// Alias for [`Self::time`].
    #[inline]
    pub const fn as_calendar_time(&self) -> Option<CalendarTime> {
        self.time()
    }

    /// Returns `true` when this value carries a time component ([`DateValue::DateTime`]).
    #[inline]
    pub const fn has_time(&self) -> bool {
        matches!(self, Self::DateTime(_))
    }

    /// Promotes `Date` to `CalendarDateTime` at midnight or returns the inner `CalendarDateTime`.
    #[inline]
    pub const fn to_date_time(&self) -> CalendarDateTime {
        match self {
            Self::Date(d) => CalendarDateTime::from_date(*d),
            Self::DateTime(dt) => *dt,
        }
    }

    /// Replaces the date component while preserving whether a time component is present.
    #[inline]
    pub const fn with_date(&self, date: CalendarDate) -> Self {
        match self {
            Self::Date(_) => Self::Date(date),
            Self::DateTime(dt) => Self::DateTime(dt.with_date(date)),
        }
    }

    /// Sets a time component, promoting `Date` to `DateTime`.
    #[inline]
    pub const fn with_time(&self, time: CalendarTime) -> Self {
        Self::DateTime(CalendarDateTime::new(self.date(), time))
    }

    /// Parses an ISO-8601 date (`YYYY-MM-DD`) or date-time (`YYYY-MM-DDTHH:MM[:SS[.mmm]]`).
    pub fn from_iso_str(raw: &str) -> Option<Self> {
        let trimmed = raw.trim();
        if trimmed.contains('T') || trimmed.contains('t') || trimmed.contains(' ') {
            CalendarDateTime::from_iso_str(trimmed).map(Self::DateTime)
        } else {
            CalendarDate::from_iso_str(trimmed).map(Self::Date)
        }
    }

    /// Formats as ISO-8601 (`YYYY-MM-DD` or `YYYY-MM-DDTHH:MM:SS(.mmm)`).
    pub fn to_iso_string(&self) -> String {
        match self {
            Self::Date(d) => d.to_iso_string(),
            Self::DateTime(dt) => dt.to_iso_string(),
        }
    }

    /// Compares two [`DateValue`]s chronologically, treating `Date(d)` as midnight on `d`.
    #[inline]
    pub fn compare_chronological(&self, other: &Self) -> Ordering {
        self.to_date_time().cmp(&other.to_date_time())
    }

    /// Clamps `self` into the optional `[min, max]` inclusive interval.
    #[inline]
    pub fn clamp(self, min: Option<Self>, max: Option<Self>) -> Self {
        let mut out = self;
        if let Some(min_val) = min
            && out < min_val
        {
            out = min_val;
        }
        if let Some(max_val) = max
            && out > max_val
        {
            out = max_val;
        }
        out
    }
}

impl Ord for DateValue {
    fn cmp(&self, other: &Self) -> Ordering {
        self.to_date_time()
            .cmp(&other.to_date_time())
            .then_with(|| self.has_time().cmp(&other.has_time()))
    }
}

impl PartialOrd for DateValue {
    fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
        Some(self.cmp(other))
    }
}

impl fmt::Display for DateValue {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(&self.to_iso_string())
    }
}

impl FromStr for DateValue {
    type Err = ();

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        Self::from_iso_str(s).ok_or(())
    }
}

impl From<CalendarDate> for DateValue {
    #[inline]
    fn from(date: CalendarDate) -> Self {
        Self::Date(date)
    }
}

impl From<CalendarDateTime> for DateValue {
    #[inline]
    fn from(dt: CalendarDateTime) -> Self {
        Self::DateTime(dt)
    }
}

/// Unified time or date-time value for segmented time fields.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub enum TimeValue {
    Time(CalendarTime),
    DateTime(CalendarDateTime),
}

impl Default for TimeValue {
    fn default() -> Self {
        Self::Time(CalendarTime::midnight())
    }
}

impl TimeValue {
    /// Returns the underlying wall-clock [`CalendarTime`].
    #[inline]
    pub const fn time(&self) -> CalendarTime {
        match self {
            Self::Time(t) => *t,
            Self::DateTime(dt) => dt.time,
        }
    }

    /// Alias for [`Self::time`].
    #[inline]
    pub const fn as_calendar_time(&self) -> CalendarTime {
        self.time()
    }

    /// Returns the [`CalendarDate`] component when this is a [`TimeValue::DateTime`].
    #[inline]
    pub const fn date(&self) -> Option<CalendarDate> {
        match self {
            Self::Time(_) => None,
            Self::DateTime(dt) => Some(dt.date),
        }
    }

    /// Alias for [`Self::date`].
    #[inline]
    pub const fn as_calendar_date(&self) -> Option<CalendarDate> {
        self.date()
    }

    /// Replaces the time component while preserving any attached date.
    #[inline]
    pub const fn with_time(&self, time: CalendarTime) -> Self {
        match self {
            Self::Time(_) => Self::Time(time),
            Self::DateTime(dt) => Self::DateTime(dt.with_time(time)),
        }
    }

    /// Parses an ISO-8601 time (`HH:MM[:SS[.mmm]]`) or date-time (`YYYY-MM-DDTHH:MM[:SS[.mmm]]`).
    pub fn from_iso_str(raw: &str) -> Option<Self> {
        let trimmed = raw.trim();
        if trimmed.contains('T') || trimmed.contains('t') || trimmed.contains(' ') {
            CalendarDateTime::from_iso_str(trimmed).map(Self::DateTime)
        } else {
            CalendarTime::from_iso_str(trimmed).map(Self::Time)
        }
    }

    /// Formats as ISO-8601 (`HH:MM:SS(.mmm)` or `YYYY-MM-DDTHH:MM:SS(.mmm)`).
    pub fn to_iso_string(&self) -> String {
        match self {
            Self::Time(t) => t.to_iso_string(),
            Self::DateTime(dt) => dt.to_iso_string(),
        }
    }

    /// Clamps `self` into the optional `[min, max]` inclusive interval.
    #[inline]
    pub fn clamp(self, min: Option<Self>, max: Option<Self>) -> Self {
        let mut out = self;
        if let Some(min_val) = min
            && out < min_val
        {
            out = min_val;
        }
        if let Some(max_val) = max
            && out > max_val
        {
            out = max_val;
        }
        out
    }
}

impl Ord for TimeValue {
    fn cmp(&self, other: &Self) -> Ordering {
        self.time()
            .cmp(&other.time())
            .then_with(|| self.date().cmp(&other.date()))
    }
}

impl PartialOrd for TimeValue {
    fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
        Some(self.cmp(other))
    }
}

impl fmt::Display for TimeValue {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(&self.to_iso_string())
    }
}

impl FromStr for TimeValue {
    type Err = ();

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        Self::from_iso_str(s).ok_or(())
    }
}

impl From<CalendarTime> for TimeValue {
    #[inline]
    fn from(time: CalendarTime) -> Self {
        Self::Time(time)
    }
}

impl From<CalendarDateTime> for TimeValue {
    #[inline]
    fn from(dt: CalendarDateTime) -> Self {
        Self::DateTime(dt)
    }
}

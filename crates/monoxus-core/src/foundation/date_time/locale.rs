use std::borrow::Cow;

use super::{
    date::{CalendarDate, DayPeriod, Granularity, HourCycle, TimeGranularity, Weekday},
    field::SegmentKind,
};

/// Weekday column header display width (`narrow` by default, matching `bits-ui`).
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, Hash)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub enum WeekdayFormat {
    #[default]
    Narrow,
    Short,
    Long,
}

/// Month display format for calendar headings and month selects.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, Hash)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub enum MonthFormat {
    #[default]
    Long,
    Short,
    Narrow,
    Numeric,
    TwoDigit,
}

/// Year display format for calendar headings and year selects.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, Hash)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub enum YearFormat {
    #[default]
    Numeric,
    TwoDigit,
}

/// Ordering style for month + year headings and full date labels.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, Hash)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub enum CalendarDateOrder {
    /// Month before year (e.g., `"January 2024"`, `"Monday, January 1, 2024"`).
    #[default]
    MonthFirst,
    /// Year before month (e.g., `"2024년 1월"`, `"2024년 1월 1일 월요일"`).
    YearFirst,
}

/// Pure-Rust localization table for calendar weekday headers, month names, headings,
/// day `aria-label` values, and live-region announcements.
#[derive(Clone, Debug, PartialEq, Eq)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub struct CalendarLabels {
    pub week_starts_on: Weekday,
    pub date_order: CalendarDateOrder,
    pub months_long: [Cow<'static, str>; 12],
    pub months_short: [Cow<'static, str>; 12],
    pub months_narrow: [Cow<'static, str>; 12],
    pub weekdays_long: [Cow<'static, str>; 7],
    pub weekdays_short: [Cow<'static, str>; 7],
    pub weekdays_narrow: [Cow<'static, str>; 7],
    pub year_suffix: Cow<'static, str>,
    pub day_suffix: Cow<'static, str>,
    pub selected_prefix: Cow<'static, str>,
    pub range_prefix: Cow<'static, str>,
    pub range_separator: Cow<'static, str>,
    pub empty_announcement: Cow<'static, str>,
}

impl CalendarLabels {
    /// Built-in `en-US` labels matching `Intl.DateTimeFormat("en-US", ...)` outputs.
    pub fn en_us() -> Self {
        Self {
            week_starts_on: Weekday::Sunday,
            date_order: CalendarDateOrder::MonthFirst,
            months_long: [
                Cow::Borrowed("January"),
                Cow::Borrowed("February"),
                Cow::Borrowed("March"),
                Cow::Borrowed("April"),
                Cow::Borrowed("May"),
                Cow::Borrowed("June"),
                Cow::Borrowed("July"),
                Cow::Borrowed("August"),
                Cow::Borrowed("September"),
                Cow::Borrowed("October"),
                Cow::Borrowed("November"),
                Cow::Borrowed("December"),
            ],
            months_short: [
                Cow::Borrowed("Jan"),
                Cow::Borrowed("Feb"),
                Cow::Borrowed("Mar"),
                Cow::Borrowed("Apr"),
                Cow::Borrowed("May"),
                Cow::Borrowed("Jun"),
                Cow::Borrowed("Jul"),
                Cow::Borrowed("Aug"),
                Cow::Borrowed("Sep"),
                Cow::Borrowed("Oct"),
                Cow::Borrowed("Nov"),
                Cow::Borrowed("Dec"),
            ],
            months_narrow: [
                Cow::Borrowed("J"),
                Cow::Borrowed("F"),
                Cow::Borrowed("M"),
                Cow::Borrowed("A"),
                Cow::Borrowed("M"),
                Cow::Borrowed("J"),
                Cow::Borrowed("J"),
                Cow::Borrowed("A"),
                Cow::Borrowed("S"),
                Cow::Borrowed("O"),
                Cow::Borrowed("N"),
                Cow::Borrowed("D"),
            ],
            weekdays_long: [
                Cow::Borrowed("Sunday"),
                Cow::Borrowed("Monday"),
                Cow::Borrowed("Tuesday"),
                Cow::Borrowed("Wednesday"),
                Cow::Borrowed("Thursday"),
                Cow::Borrowed("Friday"),
                Cow::Borrowed("Saturday"),
            ],
            weekdays_short: [
                Cow::Borrowed("Sun"),
                Cow::Borrowed("Mon"),
                Cow::Borrowed("Tue"),
                Cow::Borrowed("Wed"),
                Cow::Borrowed("Thu"),
                Cow::Borrowed("Fri"),
                Cow::Borrowed("Sat"),
            ],
            weekdays_narrow: [
                Cow::Borrowed("S"),
                Cow::Borrowed("M"),
                Cow::Borrowed("T"),
                Cow::Borrowed("W"),
                Cow::Borrowed("T"),
                Cow::Borrowed("F"),
                Cow::Borrowed("S"),
            ],
            year_suffix: Cow::Borrowed(""),
            day_suffix: Cow::Borrowed(""),
            selected_prefix: Cow::Borrowed("Selected Date: "),
            range_prefix: Cow::Borrowed("Selected Dates: "),
            range_separator: Cow::Borrowed(" to "),
            empty_announcement: Cow::Borrowed("Selected date is now empty."),
        }
    }

    /// Built-in `ko-KR` labels matching `Intl.DateTimeFormat("ko-KR", ...)` outputs.
    pub fn ko_kr() -> Self {
        Self {
            week_starts_on: Weekday::Sunday,
            date_order: CalendarDateOrder::YearFirst,
            months_long: [
                Cow::Borrowed("1월"),
                Cow::Borrowed("2월"),
                Cow::Borrowed("3월"),
                Cow::Borrowed("4월"),
                Cow::Borrowed("5월"),
                Cow::Borrowed("6월"),
                Cow::Borrowed("7월"),
                Cow::Borrowed("8월"),
                Cow::Borrowed("9월"),
                Cow::Borrowed("10월"),
                Cow::Borrowed("11월"),
                Cow::Borrowed("12월"),
            ],
            months_short: [
                Cow::Borrowed("1월"),
                Cow::Borrowed("2월"),
                Cow::Borrowed("3월"),
                Cow::Borrowed("4월"),
                Cow::Borrowed("5월"),
                Cow::Borrowed("6월"),
                Cow::Borrowed("7월"),
                Cow::Borrowed("8월"),
                Cow::Borrowed("9월"),
                Cow::Borrowed("10월"),
                Cow::Borrowed("11월"),
                Cow::Borrowed("12월"),
            ],
            months_narrow: [
                Cow::Borrowed("1월"),
                Cow::Borrowed("2월"),
                Cow::Borrowed("3월"),
                Cow::Borrowed("4월"),
                Cow::Borrowed("5월"),
                Cow::Borrowed("6월"),
                Cow::Borrowed("7월"),
                Cow::Borrowed("8월"),
                Cow::Borrowed("9월"),
                Cow::Borrowed("10월"),
                Cow::Borrowed("11월"),
                Cow::Borrowed("12월"),
            ],
            weekdays_long: [
                Cow::Borrowed("일요일"),
                Cow::Borrowed("월요일"),
                Cow::Borrowed("화요일"),
                Cow::Borrowed("수요일"),
                Cow::Borrowed("목요일"),
                Cow::Borrowed("금요일"),
                Cow::Borrowed("토요일"),
            ],
            weekdays_short: [
                Cow::Borrowed("일"),
                Cow::Borrowed("월"),
                Cow::Borrowed("화"),
                Cow::Borrowed("수"),
                Cow::Borrowed("목"),
                Cow::Borrowed("금"),
                Cow::Borrowed("토"),
            ],
            weekdays_narrow: [
                Cow::Borrowed("일"),
                Cow::Borrowed("월"),
                Cow::Borrowed("화"),
                Cow::Borrowed("수"),
                Cow::Borrowed("목"),
                Cow::Borrowed("금"),
                Cow::Borrowed("토"),
            ],
            year_suffix: Cow::Borrowed("년"),
            day_suffix: Cow::Borrowed("일"),
            selected_prefix: Cow::Borrowed("Selected Date: "),
            range_prefix: Cow::Borrowed("Selected Dates: "),
            range_separator: Cow::Borrowed(" to "),
            empty_announcement: Cow::Borrowed("Selected date is now empty."),
        }
    }

    /// Built-in `en-GB` labels (Monday week start, English month/weekday names).
    pub fn en_gb() -> Self {
        let mut labels = Self::en_us();
        labels.week_starts_on = Weekday::Monday;
        labels
    }

    /// Built-in ISO labels (Monday week start, year-first ordering, English month/weekday names).
    pub fn iso() -> Self {
        let mut labels = Self::en_us();
        labels.week_starts_on = Weekday::Monday;
        labels.date_order = CalendarDateOrder::YearFirst;
        labels
    }
}

impl Default for CalendarLabels {
    fn default() -> Self {
        Self::en_us()
    }
}

/// Pure-Rust calendar locale configuration (`en-US`, `en-GB`, `ko-KR`, `iso`, or consumer-injected [`CalendarLabels`]).
#[derive(Clone, Debug, Default, PartialEq, Eq)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub enum CalendarLocale {
    #[default]
    EnUs,
    EnGb,
    KoKr,
    Iso,
    Custom(CalendarLabels),
}

impl CalendarLocale {
    /// Parses common BCP-47 tags (`"en-US"`, `"en-GB"`, `"ko-KR"`, `"iso"`) or falls back to `EnUs`.
    pub fn from_tag(tag: &str) -> Self {
        let normalized = tag.trim().to_ascii_lowercase();
        if normalized == "ko" || normalized.starts_with("ko-") {
            Self::KoKr
        } else if normalized == "en-gb" || normalized.starts_with("en-gb-") {
            Self::EnGb
        } else if normalized == "iso" || normalized == "sv-se" {
            Self::Iso
        } else {
            Self::EnUs
        }
    }

    /// Resolves the [`CalendarLabels`] table for this locale.
    pub fn labels(&self) -> Cow<'_, CalendarLabels> {
        match self {
            Self::EnUs => Cow::Owned(CalendarLabels::en_us()),
            Self::EnGb => Cow::Owned(CalendarLabels::en_gb()),
            Self::KoKr => Cow::Owned(CalendarLabels::ko_kr()),
            Self::Iso => Cow::Owned(CalendarLabels::iso()),
            Self::Custom(labels) => Cow::Borrowed(labels),
        }
    }

    /// Resolves the pure-Rust [`FieldLocaleFormat`] for segmented date/time fields.
    pub fn field_format(&self) -> FieldLocaleFormat {
        FieldLocaleFormat::from_calendar_locale(self)
    }

    /// Resolves the owned [`FieldLocaleTemplate`] for segmented date/time fields.
    pub fn field_template(&self) -> FieldLocaleTemplate {
        self.field_format().template().into_owned()
    }

    /// Returns the locale's default first day of week.
    pub fn default_week_starts_on(&self) -> Weekday {
        self.labels().week_starts_on
    }

    /// Formats a weekday according to `format`.
    pub fn weekday_label(&self, weekday: Weekday, format: WeekdayFormat) -> String {
        let idx = weekday.to_u8() as usize;
        let labels = self.labels();
        match format {
            WeekdayFormat::Long => labels.weekdays_long[idx].to_string(),
            WeekdayFormat::Short => labels.weekdays_short[idx].to_string(),
            WeekdayFormat::Narrow => labels.weekdays_narrow[idx].to_string(),
        }
    }

    /// Formats a 1-based month (`1..=12`) according to `format`.
    pub fn month_label(&self, month: u8, format: MonthFormat) -> String {
        let clamped = month.clamp(1, 12);
        let idx = (clamped - 1) as usize;
        let labels = self.labels();
        match format {
            MonthFormat::Long => labels.months_long[idx].to_string(),
            MonthFormat::Short => labels.months_short[idx].to_string(),
            MonthFormat::Narrow => labels.months_narrow[idx].to_string(),
            MonthFormat::Numeric => clamped.to_string(),
            MonthFormat::TwoDigit => format!("{clamped:02}"),
        }
    }

    /// Formats a year according to `format`.
    pub fn year_label(&self, year: i32, format: YearFormat) -> String {
        match format {
            YearFormat::Numeric => year.to_string(),
            YearFormat::TwoDigit => format!("{:02}", year.rem_euclid(100)),
        }
    }

    /// Formats the calendar heading across one or more visible month dates.
    pub fn heading_label(
        &self,
        months: &[CalendarDate],
        month_format: MonthFormat,
        year_format: YearFormat,
    ) -> String {
        let Some(first) = months.first().copied() else {
            return String::new();
        };
        let labels = self.labels();
        if months.len() == 1 {
            let month_text = self.month_label(first.month, month_format);
            let year_text = self.year_label(first.year, year_format);
            return match labels.date_order {
                CalendarDateOrder::MonthFirst => format!("{month_text} {year_text}"),
                CalendarDateOrder::YearFirst => {
                    format!("{year_text}{} {month_text}", labels.year_suffix)
                }
            };
        }

        let last = months.last().copied().unwrap_or(first);
        let start_month = self.month_label(first.month, MonthFormat::Long);
        let end_month = self.month_label(last.month, MonthFormat::Long);
        let start_year = self.year_label(first.year, YearFormat::Numeric);
        let end_year = self.year_label(last.year, YearFormat::Numeric);

        if first.year == last.year {
            match labels.date_order {
                CalendarDateOrder::MonthFirst => {
                    format!("{start_month} - {end_month} {end_year}")
                }
                CalendarDateOrder::YearFirst => {
                    format!(
                        "{start_year}{} {start_month} - {end_month}",
                        labels.year_suffix
                    )
                }
            }
        } else {
            match labels.date_order {
                CalendarDateOrder::MonthFirst => {
                    format!("{start_month} {start_year} - {end_month} {end_year}")
                }
                CalendarDateOrder::YearFirst => {
                    format!(
                        "{start_year}{} {start_month} - {end_year}{} {end_month}",
                        labels.year_suffix, labels.year_suffix
                    )
                }
            }
        }
    }

    /// Formats the long `dateStyle: "long"` date string (e.g., `"January 1, 2024"` or `"2024년 1월 1일"`).
    pub fn long_date_label(&self, date: CalendarDate) -> String {
        let labels = self.labels();
        let month = self.month_label(date.month, MonthFormat::Long);
        match labels.date_order {
            CalendarDateOrder::MonthFirst => format!("{month} {}, {}", date.day, date.year),
            CalendarDateOrder::YearFirst => format!(
                "{}{} {month} {}{}",
                date.year, labels.year_suffix, date.day, labels.day_suffix
            ),
        }
    }

    /// Formats the accessible `aria-label` for a day cell (`weekday: long, month: long, day: numeric, year: numeric`).
    pub fn cell_aria_label(&self, date: CalendarDate) -> String {
        let labels = self.labels();
        let weekday = self.weekday_label(date.weekday(), WeekdayFormat::Long);
        let month = self.month_label(date.month, MonthFormat::Long);
        match labels.date_order {
            CalendarDateOrder::MonthFirst => {
                format!("{weekday}, {month} {}, {}", date.day, date.year)
            }
            CalendarDateOrder::YearFirst => format!(
                "{}{} {month} {}{} {weekday}",
                date.year, labels.year_suffix, date.day, labels.day_suffix
            ),
        }
    }

    /// Formats the live-region announcement for selecting a single date.
    pub fn selected_announcement(&self, date: CalendarDate) -> String {
        let labels = self.labels();
        format!("{}{}", labels.selected_prefix, self.long_date_label(date))
    }

    /// Formats the live-region announcement for completing a date range.
    pub fn range_announcement(&self, start: CalendarDate, end: CalendarDate) -> String {
        let labels = self.labels();
        format!(
            "{}{}{}{}",
            labels.range_prefix,
            self.long_date_label(start),
            labels.range_separator,
            self.long_date_label(end)
        )
    }

    /// Formats the live-region announcement when the selection becomes empty.
    pub fn empty_announcement(&self) -> String {
        self.labels().empty_announcement.to_string()
    }
}

/// A single token in a pure-Rust date/time field segment template.
#[derive(Clone, Debug, PartialEq, Eq, Hash)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub enum FieldTemplateToken {
    Segment(SegmentKind),
    Literal(Cow<'static, str>),
}

/// Localized placeholder strings for empty date/time spinbutton segments.
#[derive(Clone, Debug, PartialEq, Eq)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub struct SegmentPlaceholders {
    pub year: Cow<'static, str>,
    pub month: Cow<'static, str>,
    pub day: Cow<'static, str>,
    pub hour: Cow<'static, str>,
    pub minute: Cow<'static, str>,
    pub second: Cow<'static, str>,
    pub day_period: Cow<'static, str>,
    pub time_zone_name: Cow<'static, str>,
}

impl SegmentPlaceholders {
    /// English / ISO placeholders (`yyyy`, `mm`, `dd`, `––`, `AM`, `UTC`).
    pub fn en_us() -> Self {
        Self {
            year: Cow::Borrowed("yyyy"),
            month: Cow::Borrowed("mm"),
            day: Cow::Borrowed("dd"),
            hour: Cow::Borrowed("––"),
            minute: Cow::Borrowed("––"),
            second: Cow::Borrowed("––"),
            day_period: Cow::Borrowed("AM"),
            time_zone_name: Cow::Borrowed("UTC"),
        }
    }

    /// Korean placeholders (`연도`, `월`, `일`, `––`, `오전`, `UTC`).
    pub fn ko_kr() -> Self {
        Self {
            year: Cow::Borrowed("연도"),
            month: Cow::Borrowed("월"),
            day: Cow::Borrowed("일"),
            hour: Cow::Borrowed("––"),
            minute: Cow::Borrowed("––"),
            second: Cow::Borrowed("––"),
            day_period: Cow::Borrowed("오전"),
            time_zone_name: Cow::Borrowed("UTC"),
        }
    }

    /// Returns the placeholder text for a given [`SegmentKind`].
    pub fn for_kind(&self, kind: SegmentKind) -> &str {
        match kind {
            SegmentKind::Year => self.year.as_ref(),
            SegmentKind::Month => self.month.as_ref(),
            SegmentKind::Day => self.day.as_ref(),
            SegmentKind::Hour => self.hour.as_ref(),
            SegmentKind::Minute => self.minute.as_ref(),
            SegmentKind::Second => self.second.as_ref(),
            SegmentKind::DayPeriod => self.day_period.as_ref(),
            SegmentKind::TimeZoneName => self.time_zone_name.as_ref(),
            SegmentKind::Literal => "",
        }
    }
}

impl Default for SegmentPlaceholders {
    fn default() -> Self {
        Self::en_us()
    }
}

/// Localized `aria-label` names for date/time spinbutton segments.
#[derive(Clone, Debug, PartialEq, Eq)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub struct SegmentAriaLabels {
    pub year: Cow<'static, str>,
    pub month: Cow<'static, str>,
    pub day: Cow<'static, str>,
    pub hour: Cow<'static, str>,
    pub minute: Cow<'static, str>,
    pub second: Cow<'static, str>,
    pub day_period: Cow<'static, str>,
    pub time_zone_name: Cow<'static, str>,
}

impl SegmentAriaLabels {
    /// English `aria-label` names (`year`, `month`, `day`, `hour`, `minute`, `second`, `AM/PM`, `timezone`).
    pub fn en_us() -> Self {
        Self {
            year: Cow::Borrowed("year"),
            month: Cow::Borrowed("month"),
            day: Cow::Borrowed("day"),
            hour: Cow::Borrowed("hour"),
            minute: Cow::Borrowed("minute"),
            second: Cow::Borrowed("second"),
            day_period: Cow::Borrowed("AM/PM"),
            time_zone_name: Cow::Borrowed("timezone"),
        }
    }

    /// Korean `aria-label` names (`연도`, `월`, `일`, `시`, `분`, `초`, `오전/오후`, `시간대`).
    pub fn ko_kr() -> Self {
        Self {
            year: Cow::Borrowed("연도"),
            month: Cow::Borrowed("월"),
            day: Cow::Borrowed("일"),
            hour: Cow::Borrowed("시"),
            minute: Cow::Borrowed("분"),
            second: Cow::Borrowed("초"),
            day_period: Cow::Borrowed("오전/오후"),
            time_zone_name: Cow::Borrowed("시간대"),
        }
    }

    /// Returns the localized `aria-label` for a given [`SegmentKind`].
    pub fn for_kind(&self, kind: SegmentKind) -> &str {
        match kind {
            SegmentKind::Year => self.year.as_ref(),
            SegmentKind::Month => self.month.as_ref(),
            SegmentKind::Day => self.day.as_ref(),
            SegmentKind::Hour => self.hour.as_ref(),
            SegmentKind::Minute => self.minute.as_ref(),
            SegmentKind::Second => self.second.as_ref(),
            SegmentKind::DayPeriod => self.day_period.as_ref(),
            SegmentKind::TimeZoneName => self.time_zone_name.as_ref(),
            SegmentKind::Literal => "",
        }
    }
}

impl Default for SegmentAriaLabels {
    fn default() -> Self {
        Self::en_us()
    }
}

/// Pure-Rust segmented date/time locale template (zero JS `Intl`).
#[derive(Clone, Debug, PartialEq, Eq)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub struct FieldLocaleTemplate {
    pub default_hour_cycle: HourCycle,
    pub date_tokens: Vec<FieldTemplateToken>,
    pub time_12h_tokens: Vec<FieldTemplateToken>,
    pub time_24h_tokens: Vec<FieldTemplateToken>,
    pub date_time_separator: Cow<'static, str>,
    pub placeholders: SegmentPlaceholders,
    pub aria_labels: SegmentAriaLabels,
    pub am_label: Cow<'static, str>,
    pub pm_label: Cow<'static, str>,
    pub empty_value_text: Cow<'static, str>,
    pub selected_time_prefix: Cow<'static, str>,
}

impl FieldLocaleTemplate {
    /// Built-in `en-US` template (`MM/DD/YYYY`, 12-hour `hh:mm[:ss] AM/PM`).
    pub fn en_us() -> Self {
        Self {
            default_hour_cycle: HourCycle::H12,
            date_tokens: vec![
                FieldTemplateToken::Segment(SegmentKind::Month),
                FieldTemplateToken::Literal(Cow::Borrowed("/")),
                FieldTemplateToken::Segment(SegmentKind::Day),
                FieldTemplateToken::Literal(Cow::Borrowed("/")),
                FieldTemplateToken::Segment(SegmentKind::Year),
            ],
            time_12h_tokens: vec![
                FieldTemplateToken::Segment(SegmentKind::Hour),
                FieldTemplateToken::Literal(Cow::Borrowed(":")),
                FieldTemplateToken::Segment(SegmentKind::Minute),
                FieldTemplateToken::Literal(Cow::Borrowed(":")),
                FieldTemplateToken::Segment(SegmentKind::Second),
                FieldTemplateToken::Literal(Cow::Borrowed(" ")),
                FieldTemplateToken::Segment(SegmentKind::DayPeriod),
            ],
            time_24h_tokens: vec![
                FieldTemplateToken::Segment(SegmentKind::Hour),
                FieldTemplateToken::Literal(Cow::Borrowed(":")),
                FieldTemplateToken::Segment(SegmentKind::Minute),
                FieldTemplateToken::Literal(Cow::Borrowed(":")),
                FieldTemplateToken::Segment(SegmentKind::Second),
            ],
            date_time_separator: Cow::Borrowed(", "),
            placeholders: SegmentPlaceholders::en_us(),
            aria_labels: SegmentAriaLabels::en_us(),
            am_label: Cow::Borrowed("AM"),
            pm_label: Cow::Borrowed("PM"),
            empty_value_text: Cow::Borrowed("Empty"),
            selected_time_prefix: Cow::Borrowed("Selected Time: "),
        }
    }

    /// Built-in `en-GB` template (`DD/MM/YYYY`, 24-hour `HH:mm[:ss]`).
    pub fn en_gb() -> Self {
        let mut tpl = Self::en_us();
        tpl.default_hour_cycle = HourCycle::H24;
        tpl.date_tokens = vec![
            FieldTemplateToken::Segment(SegmentKind::Day),
            FieldTemplateToken::Literal(Cow::Borrowed("/")),
            FieldTemplateToken::Segment(SegmentKind::Month),
            FieldTemplateToken::Literal(Cow::Borrowed("/")),
            FieldTemplateToken::Segment(SegmentKind::Year),
        ];
        tpl
    }

    /// Built-in `ko-KR` template (`YYYY. MM. DD.`, 12-hour `오전/오후 hh:mm[:ss]`).
    pub fn ko_kr() -> Self {
        Self {
            default_hour_cycle: HourCycle::H12,
            date_tokens: vec![
                FieldTemplateToken::Segment(SegmentKind::Year),
                FieldTemplateToken::Literal(Cow::Borrowed(". ")),
                FieldTemplateToken::Segment(SegmentKind::Month),
                FieldTemplateToken::Literal(Cow::Borrowed(". ")),
                FieldTemplateToken::Segment(SegmentKind::Day),
                FieldTemplateToken::Literal(Cow::Borrowed(".")),
            ],
            time_12h_tokens: vec![
                FieldTemplateToken::Segment(SegmentKind::DayPeriod),
                FieldTemplateToken::Literal(Cow::Borrowed(" ")),
                FieldTemplateToken::Segment(SegmentKind::Hour),
                FieldTemplateToken::Literal(Cow::Borrowed(":")),
                FieldTemplateToken::Segment(SegmentKind::Minute),
                FieldTemplateToken::Literal(Cow::Borrowed(":")),
                FieldTemplateToken::Segment(SegmentKind::Second),
            ],
            time_24h_tokens: vec![
                FieldTemplateToken::Segment(SegmentKind::Hour),
                FieldTemplateToken::Literal(Cow::Borrowed(":")),
                FieldTemplateToken::Segment(SegmentKind::Minute),
                FieldTemplateToken::Literal(Cow::Borrowed(":")),
                FieldTemplateToken::Segment(SegmentKind::Second),
            ],
            date_time_separator: Cow::Borrowed(" "),
            placeholders: SegmentPlaceholders::ko_kr(),
            aria_labels: SegmentAriaLabels::ko_kr(),
            am_label: Cow::Borrowed("오전"),
            pm_label: Cow::Borrowed("오후"),
            empty_value_text: Cow::Borrowed("비어 있음"),
            selected_time_prefix: Cow::Borrowed("Selected Time: "),
        }
    }

    /// Built-in ISO template (`YYYY-MM-DD`, 24-hour `HH:mm[:ss]`).
    pub fn iso() -> Self {
        let mut tpl = Self::en_us();
        tpl.default_hour_cycle = HourCycle::H24;
        tpl.date_tokens = vec![
            FieldTemplateToken::Segment(SegmentKind::Year),
            FieldTemplateToken::Literal(Cow::Borrowed("-")),
            FieldTemplateToken::Segment(SegmentKind::Month),
            FieldTemplateToken::Literal(Cow::Borrowed("-")),
            FieldTemplateToken::Segment(SegmentKind::Day),
        ];
        tpl.date_time_separator = Cow::Borrowed(" ");
        tpl
    }

    /// Returns the localized meridiem label (`AM`/`PM` or `오전`/`오후`).
    pub fn day_period_label(&self, period: DayPeriod) -> &str {
        match period {
            DayPeriod::Am => self.am_label.as_ref(),
            DayPeriod::Pm => self.pm_label.as_ref(),
        }
    }

    /// Returns the filtered time token sequence for `hour_cycle` and `granularity`.
    ///
    /// Fixes `bits-ui`'s `granularity == Hour` bug (`5C D-4`) by omitting both `Minute` and `Second`
    /// along with their preceding separator literals.
    pub fn time_tokens(
        &self,
        hour_cycle: HourCycle,
        granularity: TimeGranularity,
    ) -> Vec<FieldTemplateToken> {
        let source = match hour_cycle {
            HourCycle::H12 => &self.time_12h_tokens,
            HourCycle::H24 => &self.time_24h_tokens,
        };
        let mut out: Vec<FieldTemplateToken> = Vec::with_capacity(source.len());
        for token in source {
            match token {
                FieldTemplateToken::Segment(SegmentKind::Minute)
                    if !granularity.includes_minute() =>
                {
                    if matches!(out.last(), Some(FieldTemplateToken::Literal(_))) {
                        out.pop();
                    }
                }
                FieldTemplateToken::Segment(SegmentKind::Second)
                    if !granularity.includes_second() =>
                {
                    if matches!(out.last(), Some(FieldTemplateToken::Literal(_))) {
                        out.pop();
                    }
                }
                _ => out.push(token.clone()),
            }
        }
        out
    }

    /// Returns the full date (or date + time) token sequence for `DateField`.
    pub fn date_field_tokens(
        &self,
        granularity: Granularity,
        hour_cycle: Option<HourCycle>,
        include_time_zone: bool,
    ) -> Vec<FieldTemplateToken> {
        let mut out = self.date_tokens.clone();
        if let Some(time_granularity) = granularity.to_time_granularity() {
            let cycle = hour_cycle.unwrap_or(self.default_hour_cycle);
            out.push(FieldTemplateToken::Literal(
                self.date_time_separator.clone(),
            ));
            out.extend(self.time_tokens(cycle, time_granularity));
            if include_time_zone {
                out.push(FieldTemplateToken::Literal(Cow::Borrowed(" ")));
                out.push(FieldTemplateToken::Segment(SegmentKind::TimeZoneName));
            }
        }
        out
    }
}

impl Default for FieldLocaleTemplate {
    fn default() -> Self {
        Self::en_us()
    }
}

/// Pure-Rust locale format selector for segmented date and time fields.
#[derive(Clone, Debug, Default, PartialEq, Eq)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub enum FieldLocaleFormat {
    #[default]
    EnUs,
    EnGb,
    KoKr,
    Iso,
    Custom(FieldLocaleTemplate),
}

impl FieldLocaleFormat {
    /// Derives a [`FieldLocaleFormat`] from a [`CalendarLocale`].
    pub fn from_calendar_locale(locale: &CalendarLocale) -> Self {
        match locale {
            CalendarLocale::EnUs => Self::EnUs,
            CalendarLocale::EnGb => Self::EnGb,
            CalendarLocale::KoKr => Self::KoKr,
            CalendarLocale::Iso => Self::Iso,
            CalendarLocale::Custom(labels) => match labels.date_order {
                CalendarDateOrder::YearFirst => Self::Iso,
                CalendarDateOrder::MonthFirst => Self::EnUs,
            },
        }
    }

    /// Resolves the [`FieldLocaleTemplate`] for this format.
    pub fn template(&self) -> Cow<'_, FieldLocaleTemplate> {
        match self {
            Self::EnUs => Cow::Owned(FieldLocaleTemplate::en_us()),
            Self::EnGb => Cow::Owned(FieldLocaleTemplate::en_gb()),
            Self::KoKr => Cow::Owned(FieldLocaleTemplate::ko_kr()),
            Self::Iso => Cow::Owned(FieldLocaleTemplate::iso()),
            Self::Custom(tpl) => Cow::Borrowed(tpl),
        }
    }

    /// Returns the default [`HourCycle`] for this locale format.
    pub fn default_hour_cycle(&self) -> HourCycle {
        self.template().default_hour_cycle
    }
}

use std::borrow::Cow;

use super::date::{CalendarDate, Weekday};

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
}

impl Default for CalendarLabels {
    fn default() -> Self {
        Self::en_us()
    }
}

/// Pure-Rust calendar locale configuration (`en-US`, `ko-KR`, or consumer-injected [`CalendarLabels`]).
#[derive(Clone, Debug, Default, PartialEq, Eq)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub enum CalendarLocale {
    #[default]
    EnUs,
    KoKr,
    Custom(CalendarLabels),
}

impl CalendarLocale {
    /// Parses common BCP-47 tags (`"en-US"`, `"ko-KR"`) or falls back to `EnUs`.
    pub fn from_tag(tag: &str) -> Self {
        let normalized = tag.trim().to_ascii_lowercase();
        if normalized == "ko" || normalized.starts_with("ko-") {
            Self::KoKr
        } else {
            Self::EnUs
        }
    }

    /// Resolves the [`CalendarLabels`] table for this locale.
    pub fn labels(&self) -> Cow<'_, CalendarLabels> {
        match self {
            Self::EnUs => Cow::Owned(CalendarLabels::en_us()),
            Self::KoKr => Cow::Owned(CalendarLabels::ko_kr()),
            Self::Custom(labels) => Cow::Borrowed(labels),
        }
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

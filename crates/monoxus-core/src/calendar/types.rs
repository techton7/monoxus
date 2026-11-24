use crate::foundation::date_time::{
    CalendarDate, CalendarLocale, CalendarMonth, MonthFormat, YearFormat,
};

/// All 14 compound parts exported by `calendar`.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub enum CalendarPart {
    Root,
    Header,
    Heading,
    PrevButton,
    NextButton,
    Grid,
    GridHead,
    GridBody,
    GridRow,
    HeadCell,
    Cell,
    Day,
    MonthSelect,
    YearSelect,
}

impl CalendarPart {
    pub const fn as_str(&self) -> &'static str {
        match self {
            Self::Root => "root",
            Self::Header => "header",
            Self::Heading => "heading",
            Self::PrevButton => "prev-button",
            Self::NextButton => "next-button",
            Self::Grid => "grid",
            Self::GridHead => "grid-head",
            Self::GridBody => "grid-body",
            Self::GridRow => "grid-row",
            Self::HeadCell => "head-cell",
            Self::Cell => "cell",
            Self::Day => "day",
            Self::MonthSelect => "month-select",
            Self::YearSelect => "year-select",
        }
    }
}

pub const CALENDAR_PARTS: [CalendarPart; 14] = [
    CalendarPart::Root,
    CalendarPart::Header,
    CalendarPart::Heading,
    CalendarPart::PrevButton,
    CalendarPart::NextButton,
    CalendarPart::Grid,
    CalendarPart::GridHead,
    CalendarPart::GridBody,
    CalendarPart::GridRow,
    CalendarPart::HeadCell,
    CalendarPart::Cell,
    CalendarPart::Day,
    CalendarPart::MonthSelect,
    CalendarPart::YearSelect,
];

/// Distinguishes `data-calendar-*` vs `data-range-calendar-*` attributes for shared parts.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, Hash)]
pub enum CalendarKind {
    #[default]
    Calendar,
    RangeCalendar,
}

/// Selection mode for `CalendarRoot` (`single` vs `multiple`).
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, Hash)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub enum CalendarMode {
    #[default]
    Single,
    Multiple,
}

/// Unified selection value snapshot for `Calendar`.
#[derive(Clone, Debug, PartialEq, Eq)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub enum CalendarSelectionValue {
    Single(Option<CalendarDate>),
    Multiple(Vec<CalendarDate>),
}

/// Option item rendered inside `MonthSelect` or `YearSelect`.
#[derive(Clone, Debug, PartialEq, Eq)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub struct CalendarSelectOption {
    pub value: i32,
    pub label: String,
}

impl CalendarSelectOption {
    /// Returns the stringified numeric value used by `<option>` and `SelectItem`.
    pub fn value_string(&self) -> String {
        self.value.to_string()
    }
}

/// Returns the option item matching `current`, if present in `items`.
pub fn resolve_selected_calendar_option(
    items: &[CalendarSelectOption],
    current: i32,
) -> Option<CalendarSelectOption> {
    items.iter().find(|item| item.value == current).cloned()
}

/// Builds localized month select options for `months` (defaults to `1..=12` when `None`).
pub fn build_calendar_month_options(
    months: Option<&[u8]>,
    locale: &CalendarLocale,
    format: MonthFormat,
) -> Vec<CalendarSelectOption> {
    const DEFAULT_MONTHS: [u8; 12] = [1, 2, 3, 4, 5, 6, 7, 8, 9, 10, 11, 12];
    let slice = months.unwrap_or(&DEFAULT_MONTHS);
    slice
        .iter()
        .copied()
        .map(|month| CalendarSelectOption {
            value: month as i32,
            label: locale.month_label(month, format),
        })
        .collect()
}

/// Builds localized year select options for `years`.
pub fn build_calendar_year_options(
    years: &[i32],
    locale: &CalendarLocale,
    format: YearFormat,
) -> Vec<CalendarSelectOption> {
    years
        .iter()
        .copied()
        .map(|year| CalendarSelectOption {
            value: year,
            label: locale.year_label(year, format),
        })
        .collect()
}

/// Per-cell reactive state snapshot exposed to custom `Day` / `Cell` renderers.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct CalendarCellSnapshot {
    pub date: CalendarDate,
    pub month: CalendarDate,
    pub day_text: String,
    pub is_disabled: bool,
    pub is_unavailable: bool,
    pub is_selected: bool,
    pub is_today: bool,
    pub is_outside_month: bool,
    pub is_outside_visible_months: bool,
    pub is_focused: bool,
}

/// Root view snapshot exposed by `CalendarRoot` (`render_children` and context).
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct CalendarSnapshot {
    pub placeholder: CalendarDate,
    pub months: Vec<CalendarMonth>,
    pub weekdays: Vec<String>,
    pub heading: String,
    pub full_label: String,
    pub is_invalid: bool,
    pub is_prev_disabled: bool,
    pub is_next_disabled: bool,
    pub tabbable_date: CalendarDate,
}

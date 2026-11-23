use crate::foundation::date_time::{CalendarDate, CalendarMonth};

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

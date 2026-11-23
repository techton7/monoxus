use crate::foundation::date_time::{CalendarDate, CalendarMonth, DateRange};

/// All 14 compound parts exported by `range_calendar`.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub enum RangeCalendarPart {
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

impl RangeCalendarPart {
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

pub const RANGE_CALENDAR_PARTS: [RangeCalendarPart; 14] = [
    RangeCalendarPart::Root,
    RangeCalendarPart::Header,
    RangeCalendarPart::Heading,
    RangeCalendarPart::PrevButton,
    RangeCalendarPart::NextButton,
    RangeCalendarPart::Grid,
    RangeCalendarPart::GridHead,
    RangeCalendarPart::GridBody,
    RangeCalendarPart::GridRow,
    RangeCalendarPart::HeadCell,
    RangeCalendarPart::Cell,
    RangeCalendarPart::Day,
    RangeCalendarPart::MonthSelect,
    RangeCalendarPart::YearSelect,
];

/// Per-cell reactive state snapshot exposed to custom `RangeCalendarDay` / `RangeCalendarCell` renderers.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct RangeCalendarCellSnapshot {
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
    pub is_selection_start: bool,
    pub is_selection_end: bool,
    pub is_range_start: bool,
    pub is_range_end: bool,
    pub is_range_middle: bool,
    pub is_highlighted: bool,
}

/// Root view snapshot exposed by `RangeCalendarRoot` (`render_children` and context).
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct RangeCalendarSnapshot {
    pub value: DateRange,
    pub start_value: Option<CalendarDate>,
    pub end_value: Option<CalendarDate>,
    pub highlighted_range: Option<(CalendarDate, CalendarDate)>,
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

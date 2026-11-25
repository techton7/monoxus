use crate::foundation::date_time::{CalendarDate, DateRange, DateValue, DateValueRange};

/// Structural parts belonging to the `DateRangePicker` compound primitive.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub enum DateRangePickerPart {
    Root,
    Input,
    Label,
    Segment,
    Trigger,
    Portal,
    Content,
    ContentStatic,
    Arrow,
    Close,
    Calendar,
}

impl DateRangePickerPart {
    pub const fn as_str(&self) -> &'static str {
        match self {
            Self::Root => "root",
            Self::Input => "input",
            Self::Label => "label",
            Self::Segment => "segment",
            Self::Trigger => "trigger",
            Self::Portal => "portal",
            Self::Content => "content",
            Self::ContentStatic => "content-static",
            Self::Arrow => "arrow",
            Self::Close => "close",
            Self::Calendar => "calendar",
        }
    }
}

/// Canonical list of `DateRangePicker` core parts.
pub const DATE_RANGE_PICKER_PARTS: &[DateRangePickerPart] = &[
    DateRangePickerPart::Root,
    DateRangePickerPart::Input,
    DateRangePickerPart::Label,
    DateRangePickerPart::Segment,
    DateRangePickerPart::Trigger,
    DateRangePickerPart::Portal,
    DateRangePickerPart::Content,
    DateRangePickerPart::ContentStatic,
    DateRangePickerPart::Arrow,
    DateRangePickerPart::Close,
    DateRangePickerPart::Calendar,
];

/// Read-only snapshot of `DateRangePicker` state.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct DateRangePickerSnapshot {
    pub value: DateValueRange,
    pub calendar_value: DateRange,
    pub placeholder: DateValue,
    pub calendar_placeholder: CalendarDate,
    pub open: bool,
    pub disabled: bool,
    pub read_only: bool,
    pub required: bool,
    pub close_on_range_select: bool,
    pub is_invalid: bool,
}

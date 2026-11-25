use crate::foundation::date_time::{CalendarDate, DateValue};

/// Structural parts belonging to the `DatePicker` compound primitive.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub enum DatePickerPart {
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

impl DatePickerPart {
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

/// Canonical list of `DatePicker` core parts.
pub const DATE_PICKER_PARTS: &[DatePickerPart] = &[
    DatePickerPart::Root,
    DatePickerPart::Input,
    DatePickerPart::Label,
    DatePickerPart::Segment,
    DatePickerPart::Trigger,
    DatePickerPart::Portal,
    DatePickerPart::Content,
    DatePickerPart::ContentStatic,
    DatePickerPart::Arrow,
    DatePickerPart::Close,
    DatePickerPart::Calendar,
];

/// Read-only snapshot of `DatePicker` state.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct DatePickerSnapshot {
    pub value: Option<DateValue>,
    pub calendar_value: Option<CalendarDate>,
    pub placeholder: DateValue,
    pub calendar_placeholder: CalendarDate,
    pub open: bool,
    pub disabled: bool,
    pub read_only: bool,
    pub required: bool,
    pub close_on_date_select: bool,
    pub is_invalid: bool,
}

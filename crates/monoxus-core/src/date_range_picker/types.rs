pub use crate::date_picker::types::{
    DATE_PICKER_PARTS as DATE_RANGE_PICKER_PARTS, DatePickerPart as DateRangePickerPart,
};
use crate::foundation::date_time::{CalendarDate, DateRange, DateValue, DateValueRange};

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

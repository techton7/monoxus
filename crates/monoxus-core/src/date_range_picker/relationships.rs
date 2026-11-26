use crate::date_picker::relationships::PickerRelationships;

/// Default CSS selector used by `DateRangePickerContent` for `pickerOpenFocus` (`OVL-3`).
pub const DATE_RANGE_PICKER_OPEN_FOCUS_SELECTOR: &str = "[data-range-calendar-day][data-focused]";

use super::state::DateRangePickerPolicy;

/// Deterministic scoped ID relationships for `DateRangePicker` and its composed primitives.
pub type DateRangePickerRelationships = PickerRelationships<DateRangePickerPolicy>;

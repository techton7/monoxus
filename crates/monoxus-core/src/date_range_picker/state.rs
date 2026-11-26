use crate::{
    date_picker::{
        PickerDefinitionLike, PickerPolicy, PickerSelectOutcome, PickerState,
        merge_calendar_date_into_value, merge_calendar_placeholder,
    },
    foundation::{
        date_time::{DateRange, DateValue, DateValueRange},
        shared::Direction,
    },
};

use super::{
    relationships::DATE_RANGE_PICKER_OPEN_FOCUS_SELECTOR, types::DateRangePickerSnapshot,
};

/// Immutable configuration for `DateRangePicker`.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct DateRangePickerDefinition {
    pub close_on_range_select: bool,
    pub disabled: bool,
    pub read_only: bool,
    pub required: bool,
    pub dir: Direction,
}

impl Default for DateRangePickerDefinition {
    fn default() -> Self {
        Self::new()
    }
}

impl DateRangePickerDefinition {
    pub fn new() -> Self {
        Self {
            close_on_range_select: true,
            disabled: false,
            read_only: false,
            required: false,
            dir: Direction::Ltr,
        }
    }

    pub fn with_close_on_range_select(mut self, close: bool) -> Self {
        self.close_on_range_select = close;
        self
    }

    pub fn with_disabled(mut self, disabled: bool) -> Self {
        self.disabled = disabled;
        self
    }

    pub fn with_read_only(mut self, read_only: bool) -> Self {
        self.read_only = read_only;
        self
    }

    pub fn with_required(mut self, required: bool) -> Self {
        self.required = required;
        self
    }

    pub fn with_dir(mut self, dir: Direction) -> Self {
        self.dir = dir;
        self
    }
}

impl PickerDefinitionLike for DateRangePickerDefinition {
    #[inline]
    fn is_disabled(&self) -> bool {
        self.disabled
    }

    #[inline]
    fn close_on_select(&self) -> bool {
        self.close_on_range_select
    }
}

/// Zero-sized policy marker for `DateRangePicker`.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, Hash)]
pub struct DateRangePickerPolicy;

impl PickerPolicy for DateRangePickerPolicy {
    type Value = DateValueRange;
    type CalendarValue = DateRange;
    type Definition = DateRangePickerDefinition;
    type Snapshot = DateRangePickerSnapshot;

    const OPEN_FOCUS_SELECTOR: &'static str = DATE_RANGE_PICKER_OPEN_FOCUS_SELECTOR;
    const CONTENT_DATA_ATTR: &'static str = "data-date-range-picker-content";

    #[inline]
    fn to_calendar_value(value: Self::Value) -> Self::CalendarValue {
        value.to_date_range()
    }

    #[inline]
    fn placeholder_from_field_value(value: Self::Value) -> Option<DateValue> {
        value.start.or(value.end)
    }

    fn build_snapshot(
        def: &Self::Definition,
        value: Self::Value,
        placeholder: DateValue,
        open: bool,
        is_invalid: bool,
    ) -> Self::Snapshot {
        DateRangePickerSnapshot {
            value,
            calendar_value: Self::to_calendar_value(value),
            placeholder,
            calendar_placeholder: placeholder.date(),
            open,
            disabled: def.disabled,
            read_only: def.read_only,
            required: def.required,
            close_on_range_select: def.close_on_range_select,
            is_invalid,
        }
    }
}

/// Result of a range update or range commit from `DateRangePickerCalendar`.
pub type DateRangePickerSelectOutcome = PickerSelectOutcome<DateValueRange>;

/// Merges a [`DateRange`] from `RangeCalendar` into an existing [`DateValueRange`],
/// preserving any wall-clock `CalendarTime` components on `start` / `end` (or `placeholder`).
#[inline]
pub const fn merge_calendar_range_into_value(
    current: DateValueRange,
    placeholder: DateValue,
    next: DateRange,
) -> DateValueRange {
    let start = match next.start {
        Some(d) => Some(merge_calendar_date_into_value(current.start, placeholder, d)),
        None => None,
    };
    let end = match next.end {
        Some(d) => Some(merge_calendar_date_into_value(current.end, placeholder, d)),
        None => None,
    };
    DateValueRange { start, end }
}

/// Pure headless state machine coordinating `DateRangeField`, `Popover`, and `RangeCalendar`.
pub type DateRangePickerState = PickerState<DateRangePickerPolicy>;

impl PickerState<DateRangePickerPolicy> {
    /// Applies an interim or complete range update from `RangeCalendar` (`on_value_change`).
    /// Never closes the popover by itself.
    pub fn apply_calendar_range(&mut self, range: DateRange) -> DateRangePickerSelectOutcome {
        let merged = merge_calendar_range_into_value(self.value, self.placeholder, range);
        self.value = merged;
        if let Some(start) = range.start {
            self.placeholder = merge_calendar_placeholder(self.placeholder, start);
        }
        PickerSelectOutcome {
            next_value: self.value,
            next_placeholder: self.placeholder,
            should_close: false,
        }
    }

    /// Handles a completed range commit from `RangeCalendar` (`on_range_select`).
    ///
    /// Closes the popover ONLY when `range.is_complete()` (both `start` and `end` set)
    /// and `close_on_range_select` is enabled. Partial start selections never close the popover.
    pub fn commit_calendar_range_select(
        &mut self,
        range: DateRange,
    ) -> DateRangePickerSelectOutcome {
        let merged = merge_calendar_range_into_value(self.value, self.placeholder, range);
        self.value = merged;
        if let Some(start) = range.start {
            self.placeholder = merge_calendar_placeholder(self.placeholder, start);
        }
        let should_close = range.is_complete() && self.def.close_on_range_select;
        if should_close {
            self.open = false;
        }
        PickerSelectOutcome {
            next_value: self.value,
            next_placeholder: self.placeholder,
            should_close,
        }
    }
}

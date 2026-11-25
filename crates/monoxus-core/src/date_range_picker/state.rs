use crate::{
    date_picker::{merge_calendar_date_into_value, merge_calendar_placeholder},
    foundation::{
        date_time::{CalendarDate, DateRange, DateValue, DateValueRange},
        shared::Direction,
    },
};

use super::types::DateRangePickerSnapshot;

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

/// Result of a range update or range commit from `DateRangePickerCalendar`.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct DateRangePickerSelectOutcome {
    pub next_value: DateValueRange,
    pub next_placeholder: DateValue,
    pub should_close: bool,
}

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
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct DateRangePickerState {
    def: DateRangePickerDefinition,
    value: DateValueRange,
    placeholder: DateValue,
    open: bool,
    is_invalid: bool,
}

impl DateRangePickerState {
    pub fn new(
        def: DateRangePickerDefinition,
        value: DateValueRange,
        placeholder: DateValue,
        open: bool,
    ) -> Self {
        let resolved_open = if def.disabled { false } else { open };
        Self {
            def,
            value,
            placeholder,
            open: resolved_open,
            is_invalid: false,
        }
    }

    #[inline]
    pub fn def(&self) -> &DateRangePickerDefinition {
        &self.def
    }

    #[inline]
    pub fn value(&self) -> DateValueRange {
        self.value
    }

    #[inline]
    pub fn calendar_value(&self) -> DateRange {
        self.value.to_date_range()
    }

    #[inline]
    pub fn placeholder(&self) -> DateValue {
        self.placeholder
    }

    #[inline]
    pub fn calendar_placeholder(&self) -> CalendarDate {
        self.placeholder.date()
    }

    #[inline]
    pub fn is_open(&self) -> bool {
        self.open
    }

    #[inline]
    pub fn is_invalid(&self) -> bool {
        self.is_invalid
    }

    pub fn sync_definition(&mut self, def: DateRangePickerDefinition) {
        if def.disabled {
            self.open = false;
        }
        self.def = def;
    }

    pub fn set_invalid(&mut self, invalid: bool) {
        self.is_invalid = invalid;
    }

    /// Requests an open/close transition. Returns the effective open state.
    pub fn set_open(&mut self, open: bool) -> bool {
        self.open = if self.def.disabled { false } else { open };
        self.open
    }

    /// Applies a range update originating from `DateRangeField` or external controlled state.
    pub fn set_field_value(&mut self, value: DateValueRange) {
        self.value = value;
        if let Some(start) = value.start {
            self.placeholder = start;
        } else if let Some(end) = value.end {
            self.placeholder = end;
        }
    }

    /// Applies a placeholder update originating from `DateRangeField` or external controlled state.
    pub fn set_placeholder(&mut self, placeholder: DateValue) {
        self.placeholder = placeholder;
    }

    /// Applies a placeholder update originating from `RangeCalendar` while preserving any time component.
    pub fn set_calendar_placeholder(&mut self, date: CalendarDate) -> DateValue {
        self.placeholder = merge_calendar_placeholder(self.placeholder, date);
        self.placeholder
    }

    /// Applies an interim or complete range update from `RangeCalendar` (`on_value_change`).
    /// Never closes the popover by itself.
    pub fn apply_calendar_range(&mut self, range: DateRange) -> DateRangePickerSelectOutcome {
        let merged = merge_calendar_range_into_value(self.value, self.placeholder, range);
        self.value = merged;
        if let Some(start) = range.start {
            self.placeholder = merge_calendar_placeholder(self.placeholder, start);
        }
        DateRangePickerSelectOutcome {
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
        DateRangePickerSelectOutcome {
            next_value: self.value,
            next_placeholder: self.placeholder,
            should_close,
        }
    }

    pub fn snapshot(&self) -> DateRangePickerSnapshot {
        DateRangePickerSnapshot {
            value: self.value,
            calendar_value: self.calendar_value(),
            placeholder: self.placeholder,
            calendar_placeholder: self.calendar_placeholder(),
            open: self.open,
            disabled: self.def.disabled,
            read_only: self.def.read_only,
            required: self.def.required,
            close_on_range_select: self.def.close_on_range_select,
            is_invalid: self.is_invalid,
        }
    }
}

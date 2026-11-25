use crate::foundation::{
    date_time::{CalendarDate, DateValue},
    shared::Direction,
};

use super::types::DatePickerSnapshot;

/// Immutable configuration for `DatePicker`.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct DatePickerDefinition {
    pub close_on_date_select: bool,
    pub disabled: bool,
    pub read_only: bool,
    pub required: bool,
    pub dir: Direction,
}

impl Default for DatePickerDefinition {
    fn default() -> Self {
        Self::new()
    }
}

impl DatePickerDefinition {
    pub fn new() -> Self {
        Self {
            close_on_date_select: true,
            disabled: false,
            read_only: false,
            required: false,
            dir: Direction::Ltr,
        }
    }

    pub fn with_close_on_date_select(mut self, close: bool) -> Self {
        self.close_on_date_select = close;
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

/// Result of selecting or deselecting a date from `DatePickerCalendar`.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct DatePickerSelectOutcome {
    pub next_value: Option<DateValue>,
    pub next_placeholder: DateValue,
    pub should_close: bool,
}

/// Merges a selected [`CalendarDate`] into an existing `DateValue` (or `placeholder`),
/// preserving any wall-clock `CalendarTime` component on `DateValue::DateTime`.
#[inline]
pub const fn merge_calendar_date_into_value(
    current: Option<DateValue>,
    placeholder: DateValue,
    selected: CalendarDate,
) -> DateValue {
    match current {
        Some(v) => v.with_date(selected),
        None => placeholder.with_date(selected),
    }
}

/// Replaces the calendar date component of a [`DateValue`] placeholder while preserving time.
#[inline]
pub const fn merge_calendar_placeholder(
    current_placeholder: DateValue,
    next_date: CalendarDate,
) -> DateValue {
    current_placeholder.with_date(next_date)
}

/// Pure headless state machine coordinating `DateField`, `Popover`, and `Calendar` for `DatePicker`.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct DatePickerState {
    def: DatePickerDefinition,
    value: Option<DateValue>,
    placeholder: DateValue,
    open: bool,
    is_invalid: bool,
}

impl DatePickerState {
    pub fn new(
        def: DatePickerDefinition,
        value: Option<DateValue>,
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
    pub fn def(&self) -> &DatePickerDefinition {
        &self.def
    }

    #[inline]
    pub fn value(&self) -> Option<DateValue> {
        self.value
    }

    #[inline]
    pub fn calendar_value(&self) -> Option<CalendarDate> {
        self.value.map(|v| v.date())
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

    pub fn sync_definition(&mut self, def: DatePickerDefinition) {
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

    /// Applies a value update originating from `DateField` or external controlled state.
    pub fn set_field_value(&mut self, value: Option<DateValue>) {
        self.value = value;
        if let Some(v) = value {
            self.placeholder = v;
        }
    }

    /// Applies a placeholder update originating from `DateField` or external controlled state.
    pub fn set_placeholder(&mut self, placeholder: DateValue) {
        self.placeholder = placeholder;
    }

    /// Applies a placeholder update originating from `Calendar` while preserving any time component.
    pub fn set_calendar_placeholder(&mut self, date: CalendarDate) -> DateValue {
        self.placeholder = merge_calendar_placeholder(self.placeholder, date);
        self.placeholder
    }

    /// Applies a value update originating from `Calendar` (`on_value_change`).
    pub fn apply_calendar_value(&mut self, date: Option<CalendarDate>) -> DatePickerSelectOutcome {
        match date {
            Some(selected) => {
                let merged = merge_calendar_date_into_value(self.value, self.placeholder, selected);
                self.value = Some(merged);
                self.placeholder = merge_calendar_placeholder(self.placeholder, selected);
                DatePickerSelectOutcome {
                    next_value: self.value,
                    next_placeholder: self.placeholder,
                    should_close: false,
                }
            }
            None => {
                self.value = None;
                DatePickerSelectOutcome {
                    next_value: None,
                    next_placeholder: self.placeholder,
                    should_close: false,
                }
            }
        }
    }

    /// Handles an explicit date selection commit from `Calendar` (`on_date_select`).
    ///
    /// Preserves any existing wall-clock time component and closes the popover when
    /// `close_on_date_select` is enabled.
    pub fn commit_calendar_date_select(&mut self, selected: CalendarDate) -> DatePickerSelectOutcome {
        let merged = merge_calendar_date_into_value(self.value, self.placeholder, selected);
        self.value = Some(merged);
        self.placeholder = merge_calendar_placeholder(self.placeholder, selected);
        let should_close = self.def.close_on_date_select;
        if should_close {
            self.open = false;
        }
        DatePickerSelectOutcome {
            next_value: self.value,
            next_placeholder: self.placeholder,
            should_close,
        }
    }

    pub fn snapshot(&self) -> DatePickerSnapshot {
        DatePickerSnapshot {
            value: self.value,
            calendar_value: self.calendar_value(),
            placeholder: self.placeholder,
            calendar_placeholder: self.calendar_placeholder(),
            open: self.open,
            disabled: self.def.disabled,
            read_only: self.def.read_only,
            required: self.def.required,
            close_on_date_select: self.def.close_on_date_select,
            is_invalid: self.is_invalid,
        }
    }
}

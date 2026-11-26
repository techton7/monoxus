use std::fmt::Debug;
use std::hash::Hash;
use std::marker::PhantomData;

use crate::foundation::{
    date_time::{CalendarDate, DateValue},
    shared::Direction,
};

use super::{relationships::DATE_PICKER_OPEN_FOCUS_SELECTOR, types::DatePickerSnapshot};

/// Common definition accessors shared by `DatePickerDefinition` and `DateRangePickerDefinition`.
pub trait PickerDefinitionLike: Clone + Debug + PartialEq + Eq + 'static {
    fn is_disabled(&self) -> bool;
    fn close_on_select(&self) -> bool;
}

/// Zero-overhead policy trait unifying `DatePicker` and `DateRangePicker` pure state & relationships.
pub trait PickerPolicy: Copy + Clone + Debug + Default + PartialEq + Eq + Hash + 'static {
    type Value: Copy + Debug + PartialEq + Eq + 'static;
    type CalendarValue: Copy + Debug + PartialEq + Eq + 'static;
    type Definition: PickerDefinitionLike;
    type Snapshot: Clone + Debug + PartialEq + Eq + 'static;

    const OPEN_FOCUS_SELECTOR: &'static str;
    const CONTENT_DATA_ATTR: &'static str;

    fn to_calendar_value(value: Self::Value) -> Self::CalendarValue;
    fn placeholder_from_field_value(value: Self::Value) -> Option<DateValue>;
    fn build_snapshot(
        def: &Self::Definition,
        value: Self::Value,
        placeholder: DateValue,
        open: bool,
        is_invalid: bool,
    ) -> Self::Snapshot;
}

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

impl PickerDefinitionLike for DatePickerDefinition {
    #[inline]
    fn is_disabled(&self) -> bool {
        self.disabled
    }

    #[inline]
    fn close_on_select(&self) -> bool {
        self.close_on_date_select
    }
}

/// Zero-sized policy marker for single-date `DatePicker`.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, Hash)]
pub struct DatePickerPolicy;

impl PickerPolicy for DatePickerPolicy {
    type Value = Option<DateValue>;
    type CalendarValue = Option<CalendarDate>;
    type Definition = DatePickerDefinition;
    type Snapshot = DatePickerSnapshot;

    const OPEN_FOCUS_SELECTOR: &'static str = DATE_PICKER_OPEN_FOCUS_SELECTOR;
    const CONTENT_DATA_ATTR: &'static str = "data-date-picker-content";

    #[inline]
    fn to_calendar_value(value: Self::Value) -> Self::CalendarValue {
        value.map(|v| v.date())
    }

    #[inline]
    fn placeholder_from_field_value(value: Self::Value) -> Option<DateValue> {
        value
    }

    fn build_snapshot(
        def: &Self::Definition,
        value: Self::Value,
        placeholder: DateValue,
        open: bool,
        is_invalid: bool,
    ) -> Self::Snapshot {
        DatePickerSnapshot {
            value,
            calendar_value: Self::to_calendar_value(value),
            placeholder,
            calendar_placeholder: placeholder.date(),
            open,
            disabled: def.disabled,
            read_only: def.read_only,
            required: def.required,
            close_on_date_select: def.close_on_date_select,
            is_invalid,
        }
    }
}

/// Result of selecting or updating a value from a picker calendar.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct PickerSelectOutcome<V> {
    pub next_value: V,
    pub next_placeholder: DateValue,
    pub should_close: bool,
}

/// Result of selecting or deselecting a date from `DatePickerCalendar`.
pub type DatePickerSelectOutcome = PickerSelectOutcome<Option<DateValue>>;

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

/// Policy-parameterized headless state machine shared by `DatePicker` and `DateRangePicker`.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct PickerState<P: PickerPolicy> {
    pub(crate) def: P::Definition,
    pub(crate) value: P::Value,
    pub(crate) placeholder: DateValue,
    pub(crate) open: bool,
    pub(crate) is_invalid: bool,
    _policy: PhantomData<P>,
}

impl<P: PickerPolicy> PickerState<P> {
    pub fn new(
        def: P::Definition,
        value: P::Value,
        placeholder: DateValue,
        open: bool,
    ) -> Self {
        let resolved_open = if def.is_disabled() { false } else { open };
        Self {
            def,
            value,
            placeholder,
            open: resolved_open,
            is_invalid: false,
            _policy: PhantomData,
        }
    }

    #[inline]
    pub fn def(&self) -> &P::Definition {
        &self.def
    }

    #[inline]
    pub fn value(&self) -> P::Value {
        self.value
    }

    #[inline]
    pub fn calendar_value(&self) -> P::CalendarValue {
        P::to_calendar_value(self.value)
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

    pub fn sync_definition(&mut self, def: P::Definition) {
        if def.is_disabled() {
            self.open = false;
        }
        self.def = def;
    }

    pub fn set_invalid(&mut self, invalid: bool) {
        self.is_invalid = invalid;
    }

    /// Requests an open/close transition. Returns the effective open state.
    pub fn set_open(&mut self, open: bool) -> bool {
        self.open = if self.def.is_disabled() { false } else { open };
        self.open
    }

    /// Applies a value update originating from the field or external controlled state.
    pub fn set_field_value(&mut self, value: P::Value) {
        self.value = value;
        if let Some(ph) = P::placeholder_from_field_value(value) {
            self.placeholder = ph;
        }
    }

    /// Applies a placeholder update originating from the field or external controlled state.
    pub fn set_placeholder(&mut self, placeholder: DateValue) {
        self.placeholder = placeholder;
    }

    /// Applies a placeholder update originating from the calendar while preserving any time component.
    pub fn set_calendar_placeholder(&mut self, date: CalendarDate) -> DateValue {
        self.placeholder = merge_calendar_placeholder(self.placeholder, date);
        self.placeholder
    }

    pub fn snapshot(&self) -> P::Snapshot {
        P::build_snapshot(
            &self.def,
            self.value,
            self.placeholder,
            self.open,
            self.is_invalid,
        )
    }
}

/// Pure headless state machine coordinating `DateField`, `Popover`, and `Calendar` for `DatePicker`.
pub type DatePickerState = PickerState<DatePickerPolicy>;

impl PickerState<DatePickerPolicy> {
    /// Applies a value update originating from `Calendar` (`on_value_change`).
    pub fn apply_calendar_value(&mut self, date: Option<CalendarDate>) -> DatePickerSelectOutcome {
        match date {
            Some(selected) => {
                let merged = merge_calendar_date_into_value(self.value, self.placeholder, selected);
                self.value = Some(merged);
                self.placeholder = merge_calendar_placeholder(self.placeholder, selected);
                PickerSelectOutcome {
                    next_value: self.value,
                    next_placeholder: self.placeholder,
                    should_close: false,
                }
            }
            None => {
                self.value = None;
                PickerSelectOutcome {
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
        PickerSelectOutcome {
            next_value: self.value,
            next_placeholder: self.placeholder,
            should_close,
        }
    }
}

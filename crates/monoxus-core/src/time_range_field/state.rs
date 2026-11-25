use crate::foundation::{
    date_time::{
        CalendarLocale, EditableSegmentKind, FieldInvalidReason, HourCycle, TimeGranularity,
        TimeRange, TimeValue, validate_time_range, validate_time_value,
    },
    shared::Direction,
};

use super::types::{RangeFieldInputKind, TimeRangeFieldSnapshot};

/// Static configuration for `TimeRangeField`.
#[derive(Clone, Debug, PartialEq, Eq)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub struct TimeRangeFieldDefinition {
    pub locale: CalendarLocale,
    pub granularity: TimeGranularity,
    pub hour_cycle: Option<HourCycle>,
    pub min_value: Option<TimeValue>,
    pub max_value: Option<TimeValue>,
    pub disabled: bool,
    pub read_only: bool,
    pub required: bool,
    pub readonly_segments: Vec<EditableSegmentKind>,
    pub error_message_id: Option<String>,
    pub dir: Direction,
}

impl Default for TimeRangeFieldDefinition {
    fn default() -> Self {
        Self {
            locale: CalendarLocale::EnUs,
            granularity: TimeGranularity::Minute,
            hour_cycle: None,
            min_value: None,
            max_value: None,
            disabled: false,
            read_only: false,
            required: false,
            readonly_segments: Vec::new(),
            error_message_id: None,
            dir: Direction::Ltr,
        }
    }
}

impl TimeRangeFieldDefinition {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn with_locale(mut self, locale: CalendarLocale) -> Self {
        self.locale = locale;
        self
    }

    pub fn with_granularity(mut self, granularity: TimeGranularity) -> Self {
        self.granularity = granularity;
        self
    }

    pub fn with_hour_cycle(mut self, hour_cycle: Option<HourCycle>) -> Self {
        self.hour_cycle = hour_cycle;
        self
    }

    pub fn with_min_value(mut self, min_value: Option<TimeValue>) -> Self {
        self.min_value = min_value;
        self
    }

    pub fn with_max_value(mut self, max_value: Option<TimeValue>) -> Self {
        self.max_value = max_value;
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

    pub fn with_readonly_segments(mut self, segments: Vec<EditableSegmentKind>) -> Self {
        self.readonly_segments = segments;
        self
    }

    pub fn with_error_message_id(mut self, id: Option<String>) -> Self {
        self.error_message_id = id;
        self
    }

    pub fn with_dir(mut self, dir: Direction) -> Self {
        self.dir = dir;
        self
    }
}

/// Pure-Rust state machine for `TimeRangeField`.
///
/// Overnight ranges (`end < start`, e.g. `20:00..05:00`) are valid by default (`5C D-2`),
/// matching `bits-ui` (`time-range-field.svelte.ts:186-224`); same-day ordering is opt-in
/// via the consumer `validate` callback.
#[derive(Clone, Debug, PartialEq, Eq)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub struct TimeRangeFieldState {
    def: TimeRangeFieldDefinition,
    value: TimeRange,
    placeholder: TimeValue,
    external_invalid: bool,
    local_invalid_reason: Option<FieldInvalidReason>,
}

impl TimeRangeFieldState {
    pub fn new(def: TimeRangeFieldDefinition, value: TimeRange, placeholder: TimeValue) -> Self {
        let local_invalid_reason =
            validate_time_range(value, def.min_value, def.max_value, None).err();
        Self {
            def,
            value,
            placeholder,
            external_invalid: false,
            local_invalid_reason,
        }
    }

    pub fn def(&self) -> &TimeRangeFieldDefinition {
        &self.def
    }

    pub const fn value(&self) -> TimeRange {
        self.value
    }

    pub const fn placeholder(&self) -> TimeValue {
        self.placeholder
    }

    pub const fn granularity(&self) -> TimeGranularity {
        self.def.granularity
    }

    pub fn is_invalid(&self) -> bool {
        self.external_invalid || self.local_invalid_reason.is_some()
    }

    pub fn invalid_reason(&self) -> Option<&FieldInvalidReason> {
        self.local_invalid_reason.as_ref()
    }

    pub const fn external_invalid(&self) -> bool {
        self.external_invalid
    }

    pub fn set_external_invalid(&mut self, invalid: bool) {
        self.external_invalid = invalid;
    }

    pub fn set_local_invalid_reason(&mut self, reason: Option<FieldInvalidReason>) {
        self.local_invalid_reason = reason;
    }

    /// Returns whether a specific range endpoint (`Start` or `End`) should be marked invalid (`5C D-8` / `DEF-5C-13`).
    ///
    /// - `RangeOrder`, `Custom`, `Unavailable`, or external overrides mark both endpoints invalid.
    /// - `Min` / `Max` mark ONLY the endpoint(s) whose value actually violates the bound,
    ///   leaving an untouched empty or in-bounds sibling valid.
    pub fn is_part_invalid(&self, kind: RangeFieldInputKind) -> bool {
        if self.external_invalid {
            return true;
        }
        match &self.local_invalid_reason {
            None => false,
            Some(FieldInvalidReason::RangeOrder)
            | Some(FieldInvalidReason::Custom(_))
            | Some(FieldInvalidReason::Unavailable) => true,
            Some(FieldInvalidReason::Min) | Some(FieldInvalidReason::Max) => {
                let part_val = match kind {
                    RangeFieldInputKind::Start => self.value.start,
                    RangeFieldInputKind::End => self.value.end,
                };
                validate_time_value(part_val, self.def.min_value, self.def.max_value, None).is_err()
            }
        }
    }

    pub fn sync_definition(&mut self, def: TimeRangeFieldDefinition) {
        self.def = def;
    }

    pub fn set_value(&mut self, value: TimeRange) {
        self.value = value;
    }

    pub fn set_part_value(&mut self, kind: RangeFieldInputKind, next: Option<TimeValue>) {
        match kind {
            RangeFieldInputKind::Start => self.value.start = next,
            RangeFieldInputKind::End => self.value.end = next,
        }
        if let Some(tv) = next {
            self.placeholder = tv;
        }
    }

    pub fn set_placeholder(&mut self, placeholder: TimeValue) {
        self.placeholder = placeholder;
    }

    /// Evaluates range validation (`Custom -> Min -> Max`) per `5C D-2`.
    /// Overnight ranges (`end < start`) are valid by default unless rejected by `custom_errors` or `min`/`max` bounds.
    pub fn evaluate_validation(
        &mut self,
        custom_errors: Option<Vec<String>>,
    ) -> Option<FieldInvalidReason> {
        self.local_invalid_reason = validate_time_range(
            self.value,
            self.def.min_value,
            self.def.max_value,
            custom_errors,
        )
        .err();
        self.local_invalid_reason.clone()
    }

    pub fn snapshot(&self) -> TimeRangeFieldSnapshot {
        TimeRangeFieldSnapshot {
            value: self.value,
            placeholder: self.placeholder,
            granularity: self.def.granularity,
            hour_cycle: self.def.hour_cycle,
            is_invalid: self.is_invalid(),
            invalid_reason: self.local_invalid_reason.clone(),
            disabled: self.def.disabled,
            read_only: self.def.read_only,
            required: self.def.required,
        }
    }
}

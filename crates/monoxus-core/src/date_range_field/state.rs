use crate::foundation::{
    date_time::{
        CalendarLocale, DateValue, DateValueRange, EditableSegmentKind, FieldInvalidReason,
        Granularity, HourCycle, validate_date_range_with_unavailable, validate_date_value,
        validate_date_value_range,
    },
    shared::Direction,
};

use super::types::{DateRangeFieldSnapshot, RangeFieldInputKind};

/// Static configuration for `DateRangeField`.
#[derive(Clone, Debug, PartialEq, Eq)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub struct DateRangeFieldDefinition {
    pub locale: CalendarLocale,
    pub granularity: Option<Granularity>,
    pub hour_cycle: Option<HourCycle>,
    pub hide_time_zone: bool,
    pub min_value: Option<DateValue>,
    pub max_value: Option<DateValue>,
    pub disabled: bool,
    pub read_only: bool,
    pub required: bool,
    pub readonly_segments: Vec<EditableSegmentKind>,
    pub error_message_id: Option<String>,
    pub dir: Direction,
}

impl Default for DateRangeFieldDefinition {
    fn default() -> Self {
        Self {
            locale: CalendarLocale::EnUs,
            granularity: None,
            hour_cycle: None,
            hide_time_zone: false,
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

impl DateRangeFieldDefinition {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn with_locale(mut self, locale: CalendarLocale) -> Self {
        self.locale = locale;
        self
    }

    pub fn with_granularity(mut self, granularity: Option<Granularity>) -> Self {
        self.granularity = granularity;
        self
    }

    pub fn with_hour_cycle(mut self, hour_cycle: Option<HourCycle>) -> Self {
        self.hour_cycle = hour_cycle;
        self
    }

    pub fn with_hide_time_zone(mut self, hide_time_zone: bool) -> Self {
        self.hide_time_zone = hide_time_zone;
        self
    }

    pub fn with_min_value(mut self, min_value: Option<DateValue>) -> Self {
        self.min_value = min_value;
        self
    }

    pub fn with_max_value(mut self, max_value: Option<DateValue>) -> Self {
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

    pub const fn resolved_granularity(
        &self,
        value: DateValueRange,
        placeholder: DateValue,
    ) -> Granularity {
        match self.granularity {
            Some(g) => g,
            None => {
                let sample = match value.start {
                    Some(s) => Some(s),
                    None => value.end,
                };
                Granularity::infer_from_date_value(sample, placeholder)
            }
        }
    }
}

/// Pure-Rust state machine for `DateRangeField`, including built-in `end < start` `RangeOrder` validation (`5C D-2`).
#[derive(Clone, Debug, PartialEq, Eq)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub struct DateRangeFieldState {
    def: DateRangeFieldDefinition,
    value: DateValueRange,
    placeholder: DateValue,
    external_invalid: bool,
    local_invalid_reason: Option<FieldInvalidReason>,
    start_unavailable: bool,
    end_unavailable: bool,
}

impl DateRangeFieldState {
    pub fn new(
        def: DateRangeFieldDefinition,
        value: DateValueRange,
        placeholder: DateValue,
    ) -> Self {
        let local_invalid_reason =
            validate_date_value_range(value, def.min_value, def.max_value, None).err();
        Self {
            def,
            value,
            placeholder,
            external_invalid: false,
            local_invalid_reason,
            start_unavailable: false,
            end_unavailable: false,
        }
    }

    pub fn def(&self) -> &DateRangeFieldDefinition {
        &self.def
    }

    pub const fn value(&self) -> DateValueRange {
        self.value
    }

    pub const fn placeholder(&self) -> DateValue {
        self.placeholder
    }

    pub const fn resolved_granularity(&self) -> Granularity {
        self.def.resolved_granularity(self.value, self.placeholder)
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

    pub const fn unavailable_endpoints(&self) -> (bool, bool) {
        (self.start_unavailable, self.end_unavailable)
    }

    pub fn set_unavailable_endpoints(&mut self, start_unavailable: bool, end_unavailable: bool) {
        self.start_unavailable = start_unavailable;
        self.end_unavailable = end_unavailable;
    }

    /// Returns whether a specific range endpoint (`Start` or `End`) should be marked invalid (`5C D-8` / `DEF-5C-13`).
    ///
    /// - `RangeOrder`, `Custom`, or external overrides mark both endpoints invalid.
    /// - `Unavailable` marks ONLY the endpoint(s) whose date is unavailable (or present endpoints if
    ///   per-endpoint flags were not explicitly set).
    /// - `Min` / `Max` mark ONLY the endpoint(s) whose value actually violates the bound,
    ///   leaving an untouched empty or in-bounds sibling valid.
    pub fn is_part_invalid(&self, kind: RangeFieldInputKind) -> bool {
        if self.external_invalid {
            return true;
        }
        match &self.local_invalid_reason {
            None => false,
            Some(FieldInvalidReason::RangeOrder) | Some(FieldInvalidReason::Custom(_)) => true,
            Some(FieldInvalidReason::Unavailable) => {
                if self.start_unavailable || self.end_unavailable {
                    match kind {
                        RangeFieldInputKind::Start => self.start_unavailable,
                        RangeFieldInputKind::End => self.end_unavailable,
                    }
                } else {
                    match kind {
                        RangeFieldInputKind::Start => self.value.start.is_some(),
                        RangeFieldInputKind::End => self.value.end.is_some(),
                    }
                }
            }
            Some(FieldInvalidReason::Min) | Some(FieldInvalidReason::Max) => {
                let part_val = match kind {
                    RangeFieldInputKind::Start => self.value.start,
                    RangeFieldInputKind::End => self.value.end,
                };
                validate_date_value(part_val, self.def.min_value, self.def.max_value, None).is_err()
            }
        }
    }

    pub fn sync_definition(&mut self, def: DateRangeFieldDefinition) {
        self.def = def;
    }

    pub fn set_value(&mut self, value: DateValueRange) {
        self.value = value;
    }

    pub fn set_part_value(&mut self, kind: RangeFieldInputKind, next: Option<DateValue>) {
        match kind {
            RangeFieldInputKind::Start => self.value.start = next,
            RangeFieldInputKind::End => self.value.end = next,
        }
        if let Some(dv) = next {
            self.placeholder = dv;
        }
    }

    pub fn set_placeholder(&mut self, placeholder: DateValue) {
        self.placeholder = placeholder;
    }

    /// Evaluates range validation (`Unavailable -> Custom -> RangeOrder -> Min -> Max`) per `5C D-2` / `5C D-8`.
    pub fn evaluate_validation_with_unavailable(
        &mut self,
        start_unavailable: bool,
        end_unavailable: bool,
        custom_errors: Option<Vec<String>>,
    ) -> Option<FieldInvalidReason> {
        self.start_unavailable = start_unavailable;
        self.end_unavailable = end_unavailable;
        self.local_invalid_reason = validate_date_range_with_unavailable(
            self.value,
            self.def.min_value,
            self.def.max_value,
            start_unavailable || end_unavailable,
            custom_errors,
        )
        .err();
        self.local_invalid_reason.clone()
    }

    /// Evaluates range validation (`Custom -> RangeOrder -> Min -> Max`) per `5C D-2`.
    pub fn evaluate_validation(
        &mut self,
        custom_errors: Option<Vec<String>>,
    ) -> Option<FieldInvalidReason> {
        self.evaluate_validation_with_unavailable(false, false, custom_errors)
    }

    pub fn snapshot(&self) -> DateRangeFieldSnapshot {
        DateRangeFieldSnapshot {
            value: self.value,
            placeholder: self.placeholder,
            granularity: self.resolved_granularity(),
            hour_cycle: self.def.hour_cycle,
            is_invalid: self.is_invalid(),
            invalid_reason: self.local_invalid_reason.clone(),
            disabled: self.def.disabled,
            read_only: self.def.read_only,
            required: self.def.required,
        }
    }
}

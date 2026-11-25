use crate::foundation::{
    date_time::{
        CalendarLocale, CalendarTime, EditableSegmentKind, FieldInvalidReason, FieldSegmentIntent,
        HourCycle, SegmentEngine, SegmentFocusMove, SegmentKind, SegmentTransitionResult,
        TimeGranularity, TimeValue, validate_time_value,
    },
    shared::Direction,
};

use super::{
    relationships::TimeFieldRelationships,
    types::{TimeFieldSegmentItem, TimeFieldSnapshot},
};

/// Static configuration for a `TimeField`.
#[derive(Clone, Debug, PartialEq, Eq)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub struct TimeFieldDefinition {
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

impl Default for TimeFieldDefinition {
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

impl TimeFieldDefinition {
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

    pub fn is_segment_read_only(&self, kind: SegmentKind) -> bool {
        if self.read_only {
            return true;
        }
        kind.as_editable()
            .is_some_and(|e| self.readonly_segments.contains(&e))
    }
}

/// Resolves the default placeholder [`TimeValue`] for a `TimeField` clamped into `[min, max]`.
pub fn default_time_field_placeholder(
    default_value: Option<TimeValue>,
    min_value: Option<TimeValue>,
    max_value: Option<TimeValue>,
) -> TimeValue {
    default_value
        .unwrap_or_else(|| TimeValue::Time(CalendarTime::midnight()))
        .clamp(min_value, max_value)
}

/// Pure-Rust state machine for `TimeField`, wrapping [`SegmentEngine`].
#[derive(Clone, Debug, PartialEq, Eq)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub struct TimeFieldState {
    def: TimeFieldDefinition,
    engine: SegmentEngine,
    value: Option<TimeValue>,
    placeholder: TimeValue,
    external_invalid: bool,
    local_invalid_reason: Option<FieldInvalidReason>,
}

impl TimeFieldState {
    pub fn new(
        def: TimeFieldDefinition,
        value: Option<TimeValue>,
        placeholder: TimeValue,
    ) -> Self {
        let mut engine = SegmentEngine::new_time(
            def.locale.field_format(),
            def.granularity,
            def.hour_cycle,
            placeholder,
        );
        engine.sync_from_time_value(value);

        let local_invalid_reason =
            validate_time_value(value, def.min_value, def.max_value, None).err();

        Self {
            def,
            engine,
            value,
            placeholder,
            external_invalid: false,
            local_invalid_reason,
        }
    }

    pub fn def(&self) -> &TimeFieldDefinition {
        &self.def
    }

    pub fn engine(&self) -> &SegmentEngine {
        &self.engine
    }

    pub const fn value(&self) -> Option<TimeValue> {
        self.value
    }

    pub const fn placeholder(&self) -> TimeValue {
        self.placeholder
    }

    pub const fn granularity(&self) -> TimeGranularity {
        self.def.granularity
    }

    pub fn effective_hour_cycle(&self) -> HourCycle {
        self.engine.effective_hour_cycle()
    }

    /// Returns `true` when either local validation failed OR an external override (e.g., `TimeRangeField`)
    /// marks the field invalid — fixing `bits-ui`'s `isInvalidProp` ordering bug (`5C D-4`).
    pub fn is_invalid(&self) -> bool {
        self.external_invalid || self.local_invalid_reason.is_some()
    }

    pub fn local_invalid_reason(&self) -> Option<&FieldInvalidReason> {
        self.local_invalid_reason.as_ref()
    }

    pub fn set_external_invalid(&mut self, invalid: bool) {
        self.external_invalid = invalid;
    }

    pub fn set_value(&mut self, value: Option<TimeValue>) {
        if self.value != value {
            self.value = value;
            self.engine.sync_from_time_value(value);
        }
    }

    pub fn set_placeholder(&mut self, placeholder: TimeValue) {
        if self.placeholder != placeholder {
            self.placeholder = placeholder;
            self.engine.set_placeholder_time(placeholder);
        }
    }

    pub fn sync_definition(&mut self, next_def: TimeFieldDefinition) {
        let granularity_changed = self.def.granularity != next_def.granularity;
        self.def = next_def;

        if granularity_changed {
            let mut next_engine = SegmentEngine::new_time(
                self.def.locale.field_format(),
                self.def.granularity,
                self.def.hour_cycle,
                self.placeholder,
            );
            next_engine.sync_from_time_value(self.value);
            self.engine = next_engine;
        } else {
            self.engine.set_locale(self.def.locale.field_format());
            self.engine.set_hour_cycle(self.def.hour_cycle);
        }
    }

    pub fn evaluate_validation(
        &mut self,
        custom_errors: Option<Vec<String>>,
    ) -> Option<FieldInvalidReason> {
        self.local_invalid_reason = validate_time_value(
            self.value,
            self.def.min_value,
            self.def.max_value,
            custom_errors,
        )
        .err();
        self.local_invalid_reason.clone()
    }

    pub fn apply_intent(
        &mut self,
        kind: SegmentKind,
        intent: FieldSegmentIntent,
    ) -> Option<SegmentTransitionResult> {
        if self.def.disabled {
            return None;
        }

        let is_navigation_or_blur = matches!(
            intent,
            FieldSegmentIntent::NextSegment
                | FieldSegmentIntent::PrevSegment
                | FieldSegmentIntent::Blur
        );

        if !is_navigation_or_blur && self.def.is_segment_read_only(kind) {
            return None;
        }

        let result = self.engine.apply_intent(kind, intent);
        if let Some(tv) = result.committed_time_value {
            self.value = Some(tv);
            self.placeholder = tv;
            self.engine.set_placeholder_time(tv);
        } else if result.cleared_value {
            self.value = None;
        }

        Some(result)
    }

    pub fn iso_value(&self) -> String {
        self.value
            .map(|v| v.time().to_iso_string())
            .unwrap_or_default()
    }

    pub fn description_text(&self) -> String {
        match self.value {
            Some(tv) => {
                let tpl = self.def.locale.field_template();
                format!("{}{}", tpl.selected_time_prefix, tv.time().to_iso_string())
            }
            None => String::new(),
        }
    }

    pub fn rendered_segments(&self, rels: &TimeFieldRelationships) -> Vec<TimeFieldSegmentItem> {
        self.engine
            .render_segments()
            .into_iter()
            .enumerate()
            .map(|(index, segment)| TimeFieldSegmentItem {
                id: rels.segment_id(index, segment.kind),
                index,
                segment,
            })
            .collect()
    }

    pub fn snapshot(&self, rels: &TimeFieldRelationships) -> TimeFieldSnapshot {
        TimeFieldSnapshot {
            value: self.value,
            placeholder: self.placeholder,
            granularity: self.def.granularity,
            hour_cycle: self.effective_hour_cycle(),
            segments: self.rendered_segments(rels),
            is_invalid: self.is_invalid(),
            invalid_reason: self.local_invalid_reason.clone(),
            disabled: self.def.disabled,
            read_only: self.def.read_only,
            required: self.def.required,
            iso_value: self.iso_value(),
            description_text: self.description_text(),
        }
    }
}

pub fn first_interactive_time_segment(
    segments: &[TimeFieldSegmentItem],
) -> Option<&TimeFieldSegmentItem> {
    segments.iter().find(|item| item.is_interactive())
}

pub fn last_interactive_time_segment(
    segments: &[TimeFieldSegmentItem],
) -> Option<&TimeFieldSegmentItem> {
    segments.iter().rev().find(|item| item.is_interactive())
}

pub fn resolve_adjacent_interactive_time_index(
    segments: &[TimeFieldSegmentItem],
    current_index: usize,
    move_dir: SegmentFocusMove,
) -> Result<usize, SegmentFocusMove> {
    match move_dir {
        SegmentFocusMove::Next => segments
            .iter()
            .find(|item| item.index > current_index && item.is_interactive())
            .map(|item| item.index)
            .ok_or(SegmentFocusMove::Next),
        SegmentFocusMove::Prev => segments
            .iter()
            .rev()
            .find(|item| item.index < current_index && item.is_interactive())
            .map(|item| item.index)
            .ok_or(SegmentFocusMove::Prev),
    }
}

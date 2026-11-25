use crate::foundation::{
    date_time::{
        CalendarDate, CalendarDateTime, CalendarLocale, CalendarTime, DateValue,
        EditableSegmentKind, FieldInvalidReason, FieldSegmentIntent, Granularity, HourCycle,
        SegmentEngine, SegmentFocusMove, SegmentKind, SegmentTransitionResult, validate_date_value,
    },
    shared::Direction,
};

use super::{
    relationships::DateFieldRelationships,
    types::{DateFieldSegmentItem, DateFieldSnapshot},
};

/// Static configuration for a `DateField`.
#[derive(Clone, Debug, PartialEq, Eq)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub struct DateFieldDefinition {
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

impl Default for DateFieldDefinition {
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

impl DateFieldDefinition {
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

    /// Resolves effective [`Granularity`] from explicit prop or `value`/`placeholder`.
    pub const fn resolved_granularity(
        &self,
        value: Option<DateValue>,
        placeholder: DateValue,
    ) -> Granularity {
        match self.granularity {
            Some(g) => g,
            None => Granularity::infer_from_date_value(value, placeholder),
        }
    }

    /// Returns `true` when `kind` is read-only (whole field `read_only`, `TimeZoneName`, or listed in `readonly_segments`).
    pub fn is_segment_read_only(&self, kind: SegmentKind) -> bool {
        if self.read_only {
            return true;
        }
        match kind {
            SegmentKind::TimeZoneName => true,
            SegmentKind::Literal => false,
            _ => kind
                .as_editable()
                .is_some_and(|e| self.readonly_segments.contains(&e)),
        }
    }
}

/// Resolves the default placeholder [`DateValue`] for a `DateField` clamped into `[min, max]`.
pub fn default_date_field_placeholder(
    default_value: Option<DateValue>,
    granularity: Option<Granularity>,
    min_value: Option<DateValue>,
    max_value: Option<DateValue>,
    today: CalendarDate,
) -> DateValue {
    let base = match default_value {
        Some(v) => v,
        None => {
            if granularity.is_some_and(Granularity::includes_time) {
                DateValue::DateTime(CalendarDateTime::new(today, CalendarTime::midnight()))
            } else {
                DateValue::Date(today)
            }
        }
    };
    base.clamp(min_value, max_value)
}

/// Pure-Rust state machine for `DateField`, wrapping [`SegmentEngine`].
#[derive(Clone, Debug, PartialEq, Eq)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub struct DateFieldState {
    def: DateFieldDefinition,
    engine: SegmentEngine,
    value: Option<DateValue>,
    placeholder: DateValue,
    external_invalid: bool,
    local_invalid_reason: Option<FieldInvalidReason>,
}

impl DateFieldState {
    pub fn new(
        def: DateFieldDefinition,
        value: Option<DateValue>,
        placeholder: DateValue,
    ) -> Self {
        let granularity = def.resolved_granularity(value, placeholder);
        let mut engine = SegmentEngine::new_date(
            def.locale.field_format(),
            granularity,
            def.hour_cycle,
            def.hide_time_zone,
            placeholder,
        );
        engine.sync_from_date_value(value);

        let local_invalid_reason =
            validate_date_value(value, def.min_value, def.max_value, None).err();

        Self {
            def,
            engine,
            value,
            placeholder,
            external_invalid: false,
            local_invalid_reason,
        }
    }

    pub fn def(&self) -> &DateFieldDefinition {
        &self.def
    }

    pub fn engine(&self) -> &SegmentEngine {
        &self.engine
    }

    pub const fn value(&self) -> Option<DateValue> {
        self.value
    }

    pub const fn placeholder(&self) -> DateValue {
        self.placeholder
    }

    pub const fn resolved_granularity(&self) -> Granularity {
        self.def.resolved_granularity(self.value, self.placeholder)
    }

    pub fn effective_hour_cycle(&self) -> HourCycle {
        self.engine.effective_hour_cycle()
    }

    /// Returns `true` when either local validation failed OR an external override (e.g., `DateRangeField`)
    /// marks the field invalid — fixing `bits-ui`'s `isInvalidProp` ordering bug (`5C D-4`).
    pub fn is_invalid(&self) -> bool {
        self.external_invalid || self.local_invalid_reason.is_some()
    }

    pub fn local_invalid_reason(&self) -> Option<&FieldInvalidReason> {
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

    pub fn set_value(&mut self, value: Option<DateValue>) {
        if self.value != value {
            self.value = value;
            self.engine.sync_from_date_value(value);
        }
    }

    /// Unconditionally resets `value` and clears any uncommitted partial segment buffers (`5C D-9` / `DEF-5C-03`).
    pub fn reset_value(&mut self, value: Option<DateValue>) {
        self.value = value;
        self.engine.sync_from_date_value(value);
    }

    pub fn set_placeholder(&mut self, placeholder: DateValue) {
        if self.placeholder != placeholder {
            self.placeholder = placeholder;
            self.engine.set_placeholder_date(placeholder);
        }
    }

    /// Reconciles updated [`DateFieldDefinition`] props without losing active segment buffers.
    pub fn sync_definition(&mut self, next_def: DateFieldDefinition) {
        let prev_granularity = self.resolved_granularity();
        let next_granularity = next_def.resolved_granularity(self.value, self.placeholder);
        let hide_tz_changed = self.def.hide_time_zone != next_def.hide_time_zone;

        self.def = next_def;

        if prev_granularity != next_granularity || hide_tz_changed {
            let mut next_engine = SegmentEngine::new_date(
                self.def.locale.field_format(),
                next_granularity,
                self.def.hour_cycle,
                self.def.hide_time_zone,
                self.placeholder,
            );
            next_engine.sync_from_date_value(self.value);
            self.engine = next_engine;
        } else {
            self.engine.set_locale(self.def.locale.field_format());
            self.engine.set_hour_cycle(self.def.hour_cycle);
        }
    }

    /// Evaluates single-field validation (`Custom -> Min -> Max`) and updates `local_invalid_reason`.
    pub fn evaluate_validation(
        &mut self,
        custom_errors: Option<Vec<String>>,
    ) -> Option<FieldInvalidReason> {
        self.local_invalid_reason = validate_date_value(
            self.value,
            self.def.min_value,
            self.def.max_value,
            custom_errors,
        )
        .err();
        self.local_invalid_reason.clone()
    }

    /// Applies a host-neutral [`FieldSegmentIntent`], enforcing `disabled`, `read_only`, and `readonly_segments`.
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
        if let Some(dv) = result.committed_date_value {
            self.value = Some(dv);
            self.placeholder = dv;
            self.engine.set_placeholder_date(dv);
        } else if result.cleared_value {
            self.value = None;
        }

        Some(result)
    }

    /// Formats the ISO-8601 string for hidden `<input>` form submission (`""` when empty).
    pub fn iso_value(&self) -> String {
        self.value.map(|v| v.to_iso_string()).unwrap_or_default()
    }

    /// Formats the accessible screen-reader description (`"Selected Date: ..."` or `""` when empty).
    pub fn description_text(&self) -> String {
        match self.value {
            Some(DateValue::Date(d)) => self.def.locale.selected_announcement(d),
            Some(DateValue::DateTime(dt)) => {
                format!(
                    "{} {}",
                    self.def.locale.selected_announcement(dt.date),
                    dt.time.to_iso_string()
                )
            }
            None => String::new(),
        }
    }

    /// Renders all segments paired with deterministic IDs from `rels`.
    pub fn rendered_segments(&self, rels: &DateFieldRelationships) -> Vec<DateFieldSegmentItem> {
        self.engine
            .render_segments()
            .into_iter()
            .enumerate()
            .map(|(index, segment)| DateFieldSegmentItem {
                id: rels.segment_id(index, segment.kind),
                index,
                segment,
            })
            .collect()
    }

    /// Produces a complete [`DateFieldSnapshot`] for rendering.
    pub fn snapshot(&self, rels: &DateFieldRelationships) -> DateFieldSnapshot {
        DateFieldSnapshot {
            value: self.value,
            placeholder: self.placeholder,
            granularity: self.resolved_granularity(),
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

/// Returns the first interactive (focusable) segment in `segments`.
pub fn first_interactive_segment(segments: &[DateFieldSegmentItem]) -> Option<&DateFieldSegmentItem> {
    segments.iter().find(|item| item.is_interactive())
}

/// Returns the last interactive (focusable) segment in `segments`.
pub fn last_interactive_segment(segments: &[DateFieldSegmentItem]) -> Option<&DateFieldSegmentItem> {
    segments.iter().rev().find(|item| item.is_interactive())
}

/// Resolves the next or previous interactive segment index relative to `current_index`.
///
/// Returns `Ok(next_index)` when an adjacent interactive segment exists within this field,
/// or `Err(move_dir)` when crossing the start/end boundary (for range-field or picker-trigger traversal).
pub fn resolve_adjacent_interactive_index(
    segments: &[DateFieldSegmentItem],
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

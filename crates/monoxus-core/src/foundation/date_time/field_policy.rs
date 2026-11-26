//! Zero-overhead policy-parameterized segmented field and range field state machines.
//!
//! Unifies `DateField`, `TimeField`, `DateRangeField`, and `TimeRangeField` behind zero-sized
//! [`DateFieldPolicy`] and [`TimeFieldPolicy`] markers while preserving exact divergence semantics:
//! - `DateRangeField`: `ENFORCE_RANGE_ORDER = true` (`end < start` -> `RangeOrder`), `is_date_unavailable`
//! - `TimeRangeField`: `ENFORCE_RANGE_ORDER = false` (overnight `20:00..05:00` allowed by default)
//! - Per-endpoint validation isolation (`5C D-8` / `DEF-5C-13`).

use std::fmt::Debug;
use std::marker::PhantomData;

use crate::foundation::shared::{Direction, ScopeHandle};

use super::{
    date::{
        DateValue, Granularity, HourCycle, TimeGranularity, TimeValue,
    },
    field::{
        EditableSegmentKind, FieldInvalidReason, FieldSegment, FieldSegmentIntent, SegmentEngine,
        SegmentFocusMove, SegmentKind, SegmentTransitionResult, validate_date_value_with_unavailable,
        validate_time_value,
    },
    locale::CalendarLocale,
    range::{
        ChronoComparable, ChronoRange, validate_date_range_with_unavailable, validate_time_range,
    },
};

/// Unified structural parts for segmented date/time single and range fields.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub enum FieldPart {
    Root,
    Input,
    Label,
    Segment,
}

impl FieldPart {
    #[inline]
    pub const fn as_str(&self) -> &'static str {
        match self {
            Self::Root => "root",
            Self::Input => "input",
            Self::Label => "label",
            Self::Segment => "segment",
        }
    }
}

/// Canonical parts for single segmented fields (`DateField`, `TimeField`).
pub const FIELD_PARTS: [FieldPart; 3] = [FieldPart::Input, FieldPart::Label, FieldPart::Segment];

/// Canonical parts for range segmented fields (`DateRangeField`, `TimeRangeField`).
pub const RANGE_FIELD_PARTS: [FieldPart; 4] = [
    FieldPart::Root,
    FieldPart::Input,
    FieldPart::Label,
    FieldPart::Segment,
];

/// Whether a segmented field is rendered standalone or as the `start`/`end` input of a range field.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, Hash)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub enum FieldRangeRole {
    #[default]
    Standalone,
    RangeStart,
    RangeEnd,
}

impl FieldRangeRole {
    /// Returns the `role` attribute for the `Input` container (`"group"` standalone, `"presentation"` in a range).
    #[inline]
    pub const fn input_role(self) -> &'static str {
        match self {
            Self::Standalone => "group",
            Self::RangeStart | Self::RangeEnd => "presentation",
        }
    }

    /// Returns the optional `data-range-field-part` token (`"start"` or `"end"`).
    #[inline]
    pub const fn range_part_attr(self) -> Option<&'static str> {
        match self {
            Self::Standalone => None,
            Self::RangeStart => Some("start"),
            Self::RangeEnd => Some("end"),
        }
    }
}

/// Which endpoint input a range field targets (`Start` or `End`).
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub enum FieldRangeEndpoint {
    Start,
    End,
}

impl FieldRangeEndpoint {
    #[inline]
    pub const fn as_str(self) -> &'static str {
        match self {
            Self::Start => "start",
            Self::End => "end",
        }
    }
}

/// Rendered segment paired with its template position and deterministic DOM element ID.
#[derive(Clone, Debug, PartialEq, Eq, Hash)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub struct FieldSegmentItem {
    pub index: usize,
    pub id: String,
    pub segment: FieldSegment,
}

impl FieldSegmentItem {
    #[inline]
    pub const fn kind(&self) -> SegmentKind {
        self.segment.kind
    }

    #[inline]
    pub fn value(&self) -> &str {
        &self.segment.value
    }

    #[inline]
    pub const fn is_placeholder(&self) -> bool {
        self.segment.is_placeholder
    }

    #[inline]
    pub const fn is_interactive(&self) -> bool {
        self.segment.kind.is_interactive()
    }

    #[inline]
    pub const fn is_editable(&self) -> bool {
        self.segment.kind.is_editable()
    }
}

/// Returns the first interactive (focusable) segment in `segments`.
pub fn first_interactive_field_segment(segments: &[FieldSegmentItem]) -> Option<&FieldSegmentItem> {
    segments.iter().find(|item| item.is_interactive())
}

/// Returns the last interactive (focusable) segment in `segments`.
pub fn last_interactive_field_segment(segments: &[FieldSegmentItem]) -> Option<&FieldSegmentItem> {
    segments.iter().rev().find(|item| item.is_interactive())
}

/// Resolves the next or previous interactive segment index relative to `current_index`.
pub fn resolve_adjacent_interactive_field_index(
    segments: &[FieldSegmentItem],
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

/// Deterministic ARIA ID relationships for single segmented fields (`DateField`, `TimeField`).
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct FieldRelationships {
    scope: ScopeHandle,
    input_id: String,
    label_id: String,
    description_id: String,
    live_region_id: String,
    hidden_input_id: String,
}

impl FieldRelationships {
    pub fn new(scope: ScopeHandle) -> Self {
        let input_id = scope.qualify("input");
        let label_id = scope.qualify("label");
        let description_id = scope.qualify("description");
        let live_region_id = scope.qualify("live-region");
        let hidden_input_id = scope.qualify("hidden-input");
        Self {
            scope,
            input_id,
            label_id,
            description_id,
            live_region_id,
            hidden_input_id,
        }
    }

    pub fn scope(&self) -> &ScopeHandle {
        &self.scope
    }

    pub fn input_id(&self) -> &str {
        &self.input_id
    }

    pub fn label_id(&self) -> &str {
        &self.label_id
    }

    pub fn description_id(&self) -> &str {
        &self.description_id
    }

    pub fn live_region_id(&self) -> &str {
        &self.live_region_id
    }

    pub fn hidden_input_id(&self) -> &str {
        &self.hidden_input_id
    }

    pub fn segment_id(&self, index: usize, kind: SegmentKind) -> String {
        self.scope
            .qualify(format!("segment-{index}-{}", kind.as_data_segment()))
    }
}

/// Deterministic ARIA ID relationships for range segmented fields (`DateRangeField`, `TimeRangeField`).
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct RangeFieldRelationships {
    scope: ScopeHandle,
    root_id: String,
    label_id: String,
    start_relationships: FieldRelationships,
    end_relationships: FieldRelationships,
}

impl RangeFieldRelationships {
    pub fn new(scope: ScopeHandle) -> Self {
        let root_id = scope.token();
        let label_id = scope.qualify("label");
        let start_relationships = FieldRelationships::new(scope.child("start"));
        let end_relationships = FieldRelationships::new(scope.child("end"));
        Self {
            scope,
            root_id,
            label_id,
            start_relationships,
            end_relationships,
        }
    }

    pub fn scope(&self) -> &ScopeHandle {
        &self.scope
    }

    pub fn root_id(&self) -> &str {
        &self.root_id
    }

    pub fn label_id(&self) -> &str {
        &self.label_id
    }

    pub fn start_relationships(&self) -> &FieldRelationships {
        &self.start_relationships
    }

    pub fn end_relationships(&self) -> &FieldRelationships {
        &self.end_relationships
    }
}

/// Zero-overhead policy trait specializing [`SegmentFieldDefinition`], [`SegmentFieldState`],
/// and [`SegmentFieldSnapshot`] for date vs time segmented fields.
pub trait SegmentFieldPolicy: Copy + Clone + Debug + Default + PartialEq + Eq + 'static {
    type Value: ChronoComparable + Copy + Debug + PartialEq + Eq + 'static;
    type GranularityProp: Copy + Clone + Debug + Default + PartialEq + Eq + 'static;
    type ResolvedGranularity: Copy + Clone + Debug + PartialEq + Eq + 'static;

    const SUPPORTS_TIME_ZONE_SEGMENT: bool;
    const INPUT_DATA_ATTR: &'static str;
    const LABEL_DATA_ATTR: &'static str;
    const SEGMENT_DATA_ATTR: &'static str;

    fn resolve_granularity(
        prop: Self::GranularityProp,
        value: Option<Self::Value>,
        placeholder: Self::Value,
    ) -> Self::ResolvedGranularity;

    fn new_engine(
        locale: &CalendarLocale,
        granularity: Self::ResolvedGranularity,
        hour_cycle: Option<HourCycle>,
        hide_time_zone: bool,
        placeholder: Self::Value,
    ) -> SegmentEngine;

    fn sync_engine_value(engine: &mut SegmentEngine, value: Option<Self::Value>);

    fn set_engine_placeholder(engine: &mut SegmentEngine, placeholder: Self::Value);

    fn extract_committed(result: &SegmentTransitionResult) -> Option<Self::Value>;

    fn validate_value(
        value: Option<Self::Value>,
        min: Option<Self::Value>,
        max: Option<Self::Value>,
        is_unavailable: bool,
        custom_errors: Option<Vec<String>>,
    ) -> Result<(), FieldInvalidReason>;

    fn format_iso_value(value: Option<Self::Value>) -> String;

    fn format_description(locale: &CalendarLocale, value: Option<Self::Value>) -> String;
}

/// Zero-overhead policy trait extending [`SegmentFieldPolicy`] for range fields
/// (`DateRangeField` vs `TimeRangeField`).
pub trait RangeFieldPolicy: SegmentFieldPolicy {
    /// Whether `end < start` is rejected with [`FieldInvalidReason::RangeOrder`].
    /// `true` for `DateFieldPolicy`; `false` for `TimeFieldPolicy` (overnight ranges allowed).
    const ENFORCE_RANGE_ORDER: bool;
    /// Whether individual endpoints support `is_date_unavailable` validation.
    const SUPPORTS_UNAVAILABLE: bool;
    const RANGE_ROOT_DATA_ATTR: &'static str;
    const RANGE_LABEL_DATA_ATTR: &'static str;

    fn resolve_range_granularity(
        prop: Self::GranularityProp,
        value: ChronoRange<Self::Value>,
        placeholder: Self::Value,
    ) -> Self::ResolvedGranularity;

    fn validate_range(
        range: ChronoRange<Self::Value>,
        min: Option<Self::Value>,
        max: Option<Self::Value>,
        is_unavailable: bool,
        custom_errors: Option<Vec<String>>,
    ) -> Result<(), FieldInvalidReason>;
}

/// Zero-sized policy marker for `DateField` and `DateRangeField`.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, Hash)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub struct DateFieldPolicy;

impl SegmentFieldPolicy for DateFieldPolicy {
    type Value = DateValue;
    type GranularityProp = Option<Granularity>;
    type ResolvedGranularity = Granularity;

    const SUPPORTS_TIME_ZONE_SEGMENT: bool = true;
    const INPUT_DATA_ATTR: &'static str = "data-date-field-input";
    const LABEL_DATA_ATTR: &'static str = "data-date-field-label";
    const SEGMENT_DATA_ATTR: &'static str = "data-date-field-segment";

    #[inline]
    fn resolve_granularity(
        prop: Self::GranularityProp,
        value: Option<Self::Value>,
        placeholder: Self::Value,
    ) -> Self::ResolvedGranularity {
        match prop {
            Some(g) => g,
            None => Granularity::infer_from_date_value(value, placeholder),
        }
    }

    #[inline]
    fn new_engine(
        locale: &CalendarLocale,
        granularity: Self::ResolvedGranularity,
        hour_cycle: Option<HourCycle>,
        hide_time_zone: bool,
        placeholder: Self::Value,
    ) -> SegmentEngine {
        SegmentEngine::new_date(
            locale.field_format(),
            granularity,
            hour_cycle,
            hide_time_zone,
            placeholder,
        )
    }

    #[inline]
    fn sync_engine_value(engine: &mut SegmentEngine, value: Option<Self::Value>) {
        engine.sync_from_date_value(value);
    }

    #[inline]
    fn set_engine_placeholder(engine: &mut SegmentEngine, placeholder: Self::Value) {
        engine.set_placeholder_date(placeholder);
    }

    #[inline]
    fn extract_committed(result: &SegmentTransitionResult) -> Option<Self::Value> {
        result.committed_date_value
    }

    #[inline]
    fn validate_value(
        value: Option<Self::Value>,
        min: Option<Self::Value>,
        max: Option<Self::Value>,
        is_unavailable: bool,
        custom_errors: Option<Vec<String>>,
    ) -> Result<(), FieldInvalidReason> {
        validate_date_value_with_unavailable(value, min, max, is_unavailable, custom_errors)
    }

    #[inline]
    fn format_iso_value(value: Option<Self::Value>) -> String {
        value.map(|v| v.to_iso_string()).unwrap_or_default()
    }

    fn format_description(locale: &CalendarLocale, value: Option<Self::Value>) -> String {
        match value {
            Some(DateValue::Date(d)) => locale.selected_announcement(d),
            Some(DateValue::DateTime(dt)) => {
                format!(
                    "{} {}",
                    locale.selected_announcement(dt.date),
                    dt.time.to_iso_string()
                )
            }
            None => String::new(),
        }
    }
}

impl RangeFieldPolicy for DateFieldPolicy {
    const ENFORCE_RANGE_ORDER: bool = true;
    const SUPPORTS_UNAVAILABLE: bool = true;
    const RANGE_ROOT_DATA_ATTR: &'static str = "data-date-range-field-root";
    const RANGE_LABEL_DATA_ATTR: &'static str = "data-date-range-field-label";

    #[inline]
    fn resolve_range_granularity(
        prop: Self::GranularityProp,
        value: ChronoRange<Self::Value>,
        placeholder: Self::Value,
    ) -> Self::ResolvedGranularity {
        match prop {
            Some(g) => g,
            None => Granularity::infer_from_date_value(value.start.or(value.end), placeholder),
        }
    }

    #[inline]
    fn validate_range(
        range: ChronoRange<Self::Value>,
        min: Option<Self::Value>,
        max: Option<Self::Value>,
        is_unavailable: bool,
        custom_errors: Option<Vec<String>>,
    ) -> Result<(), FieldInvalidReason> {
        validate_date_range_with_unavailable(range, min, max, is_unavailable, custom_errors)
    }
}

/// Zero-sized policy marker for `TimeField` and `TimeRangeField`.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, Hash)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub struct TimeFieldPolicy;

impl SegmentFieldPolicy for TimeFieldPolicy {
    type Value = TimeValue;
    type GranularityProp = TimeGranularity;
    type ResolvedGranularity = TimeGranularity;

    const SUPPORTS_TIME_ZONE_SEGMENT: bool = false;
    const INPUT_DATA_ATTR: &'static str = "data-time-field-input";
    const LABEL_DATA_ATTR: &'static str = "data-time-field-label";
    const SEGMENT_DATA_ATTR: &'static str = "data-time-field-segment";

    #[inline]
    fn resolve_granularity(
        prop: Self::GranularityProp,
        _value: Option<Self::Value>,
        _placeholder: Self::Value,
    ) -> Self::ResolvedGranularity {
        prop
    }

    #[inline]
    fn new_engine(
        locale: &CalendarLocale,
        granularity: Self::ResolvedGranularity,
        hour_cycle: Option<HourCycle>,
        _hide_time_zone: bool,
        placeholder: Self::Value,
    ) -> SegmentEngine {
        SegmentEngine::new_time(locale.field_format(), granularity, hour_cycle, placeholder)
    }

    #[inline]
    fn sync_engine_value(engine: &mut SegmentEngine, value: Option<Self::Value>) {
        engine.sync_from_time_value(value);
    }

    #[inline]
    fn set_engine_placeholder(engine: &mut SegmentEngine, placeholder: Self::Value) {
        engine.set_placeholder_time(placeholder);
    }

    #[inline]
    fn extract_committed(result: &SegmentTransitionResult) -> Option<Self::Value> {
        result.committed_time_value
    }

    #[inline]
    fn validate_value(
        value: Option<Self::Value>,
        min: Option<Self::Value>,
        max: Option<Self::Value>,
        _is_unavailable: bool,
        custom_errors: Option<Vec<String>>,
    ) -> Result<(), FieldInvalidReason> {
        validate_time_value(value, min, max, custom_errors)
    }

    #[inline]
    fn format_iso_value(value: Option<Self::Value>) -> String {
        value.map(|v| v.time().to_iso_string()).unwrap_or_default()
    }

    fn format_description(locale: &CalendarLocale, value: Option<Self::Value>) -> String {
        match value {
            Some(tv) => {
                let tpl = locale.field_template();
                format!("{}{}", tpl.selected_time_prefix, tv.time().to_iso_string())
            }
            None => String::new(),
        }
    }
}

impl RangeFieldPolicy for TimeFieldPolicy {
    const ENFORCE_RANGE_ORDER: bool = false;
    const SUPPORTS_UNAVAILABLE: bool = false;
    const RANGE_ROOT_DATA_ATTR: &'static str = "data-time-range-field-root";
    const RANGE_LABEL_DATA_ATTR: &'static str = "data-time-range-field-label";

    #[inline]
    fn resolve_range_granularity(
        prop: Self::GranularityProp,
        _value: ChronoRange<Self::Value>,
        _placeholder: Self::Value,
    ) -> Self::ResolvedGranularity {
        prop
    }

    #[inline]
    fn validate_range(
        range: ChronoRange<Self::Value>,
        min: Option<Self::Value>,
        max: Option<Self::Value>,
        _is_unavailable: bool,
        custom_errors: Option<Vec<String>>,
    ) -> Result<(), FieldInvalidReason> {
        validate_time_range(range, min, max, custom_errors)
    }
}

/// Policy-parameterized static configuration for single segmented fields (`DateField`, `TimeField`).
#[derive(Clone, Debug, PartialEq, Eq)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
#[cfg_attr(
    feature = "serde",
    serde(bound(
        serialize = "P::Value: serde::Serialize, P::GranularityProp: serde::Serialize",
        deserialize = "P::Value: serde::Deserialize<'de>, P::GranularityProp: serde::Deserialize<'de>"
    ))
)]
pub struct SegmentFieldDefinition<P: SegmentFieldPolicy> {
    pub locale: CalendarLocale,
    pub granularity: P::GranularityProp,
    pub hour_cycle: Option<HourCycle>,
    pub hide_time_zone: bool,
    pub min_value: Option<P::Value>,
    pub max_value: Option<P::Value>,
    pub disabled: bool,
    pub read_only: bool,
    pub required: bool,
    pub readonly_segments: Vec<EditableSegmentKind>,
    pub error_message_id: Option<String>,
    pub dir: Direction,
    #[cfg_attr(feature = "serde", serde(skip))]
    _policy: PhantomData<P>,
}

impl<P: SegmentFieldPolicy> Default for SegmentFieldDefinition<P> {
    fn default() -> Self {
        Self {
            locale: CalendarLocale::EnUs,
            granularity: P::GranularityProp::default(),
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
            _policy: PhantomData,
        }
    }
}

impl<P: SegmentFieldPolicy> SegmentFieldDefinition<P> {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn with_locale(mut self, locale: CalendarLocale) -> Self {
        self.locale = locale;
        self
    }

    pub fn with_granularity(mut self, granularity: P::GranularityProp) -> Self {
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

    pub fn with_min_value(mut self, min_value: Option<P::Value>) -> Self {
        self.min_value = min_value;
        self
    }

    pub fn with_max_value(mut self, max_value: Option<P::Value>) -> Self {
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

    pub fn resolved_granularity(
        &self,
        value: Option<P::Value>,
        placeholder: P::Value,
    ) -> P::ResolvedGranularity {
        P::resolve_granularity(self.granularity, value, placeholder)
    }

    pub fn is_segment_read_only(&self, kind: SegmentKind) -> bool {
        if self.read_only {
            return true;
        }
        match kind {
            SegmentKind::TimeZoneName => P::SUPPORTS_TIME_ZONE_SEGMENT,
            SegmentKind::Literal => false,
            _ => kind
                .as_editable()
                .is_some_and(|e| self.readonly_segments.contains(&e)),
        }
    }
}

/// Read-only snapshot of a policy-parameterized single segmented field.
#[derive(Clone, Debug, PartialEq, Eq)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
#[cfg_attr(
    feature = "serde",
    serde(bound(
        serialize = "P::Value: serde::Serialize, P::ResolvedGranularity: serde::Serialize",
        deserialize = "P::Value: serde::Deserialize<'de>, P::ResolvedGranularity: serde::Deserialize<'de>"
    ))
)]
pub struct SegmentFieldSnapshot<P: SegmentFieldPolicy> {
    pub value: Option<P::Value>,
    pub placeholder: P::Value,
    pub granularity: P::ResolvedGranularity,
    pub hour_cycle: HourCycle,
    pub segments: Vec<FieldSegmentItem>,
    pub is_invalid: bool,
    pub invalid_reason: Option<FieldInvalidReason>,
    pub disabled: bool,
    pub read_only: bool,
    pub required: bool,
    pub iso_value: String,
    pub description_text: String,
    #[cfg_attr(feature = "serde", serde(skip))]
    _policy: PhantomData<P>,
}

/// Policy-parameterized pure-Rust state machine for `DateField` and `TimeField`.
#[derive(Clone, Debug, PartialEq, Eq)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
#[cfg_attr(
    feature = "serde",
    serde(bound(
        serialize = "P::Value: serde::Serialize, P::GranularityProp: serde::Serialize",
        deserialize = "P::Value: serde::Deserialize<'de>, P::GranularityProp: serde::Deserialize<'de>"
    ))
)]
pub struct SegmentFieldState<P: SegmentFieldPolicy> {
    def: SegmentFieldDefinition<P>,
    engine: SegmentEngine,
    value: Option<P::Value>,
    placeholder: P::Value,
    external_invalid: bool,
    local_invalid_reason: Option<FieldInvalidReason>,
}

impl<P: SegmentFieldPolicy> SegmentFieldState<P> {
    pub fn new(
        def: SegmentFieldDefinition<P>,
        value: Option<P::Value>,
        placeholder: P::Value,
    ) -> Self {
        let granularity = def.resolved_granularity(value, placeholder);
        let mut engine = P::new_engine(
            &def.locale,
            granularity,
            def.hour_cycle,
            def.hide_time_zone,
            placeholder,
        );
        P::sync_engine_value(&mut engine, value);

        let local_invalid_reason =
            P::validate_value(value, def.min_value, def.max_value, false, None).err();

        Self {
            def,
            engine,
            value,
            placeholder,
            external_invalid: false,
            local_invalid_reason,
        }
    }

    pub fn def(&self) -> &SegmentFieldDefinition<P> {
        &self.def
    }

    pub fn engine(&self) -> &SegmentEngine {
        &self.engine
    }

    pub const fn value(&self) -> Option<P::Value> {
        self.value
    }

    pub const fn placeholder(&self) -> P::Value {
        self.placeholder
    }

    pub fn resolved_granularity(&self) -> P::ResolvedGranularity {
        self.def.resolved_granularity(self.value, self.placeholder)
    }

    pub fn effective_hour_cycle(&self) -> HourCycle {
        self.engine.effective_hour_cycle()
    }

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

    pub fn set_value(&mut self, value: Option<P::Value>) {
        if self.value != value {
            self.value = value;
            P::sync_engine_value(&mut self.engine, value);
        }
    }

    pub fn reset_value(&mut self, value: Option<P::Value>) {
        self.value = value;
        P::sync_engine_value(&mut self.engine, value);
    }

    pub fn set_placeholder(&mut self, placeholder: P::Value) {
        if self.placeholder != placeholder {
            self.placeholder = placeholder;
            P::set_engine_placeholder(&mut self.engine, placeholder);
        }
    }

    pub fn sync_definition(&mut self, next_def: SegmentFieldDefinition<P>) {
        let prev_granularity = self.resolved_granularity();
        let next_granularity = next_def.resolved_granularity(self.value, self.placeholder);
        let hide_tz_changed =
            P::SUPPORTS_TIME_ZONE_SEGMENT && self.def.hide_time_zone != next_def.hide_time_zone;

        self.def = next_def;

        if prev_granularity != next_granularity || hide_tz_changed {
            let mut next_engine = P::new_engine(
                &self.def.locale,
                next_granularity,
                self.def.hour_cycle,
                self.def.hide_time_zone,
                self.placeholder,
            );
            P::sync_engine_value(&mut next_engine, self.value);
            self.engine = next_engine;
        } else {
            self.engine.set_locale(self.def.locale.field_format());
            self.engine.set_hour_cycle(self.def.hour_cycle);
        }
    }

    pub fn evaluate_validation_with_unavailable(
        &mut self,
        is_unavailable: bool,
        custom_errors: Option<Vec<String>>,
    ) -> Option<FieldInvalidReason> {
        self.local_invalid_reason = P::validate_value(
            self.value,
            self.def.min_value,
            self.def.max_value,
            is_unavailable,
            custom_errors,
        )
        .err();
        self.local_invalid_reason.clone()
    }

    pub fn evaluate_validation(
        &mut self,
        custom_errors: Option<Vec<String>>,
    ) -> Option<FieldInvalidReason> {
        self.evaluate_validation_with_unavailable(false, custom_errors)
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
        if let Some(committed) = P::extract_committed(&result) {
            self.value = Some(committed);
            self.placeholder = committed;
            P::set_engine_placeholder(&mut self.engine, committed);
        } else if result.cleared_value {
            self.value = None;
        }

        Some(result)
    }

    pub fn iso_value(&self) -> String {
        P::format_iso_value(self.value)
    }

    pub fn description_text(&self) -> String {
        P::format_description(&self.def.locale, self.value)
    }

    pub fn rendered_segments(&self, rels: &FieldRelationships) -> Vec<FieldSegmentItem> {
        self.engine
            .render_segments()
            .into_iter()
            .enumerate()
            .map(|(index, segment)| FieldSegmentItem {
                id: rels.segment_id(index, segment.kind),
                index,
                segment,
            })
            .collect()
    }

    pub fn snapshot(&self, rels: &FieldRelationships) -> SegmentFieldSnapshot<P> {
        SegmentFieldSnapshot {
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
            _policy: PhantomData,
        }
    }
}

/// Policy-parameterized static configuration for range segmented fields (`DateRangeField`, `TimeRangeField`).
#[derive(Clone, Debug, PartialEq, Eq)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
#[cfg_attr(
    feature = "serde",
    serde(bound(
        serialize = "P::Value: serde::Serialize, P::GranularityProp: serde::Serialize",
        deserialize = "P::Value: serde::Deserialize<'de>, P::GranularityProp: serde::Deserialize<'de>"
    ))
)]
pub struct RangeFieldDefinition<P: RangeFieldPolicy> {
    pub locale: CalendarLocale,
    pub granularity: P::GranularityProp,
    pub hour_cycle: Option<HourCycle>,
    pub hide_time_zone: bool,
    pub min_value: Option<P::Value>,
    pub max_value: Option<P::Value>,
    pub disabled: bool,
    pub read_only: bool,
    pub required: bool,
    pub readonly_segments: Vec<EditableSegmentKind>,
    pub error_message_id: Option<String>,
    pub dir: Direction,
    #[cfg_attr(feature = "serde", serde(skip))]
    _policy: PhantomData<P>,
}

impl<P: RangeFieldPolicy> Default for RangeFieldDefinition<P> {
    fn default() -> Self {
        Self {
            locale: CalendarLocale::EnUs,
            granularity: P::GranularityProp::default(),
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
            _policy: PhantomData,
        }
    }
}

impl<P: RangeFieldPolicy> RangeFieldDefinition<P> {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn with_locale(mut self, locale: CalendarLocale) -> Self {
        self.locale = locale;
        self
    }

    pub fn with_granularity(mut self, granularity: P::GranularityProp) -> Self {
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

    pub fn with_min_value(mut self, min_value: Option<P::Value>) -> Self {
        self.min_value = min_value;
        self
    }

    pub fn with_max_value(mut self, max_value: Option<P::Value>) -> Self {
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

    pub fn resolved_granularity(
        &self,
        value: ChronoRange<P::Value>,
        placeholder: P::Value,
    ) -> P::ResolvedGranularity {
        P::resolve_range_granularity(self.granularity, value, placeholder)
    }
}

/// Read-only snapshot of a policy-parameterized range segmented field.
#[derive(Clone, Debug, PartialEq, Eq)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
#[cfg_attr(
    feature = "serde",
    serde(bound(
        serialize = "P::Value: serde::Serialize, P::ResolvedGranularity: serde::Serialize",
        deserialize = "P::Value: serde::Deserialize<'de>, P::ResolvedGranularity: serde::Deserialize<'de>"
    ))
)]
pub struct RangeFieldSnapshot<P: RangeFieldPolicy> {
    pub value: ChronoRange<P::Value>,
    pub placeholder: P::Value,
    pub granularity: P::ResolvedGranularity,
    pub hour_cycle: Option<HourCycle>,
    pub is_invalid: bool,
    pub invalid_reason: Option<FieldInvalidReason>,
    pub disabled: bool,
    pub read_only: bool,
    pub required: bool,
    #[cfg_attr(feature = "serde", serde(skip))]
    _policy: PhantomData<P>,
}

/// Policy-parameterized pure-Rust state machine for `DateRangeField` and `TimeRangeField`.
#[derive(Clone, Debug, PartialEq, Eq)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
#[cfg_attr(
    feature = "serde",
    serde(bound(
        serialize = "P::Value: serde::Serialize, P::GranularityProp: serde::Serialize",
        deserialize = "P::Value: serde::Deserialize<'de>, P::GranularityProp: serde::Deserialize<'de>"
    ))
)]
pub struct RangeFieldState<P: RangeFieldPolicy> {
    def: RangeFieldDefinition<P>,
    value: ChronoRange<P::Value>,
    placeholder: P::Value,
    external_invalid: bool,
    local_invalid_reason: Option<FieldInvalidReason>,
    start_unavailable: bool,
    end_unavailable: bool,
}

impl<P: RangeFieldPolicy> RangeFieldState<P> {
    pub fn new(
        def: RangeFieldDefinition<P>,
        value: ChronoRange<P::Value>,
        placeholder: P::Value,
    ) -> Self {
        let local_invalid_reason =
            P::validate_range(value, def.min_value, def.max_value, false, None).err();
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

    pub fn def(&self) -> &RangeFieldDefinition<P> {
        &self.def
    }

    pub const fn value(&self) -> ChronoRange<P::Value> {
        self.value
    }

    pub const fn placeholder(&self) -> P::Value {
        self.placeholder
    }

    pub fn resolved_granularity(&self) -> P::ResolvedGranularity {
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
    /// - `Unavailable` marks ONLY the endpoint(s) whose date is unavailable when `P::SUPPORTS_UNAVAILABLE`
    ///   (falling back to present endpoints if per-endpoint flags were not explicitly set).
    /// - `Min` / `Max` mark ONLY the endpoint(s) whose value actually violates the bound,
    ///   leaving an untouched empty or in-bounds sibling valid.
    pub fn is_part_invalid(&self, endpoint: FieldRangeEndpoint) -> bool {
        if self.external_invalid {
            return true;
        }
        match &self.local_invalid_reason {
            None => false,
            Some(FieldInvalidReason::RangeOrder) | Some(FieldInvalidReason::Custom(_)) => true,
            Some(FieldInvalidReason::Unavailable) => {
                if P::SUPPORTS_UNAVAILABLE {
                    if self.start_unavailable || self.end_unavailable {
                        match endpoint {
                            FieldRangeEndpoint::Start => self.start_unavailable,
                            FieldRangeEndpoint::End => self.end_unavailable,
                        }
                    } else {
                        match endpoint {
                            FieldRangeEndpoint::Start => self.value.start.is_some(),
                            FieldRangeEndpoint::End => self.value.end.is_some(),
                        }
                    }
                } else {
                    true
                }
            }
            Some(FieldInvalidReason::Min) | Some(FieldInvalidReason::Max) => {
                let part_val = match endpoint {
                    FieldRangeEndpoint::Start => self.value.start,
                    FieldRangeEndpoint::End => self.value.end,
                };
                P::validate_value(part_val, self.def.min_value, self.def.max_value, false, None)
                    .is_err()
            }
        }
    }

    pub fn sync_definition(&mut self, def: RangeFieldDefinition<P>) {
        self.def = def;
    }

    pub fn set_value(&mut self, value: ChronoRange<P::Value>) {
        self.value = value;
    }

    pub fn set_part_value(&mut self, endpoint: FieldRangeEndpoint, next: Option<P::Value>) {
        match endpoint {
            FieldRangeEndpoint::Start => self.value.start = next,
            FieldRangeEndpoint::End => self.value.end = next,
        }
        if let Some(val) = next {
            self.placeholder = val;
        }
    }

    pub fn set_placeholder(&mut self, placeholder: P::Value) {
        self.placeholder = placeholder;
    }

    pub fn evaluate_validation_with_unavailable(
        &mut self,
        start_unavailable: bool,
        end_unavailable: bool,
        custom_errors: Option<Vec<String>>,
    ) -> Option<FieldInvalidReason> {
        self.start_unavailable = P::SUPPORTS_UNAVAILABLE && start_unavailable;
        self.end_unavailable = P::SUPPORTS_UNAVAILABLE && end_unavailable;
        self.local_invalid_reason = P::validate_range(
            self.value,
            self.def.min_value,
            self.def.max_value,
            self.start_unavailable || self.end_unavailable,
            custom_errors,
        )
        .err();
        self.local_invalid_reason.clone()
    }

    pub fn evaluate_validation(
        &mut self,
        custom_errors: Option<Vec<String>>,
    ) -> Option<FieldInvalidReason> {
        self.evaluate_validation_with_unavailable(false, false, custom_errors)
    }

    pub fn snapshot(&self) -> RangeFieldSnapshot<P> {
        RangeFieldSnapshot {
            value: self.value,
            placeholder: self.placeholder,
            granularity: self.resolved_granularity(),
            hour_cycle: self.def.hour_cycle,
            is_invalid: self.is_invalid(),
            invalid_reason: self.local_invalid_reason.clone(),
            disabled: self.def.disabled,
            read_only: self.def.read_only,
            required: self.def.required,
            _policy: PhantomData,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::foundation::date_time::{CalendarDate, CalendarTime, DateValueRange, TimeRange};

    fn d(y: i32, m: u8, day: u8) -> DateValue {
        DateValue::Date(CalendarDate::new(y, m, day).expect("valid date"))
    }

    fn t(h: u8, m: u8) -> TimeValue {
        TimeValue::Time(CalendarTime::new(h, m, 0, 0).expect("valid time"))
    }

    #[test]
    fn date_range_policy_enforces_range_order_and_endpoint_isolation() {
        assert!(DateFieldPolicy::ENFORCE_RANGE_ORDER);
        assert!(DateFieldPolicy::SUPPORTS_UNAVAILABLE);

        // Inverted date range invalidates both endpoints with RangeOrder.
        let inverted = DateValueRange::new(Some(d(2026, 4, 20)), Some(d(2026, 4, 10)));
        let state = RangeFieldState::<DateFieldPolicy>::new(
            RangeFieldDefinition::new(),
            inverted,
            d(2026, 4, 1),
        );
        assert!(state.is_invalid());
        assert_eq!(state.invalid_reason(), Some(&FieldInvalidReason::RangeOrder));
        assert!(state.is_part_invalid(FieldRangeEndpoint::Start));
        assert!(state.is_part_invalid(FieldRangeEndpoint::End));

        // DEF-5C-13: start < min with end == None invalidates ONLY start.
        let partial_min = DateValueRange::new(Some(d(2026, 4, 1)), None);
        let def = RangeFieldDefinition::<DateFieldPolicy>::new()
            .with_min_value(Some(d(2026, 4, 10)));
        let min_state = RangeFieldState::<DateFieldPolicy>::new(def, partial_min, d(2026, 4, 10));
        assert_eq!(min_state.invalid_reason(), Some(&FieldInvalidReason::Min));
        assert!(min_state.is_part_invalid(FieldRangeEndpoint::Start));
        assert!(!min_state.is_part_invalid(FieldRangeEndpoint::End));

        // 5C D-8: start unavailable + end valid invalidates ONLY start.
        let valid_pair = DateValueRange::new(Some(d(2026, 4, 12)), Some(d(2026, 4, 15)));
        let mut unavail_state = RangeFieldState::<DateFieldPolicy>::new(
            RangeFieldDefinition::new(),
            valid_pair,
            d(2026, 4, 1),
        );
        let reason = unavail_state.evaluate_validation_with_unavailable(true, false, None);
        assert_eq!(reason, Some(FieldInvalidReason::Unavailable));
        assert!(unavail_state.is_part_invalid(FieldRangeEndpoint::Start));
        assert!(!unavail_state.is_part_invalid(FieldRangeEndpoint::End));
    }

    #[test]
    fn time_range_policy_allows_overnight_ranges_and_isolates_min_max() {
        assert!(!TimeFieldPolicy::ENFORCE_RANGE_ORDER);
        assert!(!TimeFieldPolicy::SUPPORTS_UNAVAILABLE);

        // Overnight 20:00..05:00 is valid by default.
        let overnight = TimeRange::new(Some(t(20, 0)), Some(t(5, 0)));
        let state = RangeFieldState::<TimeFieldPolicy>::new(
            RangeFieldDefinition::new(),
            overnight,
            t(12, 0),
        );
        assert!(!state.is_invalid());
        assert_eq!(state.invalid_reason(), None);
        assert!(!state.is_part_invalid(FieldRangeEndpoint::Start));
        assert!(!state.is_part_invalid(FieldRangeEndpoint::End));

        // Out-of-bounds start with valid end invalidates ONLY start.
        let def = RangeFieldDefinition::<TimeFieldPolicy>::new()
            .with_min_value(Some(t(9, 0)))
            .with_max_value(Some(t(17, 0)));
        let bounded = RangeFieldState::<TimeFieldPolicy>::new(
            def,
            TimeRange::new(Some(t(8, 0)), Some(t(12, 0))),
            t(9, 0),
        );
        assert_eq!(bounded.invalid_reason(), Some(&FieldInvalidReason::Min));
        assert!(bounded.is_part_invalid(FieldRangeEndpoint::Start));
        assert!(!bounded.is_part_invalid(FieldRangeEndpoint::End));
    }

    #[test]
    fn segment_field_state_and_relationships_work_for_both_policies() {
        let rels = FieldRelationships::new(ScopeHandle::root("date-field"));
        let mut date_state = SegmentFieldState::<DateFieldPolicy>::new(
            SegmentFieldDefinition::new(),
            Some(d(2026, 4, 15)),
            d(2026, 4, 1),
        );
        assert_eq!(date_state.iso_value(), "2026-04-15");
        let date_snap = date_state.snapshot(&rels);
        assert!(!date_snap.is_invalid);
        assert!(first_interactive_field_segment(&date_snap.segments).is_some());
        assert!(last_interactive_field_segment(&date_snap.segments).is_some());

        let _ = date_state.apply_intent(SegmentKind::Day, FieldSegmentIntent::Increment);
        assert_eq!(date_state.iso_value(), "2026-04-16");

        let time_rels = FieldRelationships::new(ScopeHandle::root("time-field"));
        let mut time_state = SegmentFieldState::<TimeFieldPolicy>::new(
            SegmentFieldDefinition::new().with_hour_cycle(Some(HourCycle::H24)),
            Some(t(14, 30)),
            t(0, 0),
        );
        assert_eq!(time_state.iso_value(), "14:30:00");
        let _ = time_state.apply_intent(SegmentKind::Minute, FieldSegmentIntent::Increment);
        assert_eq!(time_state.iso_value(), "14:31:00");
        assert!(!time_state.snapshot(&time_rels).is_invalid);

        let range_rels = RangeFieldRelationships::new(ScopeHandle::root("range"));
        assert_ne!(
            range_rels.start_relationships().input_id(),
            range_rels.end_relationships().input_id()
        );
    }
}

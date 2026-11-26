use crate::foundation::date_time::{
    CalendarTime, TimeValue,
    field_policy::{SegmentFieldDefinition, SegmentFieldState, TimeFieldPolicy},
};
pub use crate::foundation::date_time::field_policy::{
    first_interactive_field_segment as first_interactive_time_segment,
    last_interactive_field_segment as last_interactive_time_segment,
    resolve_adjacent_interactive_field_index as resolve_adjacent_interactive_time_index,
};

/// Static configuration for a `TimeField`.
pub type TimeFieldDefinition = SegmentFieldDefinition<TimeFieldPolicy>;

/// Pure-Rust state machine for `TimeField`, wrapping [`crate::foundation::date_time::SegmentEngine`].
pub type TimeFieldState = SegmentFieldState<TimeFieldPolicy>;

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

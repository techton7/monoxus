use crate::foundation::date_time::{
    CalendarDate, CalendarDateTime, CalendarTime, DateValue, Granularity,
    field_policy::{DateFieldPolicy, SegmentFieldDefinition, SegmentFieldState},
};
pub use crate::foundation::date_time::field_policy::{
    first_interactive_field_segment as first_interactive_segment,
    last_interactive_field_segment as last_interactive_segment,
    resolve_adjacent_interactive_field_index as resolve_adjacent_interactive_index,
};

/// Static configuration for a `DateField`.
pub type DateFieldDefinition = SegmentFieldDefinition<DateFieldPolicy>;

/// Pure-Rust state machine for `DateField`, wrapping [`crate::foundation::date_time::SegmentEngine`].
pub type DateFieldState = SegmentFieldState<DateFieldPolicy>;

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

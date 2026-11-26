use crate::foundation::date_time::field_policy::{
    RangeFieldDefinition, RangeFieldState, TimeFieldPolicy,
};

/// Static configuration for `TimeRangeField`.
pub type TimeRangeFieldDefinition = RangeFieldDefinition<TimeFieldPolicy>;

/// Pure-Rust state machine for `TimeRangeField`.
///
/// Overnight ranges (`end < start`, e.g. `20:00..05:00`) are valid by default (`5C D-2`),
/// matching `bits-ui` (`time-range-field.svelte.ts:186-224`); same-day ordering is opt-in
/// via the consumer `validate` callback.
pub type TimeRangeFieldState = RangeFieldState<TimeFieldPolicy>;

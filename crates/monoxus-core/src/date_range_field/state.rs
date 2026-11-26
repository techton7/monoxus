use crate::foundation::date_time::field_policy::{
    DateFieldPolicy, RangeFieldDefinition, RangeFieldState,
};

/// Static configuration for `DateRangeField`.
pub type DateRangeFieldDefinition = RangeFieldDefinition<DateFieldPolicy>;

/// Pure-Rust state machine for `DateRangeField`, including built-in `end < start` `RangeOrder` validation (`5C D-2`).
pub type DateRangeFieldState = RangeFieldState<DateFieldPolicy>;

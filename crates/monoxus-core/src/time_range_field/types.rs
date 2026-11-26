use crate::foundation::date_time::field_policy::{RangeFieldSnapshot, TimeFieldPolicy};
pub use crate::foundation::date_time::field_policy::{
    FieldPart as TimeRangeFieldPart, FieldRangeEndpoint as RangeFieldInputKind,
    FieldRangeEndpoint as RangeFieldPart, RANGE_FIELD_PARTS as TIME_RANGE_FIELD_PARTS,
};

/// Read-only snapshot of a `TimeRangeField`.
pub type TimeRangeFieldSnapshot = RangeFieldSnapshot<TimeFieldPolicy>;

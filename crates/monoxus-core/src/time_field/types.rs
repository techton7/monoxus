use crate::foundation::date_time::field_policy::{SegmentFieldSnapshot, TimeFieldPolicy};
pub use crate::foundation::date_time::field_policy::{
    FIELD_PARTS as TIME_FIELD_PARTS, FieldPart as TimeFieldPart,
    FieldRangeRole as TimeFieldRangeRole, FieldSegmentItem as TimeFieldSegmentItem,
};

/// Read-only snapshot of a `TimeField` state suitable for rendering and context inspection.
pub type TimeFieldSnapshot = SegmentFieldSnapshot<TimeFieldPolicy>;

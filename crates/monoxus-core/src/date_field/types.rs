use crate::foundation::date_time::field_policy::{DateFieldPolicy, SegmentFieldSnapshot};
pub use crate::foundation::date_time::field_policy::{
    FIELD_PARTS as DATE_FIELD_PARTS, FieldPart as DateFieldPart,
    FieldRangeRole as DateFieldRangeRole, FieldSegmentItem as DateFieldSegmentItem,
};

/// Read-only snapshot of a `DateField` state suitable for rendering and context inspection.
pub type DateFieldSnapshot = SegmentFieldSnapshot<DateFieldPolicy>;

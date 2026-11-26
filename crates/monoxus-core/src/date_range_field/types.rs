use crate::foundation::date_time::field_policy::{DateFieldPolicy, RangeFieldSnapshot};
pub use crate::foundation::date_time::field_policy::{
    FieldPart as DateRangeFieldPart, FieldRangeEndpoint as RangeFieldInputKind,
    FieldRangeEndpoint as RangeFieldPart, RANGE_FIELD_PARTS as DATE_RANGE_FIELD_PARTS,
};

/// Read-only snapshot of a `DateRangeField`.
pub type DateRangeFieldSnapshot = RangeFieldSnapshot<DateFieldPolicy>;

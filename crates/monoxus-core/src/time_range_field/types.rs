pub use crate::date_range_field::{RangeFieldInputKind, RangeFieldPart};

use crate::foundation::date_time::{
    FieldInvalidReason, HourCycle, TimeGranularity, TimeRange, TimeValue,
};

/// Structural parts of the `TimeRangeField` primitive.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub enum TimeRangeFieldPart {
    Root,
    Input,
    Label,
    Segment,
}

impl TimeRangeFieldPart {
    pub const fn as_str(&self) -> &'static str {
        match self {
            Self::Root => "root",
            Self::Input => "input",
            Self::Label => "label",
            Self::Segment => "segment",
        }
    }
}

pub const TIME_RANGE_FIELD_PARTS: [TimeRangeFieldPart; 4] = [
    TimeRangeFieldPart::Root,
    TimeRangeFieldPart::Input,
    TimeRangeFieldPart::Label,
    TimeRangeFieldPart::Segment,
];

/// Read-only snapshot of a `TimeRangeField`.
#[derive(Clone, Debug, PartialEq, Eq)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub struct TimeRangeFieldSnapshot {
    pub value: TimeRange,
    pub placeholder: TimeValue,
    pub granularity: TimeGranularity,
    pub hour_cycle: Option<HourCycle>,
    pub is_invalid: bool,
    pub invalid_reason: Option<FieldInvalidReason>,
    pub disabled: bool,
    pub read_only: bool,
    pub required: bool,
}

use crate::foundation::date_time::{
    DateValue, DateValueRange, FieldInvalidReason, Granularity, HourCycle,
};

/// Which endpoint input a range field input targets (`Start` or `End`).
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub enum RangeFieldInputKind {
    Start,
    End,
}

impl RangeFieldInputKind {
    pub const fn as_str(self) -> &'static str {
        match self {
            Self::Start => "start",
            Self::End => "end",
        }
    }
}

/// Alias matching ledger terminology.
pub type RangeFieldPart = RangeFieldInputKind;

/// Structural parts of the `DateRangeField` primitive.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub enum DateRangeFieldPart {
    Root,
    Input,
    Label,
    Segment,
}

impl DateRangeFieldPart {
    pub const fn as_str(&self) -> &'static str {
        match self {
            Self::Root => "root",
            Self::Input => "input",
            Self::Label => "label",
            Self::Segment => "segment",
        }
    }
}

pub const DATE_RANGE_FIELD_PARTS: [DateRangeFieldPart; 4] = [
    DateRangeFieldPart::Root,
    DateRangeFieldPart::Input,
    DateRangeFieldPart::Label,
    DateRangeFieldPart::Segment,
];

/// Read-only snapshot of a `DateRangeField`.
#[derive(Clone, Debug, PartialEq, Eq)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub struct DateRangeFieldSnapshot {
    pub value: DateValueRange,
    pub placeholder: DateValue,
    pub granularity: Granularity,
    pub hour_cycle: Option<HourCycle>,
    pub is_invalid: bool,
    pub invalid_reason: Option<FieldInvalidReason>,
    pub disabled: bool,
    pub read_only: bool,
    pub required: bool,
}

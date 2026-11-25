use crate::foundation::date_time::{
    FieldInvalidReason, FieldSegment, HourCycle, SegmentKind, TimeGranularity, TimeValue,
};

/// Structural parts of the `TimeField` primitive.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub enum TimeFieldPart {
    Input,
    Label,
    Segment,
}

impl TimeFieldPart {
    pub const fn as_str(&self) -> &'static str {
        match self {
            Self::Input => "input",
            Self::Label => "label",
            Self::Segment => "segment",
        }
    }
}

pub const TIME_FIELD_PARTS: [TimeFieldPart; 3] = [
    TimeFieldPart::Input,
    TimeFieldPart::Label,
    TimeFieldPart::Segment,
];

/// Whether a `TimeField` is rendered standalone or as the `start`/`end` input of a `TimeRangeField`.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, Hash)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub enum TimeFieldRangeRole {
    #[default]
    Standalone,
    RangeStart,
    RangeEnd,
}

impl TimeFieldRangeRole {
    pub const fn input_role(self) -> &'static str {
        match self {
            Self::Standalone => "group",
            Self::RangeStart | Self::RangeEnd => "presentation",
        }
    }

    pub const fn range_part_attr(self) -> Option<&'static str> {
        match self {
            Self::Standalone => None,
            Self::RangeStart => Some("start"),
            Self::RangeEnd => Some("end"),
        }
    }
}

/// Rendered time segment paired with its template position and deterministic DOM element ID.
#[derive(Clone, Debug, PartialEq, Eq, Hash)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub struct TimeFieldSegmentItem {
    pub index: usize,
    pub id: String,
    pub segment: FieldSegment,
}

impl TimeFieldSegmentItem {
    #[inline]
    pub const fn kind(&self) -> SegmentKind {
        self.segment.kind
    }

    #[inline]
    pub fn value(&self) -> &str {
        &self.segment.value
    }

    #[inline]
    pub const fn is_placeholder(&self) -> bool {
        self.segment.is_placeholder
    }

    #[inline]
    pub const fn is_interactive(&self) -> bool {
        self.segment.kind.is_interactive()
    }

    #[inline]
    pub const fn is_editable(&self) -> bool {
        self.segment.kind.is_editable()
    }
}

/// Read-only snapshot of a `TimeField` state suitable for rendering and context inspection.
#[derive(Clone, Debug, PartialEq, Eq)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub struct TimeFieldSnapshot {
    pub value: Option<TimeValue>,
    pub placeholder: TimeValue,
    pub granularity: TimeGranularity,
    pub hour_cycle: HourCycle,
    pub segments: Vec<TimeFieldSegmentItem>,
    pub is_invalid: bool,
    pub invalid_reason: Option<FieldInvalidReason>,
    pub disabled: bool,
    pub read_only: bool,
    pub required: bool,
    pub iso_value: String,
    pub description_text: String,
}

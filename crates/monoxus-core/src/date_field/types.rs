use crate::foundation::date_time::{
    DateValue, FieldInvalidReason, FieldSegment, Granularity, HourCycle, SegmentKind,
};

/// Structural parts of the `DateField` primitive.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub enum DateFieldPart {
    Input,
    Label,
    Segment,
}

impl DateFieldPart {
    pub const fn as_str(&self) -> &'static str {
        match self {
            Self::Input => "input",
            Self::Label => "label",
            Self::Segment => "segment",
        }
    }
}

pub const DATE_FIELD_PARTS: [DateFieldPart; 3] = [
    DateFieldPart::Input,
    DateFieldPart::Label,
    DateFieldPart::Segment,
];

/// Whether a `DateField` is rendered standalone or as the `start`/`end` input of a `DateRangeField`.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, Hash)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub enum DateFieldRangeRole {
    #[default]
    Standalone,
    RangeStart,
    RangeEnd,
}

impl DateFieldRangeRole {
    /// Returns the `role` attribute for the `Input` container (`"group"` standalone, `"presentation"` in a range).
    pub const fn input_role(self) -> &'static str {
        match self {
            Self::Standalone => "group",
            Self::RangeStart | Self::RangeEnd => "presentation",
        }
    }

    /// Returns the optional `data-range-field-part` token (`"start"` or `"end"`).
    pub const fn range_part_attr(self) -> Option<&'static str> {
        match self {
            Self::Standalone => None,
            Self::RangeStart => Some("start"),
            Self::RangeEnd => Some("end"),
        }
    }
}

/// Rendered segment paired with its template position and deterministic DOM element ID.
#[derive(Clone, Debug, PartialEq, Eq, Hash)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub struct DateFieldSegmentItem {
    pub index: usize,
    pub id: String,
    pub segment: FieldSegment,
}

impl DateFieldSegmentItem {
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

/// Read-only snapshot of a `DateField` state suitable for rendering and context inspection.
#[derive(Clone, Debug, PartialEq, Eq)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub struct DateFieldSnapshot {
    pub value: Option<DateValue>,
    pub placeholder: DateValue,
    pub granularity: Granularity,
    pub hour_cycle: HourCycle,
    pub segments: Vec<DateFieldSegmentItem>,
    pub is_invalid: bool,
    pub invalid_reason: Option<FieldInvalidReason>,
    pub disabled: bool,
    pub read_only: bool,
    pub required: bool,
    pub iso_value: String,
    pub description_text: String,
}

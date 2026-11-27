/// Structural parts of the `monoxus::field` compound primitive.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub enum FieldPart {
    Root,
    FieldSet,
    Legend,
    Label,
    Control,
    Description,
    Error,
}

/// Exhaustive ordered list of all `FieldPart` variants.
pub const FIELD_PARTS: [FieldPart; 7] = [
    FieldPart::Root,
    FieldPart::FieldSet,
    FieldPart::Legend,
    FieldPart::Label,
    FieldPart::Control,
    FieldPart::Description,
    FieldPart::Error,
];

impl FieldPart {
    pub const fn as_str(&self) -> &'static str {
        match self {
            Self::Root => "root",
            Self::FieldSet => "fieldset",
            Self::Legend => "legend",
            Self::Label => "label",
            Self::Control => "control",
            Self::Description => "description",
            Self::Error => "error",
        }
    }

    pub const fn data_attr(&self) -> &'static str {
        match self {
            Self::Root => "data-monoxus-field-root",
            Self::FieldSet => "data-monoxus-field-set",
            Self::Legend => "data-monoxus-field-legend",
            Self::Label => "data-monoxus-field-label",
            Self::Control => "data-monoxus-field-control",
            Self::Description => "data-monoxus-field-description",
            Self::Error => "data-monoxus-field-error",
        }
    }
}

/// Layout orientation hint propagated via `data-orientation`.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, Hash)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub enum FieldOrientation {
    #[default]
    Vertical,
    Horizontal,
    Responsive,
}

impl FieldOrientation {
    pub const fn as_str(&self) -> &'static str {
        match self {
            Self::Vertical => "vertical",
            Self::Horizontal => "horizontal",
            Self::Responsive => "responsive",
        }
    }
}

/// Visual styling variant propagated on `FieldLegend` via `data-variant`.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, Hash)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub enum FieldLegendVariant {
    #[default]
    Legend,
    Label,
}

impl FieldLegendVariant {
    pub const fn as_str(&self) -> &'static str {
        match self {
            Self::Legend => "legend",
            Self::Label => "label",
        }
    }
}

/// Optional `aria-live` override for `FieldError` (defaults to `Off` to avoid duplicate
/// screen-reader announcements alongside `role="alert"`).
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, Hash)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub enum FieldErrorLiveMode {
    Assertive,
    Polite,
    #[default]
    Off,
}

impl FieldErrorLiveMode {
    pub const fn as_aria_live(&self) -> Option<&'static str> {
        match self {
            Self::Assertive => Some("assertive"),
            Self::Polite => Some("polite"),
            Self::Off => None,
        }
    }
}

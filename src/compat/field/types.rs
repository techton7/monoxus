// [KEEP-MONOXUS-BRAND]
pub use ui_sdk::field::types::{FieldErrorLiveMode, FieldLegendVariant, FieldOrientation};

/// Structural parts of the `monoxus::field` compound primitive.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
#[derive(serde::Serialize, serde::Deserialize)]
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

impl From<ui_sdk::field::types::FieldPart> for FieldPart {
    fn from(part: ui_sdk::field::types::FieldPart) -> Self {
        match part {
            ui_sdk::field::types::FieldPart::Root => Self::Root,
            ui_sdk::field::types::FieldPart::FieldSet => Self::FieldSet,
            ui_sdk::field::types::FieldPart::Legend => Self::Legend,
            ui_sdk::field::types::FieldPart::Label => Self::Label,
            ui_sdk::field::types::FieldPart::Control => Self::Control,
            ui_sdk::field::types::FieldPart::Description => Self::Description,
            ui_sdk::field::types::FieldPart::Error => Self::Error,
        }
    }
}

impl From<FieldPart> for ui_sdk::field::types::FieldPart {
    fn from(part: FieldPart) -> Self {
        match part {
            FieldPart::Root => Self::Root,
            FieldPart::FieldSet => Self::FieldSet,
            FieldPart::Legend => Self::Legend,
            FieldPart::Label => Self::Label,
            FieldPart::Control => Self::Control,
            FieldPart::Description => Self::Description,
            FieldPart::Error => Self::Error,
        }
    }
}

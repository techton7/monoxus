pub use crate::foundation::shared::{Direction, Orientation};

/// Default timeout (in milliseconds) for decimal keyboard input sequences (e.g. `2` -> `.` -> `5`).
pub const RATING_GROUP_KEY_SEQUENCE_TIMEOUT_MS: u64 = 1000;

/// Parts enum representing functional sub-elements of the rating group family.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub enum RatingGroupPart {
    Root,
    Item,
}

/// Static array of all constituent parts of the rating group family.
pub const RATING_GROUP_PARTS: [RatingGroupPart; 2] =
    [RatingGroupPart::Root, RatingGroupPart::Item];

/// Visual fill state for a single item inside a `RatingGroup`.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, Hash)]
pub enum RatingGroupItemState {
    Active,
    Partial,
    #[default]
    Inactive,
}

impl RatingGroupItemState {
    pub const fn as_str(&self) -> &'static str {
        match self {
            Self::Active => "active",
            Self::Partial => "partial",
            Self::Inactive => "inactive",
        }
    }
}

/// Snapshot metadata for one item in a `RatingGroup`.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct RatingGroupItemData {
    pub index: usize,
    pub value: f64,
    pub state: RatingGroupItemState,
}

/// Host-neutral keyboard input intents consumed by `monoxus-core::rating_group`.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub enum RatingGroupKey {
    ArrowUp,
    ArrowRight,
    ArrowDown,
    ArrowLeft,
    Home,
    End,
    PageUp,
    PageDown,
    Digit(u8),
    DecimalPoint,
    Other,
}

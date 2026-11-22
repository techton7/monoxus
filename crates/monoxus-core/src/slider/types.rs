pub use crate::foundation::shared::{Direction, Orientation};

/// Parts enum representing distinct functional sub-elements of the slider family.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub enum SliderPart {
    Root,
    Track,
    Range,
    Thumb,
    Tick,
    TickLabel,
    ThumbLabel,
}

/// Static array of all constituent parts of the slider family.
pub const SLIDER_PARTS: [SliderPart; 7] = [
    SliderPart::Root,
    SliderPart::Track,
    SliderPart::Range,
    SliderPart::Thumb,
    SliderPart::Tick,
    SliderPart::TickLabel,
    SliderPart::ThumbLabel,
];

/// Policy governing multi-thumb behavior when adjacent thumbs intersect or cross over.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, Hash)]
pub enum SliderThumbOrderPolicy {
    /// Thumbs dynamically reorder when dragged past each other, transferring active focus (default).
    #[default]
    Swap,
    /// Thumbs are physically blocked from crossing each other, preserving order and respecting minimum separation.
    Preserve,
}

/// Positioning alignment policy for slider thumbs within track boundaries.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, Hash)]
pub enum SliderThumbAlignment {
    /// Thumbs are contained within the track boundaries so their outer edges never exceed the track edges (default).
    #[default]
    Contain,
    /// Thumbs are centered exactly at the percentage position along the track with standard CSS translation.
    Exact,
}

/// Positioning anchor for discrete tick and dynamic thumb value labels.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, Hash)]
pub enum SliderLabelPosition {
    #[default]
    Top,
    Bottom,
    Left,
    Right,
}

/// Semantic alias for tick label side matching ledger specifications.
pub type SliderTickLabelSide = SliderLabelPosition;

/// Semantic alias for thumb label position.
pub type SliderThumbLabelPosition = SliderLabelPosition;

/// Semantic aliases for Direction and Orientation in the slider domain.
pub type SliderDirection = Direction;
pub type SliderOrientation = Orientation;

/// Value representation for single-thumb or multi-thumb range sliders.
#[derive(Clone, Debug, PartialEq)]
pub enum SliderValue {
    Single(f64),
    Multiple(Vec<f64>),
}

impl SliderValue {
    /// Returns the values as a normalized vector of floating-point numbers.
    pub fn to_vec(&self) -> Vec<f64> {
        match self {
            Self::Single(v) => vec![*v],
            Self::Multiple(v) => v.clone(),
        }
    }

    /// Returns `true` if this instance represents a single-thumb value.
    pub fn is_single(&self) -> bool {
        matches!(self, Self::Single(_))
    }

    /// Returns `true` if this instance represents a multi-thumb value.
    pub fn is_multiple(&self) -> bool {
        matches!(self, Self::Multiple(_))
    }

    /// Returns the number of thumb values represented.
    pub fn len(&self) -> usize {
        match self {
            Self::Single(_) => 1,
            Self::Multiple(v) => v.len(),
        }
    }

    /// Returns `true` if this instance contains no thumb values.
    pub fn is_empty(&self) -> bool {
        match self {
            Self::Single(_) => false,
            Self::Multiple(v) => v.is_empty(),
        }
    }
}

impl Default for SliderValue {
    fn default() -> Self {
        Self::Single(0.0)
    }
}

impl From<f64> for SliderValue {
    fn from(v: f64) -> Self {
        Self::Single(v)
    }
}

impl From<Vec<f64>> for SliderValue {
    fn from(v: Vec<f64>) -> Self {
        Self::Multiple(v)
    }
}

impl From<&[f64]> for SliderValue {
    fn from(v: &[f64]) -> Self {
        Self::Multiple(v.to_vec())
    }
}

impl<const N: usize> From<[f64; N]> for SliderValue {
    fn from(v: [f64; N]) -> Self {
        Self::Multiple(v.to_vec())
    }
}

/// Stepping specification combining uniform scalar stepping with optional discrete step points.
#[derive(Clone, Debug, PartialEq)]
pub struct SliderStepSpec {
    pub step: f64,
    pub steps: Option<Vec<f64>>,
}

impl Default for SliderStepSpec {
    fn default() -> Self {
        Self {
            step: 1.0,
            steps: None,
        }
    }
}

impl SliderStepSpec {
    pub fn new(step: f64) -> Self {
        Self { step, steps: None }
    }

    pub fn with_steps(mut self, steps: Vec<f64>) -> Self {
        self.steps = Some(steps);
        self
    }
}

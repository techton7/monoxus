use std::fmt;

use crate::foundation::compose::{Slottable, project_as_child};

/// Orientation of the separator widget.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, Hash)]
pub enum SeparatorOrientation {
    #[default]
    Horizontal,
    Vertical,
}

impl SeparatorOrientation {
    /// Returns the string representation matching DOM and ARIA orientation tokens.
    pub const fn as_str(&self) -> &'static str {
        match self {
            Self::Horizontal => "horizontal",
            Self::Vertical => "vertical",
        }
    }
}

impl fmt::Display for SeparatorOrientation {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(self.as_str())
    }
}

impl From<SeparatorOrientation> for crate::foundation::shared::Orientation {
    fn from(orientation: SeparatorOrientation) -> Self {
        match orientation {
            SeparatorOrientation::Horizontal => crate::foundation::shared::Orientation::Horizontal,
            SeparatorOrientation::Vertical => crate::foundation::shared::Orientation::Vertical,
        }
    }
}

impl From<crate::foundation::shared::Orientation> for SeparatorOrientation {
    fn from(orientation: crate::foundation::shared::Orientation) -> Self {
        match orientation {
            crate::foundation::shared::Orientation::Horizontal => SeparatorOrientation::Horizontal,
            crate::foundation::shared::Orientation::Vertical => SeparatorOrientation::Vertical,
            crate::foundation::shared::Orientation::Both => SeparatorOrientation::Horizontal,
        }
    }
}

/// Pure state definition and attribute projector for `Separator`.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub struct Separator {
    orientation: SeparatorOrientation,
    decorative: bool,
}

impl Separator {
    pub const fn new() -> Self {
        Self {
            orientation: SeparatorOrientation::Horizontal,
            decorative: false,
        }
    }

    pub const fn with_orientation(mut self, orientation: SeparatorOrientation) -> Self {
        self.orientation = orientation;
        self
    }

    pub const fn with_decorative(mut self, decorative: bool) -> Self {
        self.decorative = decorative;
        self
    }

    pub const fn orientation(&self) -> SeparatorOrientation {
        self.orientation
    }

    pub const fn is_decorative(&self) -> bool {
        self.decorative
    }

    /// Evaluates the 4-state WAI-ARIA and data-attribute matrix.
    pub const fn attributes(&self) -> SeparatorAttributes {
        let (role, aria_orientation) = if self.decorative {
            ("none", None)
        } else {
            match self.orientation {
                SeparatorOrientation::Horizontal => ("separator", None),
                SeparatorOrientation::Vertical => ("separator", Some("vertical")),
            }
        };

        SeparatorAttributes {
            role,
            aria_orientation,
            data_orientation: self.orientation.as_str(),
            data_separator_root: "",
        }
    }
}

/// Computed WAI-ARIA and data attributes published by `Separator`.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct SeparatorAttributes {
    pub role: &'static str,
    pub aria_orientation: Option<&'static str>,
    pub data_orientation: &'static str,
    pub data_separator_root: &'static str,
}

/// Projects headless separator attributes onto a slottable child without adding wrapper elements.
pub fn project_separator_as_child<T, C>(
    state: &Separator,
    target: T,
    content: Slottable<C>,
) -> (T, SeparatorAttributes, C) {
    let (target, content) = project_as_child(target, content);
    (target, state.attributes(), content)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn separator_4_state_aria_matrix() {
        // State 1: Semantic Horizontal (default)
        let s1 = Separator::new();
        let a1 = s1.attributes();
        assert_eq!(a1.role, "separator");
        assert_eq!(a1.aria_orientation, None);
        assert_eq!(a1.data_orientation, "horizontal");
        assert_eq!(a1.data_separator_root, "");

        // State 2: Semantic Vertical
        let s2 = Separator::new().with_orientation(SeparatorOrientation::Vertical);
        let a2 = s2.attributes();
        assert_eq!(a2.role, "separator");
        assert_eq!(a2.aria_orientation, Some("vertical"));
        assert_eq!(a2.data_orientation, "vertical");
        assert_eq!(a2.data_separator_root, "");

        // State 3: Decorative Horizontal
        let s3 = Separator::new().with_decorative(true);
        let a3 = s3.attributes();
        assert_eq!(a3.role, "none");
        assert_eq!(a3.aria_orientation, None);
        assert_eq!(a3.data_orientation, "horizontal");
        assert_eq!(a3.data_separator_root, "");

        // State 4: Decorative Vertical
        let s4 = Separator::new()
            .with_orientation(SeparatorOrientation::Vertical)
            .with_decorative(true);
        let a4 = s4.attributes();
        assert_eq!(a4.role, "none");
        assert_eq!(a4.aria_orientation, None);
        assert_eq!(a4.data_orientation, "vertical");
        assert_eq!(a4.data_separator_root, "");
    }

    #[test]
    fn separator_orientation_conversions() {
        use crate::foundation::shared::Orientation;

        assert_eq!(
            Orientation::from(SeparatorOrientation::Horizontal),
            Orientation::Horizontal
        );
        assert_eq!(
            Orientation::from(SeparatorOrientation::Vertical),
            Orientation::Vertical
        );

        assert_eq!(
            SeparatorOrientation::from(Orientation::Horizontal),
            SeparatorOrientation::Horizontal
        );
        assert_eq!(
            SeparatorOrientation::from(Orientation::Vertical),
            SeparatorOrientation::Vertical
        );
        assert_eq!(
            SeparatorOrientation::from(Orientation::Both),
            SeparatorOrientation::Horizontal
        );
    }

    #[test]
    fn separator_as_child_slot_projection() {
        let state = Separator::new()
            .with_orientation(SeparatorOrientation::Vertical)
            .with_decorative(false);
        let (target, attrs, content) =
            project_separator_as_child(&state, "hr-target", Slottable::new(()));
        assert_eq!(target, "hr-target");
        assert_eq!(attrs.role, "separator");
        assert_eq!(attrs.aria_orientation, Some("vertical"));
        assert_eq!(attrs.data_orientation, "vertical");
        assert_eq!(attrs.data_separator_root, "");
        assert_eq!(content, ());
    }
}

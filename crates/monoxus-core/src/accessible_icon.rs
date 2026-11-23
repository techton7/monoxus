use crate::foundation::compose::{Slottable, project_as_child};

/// Fallback wrapper inline style used when `AccessibleIcon` wraps `children` directly
/// without a zero-wrapper `render_child` callback.
pub const ACCESSIBLE_ICON_FALLBACK_WRAPPER_STYLE: &str = "display: contents;";

/// Accessibility attributes passed to `AccessibleIcon`'s `render_child` callback.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct AccessibleIconChildProps {
    pub aria_hidden: &'static str,
    pub focusable: &'static str,
}

impl Default for AccessibleIconChildProps {
    fn default() -> Self {
        Self::new()
    }
}

impl AccessibleIconChildProps {
    pub const fn new() -> Self {
        Self {
            aria_hidden: "true",
            focusable: "false",
        }
    }
}

/// Pure attribute bundle for `AccessibleIcon` projection.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct AccessibleIconAttributes {
    pub aria_hidden: &'static str,
    pub focusable: &'static str,
    pub label: String,
}

impl AccessibleIconAttributes {
    pub fn new(label: impl Into<String>) -> Self {
        Self {
            aria_hidden: "true",
            focusable: "false",
            label: label.into(),
        }
    }
}

/// Projects `AccessibleIcon` decorative attributes and label onto a slottable target.
pub fn project_accessible_icon_as_child<T, C>(
    label: impl Into<String>,
    target: T,
    content: Slottable<C>,
) -> (T, AccessibleIconAttributes, C) {
    let (target, content) = project_as_child(target, content);
    (target, AccessibleIconAttributes::new(label), content)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn child_props_default_to_hidden_non_focusable() {
        let child_props = AccessibleIconChildProps::default();
        assert_eq!(child_props.aria_hidden, "true");
        assert_eq!(child_props.focusable, "false");
    }

    #[test]
    fn as_child_projection_preserves_target_attributes_label_and_content() {
        let (target, attrs, content) =
            project_accessible_icon_as_child("Favorite", "svg-icon", Slottable::new("<path />"));
        assert_eq!(target, "svg-icon");
        assert_eq!(attrs.aria_hidden, "true");
        assert_eq!(attrs.focusable, "false");
        assert_eq!(attrs.label, "Favorite");
        assert_eq!(content, "<path />");
    }
}

/// Canonical screen-reader-only inline CSS clipping style matching Radix UI and Bits UI.
pub const VISUALLY_HIDDEN_STYLE: &str = "position: absolute; border: 0; width: 1px; height: 1px; padding: 0; margin: -1px; overflow: hidden; clip: rect(0, 0, 0, 0); white-space: nowrap; word-wrap: normal;";

/// Merges the canonical visually-hidden style with optional consumer inline CSS.
///
/// Base SR-only properties are emitted first so consumer declarations append or
/// override via standard CSS cascade order.
pub fn merge_visually_hidden_style(consumer_style: Option<&str>) -> String {
    match consumer_style {
        Some(style) if !style.trim().is_empty() => {
            format!("{VISUALLY_HIDDEN_STYLE} {}", style.trim())
        }
        _ => VISUALLY_HIDDEN_STYLE.to_string(),
    }
}

/// Pure state marker for the `VisuallyHidden` primitive.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub struct VisuallyHiddenState;

impl VisuallyHiddenState {
    pub const fn new() -> Self {
        Self
    }

    pub fn attributes(&self) -> VisuallyHiddenAttributes {
        VisuallyHiddenAttributes::new(None)
    }

    pub fn attributes_with_style(&self, consumer_style: Option<&str>) -> VisuallyHiddenAttributes {
        VisuallyHiddenAttributes::new(consumer_style)
    }
}

/// Attributes owned by the `VisuallyHidden` root element.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct VisuallyHiddenAttributes {
    pub style: String,
    pub data_visually_hidden_root: &'static str,
}

impl Default for VisuallyHiddenAttributes {
    fn default() -> Self {
        Self::new(None)
    }
}

impl VisuallyHiddenAttributes {
    pub fn new(consumer_style: Option<&str>) -> Self {
        Self {
            style: merge_visually_hidden_style(consumer_style),
            data_visually_hidden_root: "",
        }
    }
}

#[cfg(test)]
mod tests {
    use crate::foundation::compose::{Slottable, project_attrs_as_child};
    use super::*;

    #[test]
    fn canonical_style_contains_required_sr_only_properties() {
        for decl in [
            "position: absolute;",
            "border: 0;",
            "width: 1px;",
            "height: 1px;",
            "padding: 0;",
            "margin: -1px;",
            "overflow: hidden;",
            "clip: rect(0, 0, 0, 0);",
            "white-space: nowrap;",
            "word-wrap: normal;",
        ] {
            assert!(
                VISUALLY_HIDDEN_STYLE.contains(decl),
                "missing declaration `{decl}` in VISUALLY_HIDDEN_STYLE"
            );
        }
    }

    #[test]
    fn merge_style_preserves_base_and_appends_consumer_style() {
        assert_eq!(merge_visually_hidden_style(None), VISUALLY_HIDDEN_STYLE);
        assert_eq!(
            merge_visually_hidden_style(Some("   ")),
            VISUALLY_HIDDEN_STYLE
        );

        let merged = merge_visually_hidden_style(Some("outline-color: rgb(1, 2, 3);"));
        assert!(merged.starts_with(VISUALLY_HIDDEN_STYLE));
        assert!(merged.ends_with("outline-color: rgb(1, 2, 3);"));
    }

    #[test]
    fn state_and_as_child_projection_preserve_target_and_attributes() {
        let attrs = VisuallyHiddenState::new().attributes_with_style(Some("color: red;"));
        assert_eq!(attrs.data_visually_hidden_root, "");
        assert!(attrs.style.starts_with(VISUALLY_HIDDEN_STYLE));
        assert!(attrs.style.ends_with("color: red;"));

        let (target, projected, content) =
            project_attrs_as_child(&attrs, "article", Slottable::new("Skip link"));
        assert_eq!(target, "article");
        assert_eq!(projected, attrs);
        assert_eq!(content, "Skip link");
    }
}

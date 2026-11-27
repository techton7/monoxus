//! Thin compatibility shim over [`ui_sdk::collapsible::attrs`] preserving
//! legacy `--radix-collapsible-*` and `--monoxus-collapsible-*` CSS custom
//! properties during Strangler Fig migration waves.

pub use ui_sdk::collapsible::attrs::{
    CollapsibleContentAttributes, CollapsibleRootAttributes, CollapsibleTriggerAttributes,
    format_collapsible_dimension, format_disclosure_dimension, merge_disclosure_styles,
};

use crate::foundation::shared::ElementSize;

/// Formats the four standard collapsible content dimension CSS custom properties
/// (`--radix-*` and `--monoxus-*`).
pub fn format_collapsible_css_vars(size: ElementSize) -> String {
    let h = format_disclosure_dimension(size.height);
    let w = format_disclosure_dimension(size.width);
    format!(
        "--radix-collapsible-content-height: {h}; --radix-collapsible-content-width: {w}; --monoxus-collapsible-content-height: {h}; --monoxus-collapsible-content-width: {w}"
    )
}

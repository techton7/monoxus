// [KEEP-MONOXUS-BRAND]
//! Thin compatibility shim over [`ui_sdk::accordion::attrs`] preserving
//! legacy `--radix-accordion-*` and `--monoxus-accordion-*` CSS custom
//! properties.

pub use ui_sdk::accordion::attrs::{
    AccordionContentAttributes, AccordionHeaderAttributes, AccordionItemAttributes,
    AccordionRootAttributes, AccordionTriggerAttributes, format_accordion_css_vars_with_prefix,
    format_accordion_dimension, format_disclosure_dimension, merge_disclosure_styles,
};

use crate::foundation::shared::ElementSize;

// [KEEP-MONOXUS-BRAND] Emit both `--radix-accordion-*` and `--monoxus-accordion-*` CSS custom properties.
/// Formats the four standard accordion content dimension CSS custom properties
/// (`--radix-*` and `--monoxus-*`).
pub fn format_accordion_css_vars(size: ElementSize) -> String {
    format!(
        "{} {}",
        format_accordion_css_vars_with_prefix(size, "radix"),
        format_accordion_css_vars_with_prefix(size, "monoxus"),
    )
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn format_accordion_css_vars_emits_radix_and_monoxus_prefixes() {
        let size = ElementSize {
            width: 240.0,
            height: 90.5,
        };
        assert_eq!(
            format_accordion_css_vars(size),
            "--radix-accordion-content-height: 90.5px; --radix-accordion-content-width: 240px; --monoxus-accordion-content-height: 90.5px; --monoxus-accordion-content-width: 240px;"
        );
    }
}

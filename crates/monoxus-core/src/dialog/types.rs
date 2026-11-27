pub use ui_sdk::dialog::types::{
    DEFAULT_DIALOG_RESTORE_DELAY_MS, DIALOG_PARTS, DialogCloseFocusPolicy, DialogMode,
    DialogNestedState, DialogOpenFocusPolicy, DialogOutsideDismissBehavior,
    DialogOutsideInteractionPolicy, DialogPart, DialogScrollLockPolicy, DialogStateRequest,
};

// [KEEP-MONOXUS-BRAND] Preserve `[data-monoxus-autofocus]` ahead of `[data-ui-autofocus]`.
pub const DIALOG_FOCUSABLE_SELECTOR: &str = concat!(
    "[data-monoxus-autofocus],",
    "[data-ui-autofocus],",
    "[autofocus],",
    "button:not([disabled]),",
    "[href],",
    "input:not([disabled]):not([type=\"hidden\"]),",
    "select:not([disabled]),",
    "textarea:not([disabled]),",
    "[tabindex]:not([tabindex=\"-1\"])"
);

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn dialog_focusable_selector_prioritizes_monoxus_and_ui_autofocus() {
        assert!(DIALOG_FOCUSABLE_SELECTOR.starts_with("[data-monoxus-autofocus],[data-ui-autofocus],"));
    }
}

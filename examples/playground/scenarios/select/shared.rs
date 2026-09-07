use dioxus::prelude::*;
use monoxus::select::{SelectItemIndicator, SelectItemText};

pub const CARD_STYLE: &str = "display: grid; gap: 1rem; padding: 1.25rem; border-radius: 0.75rem; border: 1px solid #c084fc; background-color: white; box-shadow: 0 10px 30px rgba(147, 51, 234, 0.08);";
pub const MUTED_STYLE: &str = "margin: 0; color: #6b21a8;";
pub const BADGE_STYLE: &str = "display: inline-block; padding: 0.2rem 0.6rem; border-radius: 9999px; font-size: 0.75rem; font-weight: 600; background-color: #f3e8ff; color: #7e22ce;";

pub const SELECT_PLAYGROUND_CSS: &str = r#"
.select-item, [role="option"] {
    display: flex;
    justify-content: space-between;
    align-items: center;
    padding: 0.375rem 0.5rem;
    border-radius: 0.25rem;
    font-size: 0.875rem;
    cursor: pointer;
    user-select: none;
    transition: background-color 0.12s ease, color 0.12s ease;
}
.select-item[data-highlighted="true"], [role="option"][data-highlighted="true"] {
    background-color: #f3e8ff !important;
    color: #7e22ce !important;
}
.select-item[data-disabled="true"], [role="option"][data-disabled="true"] {
    opacity: 0.45 !important;
    cursor: not-allowed !important;
    pointer-events: none !important;
}
"#;

#[component]
pub fn ItemRow(text: &'static str, is_selected: bool) -> Element {
    rsx! {
        div {
            style: "display: flex; justify-content: space-between; align-items: center; width: 100%;",
            SelectItemText {
                span { "{text}" }
            }
            SelectItemIndicator {
                if is_selected {
                    span { style: "color: #9333ea; font-weight: bold; margin-left: 0.5rem;", "✓" }
                }
            }
        }
    }
}

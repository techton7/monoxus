use dioxus::prelude::*;
use monoxus::select::{
    SelectContent, SelectItem, SelectItemData, SelectMode, SelectRoot, SelectTrigger, SelectValue,
    SelectViewport,
};

use super::shared::{BADGE_STYLE, ItemRow, MUTED_STYLE};

#[component]
pub fn MultipleSelectSection() -> Element {
    let mut selected_vals = use_signal(|| vec!["apple".to_string(), "japan".to_string()]);
    let mut is_open = use_signal(|| false);

    let items = vec![
        SelectItemData::new("apple", "Fresh Red Apple", false),
        SelectItemData::new("banana", "Golden Cavendish Banana", false),
        SelectItemData::new("japan", "Japan (Tokyo)", false),
    ];

    let vals_now = selected_vals();
    let count = vals_now.len();
    let display_str = if count == 0 {
        "None".to_string()
    } else {
        vals_now.join(", ")
    };

    rsx! {
        div {
            id: "select-multiple-section",
            style: "border: 1px solid #e9d5ff; border-radius: 0.5rem; padding: 1.25rem; background-color: #faf5ff;",
            div {
                style: "display: flex; justify-content: space-between; align-items: center; margin-bottom: 0.75rem;",
                h3 {
                    style: "margin: 0; font-size: 1rem; color: #581c87;",
                    "6. Multiple Selection & Closed Custom Label Retention (SL-1)"
                }
                span { id: "select-multiple-badge", style: BADGE_STYLE, "Selected ({count}): {display_str}" }
            }
            p {
                style: MUTED_STYLE,
                "Multiple mode toggles selection on click without closing dropdown, preserves exact custom item labels across menu unmount, and emits repeated hidden inputs for form submission."
            }

            div {
                style: "position: relative; width: 320px; margin-top: 1rem;",
                SelectRoot {
                    id: "select-multiple".to_string(),
                    mode: SelectMode::Multiple,
                    items: Some(items),
                    values: Some(vals_now.clone()),
                    on_values_change: move |vals: Vec<String>| selected_vals.set(vals),
                    open: Some(is_open()),
                    on_open_change: move |open: bool| is_open.set(open),
                    name: "fruits".to_string(),
                    SelectTrigger {
                        id: "select-multiple-trigger".to_string(),
                        class: "select-trigger-multiple".to_string(),
                        style: "display: flex; justify-content: space-between; align-items: center; width: 100%; padding: 0.5rem 0.75rem; border: 1px solid #d8b4fe; border-radius: 0.375rem; background: white; font-size: 0.875rem; cursor: pointer; outline: none;",
                        span {
                            id: "select-multiple-value",
                            SelectValue {
                                placeholder: "Choose multiple items...".to_string(),
                            }
                        }
                        span { style: "color: #9333ea; font-size: 0.75rem;", "▼" }
                    }
                    SelectContent {
                        id: "select-multiple-content".to_string(),
                        class: "select-content-multiple".to_string(),
                        style: "z-index: 50; background: white; border: 1px solid #d8b4fe; border-radius: 0.375rem; box-shadow: 0 10px 15px -3px rgba(0,0,0,0.1); padding: 0.25rem; outline: none;",
                        SelectViewport {
                            SelectItem {
                                value: "apple".to_string(),
                                text: "Fresh Red Apple".to_string(),
                                class: "select-item".to_string(),
                                ItemRow { text: "Fresh Red Apple", is_selected: vals_now.iter().any(|v| v == "apple") }
                            }
                            SelectItem {
                                value: "banana".to_string(),
                                text: "Golden Cavendish Banana".to_string(),
                                class: "select-item".to_string(),
                                ItemRow { text: "Golden Cavendish Banana", is_selected: vals_now.iter().any(|v| v == "banana") }
                            }
                            SelectItem {
                                value: "japan".to_string(),
                                text: "Japan (Tokyo)".to_string(),
                                class: "select-item".to_string(),
                                ItemRow { text: "Japan (Tokyo)", is_selected: vals_now.iter().any(|v| v == "japan") }
                            }
                        }
                    }
                }
            }
        }
    }
}

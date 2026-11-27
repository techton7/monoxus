use dioxus::prelude::*;
use monoxus::select::{
    SelectContent, SelectItem, SelectRoot, SelectTrigger, SelectValue, SelectViewport,
};

use super::shared::{BADGE_STYLE, ItemRow, MUTED_STYLE};

#[component]
pub fn BottomConstrainedSelectSection() -> Element {
    let mut selected_val = use_signal(|| Some("upward-1".to_string()));
    let mut is_open = use_signal(|| false);

    let curr_val = selected_val().unwrap_or_else(|| "none".into());
    let open_state = if is_open() { "Open" } else { "Closed" };

    rsx! {
        div {
            id: "select-flip-container",
            style: "border: 1px solid #e9d5ff; border-radius: 0.5rem; padding: 1.25rem; background-color: #faf5ff; margin-top: 2rem;",
            div {
                style: "display: flex; justify-content: space-between; align-items: center; margin-bottom: 0.75rem;",
                h3 {
                    style: "margin: 0; font-size: 1rem; color: #581c87;",
                    "5. Bottom-Constrained Select (Viewport Collision Flip)"
                }
                div {
                    style: "display: flex; gap: 0.5rem;",
                    span { style: BADGE_STYLE, "Selected: {curr_val}" }
                    span { style: BADGE_STYLE, "State: {open_state}" }
                }
            }
            p {
                style: MUTED_STYLE,
                "Positioned near viewport bottom. When space below is constrained (< content height), FloatingLayer flips placement to data-side=\"top\"."
            }

            div {
                style: "position: relative; width: 260px; margin-top: 1rem; margin-bottom: 0.5rem;",
                SelectRoot {
                    id: "select-flip".to_string(),
                    default_value: "upward-1".to_string(),
                    on_value_change: move |val: Option<String>| selected_val.set(val),
                    open: Some(is_open()),
                    on_open_change: move |open: bool| is_open.set(open),
                    SelectTrigger {
                        class: "select-trigger-flip".to_string(),
                        style: "display: flex; justify-content: space-between; align-items: center; width: 100%; padding: 0.5rem 0.75rem; border: 1px solid #d8b4fe; border-radius: 0.375rem; background: white; font-size: 0.875rem; cursor: pointer; outline: none; box-shadow: 0 1px 2px rgba(0,0,0,0.05);",
                        SelectValue {
                            placeholder: "Choose option...".to_string(),
                        }
                        span { style: "color: #9333ea; font-size: 0.75rem;", "▲/▼" }
                    }
                    SelectContent {
                        class: "select-content-flip".to_string(),
                        style: "z-index: 50; background: white; border: 1px solid #d8b4fe; border-radius: 0.375rem; box-shadow: 0 10px 15px -3px rgba(0,0,0,0.1); padding: 0.25rem; outline: none;",
                        SelectViewport {
                            SelectItem {
                                value: "upward-1".to_string(),
                                text: "Upward Option 1".to_string(),
                                class: "select-item".to_string(),
                                ItemRow { text: "Upward Option 1", is_selected: selected_val().as_deref() == Some("upward-1") }
                            }
                            SelectItem {
                                value: "upward-2".to_string(),
                                text: "Upward Option 2".to_string(),
                                class: "select-item".to_string(),
                                ItemRow { text: "Upward Option 2", is_selected: selected_val().as_deref() == Some("upward-2") }
                            }
                            SelectItem {
                                value: "upward-3".to_string(),
                                text: "Upward Option 3".to_string(),
                                class: "select-item".to_string(),
                                ItemRow { text: "Upward Option 3", is_selected: selected_val().as_deref() == Some("upward-3") }
                            }
                        }
                    }
                }
            }
        }
    }
}

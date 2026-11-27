use dioxus::prelude::*;
use monoxus::select::{
    SelectContent, SelectItem, SelectItemData, SelectRoot, SelectTrigger, SelectValue,
    SelectViewport,
};

use super::shared::{BADGE_STYLE, ItemRow};

#[component]
pub fn BasicFruitSelectSection() -> Element {
    let mut selected_val = use_signal(|| Some("apple".to_string()));
    let mut is_open = use_signal(|| false);

    let items = vec![
        SelectItemData::new("apple", "Apple", false),
        SelectItemData::new("banana", "Banana", false),
        SelectItemData::new("blueberry", "Blueberry", false),
        SelectItemData::new("cherry", "Cherry", false),
        SelectItemData::new("grapes", "Grapes", false),
        SelectItemData::new("green-apple", "Green Apple", false),
        SelectItemData::new("green-grapes", "Green Grapes", false),
        SelectItemData::new("pineapple", "Pineapple", false),
    ];

    let curr_val = selected_val().unwrap_or_else(|| "none".into());
    let open_state = if is_open() { "Open" } else { "Closed" };

    rsx! {
        style { "{super::shared::SELECT_PLAYGROUND_CSS}" }
        div {
            style: "border: 1px solid #e9d5ff; border-radius: 0.5rem; padding: 1.25rem; background-color: #faf5ff;",
            div {
                style: "display: flex; justify-content: space-between; align-items: center; margin-bottom: 0.75rem;",
                h3 {
                    style: "margin: 0; font-size: 1rem; color: #581c87;",
                    "1. Basic Fruit Select"
                }
                div {
                    style: "display: flex; gap: 0.5rem;",
                    span { style: BADGE_STYLE, "Selected: {curr_val}" }
                    span { style: BADGE_STYLE, "State: {open_state}" }
                }
            }

            div {
                style: "position: relative; width: 240px;",
                SelectRoot {
                    id: "select-basic".to_string(),
                    items: Some(items),
                    default_value: "apple".to_string(),
                    on_value_change: move |val: Option<String>| selected_val.set(val),
                    open: Some(is_open()),
                    on_open_change: move |open: bool| is_open.set(open),
                    SelectTrigger {
                        id: "select-trigger".to_string(),
                        class: "select-trigger-basic".to_string(),
                        style: "display: flex; justify-content: space-between; align-items: center; width: 100%; padding: 0.5rem 0.75rem; border: 1px solid #d8b4fe; border-radius: 0.375rem; background: white; font-size: 0.875rem; cursor: pointer; outline: none; box-shadow: 0 1px 2px rgba(0,0,0,0.05);",
                        SelectValue {
                            placeholder: "Select a fruit...".to_string(),
                        }
                        span { style: "color: #9333ea; font-size: 0.75rem;", "▼" }
                    }
                    SelectContent {
                        id: "select-content".to_string(),
                        style: "z-index: 50; background: white; border: 1px solid #d8b4fe; border-radius: 0.375rem; box-shadow: 0 10px 15px -3px rgba(0,0,0,0.1); padding: 0.25rem; outline: none;",
                        SelectViewport {
                            SelectItem {
                                value: "".to_string(),
                                text: "-- Clear Selection --".to_string(),
                                class: "select-item select-item-clear".to_string(),
                                ItemRow { text: "-- Clear Selection --", is_selected: false }
                            }
                            SelectItem {
                                value: "apple".to_string(),
                                text: "Apple".to_string(),
                                class: "select-item".to_string(),
                                ItemRow { text: "Apple", is_selected: selected_val().as_deref() == Some("apple") }
                            }
                            SelectItem {
                                value: "banana".to_string(),
                                text: "Banana".to_string(),
                                class: "select-item".to_string(),
                                ItemRow { text: "Banana", is_selected: selected_val().as_deref() == Some("banana") }
                            }
                            SelectItem {
                                value: "blueberry".to_string(),
                                text: "Blueberry".to_string(),
                                class: "select-item".to_string(),
                                ItemRow { text: "Blueberry", is_selected: selected_val().as_deref() == Some("blueberry") }
                            }
                            SelectItem {
                                value: "cherry".to_string(),
                                text: "Cherry".to_string(),
                                class: "select-item".to_string(),
                                ItemRow { text: "Cherry", is_selected: selected_val().as_deref() == Some("cherry") }
                            }
                            SelectItem {
                                value: "grapes".to_string(),
                                text: "Grapes".to_string(),
                                class: "select-item".to_string(),
                                ItemRow { text: "Grapes", is_selected: selected_val().as_deref() == Some("grapes") }
                            }
                            SelectItem {
                                value: "green-apple".to_string(),
                                text: "Green Apple".to_string(),
                                class: "select-item".to_string(),
                                ItemRow { text: "Green Apple", is_selected: selected_val().as_deref() == Some("green-apple") }
                            }
                            SelectItem {
                                value: "green-grapes".to_string(),
                                text: "Green Grapes".to_string(),
                                class: "select-item".to_string(),
                                ItemRow { text: "Green Grapes", is_selected: selected_val().as_deref() == Some("green-grapes") }
                            }
                            SelectItem {
                                value: "pineapple".to_string(),
                                text: "Pineapple".to_string(),
                                class: "select-item".to_string(),
                                ItemRow { text: "Pineapple", is_selected: selected_val().as_deref() == Some("pineapple") }
                            }
                        }
                    }
                }
            }
        }
    }
}

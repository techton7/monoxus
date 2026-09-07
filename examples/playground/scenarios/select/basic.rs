use dioxus::prelude::*;
use monoxus::{
    foundation::shared::ScopeHandle,
    select::{
        Select, SelectContent, SelectItem, SelectRoot, SelectTrigger, SelectValue, SelectViewport,
        use_select_runtime,
    },
};

use super::shared::{BADGE_STYLE, ItemRow};

#[component]
pub fn BasicFruitSelectSection() -> Element {
    let scope = ScopeHandle::root("playground").child("select-basic");
    let selected_val = use_signal(|| Some("apple".to_string()));
    let is_open = use_signal(|| false);

    let items = vec![
        monoxus::select::SelectItemData::new("apple", "Apple", false),
        monoxus::select::SelectItemData::new("banana", "Banana", false),
        monoxus::select::SelectItemData::new("blueberry", "Blueberry", false),
        monoxus::select::SelectItemData::new("cherry", "Cherry", false),
        monoxus::select::SelectItemData::new("grapes", "Grapes", false),
    ];

    let def = Select::new(scope.clone())
        .with_items(items)
        .with_value(selected_val())
        .with_open(is_open());

    let runtime = use_select_runtime(
        def,
        Some(move |val: Option<String>| {
            let mut s = selected_val;
            s.set(val);
        }),
        Some(move |open: bool| {
            let mut s = is_open;
            s.set(open);
        }),
    );

    let curr_val = selected_val().unwrap_or_else(|| "none".into());
    let open_state = if is_open() { "Open" } else { "Closed" };

    rsx! {
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
                    runtime: runtime.clone(),
                    SelectTrigger {
                        class: "select-trigger-basic".to_string(),
                        style: "display: flex; justify-content: space-between; align-items: center; width: 100%; padding: 0.5rem 0.75rem; border: 1px solid #d8b4fe; border-radius: 0.375rem; background: white; font-size: 0.875rem; cursor: pointer; outline: none; box-shadow: 0 1px 2px rgba(0,0,0,0.05);",
                        SelectValue {
                            placeholder: "Select a fruit...".to_string(),
                        }
                        span { style: "color: #9333ea; font-size: 0.75rem;", "▼" }
                    }
                    SelectContent {
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
                        }
                    }
                }
            }
        }
    }
}

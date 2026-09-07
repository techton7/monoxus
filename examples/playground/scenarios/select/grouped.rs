use dioxus::prelude::*;
use monoxus::{
    foundation::shared::ScopeHandle,
    select::{
        Select, SelectContent, SelectGroup, SelectItem, SelectLabel, SelectRoot, SelectSeparator,
        SelectTrigger, SelectValue, SelectViewport, use_select_runtime,
    },
};

use super::shared::{BADGE_STYLE, ItemRow};

#[component]
pub fn GroupedSelectSection() -> Element {
    let scope = ScopeHandle::root("playground").child("select-grouped");
    let selected_val = use_signal(|| Some("carrot".to_string()));
    let is_open = use_signal(|| false);

    let def = Select::new(scope.clone())
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

    rsx! {
        div {
            style: "border: 1px solid #e9d5ff; border-radius: 0.5rem; padding: 1.25rem; background-color: #faf5ff;",
            div {
                style: "display: flex; justify-content: space-between; align-items: center; margin-bottom: 0.75rem;",
                h3 {
                    style: "margin: 0; font-size: 1rem; color: #581c87;",
                    "2. Grouped Select with Category Labels"
                }
                span { style: BADGE_STYLE, "Choice: {curr_val}" }
            }

            div {
                style: "position: relative; width: 260px;",
                SelectRoot {
                    runtime: runtime.clone(),
                    SelectTrigger {
                        class: "select-trigger-grouped".to_string(),
                        style: "display: flex; justify-content: space-between; align-items: center; width: 100%; padding: 0.5rem 0.75rem; border: 1px solid #d8b4fe; border-radius: 0.375rem; background: white; font-size: 0.875rem; cursor: pointer; outline: none;",
                        SelectValue {
                            placeholder: "Choose food...".to_string(),
                        }
                        span { style: "color: #9333ea; font-size: 0.75rem;", "▼" }
                    }
                    SelectContent {
                        style: "z-index: 50; background: white; border: 1px solid #d8b4fe; border-radius: 0.375rem; box-shadow: 0 10px 15px -3px rgba(0,0,0,0.1); padding: 0.25rem; outline: none;",
                        SelectViewport {
                            SelectGroup {
                                group_key: "fruits".to_string(),
                                SelectLabel {
                                    group_key: "fruits".to_string(),
                                    class: "select-label".to_string(),
                                    span { style: "display: block; padding: 0.25rem 0.5rem; font-size: 0.75rem; font-weight: 700; color: #a855f7; text-transform: uppercase;", "Fruits" }
                                }
                                SelectItem {
                                    value: "apple".to_string(),
                                    text: "Apple".to_string(),
                                    class: "select-item".to_string(),
                                    ItemRow { text: "Apple", is_selected: selected_val().as_deref() == Some("apple") }
                                }
                                SelectItem {
                                    value: "orange".to_string(),
                                    text: "Orange".to_string(),
                                    class: "select-item".to_string(),
                                    ItemRow { text: "Orange", is_selected: selected_val().as_deref() == Some("orange") }
                                }
                            }
                            SelectSeparator {
                                class: "select-sep".to_string(),
                            }
                            SelectGroup {
                                group_key: "vegetables".to_string(),
                                SelectLabel {
                                    group_key: "vegetables".to_string(),
                                    class: "select-label".to_string(),
                                    span { style: "display: block; padding: 0.25rem 0.5rem; font-size: 0.75rem; font-weight: 700; color: #a855f7; text-transform: uppercase;", "Vegetables" }
                                }
                                SelectItem {
                                    value: "carrot".to_string(),
                                    text: "Carrot".to_string(),
                                    class: "select-item".to_string(),
                                    ItemRow { text: "Carrot", is_selected: selected_val().as_deref() == Some("carrot") }
                                }
                                SelectItem {
                                    value: "broccoli".to_string(),
                                    text: "Broccoli".to_string(),
                                    class: "select-item".to_string(),
                                    ItemRow { text: "Broccoli", is_selected: selected_val().as_deref() == Some("broccoli") }
                                }
                            }
                        }
                    }
                }
            }
        }
    }
}

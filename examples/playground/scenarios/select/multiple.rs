use dioxus::prelude::*;
use monoxus::{
    foundation::shared::ScopeHandle,
    select::{
        Select, SelectContent, SelectItem, SelectMode, SelectRoot, SelectTrigger, SelectValue,
        SelectViewport, use_select_runtime,
    },
};

use super::shared::{BADGE_STYLE, MUTED_STYLE, ItemRow};

#[component]
pub fn MultipleSelectSection() -> Element {
    let scope = ScopeHandle::root("playground").child("select-multiple");
    let selected_vals = use_signal(|| vec!["apple".to_string(), "cherry".to_string()]);
    let is_open = use_signal(|| false);

    let def = Select::new(scope.clone())
        .with_mode(SelectMode::Multiple)
        .with_values(selected_vals())
        .with_open(is_open());

    let runtime = use_select_runtime(
        def,
        None::<fn(Option<String>)>,
        Some(move |open: bool| {
            let mut s = is_open;
            s.set(open);
        }),
    );

    let count = runtime.values().len();
    let display_str = if count == 0 { "None".to_string() } else { runtime.values().join(", ") };

    rsx! {
        div {
            style: "border: 1px solid #e9d5ff; border-radius: 0.5rem; padding: 1.25rem; background-color: #faf5ff;",
            div {
                style: "display: flex; justify-content: space-between; align-items: center; margin-bottom: 0.75rem;",
                h3 {
                    style: "margin: 0; font-size: 1rem; color: #581c87;",
                    "6. Multiple Selection (Repeated Hidden Inputs)"
                }
                span { style: BADGE_STYLE, "Selected ({count}): {display_str}" }
            }
            p {
                style: MUTED_STYLE,
                "Multiple mode toggles selection on click without closing dropdown, and emits repeated hidden inputs for form submission."
            }

            div {
                style: "position: relative; width: 260px; margin-top: 1rem;",
                SelectRoot {
                    runtime: runtime.clone(),
                    name: "fruits".to_string(),
                    SelectTrigger {
                        class: "select-trigger-multiple".to_string(),
                        style: "display: flex; justify-content: space-between; align-items: center; width: 100%; padding: 0.5rem 0.75rem; border: 1px solid #d8b4fe; border-radius: 0.375rem; background: white; font-size: 0.875rem; cursor: pointer; outline: none;",
                        SelectValue {
                            placeholder: "Choose multiple fruits...".to_string(),
                        }
                        span { style: "color: #9333ea; font-size: 0.75rem;", "▼" }
                    }
                    SelectContent {
                        class: "select-content-multiple".to_string(),
                        style: "z-index: 50; background: white; border: 1px solid #d8b4fe; border-radius: 0.375rem; box-shadow: 0 10px 15px -3px rgba(0,0,0,0.1); padding: 0.25rem; outline: none;",
                        SelectViewport {
                            SelectItem {
                                value: "apple".to_string(),
                                text: "Apple".to_string(),
                                class: "select-item".to_string(),
                                ItemRow { text: "Apple", is_selected: runtime.is_selected("apple") }
                            }
                            SelectItem {
                                value: "banana".to_string(),
                                text: "Banana".to_string(),
                                class: "select-item".to_string(),
                                ItemRow { text: "Banana", is_selected: runtime.is_selected("banana") }
                            }
                            SelectItem {
                                value: "cherry".to_string(),
                                text: "Cherry".to_string(),
                                class: "select-item".to_string(),
                                ItemRow { text: "Cherry", is_selected: runtime.is_selected("cherry") }
                            }
                        }
                    }
                }
            }
        }
    }
}

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
pub fn ScrollableViewportSection() -> Element {
    let scope = ScopeHandle::root("playground").child("select-scroll");
    let selected_val = use_signal(|| None::<String>);
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
                    "3. Scrollable Viewport with Disabled Items"
                }
                span { style: BADGE_STYLE, "Choice: {curr_val}" }
            }

            div {
                style: "position: relative; width: 280px;",
                SelectRoot {
                    runtime: runtime.clone(),
                    SelectTrigger {
                        class: "select-trigger-scrollable".to_string(),
                        style: "display: flex; justify-content: space-between; align-items: center; width: 100%; padding: 0.5rem 0.75rem; border: 1px solid #d8b4fe; border-radius: 0.375rem; background: white; font-size: 0.875rem; cursor: pointer; outline: none;",
                        SelectValue {
                            placeholder: "Select country...".to_string(),
                        }
                        span { style: "color: #9333ea; font-size: 0.75rem;", "▼" }
                    }
                    SelectContent {
                        style: "z-index: 50; background: white; border: 1px solid #d8b4fe; border-radius: 0.375rem; box-shadow: 0 10px 15px -3px rgba(0,0,0,0.1); padding: 0.25rem; outline: none; max-height: 160px; overflow-y: auto;",
                        SelectViewport {
                            SelectItem {
                                value: "argentina".to_string(),
                                text: "Argentina".to_string(),
                                class: "select-item".to_string(),
                                ItemRow { text: "Argentina", is_selected: selected_val().as_deref() == Some("argentina") }
                            }
                            SelectItem {
                                value: "brazil".to_string(),
                                text: "Brazil (Disabled)".to_string(),
                                disabled: true,
                                class: "select-item".to_string(),
                                ItemRow { text: "Brazil (Disabled)", is_selected: false }
                            }
                            SelectItem {
                                value: "canada".to_string(),
                                text: "Canada".to_string(),
                                class: "select-item".to_string(),
                                ItemRow { text: "Canada", is_selected: selected_val().as_deref() == Some("canada") }
                            }
                            SelectItem {
                                value: "denmark".to_string(),
                                text: "Denmark".to_string(),
                                class: "select-item".to_string(),
                                ItemRow { text: "Denmark", is_selected: selected_val().as_deref() == Some("denmark") }
                            }
                            SelectItem {
                                value: "egypt".to_string(),
                                text: "Egypt (Disabled)".to_string(),
                                disabled: true,
                                class: "select-item".to_string(),
                                ItemRow { text: "Egypt (Disabled)", is_selected: false }
                            }
                            SelectItem {
                                value: "france".to_string(),
                                text: "France".to_string(),
                                class: "select-item".to_string(),
                                ItemRow { text: "France", is_selected: selected_val().as_deref() == Some("france") }
                            }
                            SelectItem {
                                value: "germany".to_string(),
                                text: "Germany".to_string(),
                                class: "select-item".to_string(),
                                ItemRow { text: "Germany", is_selected: selected_val().as_deref() == Some("germany") }
                            }
                            SelectItem {
                                value: "japan".to_string(),
                                text: "Japan".to_string(),
                                class: "select-item".to_string(),
                                ItemRow { text: "Japan", is_selected: selected_val().as_deref() == Some("japan") }
                            }
                            SelectItem {
                                value: "korea".to_string(),
                                text: "Korea".to_string(),
                                class: "select-item".to_string(),
                                ItemRow { text: "Korea", is_selected: selected_val().as_deref() == Some("korea") }
                            }
                        }
                    }
                }
            }
        }
    }
}

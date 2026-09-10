use dioxus::prelude::*;
use monoxus::{
    foundation::shared::ScopeHandle,
    select::{
        Select, SelectContent, SelectItem, SelectRoot, SelectTrigger, SelectValue, SelectViewport,
        use_select_runtime,
    },
};

use super::shared::{BADGE_STYLE, ItemRow, MUTED_STYLE};

#[component]
pub fn BottomConstrainedSelectSection() -> Element {
    let scope = ScopeHandle::root("playground").child("select-flip");
    let selected_val = use_signal(|| Some("upward-1".to_string()));
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
    let open_state = if is_open() { "Open" } else { "Closed" };
    let current_side = runtime.side().as_str();

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
                    span {
                        id: "select-flip-side-badge",
                        style: BADGE_STYLE,
                        "side: {current_side}"
                    }
                }
            }
            p {
                style: MUTED_STYLE,
                "Positioned near viewport bottom. When space below is constrained (< content height), FloatingLayer flips placement to data-side=\"top\"."
            }

            div {
                style: "position: relative; width: 260px; margin-top: 1rem; margin-bottom: 0.5rem;",
                SelectRoot {
                    runtime: runtime.clone(),
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

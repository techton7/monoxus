use dioxus::prelude::*;
use monoxus::{
    foundation::shared::ScopeHandle,
    select::{
        Select, SelectContentStatic, SelectItem, SelectRoot, SelectTrigger, SelectValue,
        SelectViewport, use_select_runtime,
    },
};

use super::shared::{BADGE_STYLE, ItemRow, MUTED_STYLE};

#[component]
pub fn StaticContentSection() -> Element {
    let scope = ScopeHandle::root("playground").child("select-static");
    let selected_val = use_signal(|| Some("fast".to_string()));
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

    rsx! {
        div {
            style: "border: 1px solid #e9d5ff; border-radius: 0.5rem; padding: 1.25rem; background-color: #faf5ff;",
            div {
                style: "display: flex; justify-content: space-between; align-items: center; margin-bottom: 0.75rem;",
                h3 {
                    style: "margin: 0; font-size: 1rem; color: #581c87;",
                    "7. Static Non-Floating Content Lane"
                }
                span { style: BADGE_STYLE, "Val: {selected_val().unwrap_or_default()}" }
            }
            p {
                style: MUTED_STYLE,
                "SelectContentStatic renders inline in document flow without FloatingLayer positioning styles or monitors."
            }

            div {
                style: "width: 260px; margin-top: 1rem;",
                SelectRoot {
                    runtime: runtime.clone(),
                    SelectTrigger {
                        class: "select-trigger-static".to_string(),
                        style: "display: flex; justify-content: space-between; align-items: center; width: 100%; padding: 0.5rem 0.75rem; border: 1px solid #d8b4fe; border-radius: 0.375rem; background: white; font-size: 0.875rem; cursor: pointer; outline: none;",
                        SelectValue {
                            placeholder: "Speed...".to_string(),
                        }
                        span { style: "color: #9333ea; font-size: 0.75rem;", "▼" }
                    }
                    SelectContentStatic {
                        class: "select-content-static".to_string(),
                        style: "margin-top: 0.5rem; background: white; border: 1px solid #d8b4fe; border-radius: 0.375rem; padding: 0.25rem; outline: none;".to_string(),
                        SelectViewport {
                            SelectItem {
                                value: "fast".to_string(),
                                text: "Fast".to_string(),
                                class: "select-item".to_string(),
                                ItemRow { text: "Fast", is_selected: selected_val().as_deref() == Some("fast") }
                            }
                            SelectItem {
                                value: "faster".to_string(),
                                text: "Faster".to_string(),
                                class: "select-item".to_string(),
                                ItemRow { text: "Faster", is_selected: selected_val().as_deref() == Some("faster") }
                            }
                        }
                    }
                }
            }
        }
    }
}

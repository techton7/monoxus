use dioxus::prelude::*;
use monoxus::select::{
    SelectContent, SelectItem, SelectRoot, SelectTrigger, SelectValue, SelectViewport,
};

use super::shared::{BADGE_STYLE, ItemRow};

#[component]
pub fn FormIntegrationSection() -> Element {
    let submitted_value = use_signal(|| None::<String>);
    let sub_disp = submitted_value().unwrap_or_else(|| "None yet".into());

    rsx! {
        div {
            style: "border: 1px solid #e9d5ff; border-radius: 0.5rem; padding: 1.25rem; background-color: #faf5ff;",
            div {
                style: "display: flex; justify-content: space-between; align-items: center; margin-bottom: 0.75rem;",
                h3 {
                    style: "margin: 0; font-size: 1rem; color: #581c87;",
                    "4. Native Form Integration (Uncontrolled Native FormData)"
                }
                span { id: "form-submitted-badge", style: BADGE_STYLE, "Submitted: {sub_disp}" }
            }
            p {
                style: "font-size: 0.8125rem; color: #6b7280; margin: 0 0 1rem 0;",
                "Pure uncontrolled Select with default_value=\"pro\". Form submission reads directly from native FormData (evt.values()) with zero local state mirroring."
            }

            form {
                onsubmit: move |evt| {
                    evt.prevent_default();
                    let values = evt.values();
                    let submitted_tier = values.iter().find(|(k, _)| k == "tier").and_then(|(_, v)| match v {
                        dioxus::html::FormValue::Text(s) => Some(s.clone()),
                        _ => None,
                    });
                    let mut s = submitted_value;
                    s.set(submitted_tier);
                },
                style: "display: flex; gap: 1rem; align-items: center;",
                div {
                    style: "position: relative; width: 220px;",
                    SelectRoot {
                        id: "form-tier-select".to_string(),
                        name: "tier".to_string(),
                        default_value: "pro".to_string(),
                        SelectTrigger {
                            class: "select-trigger-form".to_string(),
                            style: "display: flex; justify-content: space-between; align-items: center; width: 100%; padding: 0.5rem 0.75rem; border: 1px solid #d8b4fe; border-radius: 0.375rem; background: white; font-size: 0.875rem; cursor: pointer; outline: none;",
                            SelectValue {
                                placeholder: "Choose tier...".to_string(),
                            }
                            span { style: "color: #9333ea; font-size: 0.75rem;", "▼" }
                        }
                        SelectContent {
                            style: "z-index: 50; background: white; border: 1px solid #d8b4fe; border-radius: 0.375rem; box-shadow: 0 10px 15px -3px rgba(0,0,0,0.1); padding: 0.25rem; outline: none;",
                            SelectViewport {
                                SelectItem {
                                    value: "starter".to_string(),
                                    text: "Starter ($0)".to_string(),
                                    class: "select-item".to_string(),
                                    ItemRow { text: "Starter ($0)", is_selected: false }
                                }
                                SelectItem {
                                    value: "pro".to_string(),
                                    text: "Pro ($29)".to_string(),
                                    class: "select-item".to_string(),
                                    ItemRow { text: "Pro ($29)", is_selected: false }
                                }
                                SelectItem {
                                    value: "enterprise".to_string(),
                                    text: "Enterprise ($99)".to_string(),
                                    class: "select-item".to_string(),
                                    ItemRow { text: "Enterprise ($99)", is_selected: false }
                                }
                            }
                        }
                    }
                }
                button {
                    r#type: "submit",
                    id: "select-form-submit-btn",
                    style: "padding: 0.5rem 1rem; border-radius: 0.375rem; background-color: #9333ea; color: white; border: none; font-size: 0.875rem; font-weight: 500; cursor: pointer;",
                    "Submit Form"
                }
            }
        }
    }
}

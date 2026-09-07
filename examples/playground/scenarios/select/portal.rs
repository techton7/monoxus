use dioxus::prelude::*;
use monoxus::{
    foundation::{overlay::PortalHost, shared::ScopeHandle},
    select::{
        Select, SelectContent, SelectItem, SelectPortal, SelectRoot, SelectTrigger, SelectValue,
        SelectViewport, use_select_runtime,
    },
};

use super::shared::{BADGE_STYLE, MUTED_STYLE, ItemRow};

#[component]
pub fn PortaledSelectSection() -> Element {
    let scope = ScopeHandle::root("playground").child("select-portal");
    let selected_val = use_signal(|| Some("remote-1".to_string()));
    let is_open = use_signal(|| false);
    let mut prevent_escape = use_signal(|| false);

    let items = vec![
        monoxus::select::SelectItemData::new("remote-1", "Remote Item 1", false),
        monoxus::select::SelectItemData::new("remote-2", "Remote Item 2", false),
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
                    "8. Portaled Select (DOM Relocation & Preventable Escape)"
                }
                div {
                    style: "display: flex; gap: 0.5rem; align-items: center;",
                    span { style: BADGE_STYLE, "Selected: {curr_val}" }
                    span { style: BADGE_STYLE, "State: {open_state}" }
                }
            }
            p {
                style: MUTED_STYLE,
                "SelectPortal physically teleports the overlay into the designated #select-portal-root host element in the live DOM. Also tests preventable Escape."
            }

            // Dedicated Portal Host Container
            div {
                id: "select-portal-root",
                style: "margin-top: 1rem; margin-bottom: 1rem; border: 2px dashed #9333ea; border-radius: 0.5rem; padding: 0.75rem 1rem; background-color: #ffffff; min-height: 50px;",
                span {
                    style: "display: block; font-size: 0.75rem; font-weight: 700; color: #7e22ce; text-transform: uppercase; margin-bottom: 0.25rem;",
                    "Portal Host Destination (#select-portal-root)"
                }
                span {
                    id: "select-portal-status-label",
                    style: "font-size: 0.8125rem; color: #64748b;",
                    "When open, SelectContent is relocated here."
                }
            }

            div {
                style: "display: flex; gap: 1.5rem; align-items: center; margin-top: 1rem;",
                div {
                    style: "position: relative; width: 260px;",
                    SelectRoot {
                        runtime: runtime.clone(),
                        SelectTrigger {
                            class: "select-trigger-portaled".to_string(),
                            style: "display: flex; justify-content: space-between; align-items: center; width: 100%; padding: 0.5rem 0.75rem; border: 1px solid #d8b4fe; border-radius: 0.375rem; background: white; font-size: 0.875rem; cursor: pointer; outline: none; box-shadow: 0 1px 2px rgba(0,0,0,0.05);",
                            SelectValue {
                                placeholder: "Choose remote item...".to_string(),
                            }
                            span { style: "color: #9333ea; font-size: 0.75rem;", "▼" }
                        }
                        SelectPortal {
                            host: PortalHost::named("select-portal-root".to_string()),
                            SelectContent {
                                class: "select-content-portaled".to_string(),
                                style: "z-index: 50; background: white; border: 1px solid #d8b4fe; border-radius: 0.375rem; box-shadow: 0 10px 15px -3px rgba(0,0,0,0.1); padding: 0.25rem; outline: none;",
                                on_escape_keydown: move |evt: KeyboardEvent| {
                                    if prevent_escape() {
                                        evt.prevent_default();
                                    }
                                },
                                SelectViewport {
                                    SelectItem {
                                        value: "remote-1".to_string(),
                                        text: "Remote Item 1".to_string(),
                                        class: "select-item".to_string(),
                                        ItemRow { text: "Remote Item 1", is_selected: selected_val().as_deref() == Some("remote-1") }
                                    }
                                    SelectItem {
                                        value: "remote-2".to_string(),
                                        text: "Remote Item 2".to_string(),
                                        class: "select-item".to_string(),
                                        ItemRow { text: "Remote Item 2", is_selected: selected_val().as_deref() == Some("remote-2") }
                                    }
                                }
                            }
                        }
                    }
                }

                // Interactive prevent-escape toggle
                label {
                    style: "display: flex; align-items: center; gap: 0.5rem; font-size: 0.8125rem; color: #581c87; cursor: pointer;",
                    input {
                        r#type: "checkbox",
                        id: "prevent-escape-checkbox",
                        checked: prevent_escape(),
                        onchange: move |evt| prevent_escape.set(evt.value().parse().unwrap_or(false)),
                    }
                    span { "Prevent Dismiss on Escape Key" }
                }
            }
        }
    }
}

use dioxus::prelude::*;
use monoxus::{
    foundation::{overlay::PortalHost, shared::ScopeHandle},
    select::{
        PointerDownOutsideEvent, Select, SelectContent, SelectItem, SelectPortal, SelectRoot,
        SelectTrigger, SelectValue, SelectViewport, use_select_runtime,
    },
};

use super::shared::{BADGE_STYLE, ItemRow, MUTED_STYLE};

#[component]
pub fn PortaledSelectSection() -> Element {
    let scope = ScopeHandle::root("playground").child("select-portal");
    let selected_val = use_signal(|| Some("remote-1".to_string()));
    let is_open = use_signal(|| false);
    let mut prevent_escape = use_signal(|| false);
    let mut prevent_outside_click = use_signal(|| false);
    let mut force_mount_enabled = use_signal(|| false);
    let mut prevent_selection_enabled = use_signal(|| false);
    let mut use_custom_anchor = use_signal(|| false);
    let outside_clicks = use_signal(|| 0);

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
                    "8. Portaled Select (DOM Relocation, Outside Click & Preventable Escape)"
                }
                div {
                    style: "display: flex; gap: 0.5rem; align-items: center;",
                    span { id: "outside-clicks-badge", style: BADGE_STYLE, "Outside Clicks: {outside_clicks()}" }
                    span { style: BADGE_STYLE, "Selected: {curr_val}" }
                    span { style: BADGE_STYLE, "State: {open_state}" }
                }
            }
            p {
                style: MUTED_STYLE,
                "SelectPortal physically teleports the overlay into the designated #select-portal-root host element in the live DOM. Also tests preventable Escape and preventable Outside PointerDown."
            }

            // Dedicated Portal Host Container
            div {
                id: "select-portal-root",
                style: "position: relative; min-height: 140px; margin-top: 1rem; margin-bottom: 1rem; border: 2px dashed #9333ea; border-radius: 0.5rem; padding: 0.75rem 1rem; background-color: #faf5ff;",
                span {
                    style: "display: block; font-size: 0.75rem; font-weight: 700; color: #7e22ce; text-transform: uppercase; margin-bottom: 0.25rem;",
                    "Portal Host Destination (#select-portal-root)"
                }
                span {
                    id: "select-portal-status-label",
                    style: "font-size: 0.8125rem; color: #64748b; margin-bottom: 0.5rem; display: block;",
                    if is_open() {
                        "Status: Mounted inside #select-portal-root (DOM Teleport Active)"
                    } else {
                        "Status: Idle (Content unmounted from host)"
                    }
                }
            }

            div {
                style: "display: flex; gap: 1.5rem; align-items: flex-start; margin-top: 1rem; flex-wrap: wrap;",
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
                            force_mount: force_mount_enabled(),
                            SelectContent {
                                class: "select-content-portaled".to_string(),
                                style: "position: relative !important; top: 0.5rem !important; left: 0 !important; width: 260px !important; z-index: 50; background: white; border: 1px solid #d8b4fe; border-radius: 0.375rem; box-shadow: 0 10px 15px -3px rgba(0,0,0,0.1); padding: 0.25rem; outline: none;",
                                sticky: "always".to_string(),
                                prevent_scroll: true,
                                force_mount: force_mount_enabled(),
                                prevent_overflow_text_selection: prevent_selection_enabled(),
                                custom_anchor: if use_custom_anchor() { Some("custom-portal-anchor".to_string()) } else { None },
                                on_pointer_down_outside: move |evt: PointerDownOutsideEvent| {
                                    let mut c = outside_clicks;
                                    c.set(c() + 1);
                                    if prevent_outside_click() {
                                        evt.prevent_default();
                                    }
                                },
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

                    // Distinct custom anchor target for live testing
                    div {
                        id: "custom-portal-anchor",
                        style: "margin-top: 0.75rem; padding: 0.5rem 0.75rem; border: 2px dashed #a855f7; border-radius: 0.375rem; background-color: #f3e8ff; font-size: 0.75rem; color: #6b21a8; font-weight: 600;",
                        "Custom Anchor Target (#custom-portal-anchor)"
                    }
                }

                // Interactive control toggles
                div {
                    style: "display: flex; flex-direction: column; gap: 0.5rem;",
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
                    label {
                        style: "display: flex; align-items: center; gap: 0.5rem; font-size: 0.8125rem; color: #581c87; cursor: pointer;",
                        input {
                            r#type: "checkbox",
                            id: "prevent-outside-click-checkbox",
                            checked: prevent_outside_click(),
                            onchange: move |evt| prevent_outside_click.set(evt.value().parse().unwrap_or(false)),
                        }
                        span { "Prevent Dismiss on Outside Click" }
                    }
                    label {
                        style: "display: flex; align-items: center; gap: 0.5rem; font-size: 0.8125rem; color: #581c87; cursor: pointer;",
                        input {
                            r#type: "checkbox",
                            id: "force-mount-checkbox",
                            checked: force_mount_enabled(),
                            onchange: move |evt| force_mount_enabled.set(evt.value().parse().unwrap_or(false)),
                        }
                        span { "Force Mount Content in DOM when Closed" }
                    }
                    label {
                        style: "display: flex; align-items: center; gap: 0.5rem; font-size: 0.8125rem; color: #581c87; cursor: pointer;",
                        input {
                            r#type: "checkbox",
                            id: "prevent-selection-checkbox",
                            checked: prevent_selection_enabled(),
                            onchange: move |evt| prevent_selection_enabled.set(evt.value().parse().unwrap_or(false)),
                        }
                        span { "Prevent Text Selection when Open" }
                    }
                    label {
                        style: "display: flex; align-items: center; gap: 0.5rem; font-size: 0.8125rem; color: #581c87; cursor: pointer;",
                        input {
                            r#type: "checkbox",
                            id: "custom-anchor-checkbox",
                            checked: use_custom_anchor(),
                            onchange: move |evt| use_custom_anchor.set(evt.value().parse().unwrap_or(false)),
                        }
                        span { "Anchor to Custom Element (#custom-portal-anchor)" }
                    }
                }
            }
        }
    }
}

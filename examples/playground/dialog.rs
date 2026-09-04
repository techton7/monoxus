use dioxus::prelude::*;
use monoxus::{
    dialog::Dialog,
    foundation::{overlay::PortalHost, shared::ScopeHandle},
};

const CARD_STYLE: &str = "display: grid; gap: 1rem; padding: 1.25rem; border-radius: 0.75rem; border: 1px solid #cbd5e1; background-color: white; box-shadow: 0 10px 30px rgba(15, 23, 42, 0.08);";
const MUTED_STYLE: &str = "margin: 0; color: #475569;";

#[component]
pub fn DialogPlayground() -> Element {
    let open = use_signal(|| false);
    let dialog = Dialog::new(ScopeHandle::root("playground").child("dialog"), open())
        .with_portal_host(PortalHost::named("playground-layer"));

    let root = dialog.root();
    let trigger = dialog.trigger();
    let portal = dialog.portal();
    let overlay = dialog.overlay();
    let content = dialog.content();
    let title = dialog.title();
    let description = dialog.description();
    let close = dialog.close();

    let portal_host = portal.host().id().unwrap_or("document.body");
    let trigger_request = trigger.open_request().next_open();
    let close_request = close.close_request().next_open();
    let mut open_from_trigger = open;
    let mut open_from_overlay = open;
    let mut open_from_close = open;

    rsx! {
        section {
            style: CARD_STYLE,
            h2 {
                style: "margin: 0;",
                "Dialog"
            }
            p {
                style: MUTED_STYLE,
                "The renderer markup lives in this example module, while IDs, roles, requests, and data-state values come from "
                code { "monoxus::dialog" }
                "."
            }
            div {
                id: root.id(),
                "data-state": root.data_state().as_str(),
                style: "display: grid; gap: 1rem;",
                button {
                    id: trigger.id(),
                    r#type: "button",
                    aria_controls: trigger.aria_controls(),
                    aria_expanded: trigger.aria_expanded(),
                    "data-state": trigger.data_state().as_str(),
                    onclick: move |_| open_from_trigger.set(trigger_request),
                    style: "justify-self: start; padding: 0.65rem 0.9rem; border: 0; border-radius: 0.5rem; background-color: #2563eb; color: white; font-weight: 600; cursor: pointer;",
                    "Open dialog"
                }
                ul {
                    style: "margin: 0; padding-left: 1.25rem; color: #334155;",
                    li {
                        "root id: "
                        code { "{root.id()}" }
                    }
                    li {
                        "content role: "
                        code { "{content.role()}" }
                    }
                    li {
                        "portal host: "
                        code { "{portal_host}" }
                    }
                    li {
                        "data-state: "
                        code { "{dialog.data_state()}" }
                    }
                }
                if dialog.is_open() {
                    div {
                        style: "display: grid; gap: 0.75rem;",
                        div {
                            id: overlay.id(),
                            "data-state": overlay.data_state().as_str(),
                            onclick: move |_| open_from_overlay.set(close_request),
                            style: "padding: 0.85rem 1rem; border-radius: 0.75rem; background-color: rgba(15, 23, 42, 0.65); color: white; cursor: pointer;",
                            "Overlay — click anywhere in this band to dismiss."
                        }
                        div {
                            id: content.id(),
                            role: content.role(),
                            aria_modal: content.aria_modal(),
                            aria_labelledby: content.aria_labelledby(),
                            aria_describedby: content.aria_describedby(),
                            "data-state": content.data_state().as_str(),
                            style: "display: grid; gap: 0.75rem; padding: 1rem; border-radius: 0.75rem; border: 1px solid #bfdbfe; background-color: #eff6ff;",
                            h3 {
                                id: title.id(),
                                style: "margin: 0;",
                                "Headless dialog content"
                            }
                            p {
                                id: description.id(),
                                style: MUTED_STYLE,
                                "This is a tiny renderer harness over the public dialog data surface."
                            }
                            button {
                                id: close.id(),
                                r#type: "button",
                                "data-state": close.data_state().as_str(),
                                onclick: move |_| open_from_close.set(close_request),
                                style: "justify-self: start; padding: 0.55rem 0.8rem; border-radius: 0.5rem; border: 1px solid #94a3b8; background-color: white; cursor: pointer;",
                                "Close"
                            }
                        }
                    }
                } else {
                    p {
                        style: MUTED_STYLE,
                        "Closed. Use the trigger button to mount the content surface."
                    }
                }
            }
        }
    }
}

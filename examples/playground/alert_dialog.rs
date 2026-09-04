use dioxus::prelude::*;
use monoxus::{
    alert_dialog::AlertDialog,
    foundation::{overlay::PortalHost, shared::ScopeHandle},
};

const CARD_STYLE: &str = "display: grid; gap: 1rem; padding: 1.25rem; border-radius: 0.75rem; border: 1px solid #fecaca; background-color: white; box-shadow: 0 10px 30px rgba(127, 29, 29, 0.08);";
const MUTED_STYLE: &str = "margin: 0; color: #7f1d1d;";
const MODAL_ROOT_STYLE: &str = "position: fixed; inset: 0; z-index: 50; display: flex; align-items: center; justify-content: center; padding: 1.5rem;";
const MODAL_OVERLAY_STYLE: &str = "position: absolute; inset: 0; background-color: rgba(127, 29, 29, 0.72); backdrop-filter: blur(3px); cursor: pointer;";
const MODAL_FRAME_STYLE: &str = "position: relative; z-index: 1; width: min(100%, 34rem); max-height: calc(100vh - 3rem); overflow: auto;";
const MODAL_PANEL_STYLE: &str = "display: grid; gap: 1rem; padding: 1.35rem; border-radius: 1rem; border: 1px solid #fecaca; background-color: white; box-shadow: 0 32px 80px rgba(127, 29, 29, 0.32);";
const MODAL_WARNING_STYLE: &str = "display: grid; gap: 0.35rem; padding: 0.9rem 1rem; border-radius: 0.75rem; background-color: #fef2f2; color: #991b1b;";

#[component]
pub fn AlertDialogPlayground() -> Element {
    let open = use_signal(|| false);
    let outcome = use_signal(|| String::from("Waiting for a choice."));
    let alert = AlertDialog::new(
        ScopeHandle::root("playground").child("alert-dialog"),
        open(),
    )
    .with_portal_host(PortalHost::inline());

    let root = alert.root();
    let trigger = alert.trigger();
    let portal = alert.portal();
    let overlay = alert.overlay();
    let content = alert.content();
    let title = alert.title();
    let description = alert.description();
    let close = alert.close();
    let action = alert.action();
    let cancel = alert.cancel();

    let portal_host = if portal.host().is_inline() {
        "inline"
    } else {
        portal.host().id().unwrap_or("document.body")
    };
    let trigger_request = trigger.open_request().next_open();
    let close_request = close.close_request().next_open();
    let action_request = action.close_request().next_open();
    let cancel_request = cancel.close_request().next_open();
    let mut open_from_trigger = open;
    let mut open_from_overlay = open;
    let mut open_from_close = open;
    let mut open_from_action = open;
    let mut open_from_cancel = open;
    let mut outcome_from_overlay = outcome;
    let mut outcome_from_close = outcome;
    let mut outcome_from_action = outcome;
    let mut outcome_from_cancel = outcome;

    rsx! {
        section {
            style: CARD_STYLE,
            h2 {
                style: "margin: 0;",
                "Alert dialog"
            }
            p {
                style: MUTED_STYLE,
                "This variant reuses the same dialog lane and adds explicit "
                code { "action" }
                " / "
                code { "cancel" }
                " semantics from "
                code { "monoxus::alert_dialog" }
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
                    style: "justify-self: start; padding: 0.65rem 0.9rem; border: 0; border-radius: 0.5rem; background-color: #dc2626; color: white; font-weight: 600; cursor: pointer;",
                    "Open alert dialog"
                }
                p {
                    style: MUTED_STYLE,
                    "Last outcome: "
                    strong { "{outcome()}" }
                }
                ul {
                    style: "margin: 0; padding-left: 1.25rem; color: #7f1d1d;",
                    li {
                        "content role: "
                        code { "{content.role()}" }
                    }
                    li {
                        "portal host: "
                        code { "{portal_host}" }
                    }
                    li {
                        "action id: "
                        code { "{action.id()}" }
                    }
                    li {
                        "cancel id: "
                        code { "{cancel.id()}" }
                    }
                }
                if alert.is_open() {
                    div {
                        style: MODAL_ROOT_STYLE,
                        div {
                            id: overlay.id(),
                            "data-state": overlay.data_state().as_str(),
                            onclick: move |_| {
                                outcome_from_overlay.set(String::from("Dismissed from backdrop"));
                                open_from_overlay.set(close_request);
                            },
                            style: MODAL_OVERLAY_STYLE,
                            aria_label: "Dismiss alert dialog",
                        }
                        div {
                            style: MODAL_FRAME_STYLE,
                            div {
                                id: content.id(),
                                role: content.role(),
                                aria_modal: content.aria_modal(),
                                aria_labelledby: content.aria_labelledby(),
                                aria_describedby: content.aria_describedby(),
                                "data-state": content.data_state().as_str(),
                                style: MODAL_PANEL_STYLE,
                                div {
                                    style: "display: grid; gap: 0.5rem;",
                                    p {
                                        style: "margin: 0; color: #b91c1c; font-size: 0.85rem; font-weight: 700; text-transform: uppercase; letter-spacing: 0.04em;",
                                        "Confirm / cancel modal"
                                    }
                                    h3 {
                                        id: title.id(),
                                        style: "margin: 0;",
                                        "Delete the demo file?"
                                    }
                                    p {
                                        id: description.id(),
                                        style: MUTED_STYLE,
                                        "The example keeps renderer details local while consuming the public "
                                        code { "monoxus::alert_dialog" }
                                        " data APIs."
                                    }
                                }
                                div {
                                    style: MODAL_WARNING_STYLE,
                                    strong { "This is the primary modal experience." }
                                    p {
                                        style: "margin: 0;",
                                        "Focus the decision on the centered panel, with confirm and cancel actions grouped together above the dimmed page."
                                    }
                                }
                                div {
                                    style: "display: grid; gap: 0.45rem;",
                                    p {
                                        style: "margin: 0; font-weight: 600; color: #7f1d1d;",
                                        "Proof surface"
                                    }
                                    ul {
                                        style: "margin: 0; padding-left: 1.25rem; color: #7f1d1d;",
                                        li {
                                            "content role: "
                                            code { "{content.role()}" }
                                        }
                                        li {
                                            "portal host: "
                                            code { "{portal_host}" }
                                        }
                                        li {
                                            "action id: "
                                            code { "{action.id()}" }
                                        }
                                        li {
                                            "cancel id: "
                                            code { "{cancel.id()}" }
                                        }
                                    }
                                }
                                div {
                                    style: "display: flex; gap: 0.75rem; flex-wrap: wrap; justify-content: flex-end;",
                                    button {
                                        id: cancel.id(),
                                        r#type: "button",
                                        "data-state": cancel.data_state().as_str(),
                                        onclick: move |_| {
                                            outcome_from_cancel.set(String::from("Canceled"));
                                            open_from_cancel.set(cancel_request);
                                        },
                                        style: "padding: 0.65rem 0.95rem; border-radius: 0.65rem; border: 1px solid #fca5a5; background-color: white; cursor: pointer; font-weight: 600;",
                                        "Cancel"
                                    }
                                    button {
                                        id: action.id(),
                                        r#type: "button",
                                        "data-state": action.data_state().as_str(),
                                        onclick: move |_| {
                                            outcome_from_action.set(String::from("Confirmed"));
                                            open_from_action.set(action_request);
                                        },
                                        style: "padding: 0.65rem 0.95rem; border-radius: 0.65rem; border: 0; background-color: #dc2626; color: white; cursor: pointer; font-weight: 700;",
                                        "Confirm action"
                                    }
                                }
                                button {
                                    id: close.id(),
                                    r#type: "button",
                                    "data-state": close.data_state().as_str(),
                                    onclick: move |_| {
                                        outcome_from_close.set(String::from("Closed without choosing"));
                                        open_from_close.set(close_request);
                                    },
                                    style: "justify-self: end; padding: 0.5rem 0.75rem; border-radius: 999px; border: 1px solid #fca5a5; background-color: white; cursor: pointer; color: #991b1b;",
                                    "Dismiss"
                                }
                            }
                        }
                    }
                } else {
                    p {
                        style: MUTED_STYLE,
                        "Closed. Open it to inspect the alert role plus action/cancel IDs in a centered confirm/cancel modal."
                    }
                }
            }
        }
    }
}

use dioxus::prelude::*;
use monoxus::{
    dialog::{
        Dialog, DialogCloseFocusPolicy, DialogMode, DialogOpenFocusPolicy,
        DialogOutsideDismissBehavior, use_dialog_runtime,
    },
    foundation::{overlay::PortalHost, shared::ScopeHandle},
};

const CARD_STYLE: &str = "display: grid; gap: 1rem; padding: 1.25rem; border-radius: 0.75rem; border: 1px solid #cbd5e1; background-color: white; box-shadow: 0 10px 30px rgba(15, 23, 42, 0.08);";
const MUTED_STYLE: &str = "margin: 0; color: #475569;";
const MODAL_ROOT_STYLE: &str = "position: fixed; inset: 0; z-index: 40; display: flex; align-items: center; justify-content: center; padding: 1.5rem;";
const MODAL_OVERLAY_STYLE: &str = "position: absolute; inset: 0; background-color: rgba(15, 23, 42, 0.72); backdrop-filter: blur(3px); cursor: pointer;";
const MODAL_FRAME_STYLE: &str = "position: relative; z-index: 1; width: min(100%, 44rem); max-height: calc(100vh - 3rem); overflow: auto;";
const MODAL_PANEL_STYLE: &str = "display: grid; gap: 1rem; padding: 1.35rem; border-radius: 1rem; border: 1px solid #bfdbfe; background-color: white; box-shadow: 0 32px 80px rgba(15, 23, 42, 0.35);";
const MODAL_NOTE_STYLE: &str = "display: grid; gap: 0.35rem; padding: 0.85rem 1rem; border-radius: 0.75rem; background-color: #eff6ff; color: #1d4ed8;";

#[component]
pub fn DialogPlayground() -> Element {
    let open = use_signal(|| false);
    let dialog = use_dialog_runtime(
        Dialog::new(ScopeHandle::root("playground").child("dialog"), open())
            .with_portal_host(PortalHost::named("playground-layer")),
    );

    let root = dialog.root();
    let trigger = dialog.trigger();
    let portal = dialog.portal();
    let overlay = dialog.overlay();
    let content = dialog.content();
    let title = dialog.title();
    let description = dialog.description();
    let close = dialog.close();
    let lifecycle = dialog.lifecycle();

    let portal_host = portal.host().id().unwrap_or("document.body");
    let trigger_request = trigger.open_request().next_open();
    let close_request = close.close_request().next_open();
    let mode = dialog_mode_label(lifecycle.mode());
    let open_focus = open_focus_policy_label(lifecycle.open_focus_policy());
    let close_focus = close_focus_policy_label(lifecycle.close_focus_policy());
    let scroll_lock = if lifecycle.scroll_lock_policy().is_enabled() {
        match lifecycle.scroll_lock_policy().restore_delay() {
            Some(delay) => format!("enabled (restore delay: {delay}ms)"),
            None => String::from("enabled (restore delay: none)"),
        }
    } else {
        String::from("disabled")
    };
    let pointer_outside = outside_behavior_label(
        lifecycle
            .outside_interaction_policy()
            .pointer_down_outside(),
    );
    let focus_outside =
        outside_behavior_label(lifecycle.outside_interaction_policy().focus_outside());
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
                    onmounted: dialog.mount_trigger(),
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
                    li {
                        "mode: "
                        code { "{mode}" }
                    }
                    li {
                        "open focus: "
                        code { "{open_focus}" }
                    }
                    li {
                        "close focus restore: "
                        code { "{close_focus}" }
                    }
                    li {
                        "scroll lock: "
                        code { "{scroll_lock}" }
                    }
                    li {
                        "outside pointer: "
                        code { "{pointer_outside}" }
                        " / focus outside: "
                        code { "{focus_outside}" }
                    }
                }
                if dialog.is_open() {
                    div {
                        style: MODAL_ROOT_STYLE,
                        div {
                            id: overlay.id(),
                            "data-state": overlay.data_state().as_str(),
                            onclick: move |_| open_from_overlay.set(close_request),
                            style: MODAL_OVERLAY_STYLE,
                            aria_label: "Dismiss dialog",
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
                                onmounted: dialog.mount_content(),
                                style: MODAL_PANEL_STYLE,
                                div {
                                    style: "display: grid; gap: 0.5rem;",
                                    p {
                                        style: "margin: 0; color: #1d4ed8; font-size: 0.85rem; font-weight: 700; text-transform: uppercase; letter-spacing: 0.04em;",
                                        "Centered modal sample"
                                    }
                                    h3 {
                                        id: title.id(),
                                        style: "margin: 0;",
                                        "Headless dialog content"
                                    }
                                    p {
                                        id: description.id(),
                                        style: MUTED_STYLE,
                                        "This example keeps renderer markup local while turning the public dialog data surface into an actual floating modal."
                                    }
                                }
                                div {
                                    style: MODAL_NOTE_STYLE,
                                    strong { "Try it like a dialog." }
                                    p {
                                        style: "margin: 0;",
                                        "Click the backdrop to dismiss, or use the close button below. The page stays visible behind a full-screen overlay."
                                    }
                                }
                                div {
                                    style: "display: grid; gap: 0.45rem;",
                                    p {
                                        style: "margin: 0; font-weight: 600; color: #0f172a;",
                                        "Proof that the modal still comes from "
                                        code { "monoxus::dialog" }
                                    }
                                    ul {
                                        style: "margin: 0; padding-left: 1.25rem; color: #334155;",
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
                                        li {
                                            "close focus restore: "
                                            code { "{close_focus}" }
                                        }
                                        li {
                                            "outside pointer: "
                                            code { "{pointer_outside}" }
                                            " / focus outside: "
                                            code { "{focus_outside}" }
                                        }
                                    }
                                }
                                button {
                                    id: close.id(),
                                    r#type: "button",
                                    "data-state": close.data_state().as_str(),
                                    onmounted: dialog.mount_close(),
                                    onclick: move |_| open_from_close.set(close_request),
                                    style: "justify-self: end; padding: 0.65rem 0.95rem; border-radius: 0.65rem; border: 1px solid #94a3b8; background-color: white; cursor: pointer; font-weight: 600;",
                                    "Close dialog"
                                }
                            }
                        }
                    }
                } else {
                    p {
                        style: MUTED_STYLE,
                        "Closed. Use the trigger button to mount the floating modal surface."
                    }
                }

            }
        }
    }
}

fn dialog_mode_label(mode: DialogMode) -> &'static str {
    match mode {
        DialogMode::Modal => "modal",
        DialogMode::NonModal => "non-modal",
    }
}

fn open_focus_policy_label(policy: &DialogOpenFocusPolicy) -> String {
    match policy {
        DialogOpenFocusPolicy::FirstFocusable => String::from("first focusable"),
        DialogOpenFocusPolicy::Target(target) => format!("target:{target}"),
        DialogOpenFocusPolicy::Suppress => String::from("suppressed"),
    }
}

fn close_focus_policy_label(policy: &DialogCloseFocusPolicy) -> String {
    match policy {
        DialogCloseFocusPolicy::Trigger => String::from("trigger"),
        DialogCloseFocusPolicy::Target(target) => format!("target:{target}"),
        DialogCloseFocusPolicy::None => String::from("none"),
    }
}

fn outside_behavior_label(behavior: DialogOutsideDismissBehavior) -> &'static str {
    match behavior {
        DialogOutsideDismissBehavior::Dismiss => "dismisses",
        DialogOutsideDismissBehavior::Ignore => "ignored",
    }
}

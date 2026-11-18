use dioxus::prelude::*;
use monoxus::toggle::Toggle;

#[component]
pub fn TogglePlayground() -> Element {
    let mut bold = use_signal(|| false);
    let mut italic = use_signal(|| true);
    let mut controlled_pressed = use_signal(|| false);
    let mut lock_controlled = use_signal(|| false);

    rsx! {
        div {
            class: "max-w-4xl mx-auto space-y-8",

            div {
                class: "space-y-1.5",
                h2 {
                    class: "text-2xl font-bold tracking-tight text-slate-900",
                    "Toggle"
                }
                p {
                    class: "text-sm text-slate-500",
                    "A two-state button that can be either on or off, publishing aria-pressed=\"true\"|\"false\", data-state=\"on\"|\"off\", and omitting data-disabled when enabled."
                }
            }

            // SW1A-5: Uncontrolled, Default-On, and Disabled Toggles
            div {
                class: "rounded-2xl border border-slate-200 bg-white p-6 shadow-sm space-y-5",
                div {
                    class: "space-y-1",
                    h3 {
                        class: "text-base font-semibold text-slate-900",
                        "1. Uncontrolled, Default-On & Disabled Toggles (SW1A-5)"
                    }
                    p {
                        class: "text-xs text-slate-500",
                        "Click, Enter, and Space toggle state cleanly with zero double-fire. Styles are driven by [data-toggle-root][data-state=\"on\"|\"off\"]."
                    }
                }

                div {
                    class: "flex flex-wrap items-center gap-3",
                    Toggle {
                        id: "toggle-bold".to_string(),
                        pressed: bold(),
                        on_pressed_change: move |next| bold.set(next),
                        "Bold"
                    }

                    Toggle {
                        id: "toggle-italic".to_string(),
                        pressed: italic(),
                        on_pressed_change: move |next| italic.set(next),
                        "Italic"
                    }

                    Toggle {
                        id: "toggle-disabled-off".to_string(),
                        disabled: true,
                        default_pressed: false,
                        "Disabled (Off)"
                    }

                    Toggle {
                        id: "toggle-disabled-on".to_string(),
                        disabled: true,
                        default_pressed: true,
                        "Disabled (On)"
                    }
                }

                // Rich text preview card responding to bold and italic
                div {
                    class: "space-y-2 pt-2",
                    div { class: "text-xs font-semibold text-slate-600", "Live Typography Preview" }
                    div {
                        id: "toggle-rich-text-preview",
                        style: format!(
                            "padding: 1rem 1.25rem; border-radius: 0.5rem; background-color: #f8fafc; border: 1px solid #e2e8f0; font-size: 1rem; color: #0f172a; font-weight: {}; font-style: {}; transition: all 0.15s ease;",
                            if bold() { "700" } else { "400" },
                            if italic() { "italic" } else { "normal" }
                        ),
                        "The quick brown fox jumps over the lazy dog."
                    }
                }

                div {
                    class: "flex flex-wrap items-center gap-4 text-xs font-mono text-slate-600 pt-1",
                    span {
                        class: "inline-flex items-center gap-1.5 rounded bg-slate-100 px-2.5 py-1",
                        "Bold state: "
                        strong {
                            id: "toggle-bold-state",
                            if bold() { "true" } else { "false" }
                        }
                        span {
                            class: if bold() { "sw1a-badge-on" } else { "sw1a-badge-off" },
                            if bold() { "ON" } else { "OFF" }
                        }
                    }
                    span {
                        class: "inline-flex items-center gap-1.5 rounded bg-slate-100 px-2.5 py-1",
                        "Italic state: "
                        strong {
                            id: "toggle-italic-state",
                            if italic() { "true" } else { "false" }
                        }
                        span {
                            class: if italic() { "sw1a-badge-on" } else { "sw1a-badge-off" },
                            if italic() { "ON" } else { "OFF" }
                        }
                    }
                }
            }

            // SW1A-5: Controlled Toggle with Lock Invariance
            div {
                class: "rounded-2xl border border-slate-200 bg-white p-6 shadow-sm space-y-4",
                div {
                    class: "space-y-1",
                    h3 {
                        class: "text-base font-semibold text-slate-900",
                        "2. Controlled Toggle & Lock Invariance (SW1A-5)"
                    }
                    p {
                        class: "text-xs text-slate-500",
                        "When Lock is active, clicking the controlled Toggle fires on_pressed_change without mutating state until parent updates pressed."
                    }
                }

                div {
                    class: "flex flex-wrap items-center gap-3",
                    Toggle {
                        id: "toggle-controlled".to_string(),
                        pressed: controlled_pressed(),
                        on_pressed_change: move |next| {
                            if !lock_controlled() {
                                controlled_pressed.set(next);
                            }
                        },
                        "Pin Sidebar"
                    }

                    button {
                        id: "toggle-lock-switch",
                        r#type: "button",
                        class: "sw1a-btn-secondary text-xs",
                        onclick: move |_| {
                            let cur = *lock_controlled.peek();
                            lock_controlled.set(!cur);
                        },
                        if lock_controlled() { "Unlock Controlled State" } else { "Lock Controlled State" }
                    }

                    button {
                        id: "toggle-external-flip",
                        r#type: "button",
                        class: "sw1a-btn-secondary text-xs",
                        onclick: move |_| {
                            let cur = *controlled_pressed.peek();
                            controlled_pressed.set(!cur);
                        },
                        "External Flip"
                    }
                }

                div {
                    class: "flex items-center gap-4 text-xs font-mono text-slate-600",
                    span {
                        class: "inline-flex items-center gap-1 rounded bg-slate-100 px-2.5 py-1",
                        "Controlled state: "
                        strong {
                            id: "toggle-controlled-state",
                            if controlled_pressed() { "on" } else { "off" }
                        }
                    }
                    span {
                        class: "inline-flex items-center gap-1 rounded bg-slate-100 px-2.5 py-1",
                        "Lock: "
                        strong {
                            id: "toggle-lock-badge",
                            if lock_controlled() { "locked" } else { "unlocked" }
                        }
                    }
                }
            }
        }
    }
}

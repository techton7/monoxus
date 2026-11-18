use dioxus::prelude::keyboard_types::{Key, NamedKey};
use dioxus::prelude::*;
use monoxus::button::{Button, ButtonType};

#[component]
pub fn ButtonPlayground() -> Element {
    let mut action_count = use_signal(|| 0_u32);
    let mut anchor_count = use_signal(|| 0_u32);
    let mut submit_count = use_signal(|| 0_u32);
    let mut disabled_count = use_signal(|| 0_u32);
    let mut disabled_anchor_count = use_signal(|| 0_u32);
    let mut div_count = use_signal(|| 0_u32);
    let mut div_disabled = use_signal(|| false);
    let mut form_event = use_signal(|| String::from("none"));

    rsx! {
        div {
            class: "max-w-4xl mx-auto space-y-8",

            div {
                class: "space-y-1.5",
                h2 {
                    class: "text-2xl font-bold tracking-tight text-slate-900",
                    "Button"
                }
                p {
                    class: "text-sm text-slate-500",
                    "Polymorphic headless button primitive rendering native <button>, <a> with accessible link disablement, or custom <div role=\"button\">, with W3C :focus-visible modality parity."
                }
            }

            // SW1A-1: Enabled <button> and <a href>
            div {
                class: "rounded-2xl border border-slate-200 bg-white p-6 shadow-sm space-y-4",
                div {
                    class: "space-y-1",
                    h3 {
                        class: "text-base font-semibold text-slate-900",
                        "1. Enabled <button> & <a href> Polymorphism (SW1A-1)"
                    }
                    p {
                        class: "text-xs text-slate-500",
                        "Default type=\"button\" activates on pointer click, Enter (KeyDown), and Space (KeyUp). Anchor renders <a data-button-root href=\"...\"> without type and activates on click and Enter."
                    }
                }

                div {
                    class: "flex flex-wrap items-center gap-4",
                    div {
                        class: "flex items-center gap-2",
                        Button {
                            id: "btn-action".to_string(),
                            class: "sw1a-btn-primary".to_string(),
                            on_click: move |_| action_count += 1,
                            "Primary Action"
                        }
                        span {
                            class: "sw1a-pill-purple",
                            "<button type=\"button\">"
                        }
                    }

                    div {
                        class: "flex items-center gap-2",
                        Button {
                            id: "btn-anchor".to_string(),
                            href: "#button-anchor-target".to_string(),
                            class: "sw1a-btn-secondary".to_string(),
                            on_click: move |evt: MouseEvent| {
                                evt.prevent_default();
                                anchor_count += 1;
                            },
                            "Documentation Anchor"
                        }
                        span {
                            class: "sw1a-pill-slate",
                            "<a href=\"...\">"
                        }
                    }
                }

                div {
                    id: "button-anchor-target",
                    class: "flex flex-wrap items-center gap-4 text-xs text-slate-600 pt-1",
                    span {
                        class: "inline-flex items-center gap-1 rounded bg-slate-100 px-2.5 py-1 font-mono",
                        "Button clicks: "
                        strong { id: "btn-action-count", "{action_count()}" }
                    }
                    span {
                        class: "inline-flex items-center gap-1 rounded bg-slate-100 px-2.5 py-1 font-mono",
                        "Anchor activations: "
                        strong { id: "btn-anchor-count", "{anchor_count()}" }
                    }
                }

                form {
                    id: "btn-demo-form",
                    class: "flex flex-wrap items-center gap-3 pt-3 border-t border-slate-100",
                    onsubmit: move |evt| {
                        evt.prevent_default();
                        submit_count += 1;
                        form_event.set(String::from("submitted"));
                    },
                    onreset: move |_| {
                        form_event.set(String::from("reset"));
                    },
                    div {
                        class: "flex items-center gap-2",
                        Button {
                            id: "btn-submit".to_string(),
                            r#type: ButtonType::Submit,
                            class: "sw1a-btn-dark".to_string(),
                            "Submit Form"
                        }
                        span {
                            class: "sw1a-pill-slate",
                            "<button type=\"submit\">"
                        }
                    }
                    Button {
                        id: "btn-reset".to_string(),
                        r#type: ButtonType::Reset,
                        class: "sw1a-btn-secondary".to_string(),
                        "Reset Form"
                    }
                    span {
                        class: "inline-flex items-center gap-1 rounded bg-slate-100 px-2.5 py-1 text-xs font-mono text-slate-600",
                        "Form submits: "
                        strong { id: "btn-submit-count", "{submit_count()}" }
                    }
                    span {
                        class: "text-xs font-mono text-slate-500",
                        "Last form event: "
                        strong { id: "btn-form-event", "{form_event()}" }
                    }
                }
            }

            // SW1A-2: Disabled <button> and Disabled <a>
            div {
                class: "rounded-2xl border border-slate-200 bg-white p-6 shadow-sm space-y-4",
                div {
                    class: "space-y-1",
                    h3 {
                        class: "text-base font-semibold text-slate-900",
                        "2. Disabled <button> & Accessible Disabled <a> (SW1A-2)"
                    }
                    p {
                        class: "text-xs text-slate-500",
                        "Disabled <button> sets native disabled and data-disabled=\"\". Disabled <a> omits href, sets role=\"link\", aria-disabled=\"true\", tabindex=\"-1\", data-disabled=\"\", and suppresses click/Enter."
                    }
                }

                div {
                    class: "flex flex-wrap items-center gap-4",
                    Button {
                        id: "btn-focus-before-disabled".to_string(),
                        class: "sw1a-btn-secondary".to_string(),
                        "Tab Start"
                    }
                    div {
                        class: "flex items-center gap-2",
                        Button {
                            id: "btn-disabled-native".to_string(),
                            disabled: true,
                            class: "sw1a-btn-primary".to_string(),
                            on_click: move |_| disabled_count += 1,
                            "Disabled Button"
                        }
                        span {
                            class: "sw1a-pill-amber",
                            "<button disabled>"
                        }
                    }
                    div {
                        class: "flex items-center gap-2",
                        Button {
                            id: "btn-disabled-anchor".to_string(),
                            href: "#button-anchor-target".to_string(),
                            disabled: true,
                            class: "sw1a-btn-secondary".to_string(),
                            on_click: move |_| disabled_anchor_count += 1,
                            "Disabled Anchor Link"
                        }
                        span {
                            class: "sw1a-pill-amber",
                            "<a role=\"link\" aria-disabled=\"true\" tabindex=\"-1\">"
                        }
                    }
                    Button {
                        id: "btn-focus-after-disabled".to_string(),
                        class: "sw1a-btn-secondary".to_string(),
                        "Tab End"
                    }
                }

                div {
                    class: "flex flex-wrap items-center gap-4 text-xs text-slate-600",
                    span {
                        class: "inline-flex items-center gap-1 rounded bg-slate-100 px-2.5 py-1 font-mono",
                        "Disabled button fires: "
                        strong { id: "btn-disabled-count", "{disabled_count()}" }
                    }
                    span {
                        class: "inline-flex items-center gap-1 rounded bg-slate-100 px-2.5 py-1 font-mono",
                        "Disabled anchor fires: "
                        strong { id: "btn-anchor-disabled-count", "{disabled_anchor_count()}" }
                    }
                }
            }

            // SW1A-1 & SW1A-2: Tag Polymorphism via <div role="button">
            div {
                class: "rounded-2xl border border-slate-200 bg-white p-6 shadow-sm space-y-4",
                div {
                    class: "space-y-1",
                    h3 {
                        class: "text-base font-semibold text-slate-900",
                        "3. Custom Tag Polymorphism (<div role=\"button\">)"
                    }
                    p {
                        class: "text-xs text-slate-500",
                        "Custom elements projecting headless button semantics: WAI-ARIA role=\"button\", keyboard activation (Enter/Space), and accessible tabindex handling."
                    }
                }

                div {
                    class: "flex flex-wrap items-center gap-4",
                    button {
                        id: "toggle-div-disabled",
                        r#type: "button",
                        class: "sw1a-btn-secondary text-xs",
                        onclick: move |_| {
                            let cur = *div_disabled.peek();
                            div_disabled.set(!cur);
                        },
                        if div_disabled() { "Toggle div disabled (Currently: Disabled)" } else { "Toggle div disabled (Currently: Enabled)" }
                    }

                    div {
                        class: "flex items-center gap-2",
                        div {
                            id: "btn-div",
                            role: "button",
                            tabindex: if div_disabled() { "-1" } else { "0" },
                            "aria-disabled": if div_disabled() { "true" } else { "false" },
                            "data-button-root": "",
                            "data-disabled": if div_disabled() { Some("") } else { None },
                            class: "sw1a-btn-secondary",
                            onclick: move |_| {
                                if !div_disabled() {
                                    div_count += 1;
                                }
                            },
                            onkeydown: move |evt: KeyboardEvent| {
                                if !div_disabled() {
                                    let key = evt.key();
                                    if key == Key::Named(NamedKey::Enter) || key == Key::Character(" ".to_string()) {
                                        evt.prevent_default();
                                        div_count += 1;
                                    }
                                }
                            },
                            "Div as Button"
                        }
                        span {
                            class: "sw1a-pill-purple",
                            if div_disabled() { "<div role=\"button\" aria-disabled=\"true\" tabindex=\"-1\">" } else { "<div role=\"button\" tabindex=\"0\">" }
                        }
                    }
                }

                div {
                    class: "flex flex-wrap items-center gap-4 text-xs text-slate-600",
                    span {
                        class: "inline-flex items-center gap-1 rounded bg-slate-100 px-2.5 py-1 font-mono",
                        "Div button activations: "
                        strong { id: "btn-div-count", "{div_count()}" }
                    }
                }
            }
        }
    }
}

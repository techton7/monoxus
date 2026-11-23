use dioxus::prelude::*;
use monoxus::pin_input::*;

const PIN_INPUT_PLAYGROUND_STYLE: &str = r#"
[data-monoxus-pin-input-cell] {
    width: 44px;
    height: 52px;
    display: flex;
    align-items: center;
    justify-content: center;
    border-radius: 0.5rem;
    font-size: 1.25rem;
    font-weight: 700;
    font-family: monospace;
    color: #0f172a;
    box-sizing: border-box;
    border: 1px solid #cbd5e1;
    background-color: #f8fafc;
    transition: border-color 0.15s ease, box-shadow 0.15s ease;
}

[data-monoxus-pin-input-cell][data-filled] {
    border: 1px solid #94a3b8;
    background-color: #ffffff;
}

[data-monoxus-pin-input-cell][data-active] {
    border: 2px solid #7c3aed;
    background-color: #ffffff;
    box-shadow: 0 0 0 3px rgba(124, 58, 237, 0.18);
}

[data-monoxus-pin-input-cell][data-disabled] {
    border: 1px solid #e2e8f0;
    background-color: #f1f5f9;
    color: #94a3b8;
    opacity: 0.65;
    box-shadow: none;
}
"#;

fn render_pin_cells(
    prefix: &str,
    snap: &PinInputSnapshot,
    separator_after: Option<usize>,
) -> Element {
    rsx! {
        for cell_ref in snap.cells.iter() {
            {
                let cell = *cell_ref;
                let cell_id = format!("{prefix}-cell-{}", cell.index);
                let caret_id = format!("{prefix}-caret-{}", cell.index);
                let display_char = cell.display_char(snap.mask, '•');
                let show_separator = separator_after == Some(cell.index);

                rsx! {
                    PinInputCell {
                        key: "{cell.index}",
                        id: cell_id,
                        index: cell.index,
                        "data-filled": cell.char.is_some().then_some(""),
                        if let Some(ch) = display_char {
                            span { "{ch}" }
                        } else if cell.has_fake_caret {
                            div {
                                id: caret_id,
                                "data-monoxus-pin-fake-caret": "",
                                style: "width: 2px; height: 22px; background-color: #7c3aed; border-radius: 1px;",
                            }
                        }
                    }
                    if show_separator {
                        span {
                            key: "sep-{cell.index}",
                            aria_hidden: "true",
                            style: "color: #94a3b8; font-weight: 700; font-size: 1rem; padding: 0 2px; pointer-events: none; user-select: none;",
                            "-"
                        }
                    }
                }
            }
        }
    }
}

#[component]
pub fn PinInputPlayground() -> Element {
    rsx! {
        div {
            class: "max-w-4xl mx-auto space-y-8",
            style: "max-width: 56rem; margin: 0 auto; display: flex; flex-direction: column; gap: 2rem;",
            style { "{PIN_INPUT_PLAYGROUND_STYLE}" }

            // Header Section
            div {
                class: "space-y-1.5",
                style: "display: flex; flex-direction: column; gap: 0.375rem;",
                h2 {
                    class: "text-2xl font-bold tracking-tight text-slate-900",
                    style: "margin: 0; font-size: 1.5rem; font-weight: 700; color: #0f172a;",
                    "Pin Input"
                }
                p {
                    class: "text-sm text-slate-500",
                    style: "margin: 0; font-size: 0.875rem; color: #64748b;",
                    "Headless single-hidden-<input> segmented OTP primitive with DOM selection mirroring, fake carets, pattern filtering, paste transformation, password masking, and native <form> reset."
                }
            }

            DigitsOtpScenario {}
            PasteTransformerScenario {}
            MaskedOtpScenario {}
            FormOtpScenario {}
        }
    }
}

// ---------------------------------------------------------------------------
// Scenario 1: 6-Cell Digits OTP (SW4A-4, SW4A-5, SW4A-6, SW4A-7)
// ---------------------------------------------------------------------------
#[component]
fn DigitsOtpScenario() -> Element {
    let mut digits_val = use_signal(String::new);
    let mut complete_count = use_signal(|| 0usize);
    let mut completed_val = use_signal(String::new);

    rsx! {
        div {
            class: "rounded-2xl border border-slate-200 bg-white p-6 shadow-sm space-y-4",
            style: "border-radius: 1rem; border: 1px solid #e2e8f0; background-color: #ffffff; padding: 1.5rem; display: flex; flex-direction: column; gap: 1rem;",

            div {
                class: "space-y-1",
                style: "display: flex; flex-direction: column; gap: 0.25rem;",
                h3 {
                    class: "text-base font-semibold text-slate-900",
                    style: "margin: 0; font-size: 1rem; font-weight: 600; color: #0f172a;",
                    "1. 6-Cell Digits OTP, Caret Navigation & on_complete (SW4A-4..SW4A-7)"
                }
                p {
                    class: "text-xs text-slate-500",
                    style: "margin: 0; font-size: 0.75rem; color: #64748b;",
                    "Accepts digits only (PinInputPattern::Digits, max_length = 6). Supports ArrowLeft/ArrowRight mid-string replacement, Backspace deletion, fake caret projection, and edge-triggered on_complete."
                }
            }

            div {
                style: "padding: 0.5rem 0;",
                PinInputRoot {
                    id: "sw4a-pin-digits",
                    input_id: "sw4a-pin-digits-input",
                    max_length: 6,
                    pattern: PinInputPattern::Digits,
                    input_mode: PinInputMode::Numeric,
                    style: "display: inline-flex; align-items: center; gap: 8px;",
                    on_value_change: move |v: String| digits_val.set(v),
                    on_complete: move |full: String| {
                        complete_count += 1;
                        completed_val.set(full);
                    },
                    render_children: move |snap: PinInputSnapshot| {
                        render_pin_cells("sw4a-pin-digits", &snap, None)
                    }
                }
            }

            div {
                class: "pt-2 flex flex-wrap items-center gap-4 text-xs font-mono",
                style: "padding-top: 0.5rem; display: flex; flex-wrap: wrap; align-items: center; gap: 1rem; font-size: 0.75rem; font-family: monospace;",
                div {
                    style: "display: flex; align-items: center; gap: 0.375rem;",
                    span { style: "color: #94a3b8; font-weight: 600;", "Value:" }
                    span {
                        id: "sw4a-pin-digits-val",
                        style: "display: inline-flex; align-items: center; min-width: 4.5rem; min-height: 1.5rem; border-radius: 0.25rem; padding: 0.125rem 0.5rem; background-color: #f1f5f9; color: #334155; font-weight: 700;",
                        "{digits_val()}"
                    }
                }
                div {
                    style: "display: flex; align-items: center; gap: 0.375rem;",
                    span { style: "color: #94a3b8; font-weight: 600;", "on_complete Count:" }
                    span {
                        id: "sw4a-pin-digits-complete",
                        style: "display: inline-flex; align-items: center; border-radius: 0.25rem; padding: 0.125rem 0.5rem; background-color: #f5f3ff; color: #6d28d9; font-weight: 700;",
                        "{complete_count()}"
                    }
                }
                div {
                    style: "display: flex; align-items: center; gap: 0.375rem;",
                    span { style: "color: #94a3b8; font-weight: 600;", "Completed Value:" }
                    span {
                        id: "sw4a-pin-digits-completed-val",
                        style: "display: inline-flex; align-items: center; min-width: 4.5rem; min-height: 1.5rem; border-radius: 0.25rem; padding: 0.125rem 0.5rem; background-color: #f1f5f9; color: #334155; font-weight: 700;",
                        "{completed_val()}"
                    }
                }
            }
        }
    }
}

// ---------------------------------------------------------------------------
// Scenario 2: Paste Transformer Stripping Hyphens & Spaces (SW4A-8)
// ---------------------------------------------------------------------------
#[component]
fn PasteTransformerScenario() -> Element {
    let mut paste_val = use_signal(String::new);
    let mut paste_complete_count = use_signal(|| 0usize);

    rsx! {
        div {
            class: "rounded-2xl border border-slate-200 bg-white p-6 shadow-sm space-y-4",
            style: "border-radius: 1rem; border: 1px solid #e2e8f0; background-color: #ffffff; padding: 1.5rem; display: flex; flex-direction: column; gap: 1rem;",

            div {
                class: "space-y-1",
                style: "display: flex; flex-direction: column; gap: 0.25rem;",
                h3 {
                    class: "text-base font-semibold text-slate-900",
                    style: "margin: 0; font-size: 1rem; font-weight: 600; color: #0f172a;",
                    "2. Multi-Char Paste & paste_transformer (SW4A-8)"
                }
                p {
                    class: "text-xs text-slate-500",
                    style: "margin: 0; font-size: 0.75rem; color: #64748b;",
                    "Paste \"123-456\" or \"123 456\" into the segmented field; paste_transformer strips '-' and spaces before populating all 6 digit cells."
                }
            }

            div {
                style: "padding: 0.5rem 0;",
                PinInputRoot {
                    id: "sw4a-pin-paste",
                    input_id: "sw4a-pin-paste-input",
                    max_length: 6,
                    pattern: PinInputPattern::Digits,
                    input_mode: PinInputMode::Numeric,
                    paste_transformer: Callback::new(|s: String| s.replace('-', "").replace(' ', "")),
                    style: "display: inline-flex; align-items: center; gap: 8px;",
                    on_value_change: move |v: String| paste_val.set(v),
                    on_complete: move |_| paste_complete_count += 1,
                    render_children: move |snap: PinInputSnapshot| {
                        render_pin_cells("sw4a-pin-paste", &snap, Some(2))
                    }
                }
            }

            div {
                class: "pt-2 flex flex-wrap items-center gap-4 text-xs font-mono",
                style: "padding-top: 0.5rem; display: flex; flex-wrap: wrap; align-items: center; gap: 1rem; font-size: 0.75rem; font-family: monospace;",
                div {
                    style: "display: flex; align-items: center; gap: 0.375rem;",
                    span { style: "color: #94a3b8; font-weight: 600;", "Paste Value:" }
                    span {
                        id: "sw4a-pin-paste-val",
                        style: "display: inline-flex; align-items: center; min-width: 4.5rem; min-height: 1.5rem; border-radius: 0.25rem; padding: 0.125rem 0.5rem; background-color: #f1f5f9; color: #334155; font-weight: 700;",
                        "{paste_val()}"
                    }
                }
                div {
                    style: "display: flex; align-items: center; gap: 0.375rem;",
                    span { style: "color: #94a3b8; font-weight: 600;", "on_complete Count:" }
                    span {
                        id: "sw4a-pin-paste-complete",
                        style: "display: inline-flex; align-items: center; border-radius: 0.25rem; padding: 0.125rem 0.5rem; background-color: #f5f3ff; color: #6d28d9; font-weight: 700;",
                        "{paste_complete_count()}"
                    }
                }
            }
        }
    }
}

// ---------------------------------------------------------------------------
// Scenario 3: Password Masked OTP (SW4A-9)
// ---------------------------------------------------------------------------
#[component]
fn MaskedOtpScenario() -> Element {
    let mut mask_enabled = use_signal(|| true);
    let mut mask_val = use_signal(String::new);

    rsx! {
        div {
            class: "rounded-2xl border border-slate-200 bg-white p-6 shadow-sm space-y-4",
            style: "border-radius: 1rem; border: 1px solid #e2e8f0; background-color: #ffffff; padding: 1.5rem; display: flex; flex-direction: column; gap: 1rem;",

            div {
                class: "space-y-1",
                style: "display: flex; flex-direction: column; gap: 0.25rem;",
                h3 {
                    class: "text-base font-semibold text-slate-900",
                    style: "margin: 0; font-size: 1rem; font-weight: 600; color: #0f172a;",
                    "3. Password Masked OTP (SW4A-9)"
                }
                p {
                    class: "text-xs text-slate-500",
                    style: "margin: 0; font-size: 0.75rem; color: #64748b;",
                    "Cells render '•' when mask = true across both Web and Blitz Native while bound state retains the raw entered digits."
                }
            }

            div {
                style: "display: flex; flex-wrap: wrap; align-items: center; gap: 1rem; padding: 0.5rem 0;",
                PinInputRoot {
                    id: "sw4a-pin-mask",
                    input_id: "sw4a-pin-mask-input",
                    max_length: 6,
                    pattern: PinInputPattern::Digits,
                    input_mode: PinInputMode::Numeric,
                    mask: mask_enabled(),
                    mask_char: '•',
                    style: "display: inline-flex; align-items: center; gap: 8px;",
                    on_value_change: move |v: String| mask_val.set(v),
                    render_children: move |snap: PinInputSnapshot| {
                        render_pin_cells("sw4a-pin-mask", &snap, None)
                    }
                }

                button {
                    id: "sw4a-pin-mask-toggle",
                    r#type: "button",
                    class: "sw1a-btn-secondary",
                    style: "padding: 0.375rem 0.75rem; font-size: 0.75rem; font-weight: 600; border-radius: 0.5rem; border: 1px solid #cbd5e1; background-color: #ffffff; color: #334155; cursor: pointer;",
                    onclick: move |_| mask_enabled.toggle(),
                    if mask_enabled() {
                        "Mask: ON (•)"
                    } else {
                        "Mask: OFF"
                    }
                }
            }

            div {
                class: "pt-2 flex flex-wrap items-center gap-4 text-xs font-mono",
                style: "padding-top: 0.5rem; display: flex; flex-wrap: wrap; align-items: center; gap: 1rem; font-size: 0.75rem; font-family: monospace;",
                div {
                    style: "display: flex; align-items: center; gap: 0.375rem;",
                    span { style: "color: #94a3b8; font-weight: 600;", "Bound Value:" }
                    span {
                        id: "sw4a-pin-mask-val",
                        style: "display: inline-flex; align-items: center; min-width: 4.5rem; min-height: 1.5rem; border-radius: 0.25rem; padding: 0.125rem 0.5rem; background-color: #f1f5f9; color: #334155; font-weight: 700;",
                        "{mask_val()}"
                    }
                }
            }
        }
    }
}

// ---------------------------------------------------------------------------
// Scenario 4: Form Submission & Native Reset Sync (SW4A-10)
// ---------------------------------------------------------------------------
#[component]
fn FormOtpScenario() -> Element {
    let mut form_val = use_signal(|| String::from("123456"));
    let mut form_entries = use_signal(|| String::from("[]"));

    rsx! {
        div {
            class: "rounded-2xl border border-slate-200 bg-white p-6 shadow-sm space-y-4",
            style: "border-radius: 1rem; border: 1px solid #e2e8f0; background-color: #ffffff; padding: 1.5rem; display: flex; flex-direction: column; gap: 1rem;",

            div {
                class: "space-y-1",
                style: "display: flex; flex-direction: column; gap: 0.25rem;",
                h3 {
                    class: "text-base font-semibold text-slate-900",
                    style: "margin: 0; font-size: 1rem; font-weight: 600; color: #0f172a;",
                    "4. HTML Form Submission & Native Reset Sync (SW4A-10)"
                }
                p {
                    class: "text-xs text-slate-500",
                    style: "margin: 0; font-size: 0.75rem; color: #64748b;",
                    "Named PinInputRoot (name = \"otp\", default_value = \"123456\") inside <form id=\"sw4a-pin-form\"> serializes via FormData and restores default_value on native <button type=\"reset\">."
                }
            }

            form {
                id: "sw4a-pin-form",
                style: "display: flex; flex-direction: column; gap: 1rem;",
                onsubmit: move |evt| {
                    evt.prevent_default();
                    if let Ok(entries) = monoxus::foundation::form::collect_form_data("sw4a-pin-form") {
                        form_entries.set(serde_json::to_string(&entries).unwrap_or_default());
                    }
                },
                onreset: move |_| {
                    form_entries.set(String::from("[]"));
                },

                PinInputRoot {
                    id: "sw4a-pin-form-root",
                    input_id: "sw4a-pin-form-input",
                    name: "otp",
                    default_value: "123456",
                    max_length: 6,
                    pattern: PinInputPattern::Digits,
                    input_mode: PinInputMode::Numeric,
                    style: "display: inline-flex; align-items: center; gap: 8px;",
                    on_value_change: move |v: String| {
                        form_val.set(v);
                    },
                    render_children: move |snap: PinInputSnapshot| {
                        render_pin_cells("sw4a-pin-form", &snap, None)
                    }
                }

                div {
                    style: "display: flex; flex-wrap: wrap; align-items: center; gap: 0.75rem; padding-top: 0.25rem;",
                    button {
                        id: "sw4a-pin-reset-btn",
                        r#type: "reset",
                        class: "sw1a-btn-secondary",
                        style: "padding: 0.45rem 0.9rem; font-size: 0.8125rem; font-weight: 500; border-radius: 0.5rem; border: 1px solid #cbd5e1; background-color: #ffffff; color: #334155; cursor: pointer;",
                        "Reset Form"
                    }
                    button {
                        id: "sw4a-pin-submit-btn",
                        r#type: "submit",
                        class: "sw1a-btn-primary",
                        style: "padding: 0.45rem 0.9rem; font-size: 0.8125rem; font-weight: 600; border-radius: 0.5rem; border: 1px solid #6d28d9; background-color: #7c3aed; color: #ffffff; cursor: pointer;",
                        "Submit Form"
                    }
                    div {
                        style: "display: flex; align-items: center; gap: 0.375rem; font-size: 0.75rem; font-family: monospace;",
                        span { style: "color: #94a3b8; font-weight: 600;", "Form Value:" }
                        span {
                            id: "sw4a-pin-form-val",
                            style: "display: inline-flex; align-items: center; min-width: 4.5rem; min-height: 1.5rem; border-radius: 0.25rem; padding: 0.125rem 0.5rem; background-color: #f1f5f9; color: #334155; font-weight: 700;",
                            "{form_val()}"
                        }
                    }
                }
            }

            div {
                style: "display: flex; flex-direction: column; gap: 0.375rem;",
                span {
                    style: "font-size: 0.75rem; font-family: monospace; color: #64748b;",
                    "Serialized FormData (<form id=\"sw4a-pin-form\">):"
                }
                pre {
                    id: "sw4a-pin-form-entries",
                    style: "margin: 0; padding: 0.75rem; background-color: #f1f5f9; color: #1e293b; border-radius: 0.5rem; font-family: monospace; font-size: 0.75rem; overflow-x: auto;",
                    "{form_entries()}"
                }
            }
        }
    }
}

use dioxus::prelude::*;
use monoxus::button::{Button, ButtonType};
use monoxus::checkbox::{
    Checkbox, CheckboxGroup, CheckboxIndicator, CheckedState, use_checkbox_context,
};
use monoxus::radio_group::{RadioGroupIndicator, RadioGroupItem, RadioGroupRoot};
use monoxus::switch::{SwitchRoot, SwitchThumb};

#[component]
fn CheckboxGlyph() -> Element {
    let ctx = use_checkbox_context();
    let state = ctx.checked_state();
    match state {
        CheckedState::Checked => rsx! { span { "✓" } },
        CheckedState::Indeterminate => rsx! { span { "−" } },
        CheckedState::Unchecked => rsx! {},
    }
}

#[component]
pub fn Form1bPlayground() -> Element {
    let mut form_entries = use_signal(|| String::from("[]"));
    let mut ext_entries = use_signal(|| String::from("[]"));

    use_effect(move || {
        if let Ok(entries) = monoxus::foundation::form::collect_form_data("sw1b-form") {
            form_entries.set(serde_json::to_string(&entries).unwrap_or_default());
        }
        if let Ok(entries) = monoxus::foundation::form::collect_form_data("sw1b-ext-form") {
            ext_entries.set(serde_json::to_string(&entries).unwrap_or_default());
        }
    });

    let refresh_form = move || {
        spawn(async move {
            futures_timer::Delay::new(std::time::Duration::from_millis(16)).await;
            if let Ok(entries) = monoxus::foundation::form::collect_form_data("sw1b-form") {
                form_entries.set(serde_json::to_string(&entries).unwrap_or_default());
            }
        });
    };

    let refresh_ext_form = move || {
        spawn(async move {
            futures_timer::Delay::new(std::time::Duration::from_millis(16)).await;
            if let Ok(entries) = monoxus::foundation::form::collect_form_data("sw1b-ext-form") {
                ext_entries.set(serde_json::to_string(&entries).unwrap_or_default());
            }
        });
    };

    rsx! {
        div {
            class: "max-w-4xl mx-auto space-y-6",

            div {
                class: "space-y-1",
                h2 {
                    class: "text-2xl font-bold tracking-tight text-slate-900",
                    "Form Synchronization (Sub-wave 1B)"
                }
                p {
                    class: "text-sm text-slate-500",
                    "Demonstrates shared out-of-flow hidden input serialization, zero layout shift in flex containers, and live form reset parity across Web and Blitz Native."
                }
            }

            // SW1B-8 Part 1: Zero Layout Shift Comparison
            div {
                class: "rounded-2xl border border-slate-200 bg-white p-5 shadow-sm space-y-4",
                div {
                    class: "space-y-1",
                    h3 {
                        class: "text-base font-semibold text-slate-900",
                        "1. Zero Layout Shift Verification (SW1B-8)"
                    }
                    p {
                        class: "text-xs text-slate-500",
                        "Flex rows with gap: 12px comparing unnamed controls vs named controls with sibling HiddenFormInput. Layout rects and spacing are identical (0px shift)."
                    }
                }

                div {
                    class: "grid grid-cols-1 md:grid-cols-2 gap-4",

                    div {
                        class: "p-4 bg-slate-50 rounded-lg border border-slate-100 flex flex-col gap-2",
                        span { class: "text-xs font-semibold text-slate-700", "Unnamed Controls (No Hidden Input)" }
                        div {
                            id: "sw1b-unnamed-row",
                            style: "display: flex; gap: 12px; align-items: center;",
                            Checkbox {
                                id: "sw1b-unnamed-a".to_string(),
                                CheckboxIndicator { CheckboxGlyph {} }
                            }
                            Checkbox {
                                id: "sw1b-unnamed-b".to_string(),
                                CheckboxIndicator { CheckboxGlyph {} }
                            }
                            span { class: "text-xs text-slate-500", "gap: 12px" }
                        }
                    }

                    div {
                        class: "p-4 bg-slate-50 rounded-lg border border-slate-100 flex flex-col gap-2",
                        span { class: "text-xs font-semibold text-slate-700", "Named Controls (With Sibling Hidden Input)" }
                        div {
                            id: "sw1b-named-row",
                            style: "display: flex; gap: 12px; align-items: center;",
                            Checkbox {
                                id: "sw1b-named-a".to_string(),
                                name: "named_a".to_string(),
                                value: "val_a".to_string(),
                                CheckboxIndicator { CheckboxGlyph {} }
                            }
                            Checkbox {
                                id: "sw1b-named-b".to_string(),
                                name: "named_b".to_string(),
                                value: "val_b".to_string(),
                                CheckboxIndicator { CheckboxGlyph {} }
                            }
                            span { class: "text-xs text-slate-500", "gap: 12px" }
                        }
                    }
                }
            }

            // SW1B-8 Part 2 & SW1B-9: Ancestor <form id="sw1b-form">
            div {
                class: "rounded-2xl border border-slate-200 bg-white p-5 shadow-sm space-y-4",
                div {
                    class: "space-y-1",
                    h3 {
                        class: "text-base font-semibold text-slate-900",
                        "2. Ancestor Form Submission & Reset Synchronization (SW1B-8, SW1B-9)"
                    }
                    p {
                        class: "text-xs text-slate-500",
                        "Controls inside <form id=\"sw1b-form\"> serialize to FormData. Mutating states and clicking Reset Form restores exact initial values."
                    }
                }

                form {
                    id: "sw1b-form",
                    class: "p-4 bg-slate-50 rounded-lg border border-slate-100 flex flex-col gap-4",
                    onsubmit: move |evt| {
                        evt.prevent_default();
                        if let Ok(entries) = monoxus::foundation::form::collect_form_data("sw1b-form") {
                            form_entries.set(serde_json::to_string(&entries).unwrap_or_default());
                        }
                    },
                    onreset: move |_| {
                        refresh_form();
                    },

                    // Checkbox tri
                    div {
                        class: "flex items-center gap-2",
                        Checkbox {
                            id: "form-cb-tri".to_string(),
                            name: "tri".to_string(),
                            value: "tri-yes".to_string(),
                            default_checked: CheckedState::Indeterminate,
                            CheckboxIndicator {
                                id: "form-cb-tri-ind".to_string(),
                                CheckboxGlyph {}
                            }
                        }
                        span { class: "text-xs text-slate-700 font-medium", "Tri-state Checkbox (name=\"tri\", val=\"tri-yes\", init=Indeterminate)" }
                    }

                    // CheckboxGroup channels
                    div {
                        class: "flex flex-col gap-1.5 pt-2 border-t border-slate-200/60",
                        span { class: "text-xs font-semibold text-slate-700", "CheckboxGroup Channels (name=\"channels\", init=[\"email\"])" }
                        CheckboxGroup {
                            id: "form-cb-group".to_string(),
                            name: "channels".to_string(),
                            default_values: vec![String::from("email")],
                            div {
                                class: "flex gap-4 items-center",
                                label {
                                    class: "inline-flex items-center gap-2 text-xs text-slate-700 cursor-pointer select-none",
                                    Checkbox {
                                        id: "form-cbg-email".to_string(),
                                        value: "email".to_string(),
                                        CheckboxIndicator { id: "form-cbg-email-ind".to_string(), CheckboxGlyph {} }
                                    }
                                    span { "Email" }
                                }
                                label {
                                    class: "inline-flex items-center gap-2 text-xs text-slate-700 cursor-pointer select-none",
                                    Checkbox {
                                        id: "form-cbg-sms".to_string(),
                                        value: "sms".to_string(),
                                        CheckboxIndicator { id: "form-cbg-sms-ind".to_string(), CheckboxGlyph {} }
                                    }
                                    span { "SMS" }
                                }
                                label {
                                    class: "inline-flex items-center gap-2 text-xs text-slate-400 select-none",
                                    Checkbox {
                                        id: "form-cbg-disabled".to_string(),
                                        value: "disabled_chan".to_string(),
                                        disabled: true,
                                        CheckboxIndicator { id: "form-cbg-disabled-ind".to_string(), CheckboxGlyph {} }
                                    }
                                    span { "Disabled" }
                                }
                            }
                        }
                    }

                    // Switch marketing
                    div {
                        class: "flex items-center gap-3 pt-2 border-t border-slate-200/60",
                        SwitchRoot {
                            id: "form-sw".to_string(),
                            name: "marketing".to_string(),
                            value: "opt-in".to_string(),
                            default_checked: true,
                            SwitchThumb { id: "form-sw-thumb".to_string() }
                        }
                        span { class: "text-xs text-slate-700 font-medium", "Marketing Switch (name=\"marketing\", val=\"opt-in\", init=checked)" }
                    }

                    // RadioGroup plan
                    div {
                        class: "flex flex-col gap-1.5 pt-2 border-t border-slate-200/60",
                        span { class: "text-xs font-semibold text-slate-700", "Plan RadioGroup (name=\"plan\", init=\"pro\")" }
                        RadioGroupRoot {
                            id: "form-rg".to_string(),
                            name: "plan".to_string(),
                            default_value: Some("pro".to_string()),
                            div {
                                class: "flex gap-4 items-center",
                                label {
                                    class: "inline-flex items-center gap-2 text-xs text-slate-700 cursor-pointer select-none",
                                    RadioGroupItem { id: "form-rg-free".to_string(), value: "free".to_string(), RadioGroupIndicator { id: "form-rg-free-ind".to_string() } }
                                    span { "Free" }
                                }
                                label {
                                    class: "inline-flex items-center gap-2 text-xs text-slate-700 cursor-pointer select-none",
                                    RadioGroupItem { id: "form-rg-pro".to_string(), value: "pro".to_string(), RadioGroupIndicator { id: "form-rg-pro-ind".to_string() } }
                                    span { "Pro" }
                                }
                                label {
                                    class: "inline-flex items-center gap-2 text-xs text-slate-700 cursor-pointer select-none",
                                    RadioGroupItem { id: "form-rg-enterprise".to_string(), value: "enterprise".to_string(), RadioGroupIndicator { id: "form-rg-enterprise-ind".to_string() } }
                                    span { "Enterprise" }
                                }
                                label {
                                    class: "inline-flex items-center gap-2 text-xs text-slate-400 select-none",
                                    RadioGroupItem { id: "form-rg-disabled".to_string(), value: "disabled_plan".to_string(), disabled: true, RadioGroupIndicator { id: "form-rg-disabled-ind".to_string() } }
                                    span { "Disabled" }
                                }
                            }
                        }
                    }

                    // RadioGroup empty_plan (unselected)
                    div {
                        class: "flex flex-col gap-1.5 pt-2 border-t border-slate-200/60",
                        span { class: "text-xs font-semibold text-slate-700", "Unselected RadioGroup (name=\"empty_plan\", init=None, submits 0 entries)" }
                        RadioGroupRoot {
                            id: "form-rg-empty".to_string(),
                            name: "empty_plan".to_string(),
                            default_value: None,
                            div {
                                class: "flex gap-4 items-center",
                                label {
                                    class: "inline-flex items-center gap-2 text-xs text-slate-700 cursor-pointer select-none",
                                    RadioGroupItem { id: "form-rg-empty-a".to_string(), value: "a".to_string(), RadioGroupIndicator { id: "form-rg-empty-a-ind".to_string() } }
                                    span { "Empty Option A" }
                                }
                                label {
                                    class: "inline-flex items-center gap-2 text-xs text-slate-700 cursor-pointer select-none",
                                    RadioGroupItem { id: "form-rg-empty-b".to_string(), value: "b".to_string(), RadioGroupIndicator { id: "form-rg-empty-b-ind".to_string() } }
                                    span { "Empty Option B" }
                                }
                            }
                        }
                    }

                    // Buttons
                    div {
                        class: "flex gap-3 pt-3 border-t border-slate-200/60 items-center",
                        Button {
                            id: "sw1b-form-reset".to_string(),
                            r#type: ButtonType::Reset,
                            class: "sw1a-btn-secondary text-xs".to_string(),
                            on_click: move |_| {
                                refresh_form();
                            },
                            "Reset Form"
                        }
                        Button {
                            id: "sw1b-form-inspect".to_string(),
                            r#type: ButtonType::Button,
                            class: "sw1a-btn-primary text-xs".to_string(),
                            on_click: move |_| {
                                if let Ok(entries) = monoxus::foundation::form::collect_form_data("sw1b-form") {
                                    form_entries.set(serde_json::to_string(&entries).unwrap_or_default());
                                }
                            },
                            "Submit Form"
                        }
                    }
                }

                div {
                    class: "space-y-1.5",
                    span { class: "text-xs font-mono text-slate-500", "Serialized form entries (<form id=\"sw1b-form\">):" }
                    pre {
                        id: "sw1b-form-entries",
                        class: "p-3 bg-slate-100 text-slate-800 rounded font-mono text-xs overflow-x-auto",
                        "{form_entries()}"
                    }
                }
            }

            // SW1B-8 & SW1B-9 External Form Section
            div {
                class: "rounded-2xl border border-slate-200 bg-white p-5 shadow-sm space-y-4",
                div {
                    class: "space-y-1",
                    h3 {
                        class: "text-base font-semibold text-slate-900",
                        "3. External Form Association (form=\"sw1b-ext-form\") (SW1B-8, SW1B-9)"
                    }
                    p {
                        class: "text-xs text-slate-500",
                        "Controls associated via external form=\"id\" attribute serialize to and reset with the target form."
                    }
                }

                form {
                    id: "sw1b-ext-form",
                    onsubmit: move |evt| {
                        evt.prevent_default();
                        if let Ok(entries) = monoxus::foundation::form::collect_form_data("sw1b-ext-form") {
                            ext_entries.set(serde_json::to_string(&entries).unwrap_or_default());
                        }
                    },
                    onreset: move |_| {
                        refresh_ext_form();
                    },
                }

                div {
                    class: "p-4 bg-slate-50 rounded-lg border border-slate-100 flex flex-col gap-3",
                    div {
                        class: "flex items-center gap-2",
                        Checkbox {
                            id: "form-cb-external".to_string(),
                            form: "sw1b-ext-form".to_string(),
                            name: "ext_opt".to_string(),
                            value: "ext_on".to_string(),
                            default_checked: CheckedState::Unchecked,
                            CheckboxIndicator {
                                id: "form-cb-ext-ind".to_string(),
                                CheckboxGlyph {}
                            }
                        }
                        span { class: "text-xs text-slate-700 font-medium", "External Checkbox (form=\"sw1b-ext-form\", name=\"ext_opt\", val=\"ext_on\")" }
                    }

                    div {
                        class: "flex gap-3 items-center pt-2 border-t border-slate-200/60",
                        Button {
                            id: "sw1b-ext-reset".to_string(),
                            r#type: ButtonType::Reset,
                            form: "sw1b-ext-form".to_string(),
                            class: "sw1a-btn-secondary text-xs".to_string(),
                            on_click: move |_| {
                                refresh_ext_form();
                            },
                            "Reset External Form"
                        }
                        Button {
                            id: "sw1b-ext-inspect".to_string(),
                            r#type: ButtonType::Button,
                            class: "sw1a-btn-primary text-xs".to_string(),
                            on_click: move |_| {
                                if let Ok(entries) = monoxus::foundation::form::collect_form_data("sw1b-ext-form") {
                                    ext_entries.set(serde_json::to_string(&entries).unwrap_or_default());
                                }
                            },
                            "Submit External Form"
                        }
                    }
                }

                div {
                    class: "space-y-1.5",
                    span { class: "text-xs font-mono text-slate-500", "Serialized external form entries (<form id=\"sw1b-ext-form\">):" }
                    pre {
                        id: "sw1b-ext-entries",
                        class: "p-3 bg-slate-100 text-slate-800 rounded font-mono text-xs overflow-x-auto",
                        "{ext_entries()}"
                    }
                }
            }
        }
    }
}

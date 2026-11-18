use dioxus::prelude::*;
use monoxus::button::{Button, ButtonType};
use monoxus::checkbox::{
    Checkbox, CheckboxGroup, CheckboxGroupLabel, CheckboxIndicator, CheckedState,
    use_checkbox_context,
};
use monoxus::label::Label;

#[component]
pub fn CheckboxGlyph() -> Element {
    let ctx = use_checkbox_context();
    let state = ctx.checked_state();
    match state {
        CheckedState::Checked => rsx! { span { "✓" } },
        CheckedState::Indeterminate => rsx! { span { "−" } },
        CheckedState::Unchecked => rsx! {},
    }
}

#[component]
pub fn CheckboxPlayground() -> Element {
    let mut tristate = use_signal(|| CheckedState::Indeterminate);
    let mut show_group_label = use_signal(|| true);
    let mut group_vals = use_signal(|| vec![String::from("email")]);

    rsx! {
        div {
            class: "max-w-4xl mx-auto space-y-6",

            div {
                class: "space-y-1",
                h2 {
                    class: "text-2xl font-bold tracking-tight text-slate-900",
                    "Checkbox"
                }
                p {
                    class: "text-sm text-slate-500",
                    "A tri-state control allowing the user to toggle between checked, unchecked, or indeterminate states, with Enter suppression and form synchronization."
                }
            }

            // SW1B-1: Tri-state, Indicator Presence & Enter Suppression
            div {
                class: "rounded-2xl border border-slate-200 bg-white p-5 shadow-sm space-y-4",
                div {
                    class: "space-y-1",
                    h3 {
                        class: "text-base font-semibold text-slate-900",
                        "1. Tri-state, Indicator Presence & Enter Suppression (SW1B-1)"
                    }
                    p {
                        class: "text-xs text-slate-500",
                        "Indeterminate state (aria-checked=\"mixed\"). Enter is suppressed. Space toggles to checked. Click toggles to unchecked."
                    }
                }

                div {
                    class: "flex flex-wrap items-center gap-4",
                    div {
                        class: "flex items-center gap-3",
                        Checkbox {
                            id: "cb-tristate".to_string(),
                            default_checked: CheckedState::Indeterminate,
                            checked: tristate(),
                            on_checked_change: move |next| tristate.set(next),
                            CheckboxIndicator {
                                id: "cb-tristate-indicator".to_string(),
                                CheckboxGlyph {}
                            }
                        }
                        span {
                            class: "text-sm font-medium text-slate-700",
                            "Tri-state Task Item"
                        }
                    }

                    Button {
                        id: "cb-seed-indeterminate".to_string(),
                        r#type: ButtonType::Button,
                        class: "sw1a-btn-secondary text-xs".to_string(),
                        on_click: move |_| tristate.set(CheckedState::Indeterminate),
                        "Reset to Indeterminate"
                    }

                    div {
                        class: "flex items-center gap-1.5 text-xs font-mono text-slate-600",
                        "State: "
                        strong {
                            id: "cb-tristate-status",
                            "{tristate().data_state_str()}"
                        }
                    }
                }
            }

            // SW1B-2: Disabled, ReadOnly & Label Forwarding
            div {
                class: "rounded-2xl border border-slate-200 bg-white p-5 shadow-sm space-y-4",
                div {
                    class: "space-y-1",
                    h3 {
                        class: "text-base font-semibold text-slate-900",
                        "2. Disabled, ReadOnly & Label Forwarding (SW1B-2)"
                    }
                    p {
                        class: "text-xs text-slate-500",
                        "Disabled ignores interaction and is skipped in Tab order. ReadOnly receives focus but blocks toggle. Label forwards click/focus."
                    }
                }

                div {
                    class: "grid grid-cols-1 md:grid-cols-2 gap-4",
                    // Disabled & ReadOnly
                    div {
                        class: "flex flex-col gap-3 p-3 bg-slate-50 rounded-lg border border-slate-100",
                        div {
                            class: "flex items-center gap-2",
                            Checkbox {
                                id: "cb-disabled".to_string(),
                                disabled: true,
                                default_checked: CheckedState::Checked,
                                CheckboxIndicator {
                                    id: "cb-disabled-ind".to_string(),
                                    CheckboxGlyph {}
                                }
                            }
                            span { class: "text-xs font-mono text-slate-500", "Disabled (Checked)" }
                        }
                        div {
                            class: "flex items-center gap-2",
                            Checkbox {
                                id: "cb-readonly".to_string(),
                                read_only: true,
                                default_checked: CheckedState::Checked,
                                CheckboxIndicator {
                                    id: "cb-readonly-ind".to_string(),
                                    CheckboxGlyph {}
                                }
                            }
                            span { class: "text-xs font-mono text-slate-500", "ReadOnly (Checked)" }
                        }
                    }

                    // Explicit & Implicit Labels
                    div {
                        class: "flex flex-col gap-3 p-3 bg-slate-50 rounded-lg border border-slate-100",
                        div {
                            class: "flex items-center gap-2",
                            Checkbox {
                                id: "cb-terms".to_string(),
                                default_checked: CheckedState::Unchecked,
                                CheckboxIndicator {
                                    id: "cb-terms-ind".to_string(),
                                    CheckboxGlyph {}
                                }
                            }
                            Label {
                                id: "lbl-cb-terms".to_string(),
                                r#for: "cb-terms".to_string(),
                                class: "text-sm text-slate-700 cursor-pointer select-none".to_string(),
                                "Explicit Label (Terms of Service)"
                            }
                        }
                        div {
                            Label {
                                id: "lbl-cb-implicit".to_string(),
                                class: "inline-flex items-center gap-2 text-sm text-slate-700 cursor-pointer select-none".to_string(),
                                Checkbox {
                                    id: "cb-implicit".to_string(),
                                    default_checked: CheckedState::Unchecked,
                                    CheckboxIndicator {
                                        id: "cb-implicit-ind".to_string(),
                                        CheckboxGlyph {}
                                    }
                                }
                                span { "Implicit Label (Weekly Newsletter)" }
                            }
                        }
                    }
                }
            }

            // SW1B-3: CheckboxGroup & CheckboxGroupLabel
            div {
                class: "rounded-2xl border border-slate-200 bg-white p-5 shadow-sm space-y-4",
                div {
                    class: "space-y-1",
                    h3 {
                        class: "text-base font-semibold text-slate-900",
                        "3. CheckboxGroup Multi-Selection & Dynamic Label (SW1B-3)"
                    }
                    p {
                        class: "text-xs text-slate-500",
                        "CheckboxGroup manages multi-value synchronization. Toggle button mounts/unmounts GroupLabel, dynamically updating aria-labelledby."
                    }
                }

                div {
                    class: "flex flex-wrap items-center gap-3",
                    Button {
                        id: "cb-toggle-group-label".to_string(),
                        r#type: ButtonType::Button,
                        class: "sw1a-btn-secondary text-xs".to_string(),
                        on_click: move |_| show_group_label.toggle(),
                        "Toggle Group Label"
                    }
                    div {
                        class: "flex items-center gap-1.5 text-xs font-mono text-slate-600",
                        "Selected values: "
                        strong {
                            id: "cb-group-values",
                            "{serde_json::to_string(&group_vals()).unwrap_or_default()}"
                        }
                    }
                }

                CheckboxGroup {
                    id: "cb-group".to_string(),
                    values: group_vals(),
                    on_values_change: move |v| group_vals.set(v),
                    default_values: vec![String::from("email")],
                    class: "p-3 bg-slate-50 rounded-lg border border-slate-100 flex flex-col gap-2.5".to_string(),
                    if show_group_label() {
                        CheckboxGroupLabel {
                            id: "cb-group-label".to_string(),
                            class: "text-xs font-semibold uppercase tracking-wider text-slate-500".to_string(),
                            "Notification Preferences"
                        }
                    }
                    div {
                        class: "flex flex-wrap gap-4 items-center",
                        label {
                            class: "inline-flex items-center gap-2 text-sm text-slate-700 cursor-pointer select-none",
                            Checkbox {
                                id: "cb-group-email".to_string(),
                                value: "email".to_string(),
                                CheckboxIndicator {
                                    id: "cb-group-email-ind".to_string(),
                                    CheckboxGlyph {}
                                }
                            }
                            span { "Email" }
                        }
                        label {
                            class: "inline-flex items-center gap-2 text-sm text-slate-700 cursor-pointer select-none",
                            Checkbox {
                                id: "cb-group-sms".to_string(),
                                value: "sms".to_string(),
                                CheckboxIndicator {
                                    id: "cb-group-sms-ind".to_string(),
                                    CheckboxGlyph {}
                                }
                            }
                            span { "SMS" }
                        }
                        label {
                            class: "inline-flex items-center gap-2 text-sm text-slate-700 cursor-pointer select-none",
                            Checkbox {
                                id: "cb-group-push".to_string(),
                                value: "push".to_string(),
                                CheckboxIndicator {
                                    id: "cb-group-push-ind".to_string(),
                                    CheckboxGlyph {}
                                }
                            }
                            span { "Push" }
                        }
                    }
                }
            }
        }
    }
}

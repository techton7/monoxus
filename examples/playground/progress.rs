use dioxus::prelude::*;
use monoxus::progress::{Indicator, Progress};

const CARD_STYLE: &str = "display: grid; gap: 0.75rem; padding: 1rem; border: 1px solid #e2e8f0; border-radius: 0.75rem; background: white;";
const TRACK_STYLE: &str =
    "overflow: hidden; min-height: 0.75rem; border-radius: 9999px; background: #e2e8f0;";
const INDICATOR_STYLE: &str = "height: 0.75rem; border-radius: inherit; background: #7c3aed;";
const BUTTON_STYLE: &str = "padding: 0.375rem 0.75rem; border: 1px solid #cbd5e1; border-radius: 0.375rem; background: white; color: #334155; cursor: pointer;";

#[component]
pub fn ProgressPlayground() -> Element {
    let mut reactive_max = use_signal(|| 100.0);
    let mut reactive_value = use_signal(|| Some(40.0));
    let custom_value_label =
        use_callback(|(value, max): (f64, f64)| format!("{value} of {max} downloaded"));

    rsx! {
        section {
            id: "progress-playground",
            "data-scenario": "SW2A-progress",
            class: "max-w-4xl mx-auto space-y-6",

            div {
                class: "space-y-1",
                h2 {
                    class: "text-2xl font-bold tracking-tight text-slate-900",
                    "Progress"
                }
                p {
                    class: "text-sm text-slate-500",
                    "Headless progress roots publish synchronized WAI-ARIA and data attributes for determinate and indeterminate states."
                }
            }

            div {
                id: "progress-default",
                "data-scenario": "SW2A-1-default",
                style: CARD_STYLE,
                h3 { "Default" }
                Progress {
                    id: "progress-default-root".to_string(),
                    class: "progress-root".to_string(),
                    style: TRACK_STYLE.to_string(),
                    Indicator {
                        id: "progress-default-indicator".to_string(),
                        style: INDICATOR_STYLE.to_string(),
                    }
                }
            }

            div {
                id: "progress-loading",
                "data-scenario": "SW2A-1-loading",
                style: CARD_STYLE,
                h3 { "Loading (35 of 100)" }
                Progress {
                    id: "progress-loading-root".to_string(),
                    value: 35.0,
                    style: TRACK_STYLE.to_string(),
                    Indicator {
                        id: "progress-loading-indicator".to_string(),
                        style: INDICATOR_STYLE.to_string(),
                    }
                }
            }

            div {
                id: "progress-complete",
                "data-scenario": "SW2A-1-complete",
                style: CARD_STYLE,
                h3 { "Complete" }
                Progress {
                    id: "progress-complete-root".to_string(),
                    value: 100.0,
                    style: TRACK_STYLE.to_string(),
                    Indicator {
                        id: "progress-complete-indicator".to_string(),
                        style: INDICATOR_STYLE.to_string(),
                    }
                }
            }

            div {
                id: "progress-indeterminate",
                "data-scenario": "SW2A-1-indeterminate",
                style: CARD_STYLE,
                h3 { "Explicitly indeterminate" }
                Progress {
                    id: "progress-indeterminate-root".to_string(),
                    value: None,
                    max: 80.0,
                    style: TRACK_STYLE.to_string(),
                    Indicator {
                        id: "progress-indeterminate-indicator".to_string(),
                        style: INDICATOR_STYLE.to_string(),
                    }
                }
            }

            div {
                id: "progress-reactive-validation",
                "data-scenario": "SW2A-2-validation",
                style: CARD_STYLE,
                h3 { "Reactive validation and restoration" }
                p {
                    id: "progress-reactive-inputs",
                    class: "text-sm text-slate-500",
                    "Requested max: {reactive_max}; requested value: {reactive_value().map(|value| value.to_string()).unwrap_or_else(|| \"none\".to_string())}"
                }
                Progress {
                    id: "progress-reactive-root".to_string(),
                    value: reactive_value(),
                    max: reactive_max(),
                    style: TRACK_STYLE.to_string(),
                    Indicator {
                        id: "progress-reactive-indicator".to_string(),
                        style: INDICATOR_STYLE.to_string(),
                    }
                }
                div {
                    class: "flex flex-wrap gap-2",
                    button {
                        id: "progress-set-invalid-max",
                        "data-scenario": "invalid-max",
                        r#type: "button",
                        style: BUTTON_STYLE,
                        onclick: move |_| {
                            reactive_max.set(0.0);
                            reactive_value.set(Some(25.0));
                        },
                        "Invalid max"
                    }
                    button {
                        id: "progress-set-invalid-value",
                        "data-scenario": "invalid-value",
                        r#type: "button",
                        style: BUTTON_STYLE,
                        onclick: move |_| {
                            reactive_max.set(100.0);
                            reactive_value.set(Some(-1.0));
                        },
                        "Invalid value"
                    }
                    button {
                        id: "progress-set-over-max-value",
                        "data-scenario": "over-max-value",
                        r#type: "button",
                        style: BUTTON_STYLE,
                        onclick: move |_| {
                            reactive_max.set(80.0);
                            reactive_value.set(Some(120.0));
                        },
                        "Over-max value"
                    }
                    button {
                        id: "progress-set-nan-value",
                        "data-scenario": "nan-value",
                        r#type: "button",
                        style: BUTTON_STYLE,
                        onclick: move |_| {
                            reactive_max.set(100.0);
                            reactive_value.set(Some(f64::NAN));
                        },
                        "NaN value"
                    }
                    button {
                        id: "progress-set-nan-max",
                        "data-scenario": "nan-max",
                        r#type: "button",
                        style: BUTTON_STYLE,
                        onclick: move |_| {
                            reactive_max.set(f64::NAN);
                            reactive_value.set(Some(40.0));
                        },
                        "NaN max"
                    }
                    button {
                        id: "progress-restore-valid",
                        "data-scenario": "restore-valid",
                        r#type: "button",
                        style: BUTTON_STYLE,
                        onclick: move |_| {
                            reactive_max.set(80.0);
                            reactive_value.set(Some(40.0));
                        },
                        "Restore valid"
                    }
                }
            }

            div {
                id: "progress-custom-label",
                "data-scenario": "SW2A-2-custom-label",
                style: CARD_STYLE,
                h3 { "Custom get_value_label" }
                Progress {
                    id: "progress-custom-label-root".to_string(),
                    value: 32.0,
                    max: 80.0,
                    get_value_label: custom_value_label,
                    style: TRACK_STYLE.to_string(),
                    Indicator {
                        id: "progress-custom-label-indicator".to_string(),
                        style: INDICATOR_STYLE.to_string(),
                    }
                }
            }
        }
    }
}

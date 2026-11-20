use dioxus::prelude::*;
use monoxus::foundation::shared::ScopeHandle;
use monoxus::progress::state::Progress as ProgressModel;
use monoxus::progress::types::{ProgressState, ProgressValidation};
use monoxus::progress::{Indicator, Progress};

const CARD_CLASS: &str = "rounded-2xl border border-slate-200 bg-white p-4 shadow-sm space-y-3";
const H3_CLASS: &str = "text-base font-semibold text-slate-900";
const NOTE_CLASS: &str = "text-xs text-slate-500";
const BTN_CLASS: &str = "sw1a-btn-secondary text-xs";
const INDICATOR_BASE_STYLE: &str = "height: 100%; border-radius: inherit;";

fn model(value: Option<f64>, max: f64) -> ProgressModel {
    ProgressModel::new(ScopeHandle::root("playground"), value, max)
}

/// Consumer-owned geometry only; colors come from `[data-progress-indicator][data-state]` CSS.
fn indicator_style(value: Option<f64>, max: f64) -> String {
    let model = model(value, max);
    match model.state() {
        ProgressState::Indeterminate => {
            format!("{INDICATOR_BASE_STYLE} width: 30%; margin-left: 35%;")
        }
        ProgressState::Complete => format!("{INDICATOR_BASE_STYLE} width: 100%; margin-left: 0%;"),
        ProgressState::Loading => {
            let pct = model.percentage().unwrap_or(0.0).clamp(0.0, 100.0);
            format!("{INDICATOR_BASE_STYLE} width: {pct:.2}%; margin-left: 0%;")
        }
    }
}

fn fmt_num(value: f64) -> String {
    format!("{value}")
}

/// Emitted attribute chips for a normalized model: (label, text).
fn emitted_chips(value: Option<f64>, max: f64) -> Vec<String> {
    let m = model(value, max);
    vec![
        format!("data-state={}", m.state().as_str()),
        "aria-valuemin=0".to_string(),
        format!("aria-valuemax={}", fmt_num(m.max())),
        match m.value() {
            Some(v) => format!("aria-valuenow={}", fmt_num(v)),
            None => "aria-valuenow: omitted".to_string(),
        },
        match m.default_value_label() {
            Some(t) => format!("aria-valuetext={t}"),
            None => "aria-valuetext: omitted".to_string(),
        },
    ]
}

#[component]
fn Chips(chips: Vec<String>) -> Element {
    rsx! {
        div {
            class: "flex flex-wrap gap-1",
            for chip in chips {
                span {
                    key: "{chip}",
                    class: "sw1a-pill-slate",
                    style: "font-size: 10px; line-height: 1.2; padding: 0.0625rem 0.3rem;",
                    "{chip}"
                }
            }
        }
    }
}

#[component]
fn StateItem(
    slug: &'static str,
    scenario: &'static str,
    title: &'static str,
    note: &'static str,
    value: Option<f64>,
    max: f64,
    gui_test_id: bool,
) -> Element {
    let wrapper_id = format!("progress-{slug}");
    let root_id = format!("progress-{slug}-root");
    let indicator_id = format!("progress-{slug}-indicator");
    let chips = emitted_chips(value, max);
    let test_id = gui_test_id.then(|| root_id.clone());
    rsx! {
        div {
            id: "{wrapper_id}",
            "data-scenario": scenario,
            class: "rounded-xl border border-slate-100 bg-slate-50 p-3 space-y-2",
            div {
                class: "space-y-0",
                h4 { class: "text-sm font-semibold text-slate-900", "{title}" }
                p { class: NOTE_CLASS, "{note}" }
            }
            Progress {
                id: root_id,
                value: value,
                max: max,
                "data-gui-test-id": test_id,
                Indicator {
                    id: indicator_id,
                    style: indicator_style(value, max),
                }
            }
            Chips { chips }
        }
    }
}

fn normalization_rules(value: Option<f64>, max: f64) -> Vec<&'static str> {
    model(value, max)
        .validation_issues()
        .iter()
        .map(|issue| match issue {
            ProgressValidation::InvalidMax => "max <= 0 or NaN -> reset to 100",
            ProgressValidation::InvalidValue => {
                "value outside 0..=max or NaN -> normalized to indeterminate, not clamped"
            }
        })
        .collect()
}

fn progress_summary(value: Option<f64>, max: f64) -> String {
    let model = model(value, max);
    let state = model.state().as_str();
    match (model.value(), model.percentage()) {
        (Some(v), Some(pct)) => format!(
            "Normalized: state={state} · {v}/{:.0} ({pct:.0}%)",
            model.max()
        ),
        _ => format!(
            "Normalized: state={state} · indeterminate (max={})",
            model.max()
        ),
    }
}

#[component]
pub fn ProgressPlayground() -> Element {
    let mut reactive_max = use_signal(|| 100.0);
    let mut reactive_value = use_signal(|| Some(40.0));
    let custom_value_label =
        use_callback(|(value, max): (f64, f64)| format!("{value} of {max} downloaded"));

    let req_value = reactive_value();
    let req_max = reactive_max();
    let rules = normalization_rules(req_value, req_max);
    let emitted = emitted_chips(req_value, req_max);
    let norm = model(req_value, req_max);
    let indeterminate_flag = if norm.state() == ProgressState::Indeterminate {
        "data-indeterminate=\"\""
    } else {
        "data-indeterminate: omitted"
    };
    let indicator_sync = format!(
        "indicator data-state=\"{}\" (synced)",
        norm.state().as_str()
    );
    let requested_value = req_value
        .map(|v| v.to_string())
        .unwrap_or_else(|| "none".to_string());

    rsx! {
        section {
            id: "progress-playground",
            "data-scenario": "SW2A-progress",
            class: "max-w-4xl mx-auto space-y-4",

            div {
                class: "space-y-1",
                h2 {
                    class: "text-2xl font-bold tracking-tight text-slate-900",
                    "Progress"
                }
                p {
                    class: "text-sm text-slate-500",
                    "Synchronized WAI-ARIA and data attributes; indicator color comes from CSS keyed on data-state."
                }
            }

            div {
                class: CARD_CLASS,
                div {
                    class: "space-y-1",
                    h3 { class: H3_CLASS, "1. States and defaults (SW2A-1)" }
                    p { class: NOTE_CLASS, "Consumer sets width only; color comes from [data-progress-indicator][data-state]." }
                }
                div {
                    class: "grid grid-cols-2 gap-3",
                    StateItem {
                        slug: "default",
                        scenario: "SW2A-1-default",
                        title: "Default",
                        note: "No value -> indeterminate, not 0%.",
                        value: None,
                        max: 100.0,
                        gui_test_id: true,
                    }
                    StateItem {
                        slug: "loading",
                        scenario: "SW2A-1-loading",
                        title: "Loading (35 of 100)",
                        note: "0 <= value < max -> loading.",
                        value: Some(35.0),
                        max: 100.0,
                        gui_test_id: true,
                    }
                    StateItem {
                        slug: "complete",
                        scenario: "SW2A-1-complete",
                        title: "Complete",
                        note: "value >= max -> complete.",
                        value: Some(100.0),
                        max: 100.0,
                        gui_test_id: true,
                    }
                    StateItem {
                        slug: "indeterminate",
                        scenario: "SW2A-1-indeterminate",
                        title: "Explicitly indeterminate",
                        note: "max=80 kept; aria-valuenow omitted.",
                        value: None,
                        max: 80.0,
                        gui_test_id: false,
                    }
                }
            }

            div {
                id: "progress-reactive-validation",
                "data-scenario": "SW2A-2-validation",
                class: CARD_CLASS,
                div {
                    class: "space-y-1",
                    h3 { class: H3_CLASS, "2. Interactive progress and Radix normalization (SW2A-2)" }
                    p {
                        id: "progress-reactive-inputs",
                        class: NOTE_CLASS,
                        "Requested max: {reactive_max}; requested value: {requested_value}"
                    }
                }
                Progress {
                    id: "progress-reactive-root".to_string(),
                    value: reactive_value(),
                    max: reactive_max(),
                    Indicator {
                        id: "progress-reactive-indicator".to_string(),
                        style: indicator_style(reactive_value(), reactive_max()),
                    }
                }
                p {
                    id: "progress-reactive-normalized",
                    class: "text-xs font-mono text-slate-600",
                    "{progress_summary(reactive_value(), reactive_max())}"
                }

                div {
                    class: "space-y-2",
                    p { class: "text-xs font-semibold text-slate-700", "Normal controls" }
                    div {
                        class: "flex flex-wrap gap-2",
                        for pct in [0.0_f64, 35.0, 70.0, 100.0] {
                            button {
                                key: "{pct}",
                                r#type: "button",
                                class: BTN_CLASS,
                                onclick: move |_| {
                                    reactive_max.set(100.0);
                                    reactive_value.set(Some(pct));
                                },
                                "{pct}%"
                            }
                        }
                        button {
                            r#type: "button",
                            class: BTN_CLASS,
                            onclick: move |_| reactive_value.set(None),
                            "Indeterminate (None)"
                        }
                    }
                }

                div {
                    class: "space-y-2",
                    p { class: "text-xs font-semibold text-slate-700", "Radix normalization edge cases" }
                    div {
                        class: "flex flex-wrap gap-2",
                        button {
                            id: "progress-set-invalid-max",
                            "data-scenario": "invalid-max",
                            r#type: "button",
                            class: BTN_CLASS,
                            onclick: move |_| {
                                reactive_max.set(0.0);
                                reactive_value.set(Some(25.0));
                            },
                            "max=0, val=25"
                        }
                        button {
                            id: "progress-set-invalid-value",
                            "data-scenario": "invalid-value",
                            r#type: "button",
                            class: BTN_CLASS,
                            onclick: move |_| {
                                reactive_max.set(100.0);
                                reactive_value.set(Some(-1.0));
                            },
                            "val=-1"
                        }
                        button {
                            id: "progress-set-over-max-value",
                            "data-scenario": "over-max-value",
                            r#type: "button",
                            class: BTN_CLASS,
                            onclick: move |_| {
                                reactive_max.set(80.0);
                                reactive_value.set(Some(120.0));
                            },
                            "val=120 > max=80"
                        }
                        button {
                            id: "progress-set-nan-value",
                            "data-scenario": "nan-value",
                            r#type: "button",
                            class: BTN_CLASS,
                            onclick: move |_| {
                                reactive_max.set(100.0);
                                reactive_value.set(Some(f64::NAN));
                            },
                            "val=NaN"
                        }
                        button {
                            id: "progress-set-nan-max",
                            "data-scenario": "nan-max",
                            r#type: "button",
                            class: BTN_CLASS,
                            onclick: move |_| {
                                reactive_max.set(f64::NAN);
                                reactive_value.set(Some(40.0));
                            },
                            "max=NaN"
                        }
                        button {
                            id: "progress-restore-valid",
                            "data-scenario": "restore-valid",
                            r#type: "button",
                            class: BTN_CLASS,
                            onclick: move |_| {
                                reactive_max.set(80.0);
                                reactive_value.set(Some(40.0));
                            },
                            "Restore 40 / 80"
                        }
                    }
                }

                div {
                    class: "grid grid-cols-1 gap-4 sm:grid-cols-2",
                    div {
                        class: "rounded-xl border border-slate-100 bg-slate-50 p-3 space-y-2",
                        p { class: "text-xs font-semibold text-slate-700", "Requested props" }
                        div {
                            class: "flex flex-wrap gap-1.5",
                            span { class: "sw1a-pill-amber", "value={requested_value}" }
                            span { class: "sw1a-pill-amber", "max={reactive_max}" }
                        }
                        if rules.is_empty() {
                            p { class: NOTE_CLASS, "No normalization applied: props are valid." }
                        } else {
                            for rule in rules {
                                p { key: "{rule}", class: "text-xs text-amber-700", "{rule}" }
                            }
                        }
                    }
                    div {
                        class: "rounded-xl border border-slate-100 bg-slate-50 p-3 space-y-2",
                        p { class: "text-xs font-semibold text-slate-700", "Emitted root ARIA and synchronized indicator data" }
                        div {
                            class: "flex flex-wrap gap-1.5",
                            span { class: "sw1a-pill-purple", "role=\"progressbar\"" }
                            for chip in emitted {
                                span { key: "{chip}", class: "sw1a-pill-purple", "{chip}" }
                            }
                            span { class: "sw1a-pill-purple", "{indeterminate_flag}" }
                            span { class: "sw1a-badge-on", "{indicator_sync}" }
                        }
                    }
                }
            }

            div {
                id: "progress-custom-label",
                "data-scenario": "SW2A-2-custom-label",
                class: CARD_CLASS,
                div {
                    class: "space-y-1",
                    h3 { class: H3_CLASS, "3. Custom get_value_label (SW2A-2)" }
                    p { class: NOTE_CLASS, "get_value_label replaces the default percentage aria-valuetext." }
                }
                Progress {
                    id: "progress-custom-label-root".to_string(),
                    value: 32.0,
                    max: 80.0,
                    get_value_label: custom_value_label,
                    Indicator {
                        id: "progress-custom-label-indicator".to_string(),
                        style: indicator_style(Some(32.0), 80.0),
                    }
                }
                div {
                    class: "grid grid-cols-1 gap-3 sm:grid-cols-2",
                    div {
                        class: "space-y-1",
                        p { class: "text-xs font-semibold text-slate-700", "Default (Section 1 loading bar)" }
                        span { class: "sw1a-pill-slate", "aria-valuetext=\"40%\" (default label for 40 of 100)" }
                    }
                    div {
                        class: "space-y-1",
                        p { class: "text-xs font-semibold text-slate-700", "Custom get_value_label" }
                        span { class: "sw1a-pill-purple", "aria-valuetext=\"32 of 80 downloaded\"" }
                    }
                }
            }
        }
    }
}

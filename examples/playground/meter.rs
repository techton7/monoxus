use dioxus::prelude::*;
use monoxus::meter::MeterRoot;

struct MeterVisual {
    width_pct: f64,
    color: &'static str,
    opacity: f64,
    status: &'static str,
}

fn meter_visual_state(min: f64, value: f64, max: f64) -> MeterVisual {
    if !min.is_finite() || !value.is_finite() || !max.is_finite() {
        return MeterVisual {
            width_pct: 100.0,
            color: "#d97706",
            opacity: 0.55,
            status: "non-finite (raw ARIA/data preserved)",
        };
    }
    if max <= min {
        return MeterVisual {
            width_pct: 100.0,
            color: "#dc2626",
            opacity: 0.55,
            status: "inverted range (raw ARIA/data preserved)",
        };
    }
    let width_pct = (((value - min) / (max - min)) * 100.0).clamp(0.0, 100.0);
    if (min..=max).contains(&value) {
        MeterVisual {
            width_pct,
            color: "#0284c7",
            opacity: 1.0,
            status: "in-range",
        }
    } else {
        MeterVisual {
            width_pct,
            color: "#d97706",
            opacity: 1.0,
            status: "out-of-range (visual clamped, raw ARIA/data preserved)",
        }
    }
}

fn meter_fill_style(min: f64, value: f64, max: f64) -> String {
    let visual = meter_visual_state(min, value, max);
    format!(
        "height: 100%; width: {:.2}%; border-radius: inherit; background: {}; opacity: {:.2};",
        visual.width_pct, visual.color, visual.opacity
    )
}

fn meter_visual_summary(min: f64, value: f64, max: f64) -> String {
    let visual = meter_visual_state(min, value, max);
    format!(
        "Visual bar: {:.0}% · state: {}",
        visual.width_pct, visual.status
    )
}

#[component]
pub fn MeterPlayground() -> Element {
    let mut min = use_signal(|| 0.0_f64);
    let mut value = use_signal(|| 40.0_f64);
    let mut max = use_signal(|| 100.0_f64);

    rsx! {
        div {
            class: "max-w-4xl mx-auto space-y-8",

            div {
                class: "space-y-1.5",
                h2 {
                    class: "text-2xl font-bold tracking-tight text-slate-900",
                    "Meter"
                }
                p {
                    class: "text-sm text-slate-500",
                    "A transparent headless meter that publishes reactive min, value, and max values without clamping or validation."
                }
            }

            section {
                id: "meter-defaults-scenario",
                "data-scenario": "SW2A-3-defaults",
                class: "rounded-2xl border border-slate-200 bg-white p-6 shadow-sm space-y-4",
                div {
                    class: "space-y-1",
                    h3 {
                        class: "text-base font-semibold text-slate-900",
                        "1. Defaults (SW2A-3)"
                    }
                    p {
                        class: "text-xs text-slate-500",
                        "MeterRoot publishes min 0, value 0, and max 100 when no numeric props are supplied."
                    }
                }
                MeterRoot {
                    id: "meter-defaults".to_string(),
                    aria_label: Some("Default meter".to_string()),
                    class: Some("block h-4 overflow-hidden rounded bg-slate-100".to_string()),
                    div {
                        id: "meter-defaults-fill",
                        style: "height: 100%; width: 0.00%; border-radius: inherit; background: #0284c7;",
                    }
                }
                div {
                    class: "text-xs font-mono text-slate-600",
                    "Visual fill: 0% | Expected ARIA/data: role=meter, aria-valuemin=0, aria-valuenow=0, aria-valuemax=100"
                }
            }

            section {
                id: "meter-reactive-scenario",
                "data-scenario": "SW2A-3-reactive",
                class: "rounded-2xl border border-slate-200 bg-white p-6 shadow-sm space-y-5",
                div {
                    class: "space-y-1",
                    h3 {
                        class: "text-base font-semibold text-slate-900",
                        "2. Reactive Values and Range Edges (SW2A-3)"
                    }
                    p {
                        class: "text-xs text-slate-500",
                        "Controls update the emitted ARIA and data attributes directly, including out-of-range, inverted, and non-finite values."
                    }
                }

                MeterRoot {
                    id: "meter-reactive".to_string(),
                    min: min(),
                    value: value(),
                    max: max(),
                    aria_label: Some("Reactive meter".to_string()),
                    aria_valuetext: Some(format!("{} between {} and {}", value(), min(), max())),
                    class: Some("block h-4 overflow-hidden rounded bg-slate-100".to_string()),
                    div {
                        id: "meter-reactive-fill",
                        style: meter_fill_style(min(), value(), max()),
                    }
                }

                div {
                    class: "grid grid-cols-1 gap-4 sm:grid-cols-3",
                    label {
                        class: "space-y-1 text-xs font-medium text-slate-700",
                        "Minimum"
                        input {
                            id: "meter-min-control",
                            r#type: "number",
                            step: "any",
                            value: "{min()}",
                            class: "block w-full rounded-md border border-slate-300 px-3 py-2 text-sm",
                            oninput: move |event| {
                                if let Ok(next) = event.value().parse::<f64>() {
                                    min.set(next);
                                }
                            },
                        }
                    }
                    label {
                        class: "space-y-1 text-xs font-medium text-slate-700",
                        "Current value"
                        input {
                            id: "meter-value-control",
                            r#type: "number",
                            step: "any",
                            value: "{value()}",
                            class: "block w-full rounded-md border border-slate-300 px-3 py-2 text-sm",
                            oninput: move |event| {
                                if let Ok(next) = event.value().parse::<f64>() {
                                    value.set(next);
                                }
                            },
                        }
                    }
                    label {
                        class: "space-y-1 text-xs font-medium text-slate-700",
                        "Maximum"
                        input {
                            id: "meter-max-control",
                            r#type: "number",
                            step: "any",
                            value: "{max()}",
                            class: "block w-full rounded-md border border-slate-300 px-3 py-2 text-sm",
                            oninput: move |event| {
                                if let Ok(next) = event.value().parse::<f64>() {
                                    max.set(next);
                                }
                            },
                        }
                    }
                }

                div {
                    class: "flex flex-wrap gap-3",
                    button {
                        id: "meter-preset-valid",
                        r#type: "button",
                        class: "sw1a-btn-secondary text-xs",
                        onclick: move |_| {
                            min.set(0.0);
                            value.set(65.0);
                            max.set(100.0);
                        },
                        "Valid 0 / 65 / 100"
                    }
                    button {
                        id: "meter-preset-out-of-range",
                        r#type: "button",
                        class: "sw1a-btn-secondary text-xs",
                        onclick: move |_| {
                            min.set(20.0);
                            value.set(150.0);
                            max.set(100.0);
                        },
                        "Out of range 20 / 150 / 100"
                    }
                    button {
                        id: "meter-preset-inverted",
                        r#type: "button",
                        class: "sw1a-btn-secondary text-xs",
                        onclick: move |_| {
                            min.set(100.0);
                            value.set(50.0);
                            max.set(0.0);
                        },
                        "Inverted 100 / 50 / 0"
                    }
                    button {
                        id: "meter-preset-nonfinite",
                        r#type: "button",
                        class: "sw1a-btn-secondary text-xs",
                        onclick: move |_| {
                            min.set(f64::NEG_INFINITY);
                            value.set(f64::NAN);
                            max.set(f64::INFINITY);
                        },
                        "Non-finite -Infinity / NaN / Infinity"
                    }
                }

                p {
                    id: "meter-visual-state",
                    class: "text-xs font-mono text-slate-600",
                    "{meter_visual_summary(min(), value(), max())}"
                }

                output {
                    id: "meter-reactive-values",
                    class: "block rounded bg-slate-100 px-3 py-2 text-xs font-mono text-slate-700",
                    "min={min()} value={value()} max={max()}"
                }
            }
        }
    }
}

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

fn fmt(value: f64) -> String {
    format!("{value}")
}

fn usage_color(used: f64, total: f64) -> &'static str {
    let remaining = (total - used) / total * 100.0;
    if remaining > 40.0 {
        "#16a34a"
    } else if remaining > 15.0 {
        "#d97706"
    } else {
        "#dc2626"
    }
}

const CARD: &str = "rounded-2xl border border-slate-200 bg-white p-3 shadow-sm space-y-2";
const H3: &str = "text-base font-semibold text-slate-900";
const NOTE: &str = "text-xs text-slate-500";
const INPUT: &str = "block w-full rounded-md border border-slate-300 px-2 py-1 text-sm min-h-9";
const BTN: &str = "sw1a-btn-secondary text-xs";
const BTN_STYLE: &str = "padding: 0.25rem 0.5rem; font-size: 11px;";

#[component]
pub fn MeterPlayground() -> Element {
    let mut min = use_signal(|| 0.0_f64);
    let mut value = use_signal(|| 40.0_f64);
    let mut max = use_signal(|| 100.0_f64);
    let mut tokens = use_signal(|| 2000.0_f64);

    let (mn, v, mx) = (min(), value(), max());
    let tok = tokens();
    let tok_color = usage_color(tok, 4000.0);
    let tok_width = (tok / 4000.0 * 100.0).clamp(0.0, 100.0);

    rsx! {
        div {
            class: "max-w-4xl mx-auto space-y-4",

            div {
                class: "space-y-0.5",
                h2 {
                    class: "text-xl font-bold tracking-tight text-slate-900",
                    "Meter"
                }
                p {
                    class: "text-xs text-slate-500",
                    "Scalar range: raw min / value / max pass through, unlike Progress."
                }
            }

            section {
                id: "meter-defaults-scenario",
                "data-scenario": "SW2A-3-defaults",
                class: CARD,
                div {
                    class: "space-y-1",
                    h3 { class: H3, "1. Defaults (SW2A-3)" }
                }
                MeterRoot {
                    id: "meter-defaults".to_string(),
                    aria_label: Some("Default meter".to_string()),
                    class: Some("block h-4 overflow-hidden rounded bg-slate-100".to_string()),
                    "data-gui-test-id": "meter-defaults",
                    div {
                        id: "meter-defaults-fill",
                        style: "height: 100%; width: 0.00%; border-radius: inherit; background: #0284c7;",
                    }
                }
                div {
                    class: "flex flex-wrap gap-1",
                    for pill in ["role=\"meter\"", "aria-valuemin=\"0\"", "aria-valuenow=\"0\"", "aria-valuemax=\"100\"", "data-meter-root"] {
                        span {
                            key: "{pill}",
                            class: "sw1a-pill-purple",
                            style: "font-size: 10px; line-height: 1.2; padding: 0.0625rem 0.3rem;",
                            "{pill}"
                        }
                    }
                }
            }

            section {
                id: "meter-reactive-scenario",
                "data-scenario": "SW2A-3-reactive",
                "data-gui-test-id": "meter-reactive-scenario",
                class: CARD,
                div {
                    class: "space-y-1",
                    h3 { class: H3, "2. Reactive values and pass-through contract (SW2A-3)" }
                }

                MeterRoot {
                    id: "meter-reactive".to_string(),
                    min: mn,
                    value: v,
                    max: mx,
                    aria_label: Some("Reactive meter".to_string()),
                    aria_valuetext: Some(format!("{} between {} and {}", v, mn, mx)),
                    class: Some("block h-4 overflow-hidden rounded bg-slate-100".to_string()),
                    "data-gui-test-id": "meter-reactive",
                    div {
                        id: "meter-reactive-fill",
                        style: meter_fill_style(mn, v, mx),
                    }
                }

                div {
                    class: "grid grid-cols-3 gap-2",
                    label {
                        class: "space-y-1 text-xs font-medium text-slate-700",
                        "Minimum"
                        input {
                            id: "meter-min-control",
                            r#type: "number",
                            step: "any",
                            value: "{mn}",
                            class: INPUT,
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
                            value: "{v}",
                            class: INPUT,
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
                            value: "{mx}",
                            class: INPUT,
                            oninput: move |event| {
                                if let Ok(next) = event.value().parse::<f64>() {
                                    max.set(next);
                                }
                            },
                        }
                    }
                }

                div {
                    class: "space-y-2",
                    div {
                        class: "flex flex-wrap gap-2",
                        button {
                            id: "meter-preset-valid",
                            r#type: "button",
                            class: BTN,
                            style: BTN_STYLE,
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
                            class: BTN,
                            style: BTN_STYLE,
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
                            class: BTN,
                            style: BTN_STYLE,
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
                            class: BTN,
                            style: BTN_STYLE,
                            onclick: move |_| {
                                min.set(f64::NEG_INFINITY);
                                value.set(f64::NAN);
                                max.set(f64::INFINITY);
                            },
                            "Non-finite -inf / NaN / inf"
                        }
                    }
                }

            }

            section {
                id: "meter-contract-scenario",
                "data-scenario": "SW2A-3-contract",
                class: CARD,
                div {
                    class: "space-y-1",
                    h3 { class: H3, "Pass-through contract and ownership" }
                    p { class: NOTE, "Valid presets stay in-range; the edge-case presets prove no clamping or validation." }
                    p { class: NOTE, "Out of range: value 150 exceeds max and is emitted as-is. Inverted: max < min is emitted as-is. Non-finite: -inf / NaN / inf are the Rust f64 stringification, emitted as-is." }
                }
                div {
                    class: "grid grid-cols-1 gap-4 sm:grid-cols-2",
                    div {
                        class: "rounded-xl border border-slate-100 bg-slate-50 p-3 space-y-2",
                        p { class: "text-xs font-semibold text-slate-700", "Library-owned DOM contract (monoxus::meter)" }
                        div {
                            class: "flex flex-wrap gap-1.5",
                            span { class: "sw1a-pill-purple", "role=\"meter\"" }
                            span { class: "sw1a-pill-purple", "aria-valuemin=\"{mn}\"" }
                            span { class: "sw1a-pill-purple", "aria-valuenow=\"{v}\"" }
                            span { class: "sw1a-pill-purple", "aria-valuemax=\"{mx}\"" }
                            span { class: "sw1a-pill-slate", "data-min=\"{mn}\"" }
                            span { class: "sw1a-pill-slate", "data-value=\"{v}\"" }
                            span { class: "sw1a-pill-slate", "data-max=\"{mx}\"" }
                            span { class: "sw1a-pill-slate", "aria-valuetext=\"{v} between {mn} and {mx}\"" }
                        }
                        output {
                            id: "meter-reactive-values",
                            class: "block rounded bg-white px-3 py-2 text-xs font-mono text-slate-700",
                            "min={mn} value={v} max={mx}"
                        }
                    }
                    div {
                        class: "rounded-xl border border-slate-100 bg-slate-50 p-3 space-y-2",
                        p { class: "text-xs font-semibold text-slate-700", "Consumer-owned visual bar (playground example)" }
                        p { class: NOTE, "Fill width and blue / amber / red colors are consumer styling, not built into monoxus::meter." }
                        p {
                            id: "meter-visual-state",
                            class: "text-xs font-mono text-slate-600",
                            "{meter_visual_summary(mn, v, mx)}"
                        }
                    }
                }
            }

            section {
                id: "meter-usage-scenario",
                "data-scenario": "SW2A-3-usage",
                class: CARD,
                div {
                    class: "space-y-1",
                    h3 { class: H3, "3. Consumer-styled usage example" }
                }
                div {
                    class: "rounded-xl border border-slate-100 bg-slate-50 p-4 space-y-3",
                    div {
                        class: "space-y-0.5",
                        p { class: "text-sm font-semibold text-slate-900", "Tokens used: {fmt(tok)} / 4000" }
                        p { class: NOTE, "Consumer-styled usage example: green / amber / red is chosen by consumer code from the remaining percentage." }
                    }
                    MeterRoot {
                        id: "meter-tokens".to_string(),
                        min: 0.0,
                        value: tok,
                        max: 4000.0,
                        aria_label: Some("Tokens used".to_string()),
                        class: Some("block h-3 overflow-hidden rounded-full bg-slate-200".to_string()),
                        div {
                            id: "meter-tokens-fill",
                            style: "height: 100%; width: {tok_width:.2}%; border-radius: inherit; background: {tok_color};",
                        }
                    }
                    div {
                        class: "flex flex-wrap gap-2",
                        for step in [1200.0_f64, 2400.0, 3200.0, 3700.0] {
                            button {
                                key: "{step}",
                                r#type: "button",
                                class: BTN,
                                onclick: move |_| tokens.set(step),
                                "{fmt(step)}"
                            }
                        }
                    }
                }
            }
        }
    }
}

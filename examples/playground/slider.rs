use dioxus::prelude::*;
use monoxus::foundation::shared::{Direction, DirectionProvider, Orientation};
use monoxus::slider::*;

#[component]
pub fn SliderPlayground() -> Element {
    rsx! {
        div {
            class: "max-w-4xl mx-auto space-y-8",

            // Header Section
            div {
                class: "space-y-1.5",
                h2 {
                    class: "text-2xl font-bold tracking-tight text-slate-900",
                    "Slider"
                }
                p {
                    class: "text-sm text-slate-500",
                    "Headless interactive slider control with full keyboard navigation, continuous and discrete stepping, multi-thumb Swap/Preserve constraints, RTL orientation, and form reset integration."
                }
            }

            BasicSliderScenario {}
            MultiThumbSliderScenario {}
            RtlAndStepSliderScenario {}
            FormSliderScenario {}
        }
    }
}

// ---------------------------------------------------------------------------
// Scenario 1: Single Horizontal Slider (SW2B-3, SW2B-4)
// ---------------------------------------------------------------------------
#[component]
pub fn BasicSliderScenario() -> Element {
    let mut live_val = use_signal(|| 50.0f64);
    let mut committed_val = use_signal(|| 50.0f64);
    let mut commit_count = use_signal(|| 0usize);

    rsx! {
        div {
            class: "rounded-2xl border border-slate-200 bg-white p-6 shadow-sm space-y-4",
            div {
                class: "space-y-1",
                h3 { class: "text-base font-semibold text-slate-900", "1. Basic Horizontal Slider & Commit Events" }
                p { class: "text-xs text-slate-500",
                    "Standard single-thumb horizontal slider tracking live updates and commit events."
                }
            }

            div {
                class: "py-4",
                SliderRoot {
                    id: "sw2b-slider-basic",
                    min: 0.0,
                    max: 100.0,
                    step: 1.0,
                    default_value: SliderValue::Single(50.0),
                    style: "width: 300px; height: 24px; display: flex; align-items: center;",
                    on_value_change: move |v| {
                        if let SliderValue::Single(n) = v {
                            live_val.set(n);
                        }
                    },
                    on_value_commit: move |v| {
                        if let SliderValue::Single(n) = v {
                            committed_val.set(n);
                            commit_count += 1;
                        }
                    },
                    SliderTrack {
                        id: "sw2b-slider-basic-track",
                        class: "sw2b-slider-track",
                        style: "width: 300px; height: 8px; background: #e2e8f0; border-radius: 9999px; position: relative;",
                        SliderRange {
                            id: "sw2b-slider-basic-range",
                            class: "sw2b-slider-range",
                            style: "background: #7c3aed; border-radius: 9999px;",
                        }
                    }
                    SliderThumb {
                        id: "sw2b-slider-basic-thumb-0",
                        index: 0,
                        class: "sw2b-slider-thumb",
                        aria_label: "Master Volume".to_string(),
                    }
                }
            }

            div {
                class: "pt-2 flex flex-wrap items-center gap-4 text-xs font-mono",
                div {
                    class: "flex items-center gap-1.5",
                    span { class: "text-slate-400 font-semibold", "Live Value:" }
                    span {
                        id: "sw2b-slider-basic-value",
                        class: "inline-flex items-center rounded px-2 py-0.5 bg-slate-100 text-slate-700 font-bold",
                        "{live_val()}"
                    }
                }
                div {
                    class: "flex items-center gap-1.5",
                    span { class: "text-slate-400 font-semibold", "Commit Count:" }
                    span {
                        id: "sw2b-slider-basic-commit-count",
                        class: "inline-flex items-center rounded px-2 py-0.5 bg-purple-50 text-purple-700 font-bold",
                        "{commit_count()}"
                    }
                }
                div {
                    class: "flex items-center gap-1.5",
                    span { class: "text-slate-400 font-semibold", "Committed Value:" }
                    span {
                        id: "sw2b-slider-basic-committed",
                        class: "inline-flex items-center rounded px-2 py-0.5 bg-slate-100 text-slate-700 font-bold",
                        "{committed_val()}"
                    }
                }
            }
        }
    }
}

// ---------------------------------------------------------------------------
// Scenario 2: Multi-Thumb Swap vs. Preserve (SW2B-5)
// ---------------------------------------------------------------------------
#[component]
pub fn MultiThumbSliderScenario() -> Element {
    let mut swap_vals = use_signal(|| vec![25.0f64, 75.0f64]);
    let mut preserve_vals = use_signal(|| vec![30.0f64, 70.0f64]);
    let swap_label = format!("{:?}", swap_vals());
    let preserve_label = format!("{:?}", preserve_vals());

    rsx! {
        div {
            class: "rounded-2xl border border-slate-200 bg-white p-6 shadow-sm space-y-6",
            div {
                class: "space-y-1",
                h3 { class: "text-base font-semibold text-slate-900", "2. Multi-Thumb Range: Swap & Preserve Policies" }
                p { class: "text-xs text-slate-500",
                    "Comparing thumbs crossover behavior (Swap order policy) with distance-clamped thumbs (Preserve policy with min_steps_between_thumbs: 10)."
                }
            }

            // Swap Policy Card
            div {
                class: "space-y-3 p-4 rounded-xl border border-slate-100 bg-slate-50",
                h4 { class: "text-xs font-bold uppercase tracking-wider text-slate-500", "Swap Policy (Thumbs Cross Over & Reorder)" }
                SliderRoot {
                    id: "sw2b-slider-swap",
                    default_value: SliderValue::Multiple(vec![25.0, 75.0]),
                    order_policy: SliderThumbOrderPolicy::Swap,
                    style: "width: 300px; height: 24px; display: flex; align-items: center;",
                    on_value_change: move |v| {
                        if let SliderValue::Multiple(vals) = v {
                            swap_vals.set(vals);
                        }
                    },
                    SliderTrack {
                        id: "sw2b-slider-swap-track",
                        class: "sw2b-slider-track",
                        style: "width: 300px; height: 8px; background: #e2e8f0; border-radius: 9999px; position: relative;",
                        SliderRange {
                            id: "sw2b-slider-swap-range",
                            class: "sw2b-slider-range",
                            style: "background: #7c3aed; border-radius: 9999px;",
                        }
                    }
                    SliderThumb {
                        id: "sw2b-slider-swap-thumb-0",
                        index: 0,
                        class: "sw2b-slider-thumb",
                        aria_label: "Min Range".to_string(),
                    }
                    SliderThumb {
                        id: "sw2b-slider-swap-thumb-1",
                        index: 1,
                        class: "sw2b-slider-thumb",
                        aria_label: "Max Range".to_string(),
                    }
                }
                div {
                    class: "flex items-center gap-2 text-xs font-mono",
                    span { class: "text-slate-400 font-semibold", "Values:" }
                    span {
                        id: "sw2b-slider-swap-value",
                        class: "inline-flex items-center rounded px-2 py-0.5 bg-white border border-slate-200 text-slate-700 font-bold",
                        "{swap_label}"
                    }
                }
            }

            // Preserve Policy Card
            div {
                class: "space-y-3 p-4 rounded-xl border border-slate-100 bg-slate-50",
                h4 { class: "text-xs font-bold uppercase tracking-wider text-slate-500", "Preserve Policy (Distance Clamped, min_steps: 10)" }
                SliderRoot {
                    id: "sw2b-slider-preserve",
                    default_value: SliderValue::Multiple(vec![30.0, 70.0]),
                    min_steps_between_thumbs: 10,
                    order_policy: SliderThumbOrderPolicy::Preserve,
                    style: "width: 300px; height: 24px; display: flex; align-items: center;",
                    on_value_change: move |v| {
                        if let SliderValue::Multiple(vals) = v {
                            preserve_vals.set(vals);
                        }
                    },
                    SliderTrack {
                        id: "sw2b-slider-preserve-track",
                        class: "sw2b-slider-track",
                        style: "width: 300px; height: 8px; background: #e2e8f0; border-radius: 9999px; position: relative;",
                        SliderRange {
                            id: "sw2b-slider-preserve-range",
                            class: "sw2b-slider-range",
                            style: "background: #7c3aed; border-radius: 9999px;",
                        }
                    }
                    SliderThumb {
                        id: "sw2b-slider-preserve-thumb-0",
                        index: 0,
                        class: "sw2b-slider-thumb",
                        aria_label: "Lower Bound".to_string(),
                    }
                    SliderThumb {
                        id: "sw2b-slider-preserve-thumb-1",
                        index: 1,
                        class: "sw2b-slider-thumb",
                        aria_label: "Upper Bound".to_string(),
                    }
                }
                div {
                    class: "flex items-center gap-2 text-xs font-mono",
                    span { class: "text-slate-400 font-semibold", "Values:" }
                    span {
                        id: "sw2b-slider-preserve-value",
                        class: "inline-flex items-center rounded px-2 py-0.5 bg-white border border-slate-200 text-slate-700 font-bold",
                        "{preserve_label}"
                    }
                }
            }
        }
    }
}

// ---------------------------------------------------------------------------
// Scenario 3: RTL, Discrete Steps, Vertical & Disabled (SW2B-6)
// ---------------------------------------------------------------------------
#[component]
pub fn RtlAndStepSliderScenario() -> Element {
    let mut rtl_val = use_signal(|| 40.0f64);
    let mut custom_val = use_signal(|| 25.0f64);
    let mut vert_val = use_signal(|| 60.0f64);

    rsx! {
        div {
            class: "rounded-2xl border border-slate-200 bg-white p-6 shadow-sm space-y-6",
            div {
                class: "space-y-1",
                h3 { class: "text-base font-semibold text-slate-900", "3. RTL Direction, Discrete Custom Steps, Vertical & Disabled" }
                p { class: "text-xs text-slate-500",
                    "DirectionProvider RTL slider positioning, discrete step snapping with ticks and labels, vertical slider orientation, and disabled state."
                }
            }

            // RTL Slider
            div {
                class: "space-y-3 p-4 rounded-xl border border-slate-100 bg-slate-50",
                h4 { class: "text-xs font-bold uppercase tracking-wider text-slate-500", "Right-to-Left (RTL) Context" }
                DirectionProvider {
                    dir: Direction::Rtl,
                    SliderRoot {
                        id: "sw2b-slider-rtl",
                        default_value: SliderValue::Single(40.0),
                        style: "width: 300px; height: 24px; display: flex; align-items: center;",
                        on_value_change: move |v| {
                            if let SliderValue::Single(n) = v {
                                rtl_val.set(n);
                            }
                        },
                        SliderTrack {
                            id: "sw2b-slider-rtl-track",
                            class: "sw2b-slider-track",
                            style: "width: 300px; height: 8px; background: #e2e8f0; border-radius: 9999px; position: relative;",
                            SliderRange {
                                class: "sw2b-slider-range",
                                style: "background: #7c3aed; border-radius: 9999px;",
                            }
                        }
                        SliderThumb {
                            id: "sw2b-slider-rtl-thumb-0",
                            index: 0,
                            class: "sw2b-slider-thumb",
                            aria_label: "RTL Level".to_string(),
                        }
                    }
                }
                div {
                    class: "flex items-center gap-2 text-xs font-mono",
                    span { class: "text-slate-400 font-semibold", "RTL Value:" }
                    span {
                        id: "sw2b-slider-rtl-value",
                        class: "inline-flex items-center rounded px-2 py-0.5 bg-white border border-slate-200 text-slate-700 font-bold",
                        "{rtl_val()}"
                    }
                }
            }

            // Discrete Custom Steps Slider
            div {
                class: "space-y-4 p-4 rounded-xl border border-slate-100 bg-slate-50",
                h4 { class: "text-xs font-bold uppercase tracking-wider text-slate-500", "Discrete Custom Steps [0, 10, 25, 50, 75, 100]" }
                div {
                    class: "py-6",
                    SliderRoot {
                        id: "sw2b-slider-custom",
                        steps: vec![0.0, 10.0, 25.0, 50.0, 75.0, 100.0],
                        default_value: SliderValue::Single(25.0),
                        style: "width: 300px; height: 48px; display: flex; align-items: center; position: relative;",
                        on_value_change: move |v| {
                            if let SliderValue::Single(n) = v {
                                custom_val.set(n);
                            }
                        },
                        SliderTrack {
                            id: "sw2b-slider-custom-track",
                            class: "sw2b-slider-track",
                            style: "width: 300px; height: 8px; background: #e2e8f0; border-radius: 9999px; position: relative;",
                            SliderRange {
                                class: "sw2b-slider-range",
                                style: "background: #7c3aed; border-radius: 9999px;",
                            }
                        }
                        SliderTick { value: 0.0, style: "width: 2px; height: 12px; background: #94a3b8; pointer-events: none;" }
                        SliderTick { value: 10.0, style: "width: 2px; height: 12px; background: #94a3b8; pointer-events: none;" }
                        SliderTick { value: 25.0, style: "width: 2px; height: 12px; background: #94a3b8; pointer-events: none;" }
                        SliderTick { value: 50.0, style: "width: 2px; height: 12px; background: #94a3b8; pointer-events: none;" }
                        SliderTick { value: 75.0, style: "width: 2px; height: 12px; background: #94a3b8; pointer-events: none;" }
                        SliderTick { value: 100.0, style: "width: 2px; height: 12px; background: #94a3b8; pointer-events: none;" }
                        SliderTickLabel { value: 0.0, style: "font-size: 10px; color: #64748b; font-weight: 500; pointer-events: none; margin-top: 14px;", "0%" }
                        SliderTickLabel { value: 50.0, style: "font-size: 10px; color: #64748b; font-weight: 500; pointer-events: none; margin-top: 14px;", "50%" }
                        SliderTickLabel { value: 100.0, style: "font-size: 10px; color: #64748b; font-weight: 500; pointer-events: none; margin-top: 14px;", "100%" }
                        SliderThumb {
                            id: "sw2b-slider-custom-thumb-0",
                            index: 0,
                            class: "sw2b-slider-thumb",
                            aria_label: "Custom Step".to_string(),
                        }
                        SliderThumbLabel {
                            index: 0,
                            style: "background: #1e293b; color: #ffffff; font-size: 10px; font-weight: 600; padding: 2px 6px; border-radius: 4px; pointer-events: none; white-space: nowrap; margin-bottom: 8px;",
                            "{custom_val()}%"
                        }
                    }
                }
            }

            // Vertical & Disabled Grid
            div {
                class: "grid grid-cols-1 md:grid-cols-2 gap-4",

                // Vertical Slider Card
                div {
                    class: "space-y-3 p-4 rounded-xl border border-slate-100 bg-slate-50",
                    h4 { class: "text-xs font-bold uppercase tracking-wider text-slate-500", "Vertical Slider" }
                    div {
                        class: "h-48 flex items-center justify-center",
                        SliderRoot {
                            id: "sw2b-slider-vertical",
                            orientation: Orientation::Vertical,
                            default_value: SliderValue::Single(60.0),
                            style: "height: 160px; width: 28px; display: flex; flex-direction: column; align-items: center;",
                            on_value_change: move |v| {
                                if let SliderValue::Single(n) = v {
                                    vert_val.set(n);
                                }
                            },
                            SliderTrack {
                                id: "sw2b-slider-vertical-track",
                                class: "sw2b-slider-track",
                                style: "width: 8px; height: 160px; flex-grow: 0; background: #e2e8f0; border-radius: 9999px; position: relative;",
                                SliderRange {
                                    class: "sw2b-slider-range",
                                    style: "background: #7c3aed; border-radius: 9999px;",
                                }
                            }
                            SliderThumb {
                                id: "sw2b-slider-vertical-thumb-0",
                                index: 0,
                                class: "sw2b-slider-thumb",
                                aria_label: "Vertical Value".to_string(),
                            }
                        }
                    }
                }

                // Disabled Slider Card
                div {
                    class: "space-y-3 p-4 rounded-xl border border-slate-100 bg-slate-50",
                    h4 { class: "text-xs font-bold uppercase tracking-wider text-slate-500", "Disabled Slider" }
                    div {
                        class: "h-48 flex items-center justify-center",
                        SliderRoot {
                            id: "sw2b-slider-disabled",
                            disabled: true,
                            default_value: SliderValue::Single(35.0),
                            style: "width: 240px; height: 24px; display: flex; align-items: center;",
                            SliderTrack {
                                id: "sw2b-slider-disabled-track",
                                class: "sw2b-slider-track",
                                style: "width: 240px; height: 8px; background: #f1f5f9; border-radius: 9999px; position: relative;",
                                SliderRange {
                                    class: "sw2b-slider-range",
                                    style: "background: #94a3b8; border-radius: 9999px;",
                                }
                            }
                            SliderThumb {
                                id: "sw2b-slider-disabled-thumb-0",
                                index: 0,
                                class: "sw2b-slider-thumb",
                                aria_label: "Disabled Slider".to_string(),
                                style: "background: #f8fafc;",
                            }
                        }
                    }
                }
            }
        }
    }
}

// ---------------------------------------------------------------------------
// Scenario 4: Form Integration & Native Reset (SW2B-7)
// ---------------------------------------------------------------------------
#[component]
pub fn FormSliderScenario() -> Element {
    let mut form_val = use_signal(|| 20.0f64);

    rsx! {
        div {
            class: "rounded-2xl border border-slate-200 bg-white p-6 shadow-sm space-y-4",
            div {
                class: "space-y-1",
                h3 { class: "text-base font-semibold text-slate-900", "4. HTML Form Integration & Native Reset" }
                p { class: "text-xs text-slate-500",
                    "Hidden inputs with named form control syncing and native HTML form reset restoring initial value 20."
                }
            }

            form {
                id: "sw2b-slider-form",
                onsubmit: move |e| e.prevent_default(),
                class: "space-y-4",

                SliderRoot {
                    id: "sw2b-slider-form-root",
                    name: "volume".to_string(),
                    default_value: SliderValue::Single(20.0),
                    style: "width: 300px; height: 24px; display: flex; align-items: center;",
                    on_value_change: move |v| {
                        if let SliderValue::Single(n) = v {
                            form_val.set(n);
                        }
                    },
                    SliderTrack {
                        id: "sw2b-slider-form-track",
                        class: "sw2b-slider-track",
                        style: "width: 300px; height: 8px; background: #e2e8f0; border-radius: 9999px; position: relative;",
                        SliderRange {
                            class: "sw2b-slider-range",
                            style: "background: #7c3aed; border-radius: 9999px;",
                        }
                    }
                    SliderThumb {
                        id: "sw2b-slider-form-thumb-0",
                        index: 0,
                        class: "sw2b-slider-thumb",
                        aria_label: "Form Volume".to_string(),
                    }
                }

                div {
                    class: "flex items-center gap-4 pt-2",
                    button {
                        id: "sw2b-slider-reset",
                        r#type: "reset",
                        class: "sw1a-btn-secondary",
                        "Reset Form"
                    }
                    div {
                        class: "flex items-center gap-1.5 text-xs font-mono",
                        span { class: "text-slate-400 font-semibold", "Form Value:" }
                        span {
                            id: "sw2b-slider-form-value",
                            class: "inline-flex items-center rounded px-2 py-0.5 bg-slate-100 text-slate-700 font-bold",
                            "{form_val()}"
                        }
                    }
                }
            }
        }
    }
}

use dioxus::prelude::*;
use monoxus::foundation::form::collect_form_data;
use monoxus::rating_group::{
    Direction, Orientation, RatingGroupItem, RatingGroupItemState, RatingGroupRoot,
    format_rating_value, use_rating_group_item_context,
};

const STAR_FULL_PATH: &str =
    "M12 2l3.09 6.26L22 9.27l-5 4.87 1.18 6.88L12 17.77l-6.18 3.25L7 14.14 2 9.27l6.91-1.01L12 2z";
const STAR_LEFT_HALF_PATH: &str = "M12 2L8.91 8.26L2 9.27L7 14.14L5.82 21.02L12 17.77Z";
const STAR_RIGHT_HALF_PATH: &str = "M12 2L15.09 8.26L22 9.27L17 14.14L18.18 21.02L12 17.77Z";

const ITEM_BASE_STYLE: &str = "width: 40px; height: 40px; box-sizing: border-box; display: inline-flex; align-items: center; justify-content: center; flex-shrink: 0; border-radius: 8px; border: 1px solid #e2e8f0; background-color: #f8fafc; cursor: pointer; user-select: none;";
const ITEM_DISABLED_STYLE: &str = "width: 40px; height: 40px; box-sizing: border-box; display: inline-flex; align-items: center; justify-content: center; flex-shrink: 0; border-radius: 8px; border: 1px solid #e2e8f0; background-color: #f1f5f9; cursor: not-allowed; opacity: 0.65; user-select: none;";

#[component]
fn RatingStarVisual(#[props(default = false)] rtl: bool) -> Element {
    let item_ctx = use_rating_group_item_context();
    let state = item_ctx.state();

    match state {
        RatingGroupItemState::Active => rsx! {
            svg {
                width: "24",
                height: "24",
                view_box: "0 0 24 24",
                "aria-hidden": "true",
                style: "width: 24px; height: 24px; display: block; pointer-events: none;",
                path {
                    d: STAR_FULL_PATH,
                    fill: "#f59e0b",
                    stroke: "#d97706",
                    "stroke-width": "1.25",
                }
            }
        },
        RatingGroupItemState::Partial => {
            let half_path = if rtl {
                STAR_RIGHT_HALF_PATH
            } else {
                STAR_LEFT_HALF_PATH
            };
            rsx! {
                svg {
                    width: "24",
                    height: "24",
                    view_box: "0 0 24 24",
                    "aria-hidden": "true",
                    style: "width: 24px; height: 24px; display: block; pointer-events: none;",
                    path {
                        d: STAR_FULL_PATH,
                        fill: "#e2e8f0",
                        stroke: "#cbd5e1",
                        "stroke-width": "1.25",
                    }
                    path {
                        d: half_path,
                        fill: "#f59e0b",
                        stroke: "#d97706",
                        "stroke-width": "1.25",
                    }
                }
            }
        }
        RatingGroupItemState::Inactive => rsx! {
            svg {
                width: "24",
                height: "24",
                view_box: "0 0 24 24",
                "aria-hidden": "true",
                style: "width: 24px; height: 24px; display: block; pointer-events: none;",
                path {
                    d: STAR_FULL_PATH,
                    fill: "#e2e8f0",
                    stroke: "#94a3b8",
                    "stroke-width": "1.25",
                }
            }
        },
    }
}

#[component]
pub fn RatingGroupPlayground() -> Element {
    let mut half_val = use_signal(|| 1.0_f64);
    let mut rtl_val = use_signal(|| 2.0_f64);
    let mut vertical_val = use_signal(|| 1.5_f64);
    let mut form_val = use_signal(|| 2.0_f64);
    let mut submitted_score = use_signal(|| String::from("not submitted"));

    rsx! {
        div {
            class: "max-w-4xl mx-auto space-y-8",

            // Header Section
            div {
                class: "space-y-1.5",
                h2 {
                    class: "text-2xl font-bold tracking-tight text-slate-900",
                    "RatingGroup"
                }
                p {
                    class: "text-sm text-slate-500",
                    "WAI-ARIA role=\"slider\" rating control supporting whole and half-star pointer math, hover preview, click-to-clear, RTL & vertical axes, keyboard/decimal input, and form reset sync."
                }
            }

            // Scenario 1 (SW5A-4, SW5A-5, SW5A-6, SW5A-8): Half-Star Interactive Horizontal RatingGroup
            div {
                class: "rounded-2xl border border-slate-200 bg-white p-6 shadow-sm space-y-4",
                div {
                    class: "space-y-1",
                    h3 {
                        class: "text-base font-semibold text-slate-900",
                        "1. Half-Star Pointer Math, Hover Preview, Click-to-Clear & Keyboard (SW5A-4, SW5A-5, SW5A-6, SW5A-8)"
                    }
                    p {
                        class: "text-xs text-slate-500",
                        "40x40px items (#sw5a-rg-half-item-0..4) with allow_half: true, hover_preview: true, min: 0.0, max: 5, default_value: 1.0. Supports Arrow/PageUp/PageDown/Home/End and decimal typing (e.g. '2', '.', '5')."
                    }
                }

                div {
                    style: "display: flex; flex-wrap: wrap; align-items: center; gap: 16px;",
                    RatingGroupRoot {
                        id: "sw5a-rg-half",
                        min: 0.0,
                        max: 5,
                        allow_half: true,
                        hover_preview: true,
                        default_value: 1.0,
                        aria_label: "Half-star rating",
                        style: "display: inline-flex; flex-direction: row; align-items: center; gap: 4px; padding: 6px; border-radius: 12px; border: 1px solid #e2e8f0; background-color: #ffffff; outline: none;",
                        on_value_change: move |v| half_val.set(v),
                        for idx in 0..5 {
                            RatingGroupItem {
                                key: "{idx}",
                                id: format!("sw5a-rg-half-item-{idx}"),
                                index: idx,
                                style: ITEM_BASE_STYLE,
                                RatingStarVisual {}
                            }
                        }
                    }

                    div {
                        class: "flex items-center gap-2 text-xs font-mono",
                        span { class: "text-slate-400 font-semibold", "Live Value:" }
                        span {
                            id: "sw5a-rg-half-val",
                            style: "display: inline-flex; align-items: center; border-radius: 6px; padding: 4px 10px; background-color: #f5f3ff; color: #6d28d9; border: 1px solid #ddd6fe; font-weight: 700;",
                            "{format_rating_value(half_val())}"
                        }
                    }
                }
            }

            // Scenario 2 (SW5A-7): RTL & Vertical Orientation Geometry
            div {
                class: "rounded-2xl border border-slate-200 bg-white p-6 shadow-sm space-y-5",
                div {
                    class: "space-y-1",
                    h3 {
                        class: "text-base font-semibold text-slate-900",
                        "2. RTL & Vertical Orientation Geometry (SW5A-7)"
                    }
                    p {
                        class: "text-xs text-slate-500",
                        "RTL reverses horizontal hit halves and ArrowLeft/ArrowRight directions; Vertical keeps top-to-bottom half-star hit math."
                    }
                }

                div {
                    class: "grid grid-cols-1 md:grid-cols-2 gap-4",

                    // RTL RatingGroup
                    div {
                        style: "padding: 16px; border-radius: 0.75rem; border: 1px solid #e2e8f0; background-color: #f8fafc; display: flex; flex-direction: column; gap: 12px; min-width: 0;",
                        div {
                            style: "display: flex; flex-wrap: wrap; align-items: center; justify-content: space-between; gap: 6px;",
                            span {
                                class: "text-xs font-bold uppercase tracking-wider text-slate-600",
                                "RTL Horizontal (allow_half)"
                            }
                            span { class: "sw1a-pill-purple", "dir=\"rtl\"" }
                        }
                        RatingGroupRoot {
                            id: "sw5a-rg-rtl",
                            dir: Direction::Rtl,
                            allow_half: true,
                            default_value: 2.0,
                            max: 5,
                            aria_label: "RTL half-star rating",
                            style: "display: inline-flex; align-self: flex-start; flex-direction: row; align-items: center; gap: 4px; padding: 6px; border-radius: 12px; border: 1px solid #e2e8f0; background-color: #ffffff; direction: rtl; outline: none;",
                            on_value_change: move |v| rtl_val.set(v),
                            for idx in 0..5 {
                                RatingGroupItem {
                                    key: "{idx}",
                                    id: format!("sw5a-rg-rtl-item-{idx}"),
                                    index: idx,
                                    style: ITEM_BASE_STYLE,
                                    RatingStarVisual { rtl: true }
                                }
                            }
                        }
                        div {
                            class: "flex items-center gap-2 text-xs font-mono",
                            span { class: "text-slate-400 font-semibold", "RTL Value:" }
                            span {
                                id: "sw5a-rg-rtl-val",
                                style: "display: inline-flex; align-items: center; border-radius: 6px; padding: 2px 8px; background-color: #ffffff; border: 1px solid #cbd5e1; color: #334155; font-weight: 700;",
                                "{format_rating_value(rtl_val())}"
                            }
                        }
                    }

                    // Vertical RatingGroup
                    div {
                        style: "padding: 16px; border-radius: 0.75rem; border: 1px solid #e2e8f0; background-color: #f8fafc; display: flex; flex-direction: column; gap: 12px; min-width: 0;",
                        div {
                            style: "display: flex; flex-wrap: wrap; align-items: center; justify-content: space-between; gap: 6px;",
                            span {
                                class: "text-xs font-bold uppercase tracking-wider text-slate-600",
                                "Vertical Orientation (allow_half)"
                            }
                            span { class: "sw1a-pill-slate", "orientation=\"vertical\"" }
                        }
                        div {
                            style: "display: flex; align-items: center; gap: 16px;",
                            RatingGroupRoot {
                                id: "sw5a-rg-vertical",
                                orientation: Orientation::Vertical,
                                allow_half: true,
                                default_value: 1.5,
                                max: 5,
                                aria_label: "Vertical half-star rating",
                                style: "display: inline-flex; flex-direction: column; align-items: center; gap: 4px; padding: 6px; border-radius: 12px; border: 1px solid #e2e8f0; background-color: #ffffff; outline: none;",
                                on_value_change: move |v| vertical_val.set(v),
                                for idx in 0..5 {
                                    RatingGroupItem {
                                        key: "{idx}",
                                        id: format!("sw5a-rg-vertical-item-{idx}"),
                                        index: idx,
                                        style: ITEM_BASE_STYLE,
                                        RatingStarVisual {}
                                    }
                                }
                            }
                            div {
                                class: "flex items-center gap-2 text-xs font-mono",
                                span { class: "text-slate-400 font-semibold", "Vertical Value:" }
                                span {
                                    id: "sw5a-rg-vertical-val",
                                    style: "display: inline-flex; align-items: center; border-radius: 6px; padding: 2px 8px; background-color: #ffffff; border: 1px solid #cbd5e1; color: #334155; font-weight: 700;",
                                    "{format_rating_value(vertical_val())}"
                                }
                            }
                        }
                    }
                }
            }

            // Scenario 3 (SW5A-9): Form Submission, Native Reset Sync, Disabled & ReadOnly Guards
            div {
                class: "rounded-2xl border border-slate-200 bg-white p-6 shadow-sm space-y-6",
                div {
                    class: "space-y-1",
                    h3 {
                        class: "text-base font-semibold text-slate-900",
                        "3. Form Submission, Native Reset Sync, Disabled & Read-Only Guards (SW5A-9)"
                    }
                    p {
                        class: "text-xs text-slate-500",
                        "Hidden input name=\"score\" syncs with <form> submission and native <button type=\"reset\"> (restoring default_value: 2.0). Disabled and read-only instances block mutations."
                    }
                }

                form {
                    id: "sw5a-rg-form-el",
                    onsubmit: move |evt| {
                        evt.prevent_default();
                        let summary = match collect_form_data("sw5a-rg-form-el") {
                            Ok(entries) => entries
                                .into_iter()
                                .map(|(k, v)| format!("{k}={v}"))
                                .collect::<Vec<_>>()
                                .join(", "),
                            Err(_) => format!("score={}", format_rating_value(form_val())),
                        };
                        submitted_score.set(summary);
                    },
                    style: "padding: 16px; border-radius: 0.75rem; border: 1px solid #e2e8f0; background-color: #f8fafc; display: flex; flex-direction: column; gap: 14px;",

                    div {
                        style: "display: flex; flex-wrap: wrap; align-items: center; gap: 16px;",
                        RatingGroupRoot {
                            id: "sw5a-rg-form",
                            name: "score",
                            default_value: 2.0,
                            max: 5,
                            aria_label: "Form score rating",
                            style: "display: inline-flex; flex-direction: row; align-items: center; gap: 4px; padding: 6px; border-radius: 12px; border: 1px solid #e2e8f0; background-color: #ffffff; outline: none;",
                            on_value_change: move |v| form_val.set(v),
                            for idx in 0..5 {
                                RatingGroupItem {
                                    key: "{idx}",
                                    id: format!("sw5a-rg-form-item-{idx}"),
                                    index: idx,
                                    style: ITEM_BASE_STYLE,
                                    RatingStarVisual {}
                                }
                            }
                        }

                        div {
                            class: "flex items-center gap-2 text-xs font-mono",
                            span { class: "text-slate-400 font-semibold", "Form Value:" }
                            span {
                                id: "sw5a-rg-form-val",
                                style: "display: inline-flex; align-items: center; border-radius: 6px; padding: 4px 10px; background-color: #ffffff; border: 1px solid #cbd5e1; color: #0f172a; font-weight: 700;",
                                "{format_rating_value(form_val())}"
                            }
                        }
                    }

                    div {
                        style: "display: flex; flex-wrap: wrap; align-items: center; gap: 10px;",
                        button {
                            id: "sw5a-rg-submit-btn",
                            r#type: "submit",
                            class: "sw1a-btn-primary",
                            style: "padding: 0.375rem 0.875rem; font-size: 0.75rem;",
                            "Submit Score"
                        }
                        button {
                            id: "sw5a-rg-reset-btn",
                            r#type: "reset",
                            class: "sw1a-btn-secondary",
                            style: "padding: 0.375rem 0.875rem; font-size: 0.75rem;",
                            "Reset Form"
                        }
                        span {
                            id: "sw5a-rg-form-submitted",
                            style: "font-size: 0.75rem; font-family: monospace; color: #64748b;",
                            "Submitted: {submitted_score()}"
                        }
                    }
                }

                // Disabled & Read-Only Guards
                div {
                    class: "grid grid-cols-1 md:grid-cols-2 gap-4",
                    div {
                        style: "padding: 16px; border-radius: 0.75rem; border: 1px solid #e2e8f0; background-color: #f8fafc; display: flex; flex-direction: column; gap: 10px;",
                        div {
                            class: "flex items-center justify-between gap-2",
                            span {
                                class: "text-xs font-bold uppercase tracking-wider text-slate-600",
                                "Disabled Guard"
                            }
                            span { class: "sw1a-pill-amber", "tabindex=\"-1\"" }
                        }
                        RatingGroupRoot {
                            id: "sw5a-rg-disabled",
                            disabled: true,
                            default_value: 2.0,
                            max: 5,
                            aria_label: "Disabled rating",
                            style: "display: inline-flex; flex-direction: row; align-items: center; gap: 4px; padding: 6px; border-radius: 12px; border: 1px solid #e2e8f0; background-color: #f1f5f9; outline: none;",
                            for idx in 0..5 {
                                RatingGroupItem {
                                    key: "{idx}",
                                    id: format!("sw5a-rg-disabled-item-{idx}"),
                                    index: idx,
                                    style: ITEM_DISABLED_STYLE,
                                    RatingStarVisual {}
                                }
                            }
                        }
                    }

                    div {
                        style: "padding: 16px; border-radius: 0.75rem; border: 1px solid #e2e8f0; background-color: #f8fafc; display: flex; flex-direction: column; gap: 10px;",
                        div {
                            class: "flex items-center justify-between gap-2",
                            span {
                                class: "text-xs font-bold uppercase tracking-wider text-slate-600",
                                "Read-Only Guard"
                            }
                            span { class: "sw1a-pill-slate", "tabindex=\"0\"" }
                        }
                        RatingGroupRoot {
                            id: "sw5a-rg-readonly",
                            read_only: true,
                            default_value: 3.0,
                            max: 5,
                            aria_label: "Read-only rating",
                            style: "display: inline-flex; flex-direction: row; align-items: center; gap: 4px; padding: 6px; border-radius: 12px; border: 1px solid #e2e8f0; background-color: #ffffff; outline: none;",
                            for idx in 0..5 {
                                RatingGroupItem {
                                    key: "{idx}",
                                    id: format!("sw5a-rg-readonly-item-{idx}"),
                                    index: idx,
                                    style: ITEM_BASE_STYLE,
                                    RatingStarVisual {}
                                }
                            }
                        }
                    }
                }
            }
        }
    }
}

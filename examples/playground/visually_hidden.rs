use dioxus::prelude::*;
use monoxus::visually_hidden::{VisuallyHidden, VisuallyHiddenRenderProps};

#[component]
pub fn VisuallyHiddenPlayground() -> Element {
    let mut article_clicks = use_signal(|| 0_u32);

    rsx! {
        div {
            class: "max-w-4xl mx-auto space-y-8",

            // Header Section
            div {
                class: "space-y-1.5",
                h2 {
                    class: "text-2xl font-bold tracking-tight text-slate-900",
                    "VisuallyHidden"
                }
                p {
                    class: "text-sm text-slate-500",
                    "Screen-reader-only CSS clipping primitive (1x1px, position: absolute, clip: rect(0, 0, 0, 0)) that preserves accessibility tree exposure with zero layout shift and zero 1px paint bleed."
                }
            }

            // Scenario 1 (SW5A-1): Paint & Geometry Parity inside Bordered Container
            div {
                class: "rounded-2xl border border-slate-200 bg-white p-6 shadow-sm space-y-4",
                div {
                    class: "space-y-1",
                    h3 {
                        class: "text-base font-semibold text-slate-900",
                        "1. Zero Layout Shift & Zero 1px Paint Bleed (SW5A-1)"
                    }
                    p {
                        class: "text-xs text-slate-500",
                        "VisuallyHidden renders an out-of-flow 1x1px clipped span (#sw5a-vh-span) beside visible text inside #sw5a-vh-box, matching the control box (#sw5a-vh-control-box) height and width identically."
                    }
                }

                div {
                    class: "grid grid-cols-1 md:grid-cols-2 gap-4",
                    div {
                        class: "space-y-2",
                        span {
                            class: "text-xs font-semibold text-slate-700",
                            "With VisuallyHidden (#sw5a-vh-box)"
                        }
                        div {
                            id: "sw5a-vh-box",
                            style: "position: relative; border: 2px solid #cbd5e1; border-radius: 0.75rem; background-color: #ffffff; padding: 12px 16px; display: flex; align-items: center; justify-content: space-between; gap: 12px;",
                            span {
                                id: "sw5a-vh-visible-text",
                                style: "font-size: 0.875rem; font-weight: 500; color: #0f172a;",
                                "Visible container text"
                            }
                            VisuallyHidden {
                                id: "sw5a-vh-span",
                                "Skip to ratings"
                            }
                            span {
                                class: "sw1a-pill-purple",
                                "SR: Skip to ratings"
                            }
                        }
                    }

                    div {
                        class: "space-y-2",
                        span {
                            class: "text-xs font-semibold text-slate-700",
                            "Control Reference (No VisuallyHidden)"
                        }
                        div {
                            id: "sw5a-vh-control-box",
                            style: "position: relative; border: 2px solid #cbd5e1; border-radius: 0.75rem; background-color: #f8fafc; padding: 12px 16px; display: flex; align-items: center; justify-content: space-between; gap: 12px;",
                            span {
                                style: "font-size: 0.875rem; font-weight: 500; color: #0f172a;",
                                "Visible container text"
                            }
                            span {
                                class: "sw1a-pill-slate",
                                "Identical box metrics"
                            }
                        }
                    }
                }
            }

            // Scenario 2 (SW5A-2): as_child Style Merge & Tag Projection
            div {
                id: "sw5a-vh-as-child-card",
                class: "rounded-2xl border border-slate-200 bg-white p-6 shadow-sm space-y-4",
                style: "position: relative;",
                div {
                    class: "space-y-1",
                    h3 {
                        class: "text-base font-semibold text-slate-900",
                        "2. as_child Projection & Inline Style Merge (SW5A-2)"
                    }
                    p {
                        class: "text-xs text-slate-500",
                        "VisuallyHidden with as_child: true projects canonical SR-only declarations merged with custom style (outline-color: rgb(1, 2, 3);) onto <article id=\"sw5a-vh-article\">."
                    }
                }

                div {
                    id: "sw5a-vh-as-child-host",
                    style: "position: relative; border: 1px dashed #cbd5e1; border-radius: 0.75rem; background-color: #f8fafc; padding: 12px 16px; display: flex; flex-wrap: wrap; align-items: center; justify-content: space-between; gap: 12px;",
                    span {
                        style: "font-size: 0.8125rem; color: #334155; font-weight: 500;",
                        "Slotted <article id=\"sw5a-vh-article\"> mounted inside host"
                    }
                    VisuallyHidden {
                        id: "sw5a-vh-article",
                        as_child: true,
                        style: "outline-color: rgb(1, 2, 3);",
                        on_click: move |_| article_clicks += 1,
                        render_child: move |props: VisuallyHiddenRenderProps| {
                            rsx! {
                                article {
                                    id: props.id,
                                    class: props.class,
                                    style: props.style,
                                    "data-visually-hidden-root": props.data_visually_hidden_root,
                                    onclick: move |evt| {
                                        if let Some(handler) = props.on_click.as_ref() {
                                            handler.call(evt);
                                        }
                                    },
                                    onmounted: move |evt| {
                                        if let Some(handler) = props.on_mounted.as_ref() {
                                            handler.call(evt);
                                        }
                                    },
                                    ..props.attributes,
                                    {props.children}
                                }
                            }
                        },
                        "Projected screen-reader article"
                    }
                    div {
                        class: "flex items-center gap-2",
                        span { class: "sw1a-pill-purple", "<article>" }
                        span { class: "sw1a-pill-slate", "outline-color: rgb(1, 2, 3)" }
                        span {
                            id: "sw5a-vh-article-clicks",
                            style: "font-size: 0.75rem; font-family: monospace; color: #64748b;",
                            "Clicks: {article_clicks()}"
                        }
                    }
                }
            }
        }
    }
}

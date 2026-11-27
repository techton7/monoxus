use dioxus::prelude::*;
use monoxus::accessible_icon::{AccessibleIcon, AccessibleIconRoot};

#[component]
pub fn AccessibleIconPlayground() -> Element {
    let mut direct_clicks = use_signal(|| 0_u32);
    let mut fallback_clicks = use_signal(|| 0_u32);

    rsx! {
        div {
            class: "max-w-4xl mx-auto space-y-8",

            // Header Section
            div {
                class: "space-y-1.5",
                h2 {
                    class: "text-2xl font-bold tracking-tight text-slate-900",
                    "AccessibleIcon"
                }
                p {
                    class: "text-sm text-slate-500",
                    "Decorative icon wrapper hiding SVG artwork from assistive technology (aria-hidden=\"true\", focusable=\"false\") while providing an accessible sibling VisuallyHidden label."
                }
            }

            // Scenario (SW5A-3): Direct render_child vs Ergonomic children fallback
            div {
                class: "rounded-2xl border border-slate-200 bg-white p-6 shadow-sm space-y-5",
                div {
                    class: "space-y-1",
                    h3 {
                        class: "text-base font-semibold text-slate-900",
                        "1. Direct render_child & Ergonomic children Fallback (SW5A-3)"
                    }
                    p {
                        class: "text-xs text-slate-500",
                        "Both buttons expose the accessible name \"Favorite\" via VisuallyHidden while hiding the decorative heart <svg> from screen readers."
                    }
                }

                div {
                    class: "grid grid-cols-1 md:grid-cols-2 gap-4",

                    // Direct render_child path
                    div {
                        style: "padding: 16px; border-radius: 0.75rem; border: 1px solid #e2e8f0; background-color: #f8fafc; display: flex; flex-direction: column; gap: 12px;",
                        div {
                            class: "flex items-center justify-between gap-2",
                            span {
                                class: "text-xs font-bold uppercase tracking-wider text-slate-600",
                                "Direct render_child"
                            }
                            span { class: "sw1a-pill-purple", "zero wrapper" }
                        }
                        div {
                            class: "flex items-center gap-3",
                            button {
                                id: "sw5a-ai-btn-direct",
                                r#type: "button",
                                onclick: move |_| direct_clicks += 1,
                                style: "position: relative; display: inline-flex; align-items: center; justify-content: center; width: 40px; height: 40px; border-radius: 0.5rem; border: 1px solid #cbd5e1; background-color: #ffffff; cursor: pointer; box-shadow: 0 1px 2px rgba(15, 23, 42, 0.06);",
                                AccessibleIcon {
                                    label: "Favorite",
                                    child: move |props: Vec<Attribute>| {
                                        rsx! {
                                            svg {
                                                id: "sw5a-ai-svg-direct",
                                                width: "20",
                                                height: "20",
                                                view_box: "0 0 24 24",
                                                fill: "#ef4444",
                                                stroke: "#dc2626",
                                                "stroke-width": "1.75",
                                                style: "width: 20px; height: 20px; display: block; flex-shrink: 0;",
                                                ..props,
                                                path {
                                                    d: "M12 21.35l-1.45-1.32C5.4 15.36 2 12.28 2 8.5 2 5.42 4.42 3 7.5 3c1.74 0 3.41.81 4.5 2.09C13.09 3.81 14.76 3 16.5 3 19.58 3 22 5.42 22 8.5c0 3.78-3.4 6.86-8.55 11.54L12 21.35z",
                                                }
                                            }
                                        }
                                    }
                                }
                            }
                            div {
                                class: "space-y-0.5 text-xs",
                                p {
                                    class: "font-medium text-slate-800",
                                    "Attributes directly on <svg>"
                                }
                                p {
                                    class: "font-mono text-slate-500",
                                    "Clicks: "
                                    strong { id: "sw5a-ai-direct-count", "{direct_clicks()}" }
                                }
                            }
                        }
                    }

                    // Ergonomic children fallback path
                    div {
                        style: "padding: 16px; border-radius: 0.75rem; border: 1px solid #e2e8f0; background-color: #f8fafc; display: flex; flex-direction: column; gap: 12px;",
                        div {
                            class: "flex items-center justify-between gap-2",
                            span {
                                class: "text-xs font-bold uppercase tracking-wider text-slate-600",
                                "Children Fallback"
                            }
                            span { class: "sw1a-pill-slate", "display: contents" }
                        }
                        div {
                            class: "flex items-center gap-3",
                            button {
                                id: "sw5a-ai-btn-fallback",
                                r#type: "button",
                                onclick: move |_| fallback_clicks += 1,
                                style: "position: relative; display: inline-flex; align-items: center; justify-content: center; width: 40px; height: 40px; border-radius: 0.5rem; border: 1px solid #cbd5e1; background-color: #ffffff; cursor: pointer; box-shadow: 0 1px 2px rgba(15, 23, 42, 0.06);",
                                AccessibleIconRoot {
                                    label: "Favorite",
                                    svg {
                                        id: "sw5a-ai-svg-fallback",
                                        width: "20",
                                        height: "20",
                                        view_box: "0 0 24 24",
                                        fill: "#8b5cf6",
                                        stroke: "#7c3aed",
                                        "stroke-width": "1.75",
                                        style: "width: 20px; height: 20px; display: block; flex-shrink: 0;",
                                        path {
                                            d: "M12 21.35l-1.45-1.32C5.4 15.36 2 12.28 2 8.5 2 5.42 4.42 3 7.5 3c1.74 0 3.41.81 4.5 2.09C13.09 3.81 14.76 3 16.5 3 19.58 3 22 5.42 22 8.5c0 3.78-3.4 6.86-8.55 11.54L12 21.35z",
                                        }
                                    }
                                }
                            }
                            div {
                                class: "space-y-0.5 text-xs",
                                p {
                                    class: "font-medium text-slate-800",
                                    "Wrapped in aria-hidden display: contents"
                                }
                                p {
                                    class: "font-mono text-slate-500",
                                    "Clicks: "
                                    strong { id: "sw5a-ai-fallback-count", "{fallback_clicks()}" }
                                }
                            }
                        }
                    }
                }
            }
        }
    }
}

use dioxus::prelude::*;
use monoxus::foundation::shared::{Direction, Orientation};
use monoxus::scroll_area::*;

#[component]
pub fn ScrollAreaPlayground() -> Element {
    let mut auto_tall = use_signal(|| true);
    let mut show_dual_x = use_signal(|| true);
    let auto_height_label = if auto_tall() {
        "700px (overflowing)"
    } else {
        "80px (no overflow)"
    };

    rsx! {
        div {
            class: "max-w-5xl mx-auto space-y-10 p-6",

            div {
                class: "space-y-1.5",
                h2 {
                    class: "text-2xl font-bold tracking-tight text-slate-900",
                    "ScrollArea"
                }
                p {
                    class: "text-sm text-slate-500",
                    "Augments native scroll functionality for custom, cross-browser styling with Hover, Scroll, Auto, and Always modes."
                }
            }

            // ---------------------------------------------------------------
            // Scenario 1 (SW2B-8): Type Modes Grid (Hover, Scroll, Auto, Always)
            // ---------------------------------------------------------------
            div {
                class: "rounded-2xl border border-slate-200 bg-white p-6 shadow-sm space-y-4",
                div {
                    class: "space-y-1",
                    h3 { class: "text-base font-semibold text-slate-900", "Scenario 1: ScrollArea Type Modes (SW2B-8)" }
                    p { class: "text-xs text-slate-500",
                        "Grid of 4 ScrollArea cards: Hover, Scroll, Auto (with dynamic content resize toggle), and Always."
                    }
                }

                div {
                    class: "flex flex-wrap gap-6 items-start",

                    // 1. Hover
                    div {
                        class: "space-y-2",
                        div { class: "text-xs font-semibold text-slate-700", "Type: Hover (hide delay 600ms)" }
                        ScrollAreaRoot {
                            id: "sw2b-sa-hover-root".to_string(),
                            r#type: ScrollAreaType::Hover,
                            scroll_hide_delay: 600,
                            style: "width: 260px; height: 180px; border: 1px solid #cbd5e1; border-radius: 0.5rem; background: #ffffff;",
                            ScrollAreaViewport {
                                id: "sw2b-sa-hover-vp".to_string(),
                                class: "sw2b-sa-viewport",
                                style: "width: 100%; height: 100%;",
                                div {
                                    class: "p-3 space-y-2",
                                    for i in 1..=20 {
                                        div { class: "py-1 px-2 border-b border-slate-100 text-xs text-slate-700", "Hover Item {i}" }
                                    }
                                }
                            }
                            ScrollAreaScrollbar {
                                id: "sw2b-sa-hover-sb-y".to_string(),
                                orientation: Orientation::Vertical,
                                class: "sw2b-sa-scrollbar",
                                style: "width: 10px; background: rgba(0, 0, 0, 0.05);",
                                ScrollAreaThumb {
                                    id: "sw2b-sa-hover-thumb-y".to_string(),
                                    class: "sw2b-sa-thumb",
                                    style: "background: rgba(0, 0, 0, 0.4); border-radius: 9999px;",
                                }
                            }
                        }
                    }

                    // 2. Scroll
                    div {
                        class: "space-y-2",
                        div { class: "text-xs font-semibold text-slate-700", "Type: Scroll (hide delay 400ms)" }
                        ScrollAreaRoot {
                            id: "sw2b-sa-scroll-root".to_string(),
                            r#type: ScrollAreaType::Scroll,
                            scroll_hide_delay: 400,
                            style: "width: 260px; height: 180px; border: 1px solid #cbd5e1; border-radius: 0.5rem; background: #ffffff;",
                            ScrollAreaViewport {
                                id: "sw2b-sa-scroll-vp".to_string(),
                                class: "sw2b-sa-viewport",
                                style: "width: 100%; height: 100%;",
                                div {
                                    class: "p-3 space-y-2",
                                    for i in 1..=20 {
                                        div { class: "py-1 px-2 border-b border-slate-100 text-xs text-slate-700", "Scroll Item {i}" }
                                    }
                                }
                            }
                            ScrollAreaScrollbar {
                                id: "sw2b-sa-scroll-sb-y".to_string(),
                                orientation: Orientation::Vertical,
                                class: "sw2b-sa-scrollbar",
                                style: "width: 10px; background: rgba(0, 0, 0, 0.05);",
                                ScrollAreaThumb {
                                    id: "sw2b-sa-scroll-thumb-y".to_string(),
                                    class: "sw2b-sa-thumb",
                                    style: "background: rgba(0, 0, 0, 0.4); border-radius: 9999px;",
                                }
                            }
                        }
                    }

                    // 3. Auto (+ toggle button)
                    div {
                        class: "space-y-2",
                        div {
                            class: "flex items-center justify-between",
                            span { class: "text-xs font-semibold text-slate-700", "Type: Auto" }
                            button {
                                id: "sw2b-sa-auto-toggle",
                                class: "text-xs px-2 py-0.5 bg-indigo-600 text-white rounded font-medium hover:bg-indigo-700",
                                onclick: move |_| auto_tall.toggle(),
                                "Toggle Height"
                            }
                        }
                        ScrollAreaRoot {
                            id: "sw2b-sa-auto-root".to_string(),
                            r#type: ScrollAreaType::Auto,
                            style: "width: 260px; height: 180px; border: 1px solid #cbd5e1; border-radius: 0.5rem; background: #ffffff;",
                            ScrollAreaViewport {
                                id: "sw2b-sa-auto-vp".to_string(),
                                class: "sw2b-sa-viewport",
                                style: "width: 100%; height: 100%;",
                                div {
                                    style: if auto_tall() { "height: 700px; padding: 12px;" } else { "height: 80px; padding: 12px;" },
                                    div { class: "text-xs font-semibold text-slate-800", "Auto Content Height: {auto_height_label}" }
                                    if auto_tall() {
                                        div { class: "text-xs text-slate-500 mt-2", "Scroll down to see more items..." }
                                        for i in 1..=15 {
                                            div { class: "py-1 px-2 border-b border-slate-100 text-xs text-slate-600", "Auto item {i}" }
                                        }
                                    } else {
                                        div { class: "text-xs text-slate-500 mt-2", "Content fits within viewport height (no scrollbar)." }
                                    }
                                }
                            }
                            ScrollAreaScrollbar {
                                id: "sw2b-sa-auto-sb-y".to_string(),
                                orientation: Orientation::Vertical,
                                class: "sw2b-sa-scrollbar",
                                style: "width: 10px; background: rgba(0, 0, 0, 0.05);",
                                ScrollAreaThumb {
                                    id: "sw2b-sa-auto-thumb-y".to_string(),
                                    class: "sw2b-sa-thumb",
                                    style: "background: rgba(0, 0, 0, 0.4); border-radius: 9999px;",
                                }
                            }
                        }
                    }

                    // 4. Always
                    div {
                        class: "space-y-2",
                        div { class: "text-xs font-semibold text-slate-700", "Type: Always" }
                        ScrollAreaRoot {
                            id: "sw2b-sa-always-root".to_string(),
                            r#type: ScrollAreaType::Always,
                            style: "width: 260px; height: 180px; border: 1px solid #cbd5e1; border-radius: 0.5rem; background: #ffffff;",
                            ScrollAreaViewport {
                                id: "sw2b-sa-always-vp".to_string(),
                                class: "sw2b-sa-viewport",
                                style: "width: 100%; height: 100%;",
                                div {
                                    class: "p-3 space-y-2",
                                    for i in 1..=20 {
                                        div { class: "py-1 px-2 border-b border-slate-100 text-xs text-slate-700", "Always Item {i}" }
                                    }
                                }
                            }
                            ScrollAreaScrollbar {
                                id: "sw2b-sa-always-sb-y".to_string(),
                                orientation: Orientation::Vertical,
                                class: "sw2b-sa-scrollbar",
                                style: "width: 10px; background: rgba(0, 0, 0, 0.05);",
                                ScrollAreaThumb {
                                    id: "sw2b-sa-always-thumb-y".to_string(),
                                    class: "sw2b-sa-thumb",
                                    style: "background: rgba(0, 0, 0, 0.4); border-radius: 9999px;",
                                }
                            }
                        }
                    }
                }
            }

            // ---------------------------------------------------------------
            // Scenario 2 (SW2B-9): Extreme 10,000px Thumb Clamp (18px minimum)
            // ---------------------------------------------------------------
            div {
                class: "rounded-2xl border border-slate-200 bg-white p-6 shadow-sm space-y-4",
                div {
                    class: "space-y-1",
                    h3 { class: "text-base font-semibold text-slate-900", "Scenario 2: Extreme Thumb Size Clamp (SW2B-9)" }
                    p { class: "text-xs text-slate-500",
                        "Extreme 10,000px content inside 200px viewport, proving thumb clamps to minimum 18px size."
                    }
                }

                ScrollAreaRoot {
                    id: "sw2b-sa-clamp-root".to_string(),
                    r#type: ScrollAreaType::Always,
                    style: "width: 300px; height: 200px; border: 1px solid #cbd5e1; border-radius: 0.5rem; background: #ffffff;",
                    ScrollAreaViewport {
                        id: "sw2b-sa-clamp-vp".to_string(),
                        class: "sw2b-sa-viewport",
                        style: "width: 100%; height: 100%;",
                        div {
                            style: "height: 10000px; background: linear-gradient(to bottom, #f8fafc, #64748b); padding: 12px;",
                            div { class: "text-xs font-semibold text-slate-800", "Extreme 10,000px Content (min thumb clamp 18px)" }
                            div { class: "text-xs text-slate-500 mt-1", "Thumb height is clamped at 18px minimum." }
                        }
                    }
                    ScrollAreaScrollbar {
                        id: "sw2b-sa-clamp-sb-y".to_string(),
                        orientation: Orientation::Vertical,
                        class: "sw2b-sa-scrollbar",
                        style: "width: 12px; background: rgba(0, 0, 0, 0.05);",
                        ScrollAreaThumb {
                            id: "sw2b-sa-clamp-thumb-y".to_string(),
                            class: "sw2b-sa-thumb",
                            style: "background: rgba(0, 0, 0, 0.5); border-radius: 9999px;",
                        }
                    }
                }
            }

            // ---------------------------------------------------------------
            // Scenario 3 (SW2B-10, SW2B-11): Interactive Drag / Track Jump / Wheel
            // ---------------------------------------------------------------
            div {
                class: "rounded-2xl border border-slate-200 bg-white p-6 shadow-sm space-y-4",
                div {
                    class: "space-y-1",
                    h3 { class: "text-base font-semibold text-slate-900", "Scenario 3: Interactive Drag, Track Jump & Wheel Clamping (SW2B-10, SW2B-11)" }
                    p { class: "text-xs text-slate-500",
                        "ScrollArea with 1600px content in 240px viewport. Supports thumb dragging, track-jump clicking, and wheel scrolling without bubbling past bounds."
                    }
                }

                ScrollAreaRoot {
                    id: "sw2b-sa-interact-root".to_string(),
                    r#type: ScrollAreaType::Always,
                    style: "width: 360px; height: 240px; border: 1px solid #cbd5e1; border-radius: 0.5rem; background: #ffffff;",
                    ScrollAreaViewport {
                        id: "sw2b-sa-interact-vp".to_string(),
                        class: "sw2b-sa-viewport",
                        style: "width: 100%; height: 100%;",
                        div {
                            style: "height: 1600px; padding: 8px;",
                            for i in 1..=40 {
                                div {
                                    id: "sw2b-sa-interact-row-{i}",
                                    class: "py-2 px-3 border-b border-slate-100 text-xs font-mono text-slate-700 flex justify-between",
                                    span { "Row {i}" }
                                    span { class: "text-slate-400", "offset ~{(i - 1) * 40}px" }
                                }
                            }
                        }
                    }
                    ScrollAreaScrollbar {
                        id: "sw2b-sa-interact-sb-y".to_string(),
                        orientation: Orientation::Vertical,
                        class: "sw2b-sa-scrollbar",
                        style: "width: 14px; background: rgba(0, 0, 0, 0.05);",
                        ScrollAreaThumb {
                            id: "sw2b-sa-interact-thumb-y".to_string(),
                            class: "sw2b-sa-thumb",
                            style: "background: rgba(0, 0, 0, 0.5); border-radius: 9999px;",
                        }
                    }
                }
                div {
                    id: "sw2b-sa-interact-readout",
                    class: "text-xs font-mono text-slate-600",
                    "Interactive ScrollArea (1600px / 240px)"
                }
            }

            // ---------------------------------------------------------------
            // Scenario 4 (SW2B-12): Dual-Axis + Corner + Nonce + RTL Horizontal
            // ---------------------------------------------------------------
            div {
                class: "rounded-2xl border border-slate-200 bg-white p-6 shadow-sm space-y-4",
                div {
                    class: "space-y-1",
                    h3 { class: "text-base font-semibold text-slate-900", "Scenario 4: Dual-Axis, Corner, Nonce & RTL Horizontal (SW2B-12)" }
                    p { class: "text-xs text-slate-500",
                        "Dual-axis scroll with corner square auto-hiding when horizontal scrollbar is removed, plus RTL horizontal scroll area."
                    }
                }

                div {
                    class: "flex flex-wrap gap-6 items-start",

                    // Dual-Axis Card
                    div {
                        class: "space-y-2",
                        div {
                            class: "flex items-center justify-between",
                            span { class: "text-xs font-semibold text-slate-700", "Dual-Axis (900x900px)" }
                            button {
                                id: "sw2b-sa-dual-toggle-x",
                                class: "text-xs px-2 py-0.5 bg-slate-700 text-white rounded font-medium hover:bg-slate-800",
                                onclick: move |_| show_dual_x.toggle(),
                                "Toggle Horizontal"
                            }
                        }
                        ScrollAreaRoot {
                            id: "sw2b-sa-dual-root".to_string(),
                            r#type: ScrollAreaType::Always,
                            style: "width: 320px; height: 220px; border: 1px solid #cbd5e1; border-radius: 0.5rem; background: #ffffff;",
                            ScrollAreaViewport {
                                id: "sw2b-sa-dual-vp".to_string(),
                                nonce: "sw2b-csp-nonce".to_string(),
                                class: "sw2b-sa-viewport",
                                style: "width: 100%; height: 100%;",
                                div {
                                    style: "width: 900px; height: 900px; background: repeating-linear-gradient(45deg, #f8fafc, #f8fafc 20px, #e2e8f0 20px, #e2e8f0 40px); padding: 12px;",
                                    div { class: "text-xs font-bold text-slate-800", "Dual-Axis 900px x 900px Content" }
                                    div { class: "text-xs text-slate-600 mt-1", "Corner element displayed when both scrollbars are active." }
                                }
                            }
                            ScrollAreaScrollbar {
                                id: "sw2b-sa-dual-sb-y".to_string(),
                                orientation: Orientation::Vertical,
                                class: "sw2b-sa-scrollbar",
                                style: "width: 12px; background: rgba(0, 0, 0, 0.05);",
                                ScrollAreaThumb {
                                    id: "sw2b-sa-dual-thumb-y".to_string(),
                                    class: "sw2b-sa-thumb",
                                    style: "background: rgba(0, 0, 0, 0.5); border-radius: 9999px;",
                                }
                            }
                            if show_dual_x() {
                                ScrollAreaScrollbar {
                                    id: "sw2b-sa-dual-sb-x".to_string(),
                                    orientation: Orientation::Horizontal,
                                    class: "sw2b-sa-scrollbar",
                                    style: "height: 12px; background: rgba(0, 0, 0, 0.05);",
                                    ScrollAreaThumb {
                                        id: "sw2b-sa-dual-thumb-x".to_string(),
                                        class: "sw2b-sa-thumb",
                                        style: "background: rgba(0, 0, 0, 0.5); border-radius: 9999px;",
                                    }
                                }
                            }
                            ScrollAreaCorner {
                                id: "sw2b-sa-dual-corner".to_string(),
                                class: "sw2b-sa-corner",
                                style: "background: rgba(0, 0, 0, 0.15);",
                            }
                        }
                    }

                    // RTL Horizontal Card
                    div {
                        class: "space-y-2",
                        div { class: "text-xs font-semibold text-slate-700", "RTL Horizontal (900px content)" }
                        ScrollAreaRoot {
                            id: "sw2b-sa-rtl-root".to_string(),
                            dir: Direction::Rtl,
                            r#type: ScrollAreaType::Always,
                            style: "width: 320px; height: 140px; border: 1px solid #cbd5e1; border-radius: 0.5rem; background: #ffffff;",
                            ScrollAreaViewport {
                                id: "sw2b-sa-rtl-vp".to_string(),
                                class: "sw2b-sa-viewport",
                                style: "width: 100%; height: 100%;",
                                div {
                                    style: "width: 900px; height: 80px; padding: 12px; text-align: right;",
                                    div { class: "text-xs font-bold text-slate-800", "טקסט מימין לשמאל — RTL Horizontal Content" }
                                    div { class: "text-xs text-slate-500 mt-1", "900px wide scrollable region in RTL direction." }
                                }
                            }
                            ScrollAreaScrollbar {
                                id: "sw2b-sa-rtl-sb-x".to_string(),
                                orientation: Orientation::Horizontal,
                                class: "sw2b-sa-scrollbar",
                                style: "height: 12px; background: rgba(0, 0, 0, 0.05);",
                                ScrollAreaThumb {
                                    id: "sw2b-sa-rtl-thumb-x".to_string(),
                                    class: "sw2b-sa-thumb",
                                    style: "background: rgba(0, 0, 0, 0.5); border-radius: 9999px;",
                                }
                            }
                        }
                    }
                }
            }

            // Document scroll spacer for SW2B-11 window.scrollY stability check
            div { style: "height: 600px;" }
        }
    }
}

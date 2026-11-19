use dioxus::prelude::*;
use monoxus::aspect_ratio::{AspectRatio, AspectRatioRenderProps};

#[component]
pub fn AspectRatioPlayground() -> Element {
    let mut is_wide = use_signal(|| false);
    let mut as_child_clicks = use_signal(|| 0_u32);
    let as_child_click_handler = move |_| as_child_clicks += 1;

    rsx! {
        div {
            class: "max-w-4xl mx-auto space-y-8",

            div {
                class: "space-y-1.5",
                h2 {
                    class: "text-2xl font-bold tracking-tight text-slate-900",
                    "AspectRatio"
                }
                p {
                    class: "text-sm text-slate-500",
                    "Live geometry and as-child composition scenarios for ratios 1, 16/9, and 4/3."
                }
            }

            div {
                class: "rounded-2xl border border-slate-200 bg-white p-6 shadow-sm space-y-5",
                div {
                    class: "flex flex-wrap items-center justify-between gap-3",
                    div {
                        class: "space-y-1",
                        h3 {
                            class: "text-base font-semibold text-slate-900",
                            "Live ratios and container width (SW2A-4)"
                        }
                        p {
                            class: "text-xs text-slate-500",
                            "Toggle the same mounted container between 240px and 360px; inspect the selected wrapper descendants and roots with live host rects."
                        }
                    }
                    button {
                        id: "aspect-ratio-toggle-width",
                        r#type: "button",
                        class: "sw1a-btn-secondary text-xs",
                        onclick: move |_| is_wide.toggle(),
                        if is_wide() { "Use 240px container" } else { "Use 360px container" }
                    }
                }

                div {
                    id: "aspect-ratio-live-container",
                    "data-container-width": if is_wide() { "360" } else { "240" },
                    style: if is_wide() { "width: 360px; max-width: 100%;" } else { "width: 240px; max-width: 100%;" },
                    class: "space-y-5",

                    div {
                        class: "space-y-2",
                        p {
                            class: "text-xs font-semibold text-slate-700",
                            "ratio: 1 / 1"
                        }
                        div {
                            id: "aspect-ratio-wrapper-1",
                            AspectRatio {
                                id: "aspect-ratio-root-1",
                                ratio: 1.0,
                                class: "overflow-hidden rounded-lg bg-violet-100",
                                "aria-label": "Square aspect ratio root",
                                "data-gui-test-id": "aspect-ratio-root-1",
                                div {
                                    id: "aspect-ratio-child-1",
                                    style: "width: 100%; height: 100%; display: flex; align-items: center; justify-content: center;",
                                    "1:1"
                                }
                            }
                        }
                    }

                    div {
                        class: "space-y-2",
                        p {
                            class: "text-xs font-semibold text-slate-700",
                            "ratio: 16 / 9"
                        }
                        div {
                            id: "aspect-ratio-wrapper-16-9",
                            AspectRatio {
                                id: "aspect-ratio-root-16-9",
                                ratio: 16.0 / 9.0,
                                class: "overflow-hidden rounded-lg bg-sky-100",
                                "aria-label": "Widescreen aspect ratio root",
                                div {
                                    id: "aspect-ratio-child-16-9",
                                    style: "width: 100%; height: 100%; display: flex; align-items: center; justify-content: center;",
                                    "16:9"
                                }
                            }
                        }
                    }

                    div {
                        class: "space-y-2",
                        p {
                            class: "text-xs font-semibold text-slate-700",
                            "ratio: 4 / 3"
                        }
                        div {
                            id: "aspect-ratio-wrapper-4-3",
                            AspectRatio {
                                id: "aspect-ratio-root-4-3",
                                ratio: 4.0 / 3.0,
                                class: "overflow-hidden rounded-lg bg-emerald-100",
                                "aria-label": "Standard aspect ratio root",
                                div {
                                    id: "aspect-ratio-child-4-3",
                                    style: "width: 100%; height: 100%; display: flex; align-items: center; justify-content: center;",
                                    "4:3"
                                }
                            }
                        }
                    }
                }
            }

            div {
                class: "rounded-2xl border border-slate-200 bg-white p-6 shadow-sm space-y-5",
                div {
                    class: "space-y-1",
                    h3 {
                        class: "text-base font-semibold text-slate-900",
                        "Library-owned as_child consumer (SW2A-5)"
                    }
                    p {
                        class: "text-xs text-slate-500",
                        "AspectRatio projects its identity, attributes, style, and click event onto this real figure consumer."
                    }
                }

                div {
                    id: "aspect-ratio-as-child-wrapper",
                    style: if is_wide() { "width: 360px; max-width: 100%;" } else { "width: 240px; max-width: 100%;" },
                    AspectRatio {
                        id: "aspect-ratio-as-child-root",
                        ratio: 16.0 / 9.0,
                        as_child: true,
                        class: "aspect-ratio-consumer overflow-hidden rounded-lg bg-amber-100",
                        style: "display: flex; align-items: center; justify-content: center",
                        "aria-label": "Forwarded figure consumer",
                        "data-consumer-kind": "figure",
                        on_click: as_child_click_handler,
                        render_child: |AspectRatioRenderProps {
                            id,
                            class,
                            style,
                            on_click,
                            on_mounted,
                            attributes,
                            children,
                            data_aspect_ratio_root,
                        }| {
                            rsx! {
                                figure {
                                    id,
                                    class,
                                    style,
                                    "data-aspect-ratio-root": data_aspect_ratio_root,
                                    onclick: move |event| {
                                        if let Some(handler) = on_click.as_ref() {
                                            handler.call(event);
                                        }
                                    },
                                    onmounted: move |event| {
                                        if let Some(handler) = on_mounted.as_ref() {
                                            handler.call(event);
                                        }
                                    },
                                    ..attributes,
                                    {children}
                                }
                            }
                        },
                        figcaption {
                            id: "aspect-ratio-as-child-content",
                            "Consumer figure"
                        }
                    }
                }

                p {
                    class: "text-xs text-slate-600",
                    "Forwarded clicks: "
                    strong {
                        id: "aspect-ratio-as-child-click-count",
                        "{as_child_clicks()}"
                    }
                }
            }
        }
    }
}

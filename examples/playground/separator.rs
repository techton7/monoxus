use dioxus::prelude::*;
use monoxus::separator::{Separator, SeparatorOrientation};

#[component]
pub fn SeparatorPlayground() -> Element {
    let mut is_decorative = use_signal(|| true);

    rsx! {
        div {
            class: "max-w-4xl mx-auto space-y-8",

            div {
                class: "space-y-1.5",
                h2 {
                    class: "text-2xl font-bold tracking-tight text-slate-900",
                    "Separator"
                }
                p {
                    class: "text-sm text-slate-500",
                    "Visually or semantically separates content with strict WAI-ARIA 1.2 role and aria-orientation emission matching radix-ui."
                }
            }

            // SW1A-3: Semantic Separators (decorative = false)
            div {
                class: "rounded-2xl border border-slate-200 bg-white p-6 shadow-sm space-y-5",
                div {
                    class: "space-y-1",
                    h3 {
                        class: "text-base font-semibold text-slate-900",
                        "1. Semantic Separators (decorative: false — Default) (SW1A-3)"
                    }
                    p {
                        class: "text-xs text-slate-500",
                        "Horizontal emits role=\"separator\" with aria-orientation omitted (implicit horizontal); Vertical emits role=\"separator\" and aria-orientation=\"vertical\"."
                    }
                }

                div {
                    class: "space-y-4",
                    div {
                        class: "space-y-1",
                        div { class: "text-sm font-semibold text-slate-900", "Monoxus Headless Primitives" }
                        div { class: "text-xs text-slate-500", "Cross-platform WAI-ARIA foundation for Web and Blitz Native." }
                    }

                    div {
                        class: "space-y-2",
                        Separator {
                            id: "sep-horizontal-semantic".to_string(),
                            orientation: SeparatorOrientation::Horizontal,
                            decorative: false,
                        }
                        div {
                            class: "flex items-center gap-2",
                            span {
                                class: "sw1a-pill-purple",
                                "role=\"separator\" (aria-orientation omitted)"
                            }
                            span {
                                class: "text-xs text-slate-500",
                                "Announced by screen readers as landmark horizontal divider"
                            }
                        }
                    }

                    div {
                        class: "space-y-2 pt-2",
                        div {
                            style: "display: flex; align-items: center; gap: 0.875rem; height: 28px;",
                            span { class: "text-xs font-medium text-slate-700", "Architecture" }
                            Separator {
                                id: "sep-vertical-semantic".to_string(),
                                orientation: SeparatorOrientation::Vertical,
                                decorative: false,
                            }
                            span { class: "text-xs font-medium text-slate-700", "Primitives" }
                            Separator {
                                id: "sep-vertical-semantic-2".to_string(),
                                orientation: SeparatorOrientation::Vertical,
                                decorative: false,
                            }
                            span { class: "text-xs font-medium text-slate-700", "Verification" }
                        }
                        div {
                            class: "flex items-center gap-2",
                            span {
                                class: "sw1a-pill-purple",
                                "role=\"separator\" aria-orientation=\"vertical\""
                            }
                            span {
                                class: "text-xs text-slate-500",
                                "Announced by screen readers as vertical separator"
                            }
                        }
                    }
                }
            }

            // SW1A-3: Decorative Separators (decorative = true with interactive toggle)
            div {
                class: "rounded-2xl border border-slate-200 bg-white p-6 shadow-sm space-y-5",
                div {
                    class: "space-y-1",
                    h3 {
                        class: "text-base font-semibold text-slate-900",
                        "2. Decorative Separators & Live Accessibility Tree Inspection (SW1A-3)"
                    }
                    p {
                        class: "text-xs text-slate-500",
                        "When decorative: true, emits role=\"none\" without aria-orientation, removing the divider cleanly from assistive technology trees."
                    }
                }

                div {
                    class: "flex items-center gap-3",
                    button {
                        id: "toggle-decorative",
                        r#type: "button",
                        class: "sw1a-btn-secondary text-xs",
                        onclick: move |_| {
                            let cur = *is_decorative.peek();
                            is_decorative.set(!cur);
                        },
                        if is_decorative() { "Switch to Semantic (decorative: false)" } else { "Switch to Decorative (decorative: true)" }
                    }
                    span {
                        class: "inline-flex items-center rounded-md bg-slate-100 px-2.5 py-1 text-xs font-mono text-slate-700",
                        "Current mode: "
                        strong {
                            id: "sep-decorative-mode",
                            class: "ml-1 font-semibold",
                            if is_decorative() { "decorative (role=\"none\")" } else { "semantic (role=\"separator\")" }
                        }
                    }
                }

                div {
                    class: "space-y-4 pt-2",
                    div {
                        class: "space-y-2",
                        div { class: "text-xs text-slate-600 font-semibold", "Horizontal Toolbar Divider" }
                        Separator {
                            id: "sep-horizontal-decorative".to_string(),
                            orientation: SeparatorOrientation::Horizontal,
                            decorative: is_decorative(),
                        }
                        div {
                            class: "flex items-center gap-2",
                            span {
                                class: "sw1a-pill-slate",
                                if is_decorative() { "role=\"none\"" } else { "role=\"separator\"" }
                            }
                            span {
                                class: "text-xs text-slate-500",
                                if is_decorative() { "Omitted from screen reader accessibility tree" } else { "Exposed in accessibility tree" }
                            }
                        }
                    }

                    div {
                        class: "space-y-2 pt-2",
                        div { class: "text-xs text-slate-600 font-semibold", "Vertical Action Divider" }
                        div {
                            style: "display: flex; align-items: center; gap: 0.875rem; height: 28px;",
                            span { class: "text-xs text-slate-600 font-medium", "Cut" }
                            Separator {
                                id: "sep-vertical-decorative".to_string(),
                                orientation: SeparatorOrientation::Vertical,
                                decorative: is_decorative(),
                            }
                            span { class: "text-xs text-slate-600 font-medium", "Copy" }
                            Separator {
                                id: "sep-vertical-decorative-2".to_string(),
                                orientation: SeparatorOrientation::Vertical,
                                decorative: is_decorative(),
                            }
                            span { class: "text-xs text-slate-600 font-medium", "Paste" }
                        }
                        div {
                            class: "flex items-center gap-2",
                            span {
                                class: "sw1a-pill-slate",
                                if is_decorative() { "role=\"none\" (aria-orientation omitted)" } else { "role=\"separator\" aria-orientation=\"vertical\"" }
                            }
                        }
                    }
                }
            }
        }
    }
}

use dioxus::prelude::*;
use monoxus::collapsible::{CollapsibleContent, CollapsibleRoot, CollapsibleTrigger};

#[component]
pub fn CollapsiblePlayground() -> Element {
    rsx! {
        div {
            class: "max-w-4xl mx-auto space-y-8",

            // Header Section
            div {
                class: "space-y-1.5",
                h2 {
                    class: "text-2xl font-bold tracking-tight text-slate-900",
                    "Collapsible"
                }
                p {
                    class: "text-sm text-slate-500",
                    "An interactive disclosure component which expands/collapses a panel, providing accessible WAI-ARIA states (aria-expanded, aria-controls, data-state)."
                }
            }

            // Scenarios
            BasicCollapsibleScenario {}
            DisabledCollapsibleScenario {}
            ControlledCollapsibleScenario {}
        }
    }
}

// ---------------------------------------------------------------------------
// Scenario 1: Basic Interactive Collapsible
// ---------------------------------------------------------------------------
#[component]
pub fn BasicCollapsibleScenario() -> Element {
    let mut is_open = use_signal(|| false);

    rsx! {
        div {
            class: "rounded-2xl border border-slate-200 bg-white p-6 shadow-sm space-y-4",
            div {
                class: "space-y-1",
                h3 { class: "text-base font-semibold text-slate-900", "1. Basic Interactive Collapsible" }
                p { class: "text-xs text-slate-500",
                    "Standard disclosure toggle using CollapsibleRoot, CollapsibleTrigger, and CollapsibleContent."
                }
            }

            CollapsibleRoot {
                id: "collapsible-root".to_string(),
                default_open: false,
                on_open_change: move |open| is_open.set(open),
                div {
                    class: "flex items-center justify-between pb-3 border-b border-slate-100",
                    span {
                        class: "text-sm font-medium text-slate-700",
                        "@monoxus/primitives repository stars"
                    }
                    CollapsibleTrigger {
                        id: "collapsible-trigger".to_string(),
                        class: "inline-flex items-center justify-center rounded-lg bg-blue-600 px-3 py-1.5 text-xs font-semibold text-white shadow-sm hover:bg-blue-700 transition-colors cursor-pointer",
                        span { if is_open() { "Hide Details" } else { "Show Details" } }
                    }
                }

                div {
                    class: "pt-3 flex items-center gap-2",
                    span {
                        class: "text-xs text-slate-400 font-semibold uppercase",
                        "State:"
                    }
                    span {
                        id: "collapsible-state-badge",
                        "data-state": if is_open() { "open" } else { "closed" },
                        class: "inline-flex items-center rounded px-2 py-0.5 text-xs font-mono font-medium bg-slate-100 text-slate-700",
                        if is_open() { "open" } else { "closed" }
                    }
                }

                CollapsibleContent {
                    id: "collapsible-content".to_string(),
                    class: "pt-4 space-y-2 text-sm text-slate-600",
                    div {
                        class: "rounded-lg border border-slate-200 bg-slate-50 p-4 space-y-2",
                        p { class: "font-medium text-slate-900", "⭐ 2,450 stars on GitHub" }
                        p { class: "text-xs text-slate-500", "Headless WAI-ARIA Primitives for Rust & Dioxus cross-platform applications." }
                    }
                }
            }
        }
    }
}

// ---------------------------------------------------------------------------
// Scenario 2: Disabled Collapsible
// ---------------------------------------------------------------------------
#[component]
pub fn DisabledCollapsibleScenario() -> Element {
    rsx! {
        div {
            class: "rounded-2xl border border-slate-200 bg-white p-6 shadow-sm space-y-4",
            div {
                class: "space-y-1",
                h3 { class: "text-base font-semibold text-slate-900", "2. Disabled Collapsible" }
                p { class: "text-xs text-slate-500",
                    "A collapsible with disabled: true prevents toggling and reflects data-disabled."
                }
            }

            CollapsibleRoot {
                id: "collapsible-root-disabled".to_string(),
                disabled: true,
                default_open: false,
                div {
                    class: "flex items-center justify-between",
                    span {
                        class: "text-sm font-medium text-slate-400",
                        "Archived Project Details"
                    }
                    CollapsibleTrigger {
                        id: "collapsible-trigger-disabled".to_string(),
                        disabled: true,
                        class: "inline-flex items-center justify-center rounded-lg bg-slate-200 px-3 py-1.5 text-xs font-semibold text-slate-400 cursor-not-allowed",
                        "Locked"
                    }
                }

                CollapsibleContent {
                    id: "collapsible-content-disabled".to_string(),
                    class: "pt-4 text-sm text-slate-500",
                    "This content cannot be opened because the trigger is disabled."
                }
            }
        }
    }
}

// ---------------------------------------------------------------------------
// Scenario 3: Controlled Collapsible
// ---------------------------------------------------------------------------
#[component]
pub fn ControlledCollapsibleScenario() -> Element {
    let mut open = use_signal(|| true);

    rsx! {
        div {
            class: "rounded-2xl border border-slate-200 bg-white p-6 shadow-sm space-y-4",
            div {
                class: "space-y-1",
                h3 { class: "text-base font-semibold text-slate-900", "3. Controlled Collapsible" }
                p { class: "text-xs text-slate-500",
                    "External Signal controlling open state directly."
                }
            }

            div {
                class: "flex items-center gap-3",
                button {
                    id: "collapsible-external-toggle",
                    class: "rounded-lg border border-slate-300 px-3 py-1.5 text-xs font-medium text-slate-700 hover:bg-slate-50 cursor-pointer",
                    onclick: move |_| {
                        let cur = *open.peek();
                        open.set(!cur);
                    },
                    if open() { "External Close" } else { "External Open" }
                }
            }

            CollapsibleRoot {
                id: "collapsible-root-controlled".to_string(),
                open: open,
                div {
                    class: "flex items-center justify-between pt-2",
                    span {
                        class: "text-sm font-medium text-slate-700",
                        "Synchronized Status Panel"
                    }
                    CollapsibleTrigger {
                        id: "collapsible-trigger-controlled".to_string(),
                        class: "inline-flex items-center justify-center rounded-lg bg-indigo-600 px-3 py-1.5 text-xs font-semibold text-white hover:bg-indigo-700 cursor-pointer",
                        "Toggle Panel"
                    }
                }

                CollapsibleContent {
                    id: "collapsible-content-controlled".to_string(),
                    class: "pt-4 space-y-2 text-sm text-slate-600",
                    div {
                        class: "rounded-lg border border-indigo-100 bg-indigo-50 p-4",
                        p { class: "font-medium text-indigo-900", "Controlled content is currently visible." }
                    }
                }
            }
        }
    }
}

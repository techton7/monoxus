use dioxus::prelude::*;
use monoxus::collapsible::{CollapsibleContent, CollapsibleRoot, CollapsibleTrigger};

#[component]
pub fn CollapsibleDemo() -> Element {
    let mut open = use_signal(|| false);

    rsx! {
        CollapsibleRoot {
            open: Some(open()),
            on_open_change: move |next| open.set(next),
            class: "w-full max-w-sm space-y-2 rounded-card-sm border border-border-card bg-background p-3 shadow-mini",

            div { class: "flex items-center justify-between gap-2",
                span { class: "text-sm font-medium text-foreground",
                    "@monoxus/primitives (3 crates)"
                }
                CollapsibleTrigger {
                    class: "inline-flex h-8 items-center justify-center rounded-button border border-border-input bg-background-alt px-2.5 text-xs font-medium text-foreground shadow-mini transition-colors hover:bg-muted",
                    if open() { "Hide" } else { "Show all" }
                }
            }

            div { class: "rounded-input border border-border-card bg-background-alt px-3 py-2 font-mono text-xs text-foreground-alt",
                "monoxus"
            }

            CollapsibleContent {
                class: "space-y-2",
                div { class: "rounded-input border border-border-card bg-background-alt px-3 py-2 font-mono text-xs text-foreground-alt",
                    "ui-sdk"
                }
                div { class: "rounded-input border border-border-card bg-background-alt px-3 py-2 font-mono text-xs text-foreground-alt",
                    "oxidase"
                }
            }
        }
    }
}

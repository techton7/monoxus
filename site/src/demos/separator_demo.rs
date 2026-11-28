use dioxus::prelude::*;
use monoxus::separator::{Separator, SeparatorOrientation};

#[component]
pub fn SeparatorDemo() -> Element {
    rsx! {
        div { class: "w-full max-w-xs rounded-card-sm border border-border-card bg-background p-4 shadow-mini",
            div { class: "space-y-1",
                h4 { class: "text-sm font-medium text-foreground", "Monoxus Primitives" }
                p { class: "text-xs text-muted-foreground",
                    "Headless Dioxus components for Web and Blitz Native."
                }
            }

            Separator {
                orientation: SeparatorOrientation::Horizontal,
                class: "my-3 h-px w-full bg-border-card",
            }

            div { class: "flex h-5 items-center gap-3 text-xs text-foreground-alt",
                span { "Docs" }
                Separator {
                    orientation: SeparatorOrientation::Vertical,
                    decorative: true,
                    class: "h-full w-px bg-border-card",
                }
                span { "Primitives" }
                Separator {
                    orientation: SeparatorOrientation::Vertical,
                    decorative: true,
                    class: "h-full w-px bg-border-card",
                }
                span { "GitHub" }
            }
        }
    }
}

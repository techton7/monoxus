use dioxus::prelude::*;
use monoxus::foundation::overlay::PlacementSide;
use monoxus::tooltip::{
    TooltipArrow, TooltipContent, TooltipPortal, TooltipRoot, TooltipTrigger,
};

#[component]
pub fn TooltipDemo() -> Element {
    let mut copied = use_signal(|| false);

    rsx! {
        div { class: "flex flex-wrap items-center justify-center gap-3",
            TooltipRoot {
                side: PlacementSide::Top,
                side_offset: 8.0,

                TooltipTrigger {
                    class: "inline-flex h-10 items-center justify-center rounded-button border border-border-input bg-background px-4 text-sm font-medium text-foreground shadow-mini transition-colors hover:bg-muted",
                    span {
                        onclick: move |_| copied.set(!copied()),
                        if copied() { "Copied" } else { "Copy snippet" }
                    }
                }
                TooltipPortal {
                    TooltipContent {
                        class: "relative z-50 rounded-input border border-border-card bg-dark px-3 py-1.5 text-xs font-medium text-background shadow-popover",
                        if copied() {
                            "Copied to clipboard"
                        } else {
                            "Copy installation command"
                        }
                        TooltipArrow {
                            class: "fill-dark text-dark",
                        }
                    }
                }
            }

            TooltipRoot {
                side: PlacementSide::Bottom,
                side_offset: 8.0,

                TooltipTrigger {
                    class: "inline-flex h-10 w-10 items-center justify-center rounded-button border border-border-input bg-background text-sm font-medium text-foreground shadow-mini transition-colors hover:bg-muted",
                    "⌘"
                }
                TooltipPortal {
                    TooltipContent {
                        class: "relative z-50 rounded-input border border-border-card bg-dark px-3 py-1.5 text-xs font-medium text-background shadow-popover",
                        "Open command palette (⌘K)"
                        TooltipArrow {
                            class: "fill-dark text-dark",
                        }
                    }
                }
            }
        }
    }
}

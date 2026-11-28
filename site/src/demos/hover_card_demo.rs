use dioxus::prelude::*;
use monoxus::foundation::overlay::PlacementSide;
use monoxus::hover_card::{
    HoverCardArrow, HoverCardContent, HoverCardPortal, HoverCardRoot, HoverCardTrigger,
};

#[component]
pub fn HoverCardDemo() -> Element {
    rsx! {
        div { class: "flex flex-col items-center gap-2",
            p { class: "text-sm text-muted-foreground",
                "Hover or focus "
                HoverCardRoot {
                    open_delay: 200,
                    close_delay: 150,
                    HoverCardTrigger {
                        href: "https://github.com/dioxuslabs",
                        target: "_blank",
                        rel: "noreferrer noopener",
                        class: "font-medium text-foreground underline decoration-border-input underline-offset-4 transition-colors hover:decoration-foreground",
                        "@monoxus"
                    }
                    HoverCardPortal {
                        HoverCardContent {
                            side: PlacementSide::Bottom,
                            side_offset: 8.0,
                            class: "relative z-50 w-72 rounded-card-sm border border-border-card bg-background p-4 text-left shadow-popover outline-none",
                            div { class: "flex items-start justify-between gap-3",
                                div { class: "flex h-9 w-9 shrink-0 items-center justify-center rounded-button bg-dark text-xs font-semibold text-background",
                                    "MX"
                                }
                                div { class: "min-w-0 flex-1 space-y-1",
                                    p { class: "text-sm font-semibold leading-none text-foreground", "monoxus" }
                                    p { class: "text-xs text-muted-foreground", "v0.2.0 · Headless UI for Dioxus" }
                                }
                            }
                            p { class: "mt-3 text-xs leading-relaxed text-muted-foreground",
                                "Unstyled, WAI-ARIA compliant primitives compiled across Web (WASM) and Blitz Native from a single Rust codebase."
                            }
                            div { class: "mt-3 flex items-center gap-4 border-t border-border pt-2.5 text-xs text-muted-foreground",
                                span {
                                    strong { class: "font-medium text-foreground", "35" }
                                    " primitives"
                                }
                                span {
                                    strong { class: "font-medium text-foreground", "MIT/Apache" }
                                }
                            }
                            HoverCardArrow {
                                class: "fill-background text-background",
                            }
                        }
                    }
                }
                " to inspect package metadata."
            }
        }
    }
}

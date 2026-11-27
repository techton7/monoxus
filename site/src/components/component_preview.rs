use dioxus::prelude::*;
use monoxus::foundation::overlay::PlacementSide;
use monoxus::scroll_area::{
    Orientation, ScrollAreaRoot, ScrollAreaScrollbar, ScrollAreaThumb, ScrollAreaType,
    ScrollAreaViewport,
};
use monoxus::tabs::{TabsContent, TabsList, TabsRoot, TabsTrigger};
use monoxus::tooltip::{TooltipContent, TooltipPortal, TooltipRoot, TooltipTrigger};

use crate::components::doc_page_header::schedule_copied_reset;
use crate::components::icons::{CheckIcon, ChevronDownIcon, CodeIcon, CopyIcon};
use crate::demos::{demo_filename, demo_source, render_demo};
use crate::docs_data::{CSS_TOKENS_SNIPPET, HIGHLIGHTED_CSS_TOKENS, highlighted_demo};
use crate::theme::copy_to_clipboard;

/// Live component preview box paired with an expandable syntax-highlighted code panel.
#[component]
pub fn ComponentPreview(slug: String, #[props(default = false)] default_open: bool) -> Element {
    let mut code_open = use_signal(|| default_open);
    let mut copied = use_signal(|| false);
    let active_tab = use_signal(|| "demo".to_string());

    let source = demo_source(&slug);
    let filename = demo_filename(&slug);
    let highlighted_demo = highlighted_demo(&slug);
    let highlighted_css = HIGHLIGHTED_CSS_TOKENS;

    let copy_target = if active_tab() == "css" {
        CSS_TOKENS_SNIPPET
    } else {
        source
    };

    rsx! {
        div {
            class: "my-6 overflow-hidden rounded-card border-2 border-muted bg-background shadow-card",

            // Live Interactive Preview Surface
            div {
                class: "flex min-h-[320px] w-full items-center justify-center bg-zinc-50/80 p-6 sm:p-10 dark:bg-neutral-900/50",
                {render_demo(&slug)}
            }

            // Attached Expandable Code Tabs
            TabsRoot {
                value: Some(active_tab),
                div {
                    class: "border-t border-border-card bg-background",
                    div {
                        class: "flex flex-wrap items-center justify-between gap-2 border-b border-border-card bg-muted/30 px-3 py-2",
                        TabsList {
                            class: "inline-flex items-center gap-1 rounded-button bg-muted/80 p-0.5",
                            TabsTrigger {
                                value: "demo".to_string(),
                                class: "rounded-[4px] px-2.5 py-1 font-mono text-xs font-medium text-foreground-alt transition-colors data-[state=active]:bg-background data-[state=active]:text-foreground data-[state=active]:shadow-mini",
                                "{filename}"
                            }
                            TabsTrigger {
                                value: "css".to_string(),
                                class: "rounded-[4px] px-2.5 py-1 font-mono text-xs font-medium text-foreground-alt transition-colors data-[state=active]:bg-background data-[state=active]:text-foreground data-[state=active]:shadow-mini",
                                "tokens.css"
                            }
                        }

                        div {
                            class: "flex items-center gap-2",
                            TooltipRoot {
                                side: PlacementSide::Top,
                                side_offset: 6.0,
                                span {
                                    class: "inline-flex",
                                    onclick: move |_| {
                                        copy_to_clipboard(copy_target);
                                        copied.set(true);
                                        schedule_copied_reset(copied);
                                    },
                                    TooltipTrigger {
                                        class: "inline-flex h-7 items-center gap-1.5 rounded-button border border-border-input bg-background px-2.5 text-xs font-medium text-foreground-alt transition-colors hover:border-border-input-hover hover:text-foreground",
                                        if copied() {
                                            CheckIcon { class: "h-3.5 w-3.5 text-foreground" }
                                            span { "Copied" }
                                        } else {
                                            CopyIcon { class: "h-3.5 w-3.5" }
                                            span { "Copy" }
                                        }
                                    }
                                }
                                TooltipPortal {
                                    TooltipContent {
                                        class: "relative z-50 rounded-button border border-border-card bg-dark px-2.5 py-1 text-xs font-medium text-background shadow-popover",
                                        if copied() {
                                            "Copied to clipboard"
                                        } else {
                                            "Copy active snippet"
                                        }
                                    }
                                }
                            }

                            button {
                                r#type: "button",
                                onclick: move |_| code_open.toggle(),
                                class: "inline-flex h-7 items-center gap-1.5 rounded-button border border-border-input bg-background px-2.5 text-xs font-medium text-foreground-alt transition-colors hover:border-border-input-hover hover:text-foreground",
                                CodeIcon { class: "h-3.5 w-3.5" }
                                span {
                                    if code_open() { "Collapse code" } else { "Expand code" }
                                }
                                ChevronDownIcon {
                                    class: if code_open() {
                                        "h-3.5 w-3.5 rotate-180 transition-transform duration-150"
                                    } else {
                                        "h-3.5 w-3.5 transition-transform duration-150"
                                    },
                                }
                            }
                        }
                    }

                    div {
                        class: if code_open() {
                            "relative max-h-none bg-zinc-50/60 dark:bg-neutral-900/70"
                        } else {
                            "relative max-h-[236px] overflow-hidden bg-zinc-50/60 dark:bg-neutral-900/70"
                        },
                        TabsContent {
                            value: "demo".to_string(),
                            class: "outline-hidden",
                            ScrollAreaRoot {
                                r#type: ScrollAreaType::Hover,
                                class: "w-full",
                                ScrollAreaViewport {
                                    class: "w-full",
                                    pre {
                                        class: "p-4 font-mono text-[13px] leading-relaxed text-foreground",
                                        code {
                                            dangerous_inner_html: "{highlighted_demo}",
                                        }
                                    }
                                }
                                ScrollAreaScrollbar {
                                    orientation: Orientation::Horizontal,
                                    class: "flex h-2.5 touch-none select-none p-0.5 opacity-0 transition-opacity duration-150 data-[state=visible]:opacity-100",
                                    ScrollAreaThumb {
                                        class: "relative flex-1 rounded-full bg-dark-40 hover:bg-foreground/60",
                                    }
                                }
                            }
                        }
                        TabsContent {
                            value: "css".to_string(),
                            class: "outline-hidden",
                            ScrollAreaRoot {
                                r#type: ScrollAreaType::Hover,
                                class: "w-full",
                                ScrollAreaViewport {
                                    class: "w-full",
                                    pre {
                                        class: "p-4 font-mono text-[13px] leading-relaxed text-foreground",
                                        code {
                                            dangerous_inner_html: "{highlighted_css}",
                                        }
                                    }
                                }
                                ScrollAreaScrollbar {
                                    orientation: Orientation::Horizontal,
                                    class: "flex h-2.5 touch-none select-none p-0.5 opacity-0 transition-opacity duration-150 data-[state=visible]:opacity-100",
                                    ScrollAreaThumb {
                                        class: "relative flex-1 rounded-full bg-dark-40 hover:bg-foreground/60",
                                    }
                                }
                            }
                        }
                        if !code_open() {
                            div {
                                class: "pointer-events-none absolute inset-x-0 bottom-0 h-20 bg-gradient-to-t from-zinc-50 via-zinc-50/80 to-transparent dark:from-neutral-900 dark:via-neutral-900/80",
                            }
                            div {
                                class: "absolute inset-x-0 bottom-3 flex justify-center",
                                button {
                                    r#type: "button",
                                    onclick: move |_| code_open.set(true),
                                    class: "inline-flex h-8 items-center gap-1.5 rounded-button border border-border-input bg-background/95 px-3 text-xs font-medium text-foreground shadow-mini backdrop-blur-xs transition-colors hover:border-border-input-hover hover:bg-muted",
                                    span { "Expand code" }
                                    ChevronDownIcon { class: "h-3.5 w-3.5" }
                                }
                            }
                        }
                    }
                }
            }
        }
    }
}

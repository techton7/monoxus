use dioxus::prelude::*;
use monoxus::button::Button;
use monoxus::dialog::{
    DialogClose, DialogContent, DialogDescription, DialogOverlay, DialogPortal, DialogRoot,
    DialogTitle, DialogTrigger,
};
use monoxus::foundation::overlay::PlacementSide;
use monoxus::toggle::Toggle;
use monoxus::tooltip::{TooltipContent, TooltipPortal, TooltipRoot, TooltipTrigger};

use crate::components::icons::{
    CloseIcon, GithubIcon, MenuIcon, MonoxusMark, MoonIcon, SearchIcon, SunIcon,
};
use crate::components::search_dialog::SearchDialog;
use crate::components::sidebar_nav::SidebarNavList;
use crate::routes::Route;
use crate::theme::use_theme;

/// Sticky monochrome header shared by the landing page and documentation layout.
#[component]
pub fn SiteHeader() -> Element {
    let mut theme = use_theme();
    let is_dark = theme.is_dark();
    let mut search_open = use_signal(|| false);
    let mut mobile_open = use_signal(|| false);

    rsx! {
        header {
            class: "sticky top-0 z-40 h-[71px] w-full border-b border-border-card bg-background/85 backdrop-blur-md",
            div {
                class: "mx-auto flex h-full max-w-[1440px] items-center justify-between gap-4 px-4 sm:px-6 lg:px-8",

                // Left: Brand + Primary Navigation
                div {
                    class: "flex items-center gap-6",
                    div {
                        class: "flex items-center gap-2.5",
                        Link {
                            to: Route::Home {},
                            class: "inline-flex items-center gap-2.5 text-foreground transition-opacity hover:opacity-85",
                            MonoxusMark { class: "h-6 w-6" }
                            span {
                                class: "text-[15px] font-semibold tracking-tight",
                                "Monoxus"
                            }
                        }
                        TooltipRoot {
                            side: PlacementSide::Bottom,
                            side_offset: 8.0,
                            TooltipTrigger {
                                class: "inline-flex items-center rounded-full border border-border-card bg-muted/80 px-2 py-0.5 font-mono text-[11px] font-medium text-foreground-alt transition-colors hover:border-border-input-hover hover:text-foreground",
                                "v0.1.2"
                            }
                            TooltipPortal {
                                TooltipContent {
                                    class: "relative z-50 rounded-button border border-border-card bg-background px-2.5 py-1.5 text-xs font-medium text-foreground shadow-popover",
                                    "Dioxus 0.8 · Web + Blitz Native"
                                }
                            }
                        }
                    }

                    nav {
                        aria_label: "Primary",
                        class: "hidden items-center gap-5 text-sm font-medium md:flex",
                        Link {
                            to: Route::OverviewDoc { slug: "introduction".to_string() },
                            class: "text-foreground-alt transition-colors hover:text-foreground",
                            "Docs"
                        }
                        Link {
                            to: Route::ComponentDocPage { slug: "button".to_string() },
                            class: "text-foreground-alt transition-colors hover:text-foreground",
                            "Components"
                        }
                    }
                }

                // Right: Search, Theme Toggle, GitHub, Mobile Drawer
                div {
                    class: "flex items-center gap-2",
                    Button {
                        class: "hidden h-9 items-center gap-2.5 rounded-input border border-border-input bg-background-alt px-3 text-xs font-medium text-foreground-alt shadow-mini transition-colors hover:border-border-input-hover hover:text-foreground sm:inline-flex",
                        aria_label: Some("Search documentation".to_string()),
                        on_click: move |_| search_open.set(true),
                        SearchIcon { class: "h-3.5 w-3.5 text-muted-foreground" }
                        span { "Search docs..." }
                        kbd {
                            class: "ml-2 rounded border border-border-input bg-muted/70 px-1.5 py-0.5 font-mono text-[10px] font-medium text-foreground-alt shadow-kbd",
                            "⌘K"
                        }
                    }

                    Button {
                        class: "inline-flex h-9 w-9 items-center justify-center rounded-input border border-border-input bg-background-alt text-foreground-alt shadow-mini transition-colors hover:border-border-input-hover hover:text-foreground sm:hidden",
                        aria_label: Some("Search documentation".to_string()),
                        on_click: move |_| search_open.set(true),
                        SearchIcon { class: "h-4 w-4" }
                    }

                    Toggle {
                        pressed: Some(is_dark),
                        on_pressed_change: move |_| theme.toggle(),
                        aria_label: if is_dark { "Switch to light mode" } else { "Switch to dark mode" },
                        title: if is_dark { "Switch to light mode" } else { "Switch to dark mode" },
                        class: "inline-flex h-9 w-9 items-center justify-center rounded-input border border-border-input bg-background-alt text-foreground-alt shadow-mini transition-colors hover:border-border-input-hover hover:text-foreground",
                        if is_dark {
                            SunIcon { class: "h-4 w-4" }
                        } else {
                            MoonIcon { class: "h-4 w-4" }
                        }
                    }

                    Button {
                        href: Some("https://github.com/techton7/monoxus".to_string()),
                        target: Some("_blank".to_string()),
                        rel: Some("noreferrer".to_string()),
                        aria_label: Some("Monoxus on GitHub".to_string()),
                        class: "inline-flex h-9 w-9 items-center justify-center rounded-input border border-border-input bg-background-alt text-foreground-alt shadow-mini transition-colors hover:border-border-input-hover hover:text-foreground",
                        GithubIcon { class: "h-4 w-4" }
                    }

                    // Mobile navigation drawer
                    div {
                        class: "lg:hidden",
                        DialogRoot {
                            open: Some(mobile_open),
                            DialogTrigger {
                                class: "inline-flex h-9 w-9 items-center justify-center rounded-input border border-border-input bg-background-alt text-foreground-alt shadow-mini transition-colors hover:border-border-input-hover hover:text-foreground",
                                span { class: "sr-only", "Open navigation menu" }
                                MenuIcon { class: "h-4 w-4" }
                            }
                            DialogPortal {
                                DialogOverlay {
                                    class: "fixed inset-0 z-50 bg-black/50 backdrop-blur-[2px] animate-fade-in",
                                }
                                DialogContent {
                                    class: "fixed inset-y-0 left-0 z-50 flex w-[min(85vw,20rem)] flex-col border-r border-border-card bg-background p-5 shadow-popover outline-hidden",
                                    div {
                                        class: "mb-5 flex items-center justify-between border-b border-border-card pb-4",
                                        DialogTitle {
                                            class: "flex items-center gap-2 text-sm font-semibold text-foreground",
                                            MonoxusMark { class: "h-5 w-5" }
                                            "Monoxus Docs"
                                        }
                                        DialogDescription {
                                            class: "sr-only",
                                            "Navigate Monoxus documentation sections and components."
                                        }
                                        DialogClose {
                                            class: "inline-flex h-8 w-8 items-center justify-center rounded-button text-foreground-alt hover:bg-muted hover:text-foreground",
                                            span { class: "sr-only", "Close navigation menu" }
                                            CloseIcon { class: "h-4 w-4" }
                                        }
                                    }

                                    div {
                                        class: "mb-4 flex flex-wrap gap-2 border-b border-border-card pb-4 text-xs font-medium",
                                        Link {
                                            to: Route::OverviewDoc { slug: "introduction".to_string() },
                                            onclick: move |_| mobile_open.set(false),
                                            class: "rounded-button border border-border-card px-2.5 py-1.5 text-foreground hover:bg-muted",
                                            "Docs"
                                        }
                                        Link {
                                            to: Route::ComponentDocPage { slug: "button".to_string() },
                                            onclick: move |_| mobile_open.set(false),
                                            class: "rounded-button border border-border-card px-2.5 py-1.5 text-foreground hover:bg-muted",
                                            "Components"
                                        }
                                    }

                                    div {
                                        class: "min-h-0 flex-1 overflow-y-auto pr-1",
                                        SidebarNavList {
                                            on_navigate: move |_| mobile_open.set(false),
                                        }
                                    }
                                }
                            }
                        }
                    }
                }
            }
        }
        SearchDialog { open: search_open }
    }
}

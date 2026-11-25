use dioxus::prelude::*;
use monoxus::separator::{SeparatorOrientation, SeparatorRoot};

use crate::components::icons::{ChevronLeftIcon, ChevronRightIcon};
use crate::components::sidebar_nav::SidebarNav;
use crate::components::site_header::SiteHeader;
use crate::docs_data::{TocItem, prev_next_nav};
use crate::routes::Route;

/// Outer 3-column documentation layout (`SiteHeader` + `SidebarNav` + routed page outlet).
#[component]
pub fn DocsLayout() -> Element {
    rsx! {
        div {
            class: "min-h-screen bg-background text-foreground",
            SiteHeader {}
            div {
                class: "mx-auto flex w-full max-w-[1440px] px-4 sm:px-6 lg:px-8",
                SidebarNav {}
                main {
                    class: "min-w-0 flex-1 py-8 lg:pl-8",
                    Outlet::<Route> {}
                }
            }
        }
    }
}

/// Page wrapper pairing main documentation content + bottom pager with the sticky right TOC.
#[component]
pub fn DocPageShell(toc: Vec<TocItem>, current_path: String, children: Element) -> Element {
    rsx! {
        div {
            class: "grid grid-cols-1 gap-10 xl:grid-cols-[minmax(0,1fr)_220px]",
            article {
                class: "min-w-0 max-w-[780px]",
                {children}
                DocsPager { current_path }
            }
            aside {
                aria_label: "On this page",
                class: "hidden xl:block",
                if !toc.is_empty() {
                    div {
                        class: "sticky top-[103px] max-h-[calc(100vh-120px)] overflow-y-auto",
                        h4 {
                            class: "mb-3 text-xs font-semibold tracking-wider text-foreground uppercase",
                            "On this page"
                        }
                        ul {
                            class: "space-y-1.5 border-l border-border-card text-[13px]",
                            for item in toc.iter() {
                                {
                                    let pad = if item.level >= 3 { "pl-6" } else { "pl-3.5" };
                                    let href = format!("#{}", item.id);
                                    rsx! {
                                        li {
                                            key: "{item.id}",
                                            a {
                                                href: "{href}",
                                                class: "block py-0.5 text-foreground-alt transition-colors hover:text-foreground {pad}",
                                                "{item.title}"
                                            }
                                        }
                                    }
                                }
                            }
                        }
                    }
                }
            }
        }
    }
}

/// Previous / Next navigation cards rendered at the bottom of each documentation page.
#[component]
pub fn DocsPager(current_path: String) -> Element {
    let (prev, next) = prev_next_nav(&current_path);
    if prev.is_none() && next.is_none() {
        return rsx! {};
    }

    rsx! {
        div {
            class: "mt-14 space-y-6",
            SeparatorRoot {
                orientation: SeparatorOrientation::Horizontal,
                decorative: true,
                class: "h-px w-full bg-border-card",
            }
            div {
                class: "grid grid-cols-1 gap-4 sm:grid-cols-2",
                if let Some(prev_item) = prev {
                    Link {
                        to: prev_item.path,
                        class: "group flex items-center gap-3 rounded-card-sm border border-border-card bg-background-alt p-4 transition-colors hover:border-border-input-hover hover:bg-muted/40",
                        ChevronLeftIcon { class: "h-4 w-4 shrink-0 text-muted-foreground transition-transform group-hover:-translate-x-0.5 group-hover:text-foreground" }
                        div {
                            class: "min-w-0",
                            div { class: "text-xs text-muted-foreground", "Previous" }
                            div { class: "truncate text-sm font-medium text-foreground", "{prev_item.title}" }
                        }
                    }
                } else {
                    div {}
                }
                if let Some(next_item) = next {
                    Link {
                        to: next_item.path,
                        class: "group flex items-center justify-end gap-3 rounded-card-sm border border-border-card bg-background-alt p-4 text-right transition-colors hover:border-border-input-hover hover:bg-muted/40",
                        div {
                            class: "min-w-0",
                            div { class: "text-xs text-muted-foreground", "Next" }
                            div { class: "truncate text-sm font-medium text-foreground", "{next_item.title}" }
                        }
                        ChevronRightIcon { class: "h-4 w-4 shrink-0 text-muted-foreground transition-transform group-hover:translate-x-0.5 group-hover:text-foreground" }
                    }
                }
            }
        }
    }
}

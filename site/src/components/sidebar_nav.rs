use dioxus::prelude::*;
use monoxus::foundation::shared::Orientation;
use monoxus::scroll_area::{
    ScrollAreaRoot, ScrollAreaScrollbar, ScrollAreaThumb, ScrollAreaType, ScrollAreaViewport,
};

use crate::docs_data::sidebar_groups;
use crate::routes::Route;

/// Sticky left documentation sidebar (`240px`) on desktop viewports.
#[component]
pub fn SidebarNav() -> Element {
    rsx! {
        aside {
            aria_label: "Documentation navigation",
            class: "sticky top-[71px] hidden h-[calc(100vh-71px)] w-[240px] shrink-0 self-start lg:block",
            // TODO(monoxus): remove once auto-ID & wrapper z-index hoisting land
            ScrollAreaRoot {
                id: Some("docs-sidebar-scroll-area".into()),
                r#type: ScrollAreaType::Hover,
                class: "h-full w-full overflow-hidden border-r border-border-card pr-3",
                ScrollAreaViewport {
                    class: "h-full w-full py-6",
                    SidebarNavList {}
                }
                ScrollAreaScrollbar {
                    orientation: Orientation::Vertical,
                    class: "flex w-2 touch-none select-none p-0.5 transition-colors",
                    ScrollAreaThumb {
                        class: "relative flex-1 rounded-full bg-dark-10 hover:bg-dark-40",
                    }
                }
            }
        }
    }
}

/// Shared grouped sidebar navigation list used by both desktop sidebar and mobile drawer.
#[component]
pub fn SidebarNavList(#[props(default)] on_navigate: Option<EventHandler<()>>) -> Element {
    let current_route = use_route::<Route>().to_string();
    let groups = sidebar_groups();

    rsx! {
        nav {
            class: "space-y-6",
            for group in groups.iter() {
                div {
                    key: "{group.title}",
                    class: "space-y-1.5",
                    h4 {
                        class: "px-2.5 text-xs font-semibold tracking-wider text-muted-foreground uppercase",
                        "{group.title}"
                    }
                    ul {
                        class: "space-y-0.5",
                        for item in group.items.iter() {
                            {
                                let is_active = current_route == item.path;
                                let link_cls = if is_active {
                                    "flex items-center justify-between rounded-button px-2.5 py-1.5 text-sm transition-colors"
                                } else {
                                    "flex items-center justify-between rounded-button px-2.5 py-1.5 text-sm text-foreground-alt transition-colors hover:bg-muted/60 hover:text-foreground aria-[current=page]:text-foreground"
                                };
                                let path = item.path;
                                rsx! {
                                    li {
                                        key: "{item.path}",
                                        Link {
                                            to: path,
                                            class: "{link_cls}",
                                            active_class: "bg-muted font-medium text-foreground",
                                            onclick: move |_| {
                                                if let Some(cb) = on_navigate {
                                                    cb.call(());
                                                }
                                            },
                                            span { "{item.title}" }
                                            if let Some(badge) = item.badge {
                                                span {
                                                    class: "rounded-full border border-border-card bg-background px-1.5 py-0.5 text-[10px] font-medium text-foreground-alt",
                                                    "{badge}"
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
}

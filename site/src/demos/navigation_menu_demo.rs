use dioxus::prelude::*;
use monoxus::navigation_menu::{
    NavigationMenuContent, NavigationMenuIndicator, NavigationMenuItem, NavigationMenuLink,
    NavigationMenuList, NavigationMenuRoot, NavigationMenuTrigger, NavigationMenuViewport,
};

#[component]
pub fn NavigationMenuDemo() -> Element {
    let mut selected_link = use_signal(|| String::from("Overview"));

    rsx! {
        div { class: "flex flex-col items-center gap-4",
            NavigationMenuRoot {
                class: "relative z-20 flex flex-col items-center",

                NavigationMenuList {
                    class: "flex list-none items-center gap-1 rounded-card-sm border border-border-card bg-background p-1 shadow-mini",

                    NavigationMenuItem {
                        value: "primitives",
                        NavigationMenuTrigger {
                            class: "inline-flex items-center gap-1.5 rounded-button px-3 py-1.5 text-sm font-medium text-foreground transition-colors hover:bg-muted data-[state=open]:bg-muted",
                            span { "Primitives" }
                            span { class: "text-[10px] text-muted-foreground", "▾" }
                        }
                        NavigationMenuContent {
                            class: "w-72 p-2",
                            ul { class: "grid list-none gap-1",
                                li {
                                    NavigationMenuLink {
                                        href: "#dialog",
                                        active: selected_link() == "Dialog",
                                        on_click: move |evt: MouseEvent| {
                                            evt.prevent_default();
                                            selected_link.set(String::from("Dialog"));
                                        },
                                        class: "block rounded-button p-2.5 transition-colors hover:bg-muted data-[active]:bg-muted",
                                        div { class: "text-sm font-medium text-foreground", "Dialog" }
                                        p { class: "text-xs text-muted-foreground", "Modal overlays with focus trap and scroll lock." }
                                    }
                                }
                                li {
                                    NavigationMenuLink {
                                        href: "#combobox",
                                        active: selected_link() == "Combobox",
                                        on_click: move |evt: MouseEvent| {
                                            evt.prevent_default();
                                            selected_link.set(String::from("Combobox"));
                                        },
                                        class: "block rounded-button p-2.5 transition-colors hover:bg-muted data-[active]:bg-muted",
                                        div { class: "text-sm font-medium text-foreground", "Combobox" }
                                        p { class: "text-xs text-muted-foreground", "Filterable single and multi-select input." }
                                    }
                                }
                            }
                        }
                    }

                    NavigationMenuItem {
                        value: "guides",
                        NavigationMenuTrigger {
                            class: "inline-flex items-center gap-1.5 rounded-button px-3 py-1.5 text-sm font-medium text-foreground transition-colors hover:bg-muted data-[state=open]:bg-muted",
                            span { "Guides" }
                            span { class: "text-[10px] text-muted-foreground", "▾" }
                        }
                        NavigationMenuContent {
                            class: "w-72 p-2",
                            ul { class: "grid list-none gap-1",
                                li {
                                    NavigationMenuLink {
                                        href: "#styling",
                                        active: selected_link() == "Styling",
                                        on_click: move |evt: MouseEvent| {
                                            evt.prevent_default();
                                            selected_link.set(String::from("Styling"));
                                        },
                                        class: "block rounded-button p-2.5 transition-colors hover:bg-muted data-[active]:bg-muted",
                                        div { class: "text-sm font-medium text-foreground", "Styling" }
                                        p { class: "text-xs text-muted-foreground", "Style via data-* states and CSS custom properties." }
                                    }
                                }
                                li {
                                    NavigationMenuLink {
                                        href: "#accessibility",
                                        active: selected_link() == "Accessibility",
                                        on_click: move |evt: MouseEvent| {
                                            evt.prevent_default();
                                            selected_link.set(String::from("Accessibility"));
                                        },
                                        class: "block rounded-button p-2.5 transition-colors hover:bg-muted data-[active]:bg-muted",
                                        div { class: "text-sm font-medium text-foreground", "Accessibility" }
                                        p { class: "text-xs text-muted-foreground", "WAI-ARIA roles, roving focus, and keyboard support." }
                                    }
                                }
                            }
                        }
                    }

                    NavigationMenuItem {
                        value: "overview",
                        NavigationMenuLink {
                            href: "#overview",
                            active: selected_link() == "Overview",
                            on_click: move |evt: MouseEvent| {
                                evt.prevent_default();
                                selected_link.set(String::from("Overview"));
                            },
                            class: "inline-flex items-center rounded-button px-3 py-1.5 text-sm font-medium text-foreground transition-colors hover:bg-muted data-[active]:bg-muted",
                            "Overview"
                        }
                    }

                    NavigationMenuIndicator {
                        class: "top-full z-10 flex h-2 items-end justify-center overflow-hidden",
                        div { class: "relative top-[60%] h-2 w-2 rotate-45 rounded-tl-sm border-l border-t border-border-card bg-background" }
                    }
                }

                div { class: "absolute left-0 top-full flex w-full justify-center pt-2",
                    NavigationMenuViewport {
                        class: "relative overflow-hidden rounded-card-sm border border-border-card bg-background shadow-popover",
                    }
                }
            }

            p { class: "text-xs text-muted-foreground",
                "Active link: "
                span { class: "font-medium text-foreground", "{selected_link()}" }
            }
        }
    }
}

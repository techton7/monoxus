use dioxus::prelude::*;
use monoxus::menu::CheckedState;
use monoxus::menubar::{
    MenubarCheckboxItem, MenubarContent, MenubarItem, MenubarItemIndicator, MenubarMenu,
    MenubarPortal, MenubarRadioGroup, MenubarRadioItem, MenubarRoot, MenubarSeparator, MenubarSub,
    MenubarSubContent, MenubarSubTrigger, MenubarTrigger,
};

#[component]
pub fn MenubarDemo() -> Element {
    let mut show_sidebar = use_signal(|| CheckedState::Checked);
    let mut zoom = use_signal(|| String::from("100%"));
    let mut status = use_signal(|| String::from("Click a menu or use Arrow keys"));

    rsx! {
        div { class: "flex flex-col items-center gap-3",
            MenubarRoot {
                class: "inline-flex items-center gap-1 rounded-button border border-border-input bg-background p-1 shadow-mini",

                MenubarMenu { value: "file",
                    MenubarTrigger {
                        class: "inline-flex h-8 cursor-default select-none items-center rounded-input px-3 text-sm font-medium text-foreground outline-none transition-colors hover:bg-muted data-[highlighted]:bg-muted data-[state=open]:bg-muted",
                        "File"
                    }
                    MenubarPortal {
                        MenubarContent {
                            side_offset: 6.0,
                            class: "z-50 min-w-[190px] rounded-card-sm border border-border-card bg-background p-1 text-sm text-foreground shadow-popover outline-none",
                            MenubarItem {
                                class: "flex cursor-default select-none items-center justify-between rounded-button px-2.5 py-1.5 text-sm outline-none data-[highlighted]:bg-muted",
                                on_select: move |_| status.set("File -> New Tab".into()),
                                span { "New Tab" }
                                span { class: "text-xs text-muted-foreground", "⌘T" }
                            }
                            MenubarItem {
                                class: "flex cursor-default select-none items-center justify-between rounded-button px-2.5 py-1.5 text-sm outline-none data-[highlighted]:bg-muted",
                                on_select: move |_| status.set("File -> Save".into()),
                                span { "Save" }
                                span { class: "text-xs text-muted-foreground", "⌘S" }
                            }
                            MenubarSeparator { class: "my-1 h-px bg-border" }
                            MenubarSub {
                                MenubarSubTrigger {
                                    class: "flex cursor-default select-none items-center justify-between rounded-button px-2.5 py-1.5 text-sm outline-none data-[highlighted]:bg-muted data-[state=open]:bg-muted",
                                    span { "Share" }
                                    span { class: "text-xs text-muted-foreground", "›" }
                                }
                                MenubarPortal {
                                    MenubarSubContent {
                                        side_offset: 4.0,
                                        class: "z-50 min-w-[150px] rounded-card-sm border border-border-card bg-background p-1 text-sm text-foreground shadow-popover outline-none",
                                        MenubarItem {
                                            class: "flex cursor-default select-none items-center rounded-button px-2.5 py-1.5 text-sm outline-none data-[highlighted]:bg-muted",
                                            on_select: move |_| status.set("Shared via Email".into()),
                                            "Email Link"
                                        }
                                        MenubarItem {
                                            class: "flex cursor-default select-none items-center rounded-button px-2.5 py-1.5 text-sm outline-none data-[highlighted]:bg-muted",
                                            on_select: move |_| status.set("Copied Markdown".into()),
                                            "Copy Markdown"
                                        }
                                    }
                                }
                            }
                        }
                    }
                }

                MenubarMenu { value: "edit",
                    MenubarTrigger {
                        class: "inline-flex h-8 cursor-default select-none items-center rounded-input px-3 text-sm font-medium text-foreground outline-none transition-colors hover:bg-muted data-[highlighted]:bg-muted data-[state=open]:bg-muted",
                        "Edit"
                    }
                    MenubarPortal {
                        MenubarContent {
                            side_offset: 6.0,
                            class: "z-50 min-w-[180px] rounded-card-sm border border-border-card bg-background p-1 text-sm text-foreground shadow-popover outline-none",
                            MenubarItem {
                                class: "flex cursor-default select-none items-center justify-between rounded-button px-2.5 py-1.5 text-sm outline-none data-[highlighted]:bg-muted",
                                on_select: move |_| status.set("Edit -> Undo".into()),
                                span { "Undo" }
                                span { class: "text-xs text-muted-foreground", "⌘Z" }
                            }
                            MenubarItem {
                                class: "flex cursor-default select-none items-center justify-between rounded-button px-2.5 py-1.5 text-sm outline-none data-[highlighted]:bg-muted",
                                on_select: move |_| status.set("Edit -> Redo".into()),
                                span { "Redo" }
                                span { class: "text-xs text-muted-foreground", "⇧⌘Z" }
                            }
                            MenubarSeparator { class: "my-1 h-px bg-border" }
                            MenubarItem {
                                class: "flex cursor-default select-none items-center justify-between rounded-button px-2.5 py-1.5 text-sm outline-none data-[highlighted]:bg-muted",
                                on_select: move |_| status.set("Edit -> Find".into()),
                                span { "Find" }
                                span { class: "text-xs text-muted-foreground", "⌘F" }
                            }
                        }
                    }
                }

                MenubarMenu { value: "view",
                    MenubarTrigger {
                        class: "inline-flex h-8 cursor-default select-none items-center rounded-input px-3 text-sm font-medium text-foreground outline-none transition-colors hover:bg-muted data-[highlighted]:bg-muted data-[state=open]:bg-muted",
                        "View"
                    }
                    MenubarPortal {
                        MenubarContent {
                            side_offset: 6.0,
                            class: "z-50 min-w-[190px] rounded-card-sm border border-border-card bg-background p-1 text-sm text-foreground shadow-popover outline-none",
                            MenubarCheckboxItem {
                                checked: Some(show_sidebar()),
                                on_checked_change: move |next| show_sidebar.set(next),
                                close_on_select: false,
                                class: "flex cursor-default select-none items-center gap-2 rounded-button px-2.5 py-1.5 text-sm outline-none data-[highlighted]:bg-muted",
                                span { class: "inline-flex w-4 items-center justify-center text-xs",
                                    MenubarItemIndicator { "✓" }
                                }
                                "Show Sidebar"
                            }
                            MenubarSeparator { class: "my-1 h-px bg-border" }
                            MenubarRadioGroup {
                                value: Some(zoom()),
                                on_value_change: move |next| {
                                    status.set(format!("Zoom set to {next}"));
                                    zoom.set(next);
                                },
                                MenubarRadioItem {
                                    value: "90%",
                                    class: "flex cursor-default select-none items-center gap-2 rounded-button px-2.5 py-1.5 text-sm outline-none data-[highlighted]:bg-muted",
                                    span { class: "inline-flex w-4 items-center justify-center text-xs",
                                        MenubarItemIndicator { "•" }
                                    }
                                    "90% Zoom"
                                }
                                MenubarRadioItem {
                                    value: "100%",
                                    class: "flex cursor-default select-none items-center gap-2 rounded-button px-2.5 py-1.5 text-sm outline-none data-[highlighted]:bg-muted",
                                    span { class: "inline-flex w-4 items-center justify-center text-xs",
                                        MenubarItemIndicator { "•" }
                                    }
                                    "100% Zoom"
                                }
                                MenubarRadioItem {
                                    value: "125%",
                                    class: "flex cursor-default select-none items-center gap-2 rounded-button px-2.5 py-1.5 text-sm outline-none data-[highlighted]:bg-muted",
                                    span { class: "inline-flex w-4 items-center justify-center text-xs",
                                        MenubarItemIndicator { "•" }
                                    }
                                    "125% Zoom"
                                }
                            }
                        }
                    }
                }
            }

            p { class: "text-xs text-muted-foreground",
                "{status} · Zoom: "
                span { class: "font-medium text-foreground", "{zoom}" }
            }
        }
    }
}

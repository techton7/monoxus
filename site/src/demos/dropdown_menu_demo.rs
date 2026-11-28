use dioxus::prelude::*;
use monoxus::dropdown_menu::{
    DropdownMenuCheckboxItem, DropdownMenuContent, DropdownMenuGroup, DropdownMenuGroupHeading,
    DropdownMenuItem, DropdownMenuItemIndicator, DropdownMenuPortal, DropdownMenuRadioGroup,
    DropdownMenuRadioItem, DropdownMenuRoot, DropdownMenuSeparator, DropdownMenuSub,
    DropdownMenuSubContent, DropdownMenuSubTrigger, DropdownMenuTrigger,
};
use monoxus::foundation::overlay::{PlacementAlign, PlacementSide};
use monoxus::menu::CheckedState;

#[component]
pub fn DropdownMenuDemo() -> Element {
    let mut show_minimap = use_signal(|| CheckedState::Checked);
    let mut sort_by = use_signal(|| String::from("updated"));
    let mut last_event = use_signal(|| String::from("No command selected"));

    rsx! {
        div { class: "flex flex-col items-center gap-3",
            DropdownMenuRoot {
                DropdownMenuTrigger {
                    class: "inline-flex h-10 items-center gap-2 rounded-button border border-border-input bg-background px-4 text-sm font-medium text-foreground shadow-mini transition-colors hover:bg-muted data-[state=open]:bg-muted",
                    span { "Workspace" }
                    span { class: "text-xs text-muted-foreground", "▾" }
                }

                DropdownMenuPortal {
                    DropdownMenuContent {
                        side: PlacementSide::Bottom,
                        align: PlacementAlign::Start,
                        side_offset: 6.0,
                        class: "z-50 min-w-[220px] rounded-card-sm border border-border-card bg-background p-1 text-sm text-foreground shadow-popover outline-none",

                        DropdownMenuGroup {
                            DropdownMenuGroupHeading {
                                class: "px-2.5 py-1 text-[11px] font-medium uppercase tracking-wider text-muted-foreground",
                                "Actions"
                            }
                            DropdownMenuItem {
                                class: "flex cursor-default select-none items-center justify-between rounded-button px-2.5 py-1.5 text-sm outline-none data-[highlighted]:bg-muted",
                                on_select: move |_| last_event.set("Created new branch".into()),
                                span { "New branch" }
                                span { class: "text-xs text-muted-foreground", "⌘B" }
                            }
                            DropdownMenuItem {
                                class: "flex cursor-default select-none items-center justify-between rounded-button px-2.5 py-1.5 text-sm outline-none data-[highlighted]:bg-muted",
                                on_select: move |_| last_event.set("Opened settings".into()),
                                span { "Settings" }
                                span { class: "text-xs text-muted-foreground", "⌘," }
                            }
                            DropdownMenuItem {
                                disabled: true,
                                class: "flex cursor-default select-none items-center justify-between rounded-button px-2.5 py-1.5 text-sm outline-none data-[disabled]:pointer-events-none data-[disabled]:opacity-40",
                                span { "Archive workspace" }
                            }
                        }

                        DropdownMenuSeparator { class: "my-1 h-px bg-border" }

                        DropdownMenuSub {
                            DropdownMenuSubTrigger {
                                class: "flex cursor-default select-none items-center justify-between rounded-button px-2.5 py-1.5 text-sm outline-none data-[highlighted]:bg-muted data-[state=open]:bg-muted",
                                span { "Invite members" }
                                span { class: "text-xs text-muted-foreground", "›" }
                            }
                            DropdownMenuPortal {
                                DropdownMenuSubContent {
                                    side_offset: 4.0,
                                    class: "z-50 min-w-[165px] rounded-card-sm border border-border-card bg-background p-1 text-sm text-foreground shadow-popover outline-none",
                                    DropdownMenuItem {
                                        class: "flex cursor-default select-none items-center rounded-button px-2.5 py-1.5 text-sm outline-none data-[highlighted]:bg-muted",
                                        on_select: move |_| last_event.set("Copied invite link".into()),
                                        "Copy invite link"
                                    }
                                    DropdownMenuItem {
                                        class: "flex cursor-default select-none items-center rounded-button px-2.5 py-1.5 text-sm outline-none data-[highlighted]:bg-muted",
                                        on_select: move |_| last_event.set("Sent email invite".into()),
                                        "Send via email"
                                    }
                                }
                            }
                        }

                        DropdownMenuSeparator { class: "my-1 h-px bg-border" }

                        DropdownMenuCheckboxItem {
                            checked: Some(show_minimap()),
                            on_checked_change: move |next| show_minimap.set(next),
                            close_on_select: false,
                            class: "flex cursor-default select-none items-center gap-2 rounded-button px-2.5 py-1.5 text-sm outline-none data-[highlighted]:bg-muted",
                            span { class: "inline-flex w-4 items-center justify-center text-xs",
                                DropdownMenuItemIndicator { "✓" }
                            }
                            "Show minimap"
                        }

                        DropdownMenuSeparator { class: "my-1 h-px bg-border" }

                        DropdownMenuRadioGroup {
                            value: Some(sort_by()),
                            on_value_change: move |next| {
                                last_event.set(format!("Sorted by {next}"));
                                sort_by.set(next);
                            },
                            DropdownMenuRadioItem {
                                value: "updated",
                                class: "flex cursor-default select-none items-center gap-2 rounded-button px-2.5 py-1.5 text-sm outline-none data-[highlighted]:bg-muted",
                                span { class: "inline-flex w-4 items-center justify-center text-xs",
                                    DropdownMenuItemIndicator { "•" }
                                }
                                "Last updated"
                            }
                            DropdownMenuRadioItem {
                                value: "name",
                                class: "flex cursor-default select-none items-center gap-2 rounded-button px-2.5 py-1.5 text-sm outline-none data-[highlighted]:bg-muted",
                                span { class: "inline-flex w-4 items-center justify-center text-xs",
                                    DropdownMenuItemIndicator { "•" }
                                }
                                "Alphabetical"
                            }
                        }
                    }
                }
            }

            p { class: "text-xs text-muted-foreground",
                "{last_event} · Sort: "
                span { class: "font-medium text-foreground", "{sort_by}" }
            }
        }
    }
}

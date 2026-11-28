use dioxus::prelude::*;
use monoxus::context_menu::{
    ContextMenuCheckboxItem, ContextMenuContent, ContextMenuItem, ContextMenuItemIndicator,
    ContextMenuPortal, ContextMenuRadioGroup, ContextMenuRadioItem, ContextMenuRoot,
    ContextMenuSeparator, ContextMenuSub, ContextMenuSubContent, ContextMenuSubTrigger,
    ContextMenuTrigger,
};
use monoxus::menu::CheckedState;

#[component]
pub fn ContextMenuDemo() -> Element {
    let mut show_grid = use_signal(|| CheckedState::Checked);
    let mut snap_guides = use_signal(|| CheckedState::Unchecked);
    let mut density = use_signal(|| String::from("comfortable"));
    let mut last_action = use_signal(|| String::from("Right-click the canvas below"));

    rsx! {
        div { class: "flex w-full max-w-md flex-col items-center gap-3",
            ContextMenuRoot {
                ContextMenuTrigger {
                    class: "flex h-36 w-full select-none flex-col items-center justify-center rounded-card-sm border border-dashed border-border-input bg-background-alt text-center transition-colors hover:border-foreground/40",
                    p { class: "text-sm font-medium text-foreground", "Right-click anywhere here" }
                    p { class: "mt-1 text-xs text-muted-foreground",
                        "Or long-press on touch devices (700ms)"
                    }
                }

                ContextMenuPortal {
                    ContextMenuContent {
                        class: "z-50 min-w-[210px] rounded-card-sm border border-border-card bg-background p-1 text-sm text-foreground shadow-popover outline-none",

                        ContextMenuItem {
                            class: "flex cursor-default select-none items-center justify-between rounded-button px-2.5 py-1.5 text-sm outline-none data-[highlighted]:bg-muted data-[disabled]:pointer-events-none data-[disabled]:opacity-40",
                            on_select: move |_| last_action.set("Duplicated selection".into()),
                            span { "Duplicate" }
                            span { class: "text-xs text-muted-foreground", "⌘D" }
                        }
                        ContextMenuItem {
                            class: "flex cursor-default select-none items-center justify-between rounded-button px-2.5 py-1.5 text-sm outline-none data-[highlighted]:bg-muted data-[disabled]:pointer-events-none data-[disabled]:opacity-40",
                            on_select: move |_| last_action.set("Copied CSS tokens".into()),
                            span { "Copy tokens" }
                            span { class: "text-xs text-muted-foreground", "⇧⌘C" }
                        }

                        ContextMenuSub {
                            ContextMenuSubTrigger {
                                class: "flex cursor-default select-none items-center justify-between rounded-button px-2.5 py-1.5 text-sm outline-none data-[highlighted]:bg-muted data-[state=open]:bg-muted",
                                span { "Export as" }
                                span { class: "text-xs text-muted-foreground", "›" }
                            }
                            ContextMenuPortal {
                                ContextMenuSubContent {
                                    side_offset: 4.0,
                                    class: "z-50 min-w-[160px] rounded-card-sm border border-border-card bg-background p-1 text-sm text-foreground shadow-popover outline-none",
                                    ContextMenuItem {
                                        class: "flex cursor-default select-none items-center rounded-button px-2.5 py-1.5 text-sm outline-none data-[highlighted]:bg-muted",
                                        on_select: move |_| last_action.set("Exported SVG".into()),
                                        "SVG Vector"
                                    }
                                    ContextMenuItem {
                                        class: "flex cursor-default select-none items-center rounded-button px-2.5 py-1.5 text-sm outline-none data-[highlighted]:bg-muted",
                                        on_select: move |_| last_action.set("Exported PNG @2x".into()),
                                        "PNG @2x"
                                    }
                                }
                            }
                        }

                        ContextMenuSeparator { class: "my-1 h-px bg-border" }

                        ContextMenuCheckboxItem {
                            checked: Some(show_grid()),
                            on_checked_change: move |next| show_grid.set(next),
                            close_on_select: false,
                            class: "flex cursor-default select-none items-center gap-2 rounded-button px-2.5 py-1.5 text-sm outline-none data-[highlighted]:bg-muted",
                            span { class: "inline-flex w-4 items-center justify-center text-xs",
                                ContextMenuItemIndicator { "✓" }
                            }
                            "Show pixel grid"
                        }
                        ContextMenuCheckboxItem {
                            checked: Some(snap_guides()),
                            on_checked_change: move |next| snap_guides.set(next),
                            close_on_select: false,
                            class: "flex cursor-default select-none items-center gap-2 rounded-button px-2.5 py-1.5 text-sm outline-none data-[highlighted]:bg-muted",
                            span { class: "inline-flex w-4 items-center justify-center text-xs",
                                ContextMenuItemIndicator { "✓" }
                            }
                            "Snap to guides"
                        }

                        ContextMenuSeparator { class: "my-1 h-px bg-border" }

                        ContextMenuRadioGroup {
                            value: Some(density()),
                            on_value_change: move |next| {
                                last_action.set(format!("Density: {next}"));
                                density.set(next);
                            },
                            ContextMenuRadioItem {
                                value: "compact",
                                class: "flex cursor-default select-none items-center gap-2 rounded-button px-2.5 py-1.5 text-sm outline-none data-[highlighted]:bg-muted",
                                span { class: "inline-flex w-4 items-center justify-center text-xs",
                                    ContextMenuItemIndicator { "•" }
                                }
                                "Compact"
                            }
                            ContextMenuRadioItem {
                                value: "comfortable",
                                class: "flex cursor-default select-none items-center gap-2 rounded-button px-2.5 py-1.5 text-sm outline-none data-[highlighted]:bg-muted",
                                span { class: "inline-flex w-4 items-center justify-center text-xs",
                                    ContextMenuItemIndicator { "•" }
                                }
                                "Comfortable"
                            }
                        }
                    }
                }
            }

            div { class: "flex flex-wrap items-center justify-center gap-x-3 gap-y-1 text-xs text-muted-foreground",
                span { "{last_action}" }
                span { "·" }
                span {
                    "Grid: "
                    span { class: "font-medium text-foreground",
                        if show_grid() == CheckedState::Checked { "on" } else { "off" }
                    }
                }
                span { "·" }
                span {
                    "Density: "
                    span { class: "font-medium text-foreground", "{density}" }
                }
            }
        }
    }
}

use dioxus::prelude::*;
use monoxus::command::{
    CommandEmpty, CommandGroup, CommandGroupHeading, CommandGroupItems, CommandInput, CommandItem,
    CommandList, CommandRoot, CommandSeparator, CommandViewport,
};

#[component]
pub fn CommandDemo() -> Element {
    let mut last_action = use_signal(|| Option::<String>::None);

    rsx! {
        div { class: "flex w-full max-w-md flex-col items-center gap-3",
            CommandRoot {
                label: "Command Menu",
                loop_focus: true,
                class: "w-full overflow-hidden rounded-card-sm border border-border-card bg-background shadow-popover outline-none",

                div { class: "border-b border-border-card px-3 py-2.5",
                    CommandInput {
                        placeholder: "Type a command or search...",
                        class: "w-full bg-transparent text-sm text-foreground outline-none placeholder:text-muted-foreground",
                    }
                }

                CommandList {
                    class: "max-h-64 overflow-y-auto p-1.5",
                    CommandViewport {
                        CommandEmpty {
                            class: "py-6 text-center text-xs text-muted-foreground",
                            "No commands found."
                        }

                        CommandGroup {
                            value: "navigation",
                            class: "space-y-0.5",
                            CommandGroupHeading {
                                class: "px-2.5 py-1.5 text-xs font-medium text-muted-foreground",
                                "Navigation"
                            }
                            CommandGroupItems {
                                class: "space-y-0.5",
                                CommandItem {
                                    value: "Getting Started",
                                    keywords: vec![String::from("docs"), String::from("install"), String::from("setup")],
                                    on_select: move |_| last_action.set(Some(String::from("Getting Started"))),
                                    class: "flex cursor-pointer select-none items-center justify-between rounded-button px-2.5 py-2 text-sm text-foreground transition-colors data-[selected]:bg-muted",
                                    span { "Getting Started" }
                                    span { class: "text-xs text-muted-foreground", "G S" }
                                }
                                CommandItem {
                                    value: "Styling Guide",
                                    keywords: vec![String::from("css"), String::from("tailwind"), String::from("tokens")],
                                    on_select: move |_| last_action.set(Some(String::from("Styling Guide"))),
                                    class: "flex cursor-pointer select-none items-center justify-between rounded-button px-2.5 py-2 text-sm text-foreground transition-colors data-[selected]:bg-muted",
                                    span { "Styling Guide" }
                                    span { class: "text-xs text-muted-foreground", "G T" }
                                }
                                CommandItem {
                                    value: "Accessibility",
                                    keywords: vec![String::from("aria"), String::from("keyboard"), String::from("focus")],
                                    on_select: move |_| last_action.set(Some(String::from("Accessibility"))),
                                    class: "flex cursor-pointer select-none items-center justify-between rounded-button px-2.5 py-2 text-sm text-foreground transition-colors data-[selected]:bg-muted",
                                    span { "Accessibility" }
                                    span { class: "text-xs text-muted-foreground", "G A" }
                                }
                            }
                        }

                        CommandSeparator { class: "my-1.5 h-px bg-border-card" }

                        CommandGroup {
                            value: "actions",
                            class: "space-y-0.5",
                            CommandGroupHeading {
                                class: "px-2.5 py-1.5 text-xs font-medium text-muted-foreground",
                                "Actions"
                            }
                            CommandGroupItems {
                                class: "space-y-0.5",
                                CommandItem {
                                    value: "Toggle Theme",
                                    keywords: vec![String::from("dark"), String::from("light"), String::from("mode")],
                                    on_select: move |_| last_action.set(Some(String::from("Toggle Theme"))),
                                    class: "flex cursor-pointer select-none items-center justify-between rounded-button px-2.5 py-2 text-sm text-foreground transition-colors data-[selected]:bg-muted",
                                    span { "Toggle Theme" }
                                    span { class: "text-xs text-muted-foreground", "⌘ J" }
                                }
                                CommandItem {
                                    value: "Copy Page Link",
                                    keywords: vec![String::from("url"), String::from("share"), String::from("clipboard")],
                                    on_select: move |_| last_action.set(Some(String::from("Copy Page Link"))),
                                    class: "flex cursor-pointer select-none items-center justify-between rounded-button px-2.5 py-2 text-sm text-foreground transition-colors data-[selected]:bg-muted",
                                    span { "Copy Page Link" }
                                    span { class: "text-xs text-muted-foreground", "⌘ C" }
                                }
                            }
                        }
                    }
                }
            }

            p { class: "text-xs text-muted-foreground",
                "Last run: "
                span { class: "font-medium text-foreground",
                    "{last_action().unwrap_or_else(|| String::from(\"none\"))}"
                }
            }
        }
    }
}

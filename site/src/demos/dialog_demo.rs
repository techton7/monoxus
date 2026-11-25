use dioxus::prelude::*;
use monoxus::dialog::{
    DialogClose, DialogContent, DialogDescription, DialogOverlay, DialogPortal, DialogRoot,
    DialogTitle, DialogTrigger,
};

#[component]
pub fn DialogDemo() -> Element {
    let mut open = use_signal(|| false);
    let mut key_name = use_signal(|| String::from("Production Read-Only"));
    let mut last_created = use_signal(|| None::<String>);

    rsx! {
        div { class: "flex flex-col items-center gap-3",
            DialogRoot {
                open: open,
                on_open_change: move |next| open.set(next),

                DialogTrigger {
                    class: "inline-flex h-10 items-center justify-center rounded-button bg-dark px-4 text-sm font-medium text-background shadow-btn transition-colors hover:bg-dark/90",
                    "Create API Key"
                }

                DialogPortal {
                    DialogOverlay {
                        class: "fixed inset-0 z-50 bg-black/50 backdrop-blur-[1px]",
                    }
                    DialogContent {
                        class: "fixed left-1/2 top-1/2 z-50 w-[calc(100%-2rem)] max-w-md -translate-x-1/2 -translate-y-1/2 rounded-card-sm border border-border-card bg-background p-6 shadow-popover outline-none",

                        div { class: "flex items-start justify-between gap-4",
                            div { class: "space-y-1",
                                DialogTitle {
                                    class: "text-base font-semibold tracking-tight text-foreground",
                                    "Create API key"
                                }
                                DialogDescription {
                                    class: "text-sm text-muted-foreground",
                                    "Generate a scoped token for CI deployments. Press Escape or click outside to dismiss."
                                }
                            }
                            DialogClose {
                                class: "inline-flex h-7 w-7 items-center justify-center rounded-button text-sm text-muted-foreground transition-colors hover:bg-muted hover:text-foreground",
                                "✕"
                            }
                        }

                        div { class: "mt-5 space-y-2",
                            label {
                                r#for: "api-key-name",
                                class: "block text-xs font-medium text-foreground-alt",
                                "Key label"
                            }
                            input {
                                id: "api-key-name",
                                r#type: "text",
                                value: "{key_name}",
                                oninput: move |evt| key_name.set(evt.value()),
                                class: "h-10 w-full rounded-input border border-border-input bg-background-alt px-3 text-sm text-foreground outline-none focus:border-foreground",
                            }
                        }

                        div { class: "mt-6 flex items-center justify-end gap-2",
                            DialogClose {
                                class: "inline-flex h-9 items-center justify-center rounded-button border border-border-input bg-background px-3.5 text-sm font-medium text-foreground shadow-mini transition-colors hover:bg-muted",
                                "Cancel"
                            }
                            button {
                                r#type: "button",
                                class: "inline-flex h-9 items-center justify-center rounded-button bg-dark px-3.5 text-sm font-medium text-background shadow-btn transition-colors hover:bg-dark/90",
                                onclick: move |_| {
                                    let trimmed = key_name().trim().to_string();
                                    if !trimmed.is_empty() {
                                        last_created.set(Some(trimmed));
                                    }
                                    open.set(false);
                                },
                                "Generate key"
                            }
                        }
                    }
                }
            }

            if let Some(name) = last_created() {
                p { class: "text-xs text-muted-foreground",
                    "Created token: "
                    span { class: "font-medium text-foreground", "{name}" }
                }
            }
        }
    }
}

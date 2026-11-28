use dioxus::prelude::*;
use monoxus::alert_dialog::{
    AlertDialogAction, AlertDialogCancel, AlertDialogRoot, DialogContent, DialogDescription,
    DialogOverlay, DialogPortal, DialogTitle, DialogTrigger,
};

#[component]
pub fn AlertDialogDemo() -> Element {
    let mut open = use_signal(|| false);
    let mut status = use_signal(|| String::from("Workspace active"));

    rsx! {
        div { class: "flex flex-col items-center gap-3",
            AlertDialogRoot {
                open: Some(open()),
                on_open_change: move |next| open.set(next),

                DialogTrigger {
                    class: "inline-flex h-10 items-center justify-center rounded-button bg-dark px-4 text-sm font-medium text-background shadow-btn transition-colors hover:bg-dark/90",
                    "Delete Workspace"
                }

                DialogPortal {
                    DialogOverlay {
                        class: "fixed inset-0 z-50 bg-black/50 backdrop-blur-[1px]",
                    }
                    DialogContent {
                        class: "fixed left-1/2 top-1/2 z-50 w-[calc(100%-2rem)] max-w-md -translate-x-1/2 -translate-y-1/2 rounded-card-sm border border-border-card bg-background p-6 shadow-popover outline-none",

                        div { class: "space-y-2",
                            DialogTitle {
                                class: "text-base font-semibold tracking-tight text-foreground",
                                "Delete production workspace?"
                            }
                            DialogDescription {
                                class: "text-sm text-muted-foreground",
                                "This permanently removes all deployments, tokens, and audit logs. Initial focus defaults to Cancel to prevent accidental confirmation."
                            }
                        }

                        div { class: "mt-6 flex items-center justify-end gap-2",
                            AlertDialogCancel {
                                class: "inline-flex h-9 items-center justify-center rounded-button border border-border-input bg-background px-3.5 text-sm font-medium text-foreground shadow-mini transition-colors hover:bg-muted focus-visible:outline-none focus-visible:ring-2 focus-visible:ring-foreground",
                                on_click: move |_| status.set(String::from("Deletion canceled")),
                                "Cancel"
                            }
                            AlertDialogAction {
                                class: "inline-flex h-9 items-center justify-center rounded-button bg-dark px-3.5 text-sm font-medium text-background shadow-btn transition-colors hover:bg-dark/90 focus-visible:outline-none focus-visible:ring-2 focus-visible:ring-foreground",
                                on_click: move |_| status.set(String::from("Workspace scheduled for deletion")),
                                "Delete permanently"
                            }
                        }
                    }
                }
            }

            p { class: "text-xs text-muted-foreground",
                "Status: "
                span { class: "font-medium text-foreground", "{status}" }
            }
        }
    }
}

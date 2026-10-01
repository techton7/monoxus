use dioxus::prelude::*;
use monoxus::alert_dialog::{
    AlertDialogAction, AlertDialogCancel, AlertDialogRoot, DialogContent, DialogDescription,
    DialogOverlay, DialogPortal, DialogTitle, DialogTrigger,
};

#[component]
pub fn AlertDialogPlayground() -> Element {
    let open = use_signal(|| false);
    let mut outcome = use_signal(|| String::from("Waiting for a choice."));

    rsx! {
        div {
            class: "max-w-2xl mx-auto space-y-6",

            // Header Section
            div {
                class: "space-y-1.5",
                h2 {
                    class: "text-2xl font-bold tracking-tight text-slate-900",
                    "Alert Dialog"
                }
                p {
                    class: "text-sm text-slate-500",
                    "A modal dialog that interrupts the user with important content and expects an active confirmation or cancellation response."
                }
            }

            // Interactive Showcase Card
            div {
                class: "rounded-2xl border border-slate-200 bg-white p-8 shadow-sm space-y-6",

                div {
                    class: "space-y-2",
                    h3 {
                        class: "text-base font-semibold text-slate-900",
                        "Declarative Destructive Confirmation Modal"
                    }
                    p {
                        class: "text-sm text-slate-600",
                        "Composed using "
                        code { class: "rounded bg-slate-100 px-1.5 py-0.5 text-xs font-mono text-red-600", "<AlertDialogRoot>" }
                        ", "
                        code { class: "rounded bg-slate-100 px-1.5 py-0.5 text-xs font-mono text-red-600", "<AlertDialogAction>" }
                        ", and "
                        code { class: "rounded bg-slate-100 px-1.5 py-0.5 text-xs font-mono text-red-600", "<AlertDialogCancel>" }
                        ". Backdrop dismissal is strictly disabled."
                    }
                }

                div {
                    class: "flex items-center gap-4",
                    AlertDialogRoot {
                        open: open,
                        DialogTrigger {
                            id: "alert-dialog-trigger",
                            class: "inline-flex items-center justify-center rounded-lg bg-red-600 px-4 py-2.5 text-sm font-semibold text-white shadow-sm hover:bg-red-700 focus-visible:outline-2 focus-visible:outline-offset-2 focus-visible:outline-red-600 transition-colors cursor-pointer",
                            "Delete Account"
                        }
                        DialogPortal {
                            DialogOverlay {
                                id: "alert-dialog-overlay",
                                class: "fixed inset-0 z-50 bg-slate-900/60 backdrop-blur-sm cursor-not-allowed",
                            }
                            div {
                                class: "fixed inset-0 z-50 flex items-center justify-center p-4 pointer-events-none",
                                DialogContent {
                                    id: "alert-dialog-content",
                                    aria_labelledby: "alert-dialog-title".to_string(),
                                    aria_describedby: "alert-dialog-description".to_string(),
                                    class: "pointer-events-auto w-full max-w-lg rounded-2xl bg-white p-6 shadow-2xl border border-red-100 space-y-5 will-change-[opacity,transform]",
                                    div {
                                        class: "space-y-1.5",
                                        DialogTitle {
                                            id: "alert-dialog-title",
                                            class: "text-lg font-semibold text-slate-900 tracking-tight",
                                            "Are you absolutely sure?"
                                        }
                                        DialogDescription {
                                            id: "alert-dialog-description",
                                            class: "text-sm text-slate-500 leading-relaxed",
                                            "This action cannot be undone. This will permanently delete your account, wipe all workspace projects, and cancel all active team subscriptions."
                                        }
                                    }

                                    div {
                                        class: "rounded-lg bg-red-50 border border-red-200/60 p-3 text-xs text-red-800 leading-relaxed",
                                        strong { class: "font-semibold", "Warning: " }
                                        "Backdrop clicking is disabled by alert dialog policy to prevent accidental dismissal."
                                    }

                                    div {
                                        class: "flex justify-end gap-3 pt-2",
                                        AlertDialogCancel {
                                            id: "alert-dialog-cancel",
                                            on_click: move |_| outcome.set(String::from("Canceled deletion")),
                                            class: "inline-flex items-center justify-center rounded-lg border border-slate-300 bg-white px-4 py-2 text-sm font-medium text-slate-700 shadow-sm hover:bg-slate-50 transition-colors cursor-pointer",
                                            "Cancel"
                                        }
                                        AlertDialogAction {
                                            id: "alert-dialog-confirm",
                                            on_click: move |_| outcome.set(String::from("Confirmed permanent deletion")),
                                            class: "inline-flex items-center justify-center rounded-lg bg-red-600 px-4 py-2 text-sm font-semibold text-white shadow-sm hover:bg-red-700 transition-colors cursor-pointer",
                                            "Yes, delete account"
                                        }
                                    }
                                }
                            }
                        }
                    }
                }

                div {
                    class: "flex items-center gap-2 pt-2 text-sm text-slate-600",
                    span { class: "text-xs font-semibold uppercase tracking-wider text-slate-400", "State:" }
                    span {
                        id: "alert-dialog-outcome",
                        class: "inline-flex items-center rounded-md bg-slate-100 px-2.5 py-1 text-xs font-medium text-slate-800",
                        "{outcome()}"
                    }
                }
            }

            // Architecture & Invariants Card
            div {
                class: "rounded-xl border border-slate-200 bg-slate-50/50 p-5 space-y-3",
                h4 {
                    class: "text-xs font-semibold uppercase tracking-wider text-slate-500",
                    "WAI-ARIA & Behavioral Guarantees"
                }
                ul {
                    class: "space-y-1.5 text-xs text-slate-600",
                    li {
                        class: "flex items-center gap-2",
                        span { class: "h-1.5 w-1.5 rounded-full bg-red-500" }
                        "Role "
                        code { class: "font-mono text-slate-800", "alertdialog" }
                        " requiring explicit confirmation or cancel choice"
                    }
                    li {
                        class: "flex items-center gap-2",
                        span { class: "h-1.5 w-1.5 rounded-full bg-red-500" }
                        "Outside backdrop clicks are ignored by default policy"
                    }
                    li {
                        class: "flex items-center gap-2",
                        span { class: "h-1.5 w-1.5 rounded-full bg-red-500" }
                        "Auto-managed ARIA relationships and focus restoration on dismiss"
                    }
                }
            }
        }
    }
}

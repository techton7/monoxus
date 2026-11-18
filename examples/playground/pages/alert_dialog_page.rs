use crate::alert_dialog::AlertDialogPlayground;
use dioxus::prelude::*;
use monoxus::alert_dialog::{
    AlertDialogAction, AlertDialogCancel, AlertDialogRoot, DialogContent, DialogDescription,
    DialogOverlay, DialogPortal, DialogTitle, DialogTrigger,
};

#[component]
pub fn AlertDialogPage() -> Element {
    let mut outside_clicks = use_signal(|| 0_u32);
    let order_open = use_signal(|| false);

    rsx! {
        div {
            class: "space-y-8",
            AlertDialogPlayground {}

            // Track 1 Regression Hook (MF-3, R1-3): Action rendered BEFORE Cancel in DOM
            div {
                id: "alert-dialog-external-sibling",
                class: "max-w-2xl mx-auto rounded-2xl border border-slate-200 bg-white p-6 shadow-sm space-y-4",
                h3 {
                    class: "text-base font-semibold text-slate-900",
                    "Track 1 Regression: Cancel Open-Focus Priority & Outside Click Guard"
                }
                div {
                    class: "flex flex-wrap items-center gap-3",
                    button {
                        id: "alert-outside-programmatic-btn",
                        r#type: "button",
                        class: "rounded-lg border border-slate-300 bg-slate-50 px-3 py-1.5 text-xs font-medium text-slate-700 cursor-pointer",
                        onclick: move |_| outside_clicks += 1,
                        "Outside Programmatic Button ({outside_clicks()})"
                    }

                    AlertDialogRoot {
                        id: "alert-focus-order",
                        open: order_open,
                        DialogTrigger {
                            id: "alert-focus-order-trigger",
                            class: "rounded-lg bg-red-700 px-3.5 py-2 text-xs font-semibold text-white cursor-pointer",
                            "Open Action-First AlertDialog"
                        }
                        DialogPortal {
                            DialogOverlay {
                                id: "alert-focus-order-overlay",
                                class: "fixed inset-0 z-50 bg-slate-900/50",
                                style: "position: fixed; inset: 0; width: 100%; height: 100%;",
                            }
                            div {
                                class: "fixed inset-0 z-50 flex items-center justify-center p-4 pointer-events-none",
                                style: "position: fixed; inset: 0; pointer-events: none;",
                                DialogContent {
                                    id: "alert-focus-order-content",
                                    class: "w-full max-w-md rounded-2xl bg-white p-6 shadow-xl border border-red-200 space-y-4",
                                    DialogTitle {
                                        class: "text-base font-semibold text-slate-900",
                                        "Destructive Action Rendered First"
                                    }
                                    DialogDescription {
                                        class: "text-xs text-slate-600",
                                        "Even though Confirm appears before Cancel in DOM order, initial focus lands safely on AlertDialogCancel."
                                    }
                                    div {
                                        class: "flex justify-end gap-3 pt-2",
                                        AlertDialogAction {
                                            id: "alert-action-first",
                                            class: "rounded-lg bg-red-600 px-3 py-1.5 text-xs font-semibold text-white cursor-pointer",
                                            "Confirm Destructive"
                                        }
                                        AlertDialogCancel {
                                            id: "alert-cancel-second",
                                            class: "rounded-lg border border-slate-300 bg-white px-3 py-1.5 text-xs font-semibold text-slate-700 cursor-pointer",
                                            "Safe Cancel"
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

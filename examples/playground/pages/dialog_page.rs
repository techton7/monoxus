use crate::dialog::DialogPlayground;
use dioxus::prelude::*;
use monoxus::{
    dialog::{DialogClose, DialogContent, DialogOverlay, DialogPortal, DialogRoot, DialogTrigger},
    tooltip::components::{
        TooltipContent, TooltipPortal, TooltipProvider as TooltipProviderComp, TooltipRoot,
        TooltipTrigger,
    },
};

#[component]
pub fn DialogPage() -> Element {
    let mut outside_clicks = use_signal(|| 0_u32);
    let mut minimal_open = use_signal(|| false);

    rsx! {
        div {
            class: "space-y-8",
            DialogPlayground {}

            // Track 1 Regression Hooks (MF-1, MF-4, MF-11, R1-3, R1-4)
            div {
                id: "dialog-external-sibling",
                class: "max-w-2xl mx-auto rounded-2xl border border-slate-200 bg-white p-6 shadow-sm space-y-4",
                h3 {
                    class: "text-base font-semibold text-slate-900",
                    "Track 1 Regression Suite: Omitted ARIA Title/Description, Modal Shield & Nested Tooltip"
                }
                div {
                    class: "flex flex-wrap items-center gap-3",
                    button {
                        id: "dialog-outside-programmatic-btn",
                        r#type: "button",
                        class: "rounded-lg border border-slate-300 bg-slate-50 px-3 py-1.5 text-xs font-medium text-slate-700 cursor-pointer",
                        onclick: move |_| outside_clicks += 1,
                        "Outside Sibling Action ({outside_clicks()})"
                    }

                    DialogRoot {
                        id: "dialog-minimal",
                        open: minimal_open(),
                        on_open_change: move |next: bool| minimal_open.set(next),
                        DialogTrigger {
                            id: "dialog-minimal-trigger",
                            class: "rounded-lg bg-slate-900 px-3.5 py-2 text-xs font-semibold text-white cursor-pointer",
                            "Open Minimal Dialog (No Title/Description)"
                        }
                        DialogPortal {
                            DialogOverlay {
                                id: "dialog-minimal-overlay",
                                class: "fixed inset-0 z-50 bg-slate-900/50",
                                style: "position: fixed; inset: 0; width: 100%; height: 100%;",
                            }
                            div {
                                class: "fixed inset-0 z-50 flex items-center justify-center p-4 pointer-events-none",
                                style: "position: fixed; inset: 0; pointer-events: none;",
                                DialogContent {
                                    id: "dialog-minimal-content",
                                    class: "w-full max-w-md rounded-2xl bg-white p-6 shadow-xl border border-slate-200 space-y-4",
                                    p {
                                        class: "text-sm text-slate-700",
                                        "DialogTitle and DialogDescription are omitted here, so aria-labelledby and aria-describedby are omitted. Includes nested portalled Tooltip branch:"
                                    }
                                    div {
                                        class: "flex items-center justify-between pt-2",
                                        TooltipProviderComp {
                                            delay_duration: 0,
                                            TooltipRoot {
                                                id: "dialog-nested-tooltip".to_string(),
                                                TooltipTrigger {
                                                    id: "dialog-nested-tooltip-trigger".to_string(),
                                                    class: "rounded-lg border border-cyan-300 bg-cyan-50 px-3 py-1.5 text-xs font-semibold text-cyan-800 cursor-pointer".to_string(),
                                                    "Nested Tooltip Trigger"
                                                }
                                                TooltipPortal {
                                                    TooltipContent {
                                                        id: "dialog-nested-tooltip-content".to_string(),
                                                        class: "z-50 rounded-lg bg-slate-900 px-3 py-1.5 text-xs text-white shadow-lg".to_string(),
                                                        "Registered in Dialog OverlayBranchContext"
                                                    }
                                                }
                                            }
                                        }
                                        DialogClose {
                                            id: "dialog-minimal-close",
                                            class: "rounded-lg border border-slate-300 bg-white px-3 py-1.5 text-xs font-semibold text-slate-700 cursor-pointer",
                                            "Close Minimal"
                                        }
                                    }
                                }
                            }
                        }
                    }
                }
            }

            div {
                id: "dialog-aria-live-probe",
                "aria-live": "polite",
                class: "max-w-2xl mx-auto rounded-lg border border-emerald-200 bg-emerald-50/70 px-4 py-2 text-xs text-emerald-800",
                "Live Region Probe ([aria-live=\"polite\"]) — remains accessible to screen readers while modal Dialog is open."
            }
        }
    }
}

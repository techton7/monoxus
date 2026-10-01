use dioxus::prelude::*;
use monoxus::dialog::{
    DialogClose, DialogContent, DialogDescription, DialogOverlay, DialogPortal, DialogRoot,
    DialogTitle, DialogTrigger,
};

#[component]
pub fn DialogPlayground() -> Element {
    let open = use_signal(|| false);

    rsx! {
        div {
            class: "max-w-2xl mx-auto space-y-6",

            // Header Section
            div {
                class: "space-y-1.5",
                h2 {
                    class: "text-2xl font-bold tracking-tight text-slate-900",
                    "Dialog"
                }
                p {
                    class: "text-sm text-slate-500",
                    "A window overlaid on either the primary window or another dialog window, rendering the content underneath inert with full WAI-ARIA compliance."
                }
            }

            // Interactive Showcase Card
            div {
                class: "rounded-2xl border border-slate-200 bg-white p-8 shadow-sm space-y-6",

                div {
                    class: "space-y-2",
                    h3 {
                        class: "text-base font-semibold text-slate-900",
                        "Declarative Compound Modal"
                    }
                    p {
                        class: "text-sm text-slate-600",
                        "Composed using "
                        code { class: "rounded bg-slate-100 px-1.5 py-0.5 text-xs font-mono text-blue-600", "<DialogRoot>" }
                        ", "
                        code { class: "rounded bg-slate-100 px-1.5 py-0.5 text-xs font-mono text-blue-600", "<DialogTrigger>" }
                        ", "
                        code { class: "rounded bg-slate-100 px-1.5 py-0.5 text-xs font-mono text-blue-600", "<DialogPortal>" }
                        ", "
                        code { class: "rounded bg-slate-100 px-1.5 py-0.5 text-xs font-mono text-blue-600", "<DialogOverlay>" }
                        ", and "
                        code { class: "rounded bg-slate-100 px-1.5 py-0.5 text-xs font-mono text-blue-600", "<DialogContent>" }
                        "."
                    }
                }

                DialogRoot {
                    open: open,
                    DialogTrigger {
                        id: "dialog-trigger",
                        class: "inline-flex items-center justify-center rounded-lg bg-blue-600 px-4 py-2.5 text-sm font-semibold text-white shadow-sm hover:bg-blue-700 focus-visible:outline-2 focus-visible:outline-offset-2 focus-visible:outline-blue-600 transition-colors cursor-pointer",
                        "Edit Profile"
                    }
                    DialogPortal {
                        DialogOverlay {
                            id: "dialog-overlay",
                            class: "fixed inset-0 z-50 bg-slate-900/60 backdrop-blur-sm",
                        }
                        div {
                            class: "fixed inset-0 z-50 flex items-center justify-center p-4 pointer-events-none",
                            DialogContent {
                                id: "dialog-content",
                                aria_labelledby: "dialog-title".to_string(),
                                aria_describedby: "dialog-description".to_string(),
                                class: "pointer-events-auto w-full max-w-lg rounded-2xl bg-white p-6 shadow-2xl border border-slate-200 space-y-5 will-change-[opacity,transform]",
                                div {
                                    class: "space-y-1.5",
                                    DialogTitle {
                                        id: "dialog-title",
                                        class: "text-lg font-semibold text-slate-900 tracking-tight",
                                        "Edit Profile"
                                    }
                                    DialogDescription {
                                        id: "dialog-description",
                                        class: "text-sm text-slate-500",
                                        "Make changes to your profile here. Click save when you're done."
                                    }
                                }

                                div {
                                    class: "grid gap-3 py-1",
                                    div {
                                        class: "grid grid-cols-4 items-center gap-4",
                                        label {
                                            r#for: "name",
                                            class: "text-right text-sm font-medium text-slate-700",
                                            "Name"
                                        }
                                        input {
                                            id: "name",
                                            class: "col-span-3 rounded-lg border border-slate-300 px-3 py-2 text-sm text-slate-900 shadow-sm focus:border-blue-500 focus:outline-none focus:ring-1 focus:ring-blue-500",
                                            value: "Pedro Duarte",
                                        }
                                    }
                                    div {
                                        class: "grid grid-cols-4 items-center gap-4",
                                        label {
                                            r#for: "username",
                                            class: "text-right text-sm font-medium text-slate-700",
                                            "Username"
                                        }
                                        input {
                                            id: "username",
                                            class: "col-span-3 rounded-lg border border-slate-300 px-3 py-2 text-sm text-slate-900 shadow-sm focus:border-blue-500 focus:outline-none focus:ring-1 focus:ring-blue-500",
                                            value: "@peduarte",
                                        }
                                    }
                                }

                                div {
                                    class: "flex justify-end gap-3 pt-2",
                                    DialogClose {
                                        id: "dialog-close",
                                        class: "inline-flex items-center justify-center rounded-lg border border-slate-300 bg-white px-4 py-2 text-sm font-medium text-slate-700 shadow-sm hover:bg-slate-50 transition-colors cursor-pointer",
                                        "Cancel"
                                    }
                                    DialogClose {
                                        id: "dialog-save",
                                        class: "inline-flex items-center justify-center rounded-lg bg-blue-600 px-4 py-2 text-sm font-semibold text-white shadow-sm hover:bg-blue-700 transition-colors cursor-pointer",
                                        "Save changes"
                                    }
                                }
                            }
                        }
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
                        span { class: "h-1.5 w-1.5 rounded-full bg-blue-500" }
                        "Role "
                        code { class: "font-mono text-slate-800", "dialog" }
                        " with "
                        code { class: "font-mono text-slate-800", "aria-modal=\"true\"" }
                        " and auto-linked labelledby / describedby IDs"
                    }
                    li {
                        class: "flex items-center gap-2",
                        span { class: "h-1.5 w-1.5 rounded-full bg-blue-500" }
                        "Initial focus moves to first focusable control; focus restored to trigger upon dismissal"
                    }
                    li {
                        class: "flex items-center gap-2",
                        span { class: "h-1.5 w-1.5 rounded-full bg-blue-500" }
                        "Dismissible via Escape keypress or clicking the dimmed backdrop overlay"
                    }
                    li {
                        class: "flex items-center gap-2",
                        span { class: "h-1.5 w-1.5 rounded-full bg-blue-500" }
                        "Outer flexbox centering ensures 200ms scale-in/out transitions never displace modal coordinates"
                    }
                }
            }
        }
    }
}

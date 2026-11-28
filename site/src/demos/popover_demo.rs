use dioxus::prelude::*;
use monoxus::foundation::overlay::PlacementSide;
use monoxus::popover::{
    PopoverArrow, PopoverClose, PopoverContent, PopoverDescription, PopoverPortal, PopoverRoot,
    PopoverTitle, PopoverTrigger,
};

#[component]
pub fn PopoverDemo() -> Element {
    let mut open = use_signal(|| false);
    let mut width = use_signal(|| String::from("320px"));
    let mut height = use_signal(|| String::from("180px"));

    rsx! {
        div { class: "flex flex-col items-center gap-3",
            PopoverRoot {
                open: Some(open()),
                on_open_change: move |next| open.set(next),

                PopoverTrigger {
                    class: "inline-flex h-10 items-center justify-center rounded-button border border-border-input bg-background px-4 text-sm font-medium text-foreground shadow-mini transition-colors hover:bg-muted data-[state=open]:bg-muted",
                    "Dimensions"
                }

                PopoverPortal {
                    PopoverContent {
                        side: PlacementSide::Bottom,
                        side_offset: 8.0,
                        class: "relative z-50 w-72 rounded-card-sm border border-border-card bg-background p-4 shadow-popover outline-none",

                        div { class: "flex items-start justify-between gap-2",
                            div { class: "space-y-1",
                                PopoverTitle {
                                    class: "text-sm font-semibold text-foreground",
                                    "Layer dimensions"
                                }
                                PopoverDescription {
                                    class: "text-xs text-muted-foreground",
                                    "Configure the frame size for the active surface."
                                }
                            }
                            PopoverClose {
                                class: "inline-flex h-6 w-6 items-center justify-center rounded-input text-xs text-muted-foreground transition-colors hover:bg-muted hover:text-foreground",
                                "✕"
                            }
                        }

                        div { class: "mt-4 grid gap-2.5",
                            div { class: "grid grid-cols-3 items-center gap-2",
                                label {
                                    r#for: "popover-width",
                                    class: "text-xs font-medium text-foreground-alt",
                                    "Width"
                                }
                                input {
                                    id: "popover-width",
                                    r#type: "text",
                                    value: "{width}",
                                    oninput: move |evt| width.set(evt.value()),
                                    class: "col-span-2 h-8 rounded-input border border-border-input bg-background-alt px-2.5 text-xs text-foreground outline-none focus:border-foreground",
                                }
                            }
                            div { class: "grid grid-cols-3 items-center gap-2",
                                label {
                                    r#for: "popover-height",
                                    class: "text-xs font-medium text-foreground-alt",
                                    "Height"
                                }
                                input {
                                    id: "popover-height",
                                    r#type: "text",
                                    value: "{height}",
                                    oninput: move |evt| height.set(evt.value()),
                                    class: "col-span-2 h-8 rounded-input border border-border-input bg-background-alt px-2.5 text-xs text-foreground outline-none focus:border-foreground",
                                }
                            }
                        }

                        div { class: "mt-4 flex justify-end",
                            PopoverClose {
                                class: "inline-flex h-8 items-center justify-center rounded-button bg-dark px-3 text-xs font-medium text-background shadow-btn transition-colors hover:bg-dark/90",
                                "Done"
                            }
                        }

                        PopoverArrow {
                            class: "fill-background text-background",
                        }
                    }
                }
            }

            p { class: "text-xs text-muted-foreground",
                "Current frame: "
                span { class: "font-medium text-foreground", "{width} × {height}" }
            }
        }
    }
}

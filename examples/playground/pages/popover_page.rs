use crate::popover::PopoverPlayground;
use dioxus::prelude::*;
use monoxus::popover::{
    PopoverClose, PopoverContent, PopoverPortal, PopoverRoot, PopoverTrigger,
};

#[component]
pub fn PopoverPage() -> Element {
    rsx! {
        div {
            class: "space-y-6",
            PopoverPlayground {}

            // Track 1 Regression Hook (MF-6, R1-4): Declarative custom Content ID & role override
            div {
                style: "padding: 0 1rem 2rem 1rem;",
                section {
                    style: "display: grid; gap: 0.75rem; padding: 1.25rem; border-radius: 0.75rem; border: 1px solid #d8b4fe; background-color: white;",
                    h3 {
                        style: "margin: 0; font-size: 1rem; color: #581c87;",
                        "Declarative Compound Popover (Custom Content ID & role=\"region\")"
                    }
                    div {
                        style: "display: flex; align-items: center; gap: 1rem;",
                        PopoverRoot {
                            id: "popover-declarative".to_string(),
                            PopoverTrigger {
                                id: "popover-decl-trigger".to_string(),
                                style: "padding: 0.55rem 0.9rem; border: 0; border-radius: 0.5rem; background-color: #7e22ce; color: white; font-weight: 600; cursor: pointer;".to_string(),
                                "Open Custom Region Popover"
                            }
                            PopoverPortal {
                                PopoverContent {
                                    id: "popover-custom-region-content".to_string(),
                                    role: "region".to_string(),
                                    style: "padding: 0.9rem; border-radius: 0.65rem; border: 1px solid #c084fc; background-color: white; box-shadow: 0 12px 24px rgba(88, 28, 135, 0.14); display: grid; gap: 0.5rem; width: 220px;".to_string(),
                                    span {
                                        style: "font-size: 0.8125rem; color: #581c87; font-weight: 600;",
                                        "Custom ID & Region Role"
                                    }
                                    PopoverClose {
                                        id: "popover-decl-close".to_string(),
                                        style: "padding: 0.35rem 0.65rem; border-radius: 0.4rem; border: 1px solid #d8b4fe; background-color: #faf5ff; color: #6b21a8; cursor: pointer; font-size: 0.75rem;".to_string(),
                                        "Dismiss"
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

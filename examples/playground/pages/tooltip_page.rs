use crate::tooltip::TooltipPlayground;
use dioxus::prelude::*;
use monoxus::tooltip::components::{
    TooltipContent, TooltipPortal, TooltipProvider as TooltipProviderComp, TooltipRoot,
    TooltipTrigger,
};

#[component]
pub fn TooltipPage() -> Element {
    rsx! {
        div {
            class: "space-y-6",
            TooltipPlayground {}

            // Track 1 Regression Hook (MF-8..MF-10, R1-4): Keyboard-focus hover-leave guard & ancestor scroll dismiss
            div {
                style: "padding: 0 1rem 2rem 1rem;",
                section {
                    style: "display: grid; gap: 0.75rem; padding: 1.25rem; border-radius: 0.75rem; border: 1px solid #a5f3fc; background-color: white;",
                    h3 {
                        style: "margin: 0; font-size: 1rem; color: #0f766e;",
                        "Declarative Focus-Hover Guard & Scrollable Ancestor Dismissal"
                    }
                    div {
                        id: "tooltip-scroll-container",
                        style: "height: 120px; overflow-y: auto; padding: 1rem; border-radius: 0.5rem; border: 1px dashed #67e8f9; background-color: #f0fdfa;",
                        div {
                            style: "height: 240px; display: flex; flex-direction: column; justify-content: flex-start; gap: 0.75rem;",
                            TooltipProviderComp {
                                id: "tooltip-focus-guard-provider".to_string(),
                                delay_duration: 0,
                                TooltipRoot {
                                    id: "tooltip-focus-guard".to_string(),
                                    TooltipTrigger {
                                        id: "tooltip-focus-guard-trigger".to_string(),
                                        style: "align-self: flex-start; padding: 0.5rem 0.85rem; border: 1px solid #0891b2; border-radius: 0.5rem; background-color: white; color: #0e7490; font-weight: 600; cursor: pointer;".to_string(),
                                        "Keyboard Focus Guard Trigger"
                                    }
                                    TooltipPortal {
                                        TooltipContent {
                                            id: "tooltip-focus-guard-content".to_string(),
                                            aria_label: "Accessible screen-reader tooltip summary".to_string(),
                                            style: "padding: 0.5rem 0.75rem; border-radius: 0.5rem; background-color: #0f172a; color: white; font-size: 0.75rem;".to_string(),
                                            "Remains open on pointer leave when opened via keyboard focus"
                                        }
                                    }
                                }
                            }
                            p {
                                style: "margin: 0; font-size: 0.75rem; color: #0f766e;",
                                "Scroll this container after opening the tooltip to verify user scroll dismissal."
                            }
                        }
                    }
                }
            }
        }
    }
}

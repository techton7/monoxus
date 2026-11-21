use crate::accordion::AccordionPlayground;
use dioxus::prelude::*;
use monoxus::{
    accordion::{
        AccordionContent, AccordionHeader, AccordionItem, AccordionMode, AccordionRoot,
        AccordionTrigger,
    },
    foundation::shared::{Direction, DirectionProvider},
};

#[component]
pub fn AccordionPage() -> Element {
    let mut show_dynamic = use_signal(|| true);

    rsx! {
        div {
            class: "space-y-6",
            AccordionPlayground {}

            // Track 1 Regression Hook (DN-1, DN-3..DN-5, R1-2, R1-5): Declarative AccordionHeader level=2, DirectionProvider, mount guard & unmount cleanup
            section {
                style: "display: grid; gap: 0.75rem; padding: 1.25rem; border-radius: 0.75rem; border: 1px solid #93c5fd; background-color: white;",
                h3 {
                    style: "margin: 0; font-size: 1rem; color: #1e3a8a;",
                    "6. Declarative Compound Accordion (Header Level 2, RTL Context, Mount Guard & Dynamic Unmount)"
                }
                div {
                    style: "display: flex; gap: 0.5rem;",
                    button {
                        id: "accordion-dynamic-toggle",
                        r#type: "button",
                        style: "padding: 0.4rem 0.75rem; border-radius: 0.375rem; border: 1px solid #cbd5e1; background: #f8fafc; font-size: 0.75rem; font-weight: 600; cursor: pointer;",
                        onclick: move |_| {
                            let cur = *show_dynamic.peek();
                            show_dynamic.set(!cur);
                        },
                        if show_dynamic() { "Unmount Dynamic Item" } else { "Mount Dynamic Item" }
                    }
                }
                DirectionProvider {
                    dir: Direction::Rtl,
                    AccordionRoot {
                        id: "accordion-declarative".to_string(),
                        mode: AccordionMode::Single { collapsible: true },
                        default_value: "decl-1".to_string(),
                        style: "max-width: 32rem; display: grid; gap: 0.5rem;".to_string(),
                        AccordionItem {
                            value: "decl-1".to_string(),
                            style: "border: 1px solid #e2e8f0; border-radius: 0.5rem; overflow: hidden;".to_string(),
                            AccordionHeader {
                                level: 2,
                                AccordionTrigger {
                                    id: "accordion-decl-trigger-1".to_string(),
                                    style: "width: 100%; padding: 0.65rem 0.9rem; background: #f8fafc; border: none; font-weight: 600; cursor: pointer; text-align: right;".to_string(),
                                    "Heading Level 2 (Default Open)"
                                }
                            }
                            AccordionContent {
                                style: "padding: 0.65rem 0.9rem; font-size: 0.85rem; color: #334155;".to_string(),
                                "Mounted open with initial mount-animation suppression guard."
                            }
                        }
                        if show_dynamic() {
                            AccordionItem {
                                value: "decl-dyn".to_string(),
                                style: "border: 1px solid #e2e8f0; border-radius: 0.5rem; overflow: hidden;".to_string(),
                                AccordionHeader {
                                    level: 2,
                                    AccordionTrigger {
                                        id: "accordion-decl-trigger-dyn".to_string(),
                                        style: "width: 100%; padding: 0.65rem 0.9rem; background: #f8fafc; border: none; font-weight: 600; cursor: pointer; text-align: right;".to_string(),
                                        "Dynamic Removable Item"
                                    }
                                }
                                AccordionContent {
                                    style: "padding: 0.65rem 0.9rem; font-size: 0.85rem; color: #334155;".to_string(),
                                    "Unregisters cleanly on unmount via use_drop."
                                }
                            }
                        }
                        AccordionItem {
                            value: "decl-2".to_string(),
                            style: "border: 1px solid #e2e8f0; border-radius: 0.5rem; overflow: hidden;".to_string(),
                            AccordionHeader {
                                level: 2,
                                AccordionTrigger {
                                    id: "accordion-decl-trigger-2".to_string(),
                                    style: "width: 100%; padding: 0.65rem 0.9rem; background: #f8fafc; border: none; font-weight: 600; cursor: pointer; text-align: right;".to_string(),
                                    "Surviving Sibling Item"
                                }
                            }
                            AccordionContent {
                                style: "padding: 0.65rem 0.9rem; font-size: 0.85rem; color: #334155;".to_string(),
                                "Receives roving focus cleanly after dynamic sibling unmounts."
                            }
                        }
                    }
                }
            }

            // Track 2 (SW2B-2): Measured Dimensions & Variable Sizing Accordion
            section {
                style: "display: grid; gap: 0.75rem; padding: 1.25rem; border-radius: 0.75rem; border: 1px solid #cbd5e1; background-color: white;",
                h3 {
                    style: "margin: 0; font-size: 1rem; color: #0f172a;",
                    "7. SW2B Disclosure Dimensions Accordion"
                }
                AccordionRoot {
                    id: "sw2b-accordion-root".to_string(),
                    mode: AccordionMode::Single { collapsible: true },
                    style: "width: 320px; display: grid; gap: 0.5rem;".to_string(),
                    AccordionItem {
                        value: "sw2b-item-1".to_string(),
                        style: "width: 320px; border: 1px solid #cbd5e1; border-radius: 0.5rem; overflow: hidden;".to_string(),
                        AccordionHeader {
                            level: 3,
                            AccordionTrigger {
                                id: "sw2b-accordion-trigger-1".to_string(),
                                class: "sw1a-btn-secondary".to_string(),
                                style: "width: 100%; justify-content: space-between;".to_string(),
                                "Expand Dimension Panel"
                            }
                        }
                        AccordionContent {
                            style: "width: 320px; box-sizing: border-box;".to_string(),
                            div {
                                style: "height: 110px; width: 320px; padding: 12px; box-sizing: border-box; background: #f8fafc;",
                                "Accordion measured dimensions panel (--radix-accordion-content-height / width)."
                            }
                        }
                    }
                }
            }
        }
    }
}

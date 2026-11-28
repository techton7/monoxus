use dioxus::prelude::*;
use monoxus::accordion::{
    AccordionContent, AccordionHeader, AccordionItem, AccordionMode, AccordionRoot,
    AccordionTrigger,
};

const ITEMS: [(&str, &str, &str); 3] = [
    (
        "accessible",
        "Is it accessible?",
        "Yes. Every trigger follows WAI-ARIA Accordion semantics with roving arrow-key focus and aria-expanded/aria-controls wiring.",
    ),
    (
        "unstyled",
        "Is it unstyled?",
        "Yes. Primitives publish semantic data-state and data-orientation attributes so you can style them with Tailwind or plain CSS.",
    ),
    (
        "native",
        "Does it work on Blitz Native?",
        "Yes. State machines live in ui-sdk and DOM/focus bridges run through oxidase on both Web (WASM) and Blitz Native.",
    ),
];

#[component]
pub fn AccordionDemo() -> Element {
    rsx! {
        AccordionRoot {
            mode: AccordionMode::Single { collapsible: true },
            default_value: "accessible".to_string(),
            class: "w-full max-w-md divide-y divide-border-card rounded-card-sm border border-border-card bg-background shadow-mini",

            for (value, title, body) in ITEMS {
                AccordionItem {
                    key: "{value}",
                    value: value.to_string(),
                    class: "group",

                    AccordionHeader {
                        level: 3,
                        class: "flex",
                        AccordionTrigger {
                            class: "flex w-full items-center justify-between px-4 py-3 text-left text-sm font-medium text-foreground transition-colors hover:bg-muted/50",
                            span { "{title}" }
                            span { class: "text-xs text-muted-foreground transition-transform duration-200 group-data-[state=open]:rotate-180",
                                "▾"
                            }
                        }
                    }

                    AccordionContent {
                        class: "px-4 pb-3.5 text-xs leading-relaxed text-muted-foreground",
                        "{body}"
                    }
                }
            }
        }
    }
}

use dioxus::prelude::*;
use monoxus::accordion::{
    AccordionContent, AccordionHeader, AccordionItem, AccordionMode, AccordionRoot,
    AccordionTrigger,
};
use monoxus::dialog::{
    DialogClose, DialogContent, DialogDescription, DialogOverlay, DialogPortal, DialogRoot,
    DialogTitle, DialogTrigger,
};
use monoxus::foundation::overlay::PlacementSide;
use monoxus::slider::{SliderRange, SliderRoot, SliderThumb, SliderTrack, SliderValue};
use monoxus::switch::{SwitchRoot, SwitchThumb};
use monoxus::tabs::{TabsContent, TabsList, TabsRoot, TabsTrigger};
use monoxus::tooltip::{TooltipContent, TooltipPortal, TooltipRoot, TooltipTrigger};

use crate::components::SiteHeader;
use crate::components::icons::{ChevronDownIcon, ChevronRightIcon, CloseIcon};
use crate::routes::Route;

/// Monochrome high-contrast landing page with 4 live interactive Monoxus showcase cards.
#[component]
pub fn Home() -> Element {
    rsx! {
        document::Title { "Monoxus — Headless Primitives for Dioxus" }
        div {
            class: "min-h-screen bg-background text-foreground",
            SiteHeader {}

            main {
                class: "mx-auto max-w-[1440px] px-4 sm:px-6 lg:px-8",

                // Hero
                section {
                    class: "mx-auto max-w-3xl pt-16 pb-14 text-center sm:pt-24 sm:pb-18",
                    div {
                        class: "mb-6 inline-flex items-center gap-2 rounded-full border border-border-card bg-muted/70 px-3 py-1 text-xs font-medium text-foreground-alt",
                        span { class: "h-1.5 w-1.5 rounded-full bg-foreground" }
                        span { "v0.1.2 · 44 Headless Primitives for Dioxus 0.8" }
                    }
                    h1 {
                        class: "text-4xl font-semibold tracking-tight text-foreground sm:text-6xl",
                        "Headless primitives for Dioxus."
                    }
                    p {
                        class: "mx-auto mt-5 max-w-2xl text-base leading-7 text-foreground-alt sm:text-lg",
                        "Unstyled, WAI-ARIA compliant UI components built in Rust. Write your markup and state once—run identically across Web (WASM) and Blitz Native."
                    }
                    div {
                        class: "mt-8 flex flex-wrap items-center justify-center gap-3",
                        Link {
                            to: Route::OverviewDoc { slug: "getting-started".to_string() },
                            class: "inline-flex h-11 items-center gap-2 rounded-input bg-dark px-5 text-sm font-semibold text-background shadow-btn transition-opacity hover:opacity-90",
                            span { "Start building" }
                            ChevronRightIcon { class: "h-4 w-4" }
                        }
                        Link {
                            to: Route::ComponentDocPage { slug: "button".to_string() },
                            class: "inline-flex h-11 items-center gap-2 rounded-input border border-border-input bg-background-alt px-5 text-sm font-medium text-foreground shadow-mini transition-colors hover:border-border-input-hover hover:bg-muted/50",
                            "View Components"
                        }
                    }
                }

                // 4 Live Interactive Showcase Cards
                section {
                    aria_label: "Interactive primitive demos",
                    class: "grid grid-cols-1 gap-5 md:grid-cols-2 desktop:grid-cols-4",
                    AudioControlsCard {}
                    QuickSettingsCard {}
                    ModalTooltipCard {}
                    FaqAccordionCard {}
                }

                // 3 Value-Prop Cards
                section {
                    aria_label: "Core principles",
                    class: "mt-16 grid grid-cols-1 gap-5 md:grid-cols-3",
                    ValuePropCard {
                        title: "Unstyled by Default".to_string(),
                        description: "Zero CSS assumptions or bundled themes. Style with Tailwind v4 utilities, CSS variables, or state selectors like data-[state=open].".to_string(),
                    }
                    ValuePropCard {
                        title: "WAI-ARIA & Keyboard First".to_string(),
                        description: "Roving tabindex, focus trapping, outside-dismiss layers, and deterministic ARIA IDs are verified inside monoxus-core.".to_string(),
                    }
                    ValuePropCard {
                        title: "Web + Blitz Native".to_string(),
                        description: "Pure state and attribute projection in monoxus-core paired with dual-host runtimes for browser WASM and Blitz desktop windows.".to_string(),
                    }
                }
            }

            // Minimal Footer
            footer {
                class: "mt-24 border-t border-border-card",
                div {
                    class: "mx-auto flex max-w-[1440px] flex-col items-center justify-between gap-4 px-4 py-8 text-xs text-foreground-alt sm:flex-row sm:px-6 lg:px-8",
                    div {
                        class: "font-medium text-foreground",
                        "monoxus.com"
                    }
                    div {
                        class: "flex items-center gap-6",
                        a {
                            href: "/llms.txt",
                            class: "transition-colors hover:text-foreground",
                            "LLMs"
                        }
                        a {
                            href: "https://github.com/techton7/monoxus",
                            target: "_blank",
                            rel: "noreferrer",
                            class: "transition-colors hover:text-foreground",
                            "GitHub"
                        }
                    }
                }
            }
        }
    }
}

#[component]
fn AudioControlsCard() -> Element {
    let mut gain = use_signal(|| 64.0f64);
    let mut muted = use_signal(|| false);
    let display_gain = if muted() { 0 } else { gain().round() as i64 };

    rsx! {
        div {
            class: "flex flex-col justify-between rounded-card border border-border-card bg-background-alt p-5 shadow-card",
            div {
                class: "space-y-1",
                div {
                    class: "flex items-center justify-between",
                    span { class: "text-xs font-medium text-muted-foreground uppercase tracking-wider", "Slider + Switch" }
                    span { class: "font-mono text-xs font-semibold text-foreground", "{display_gain}%" }
                }
                h3 { class: "text-base font-semibold text-foreground", "Audio Gain" }
            }

            div {
                class: "my-6 space-y-5",
                SliderRoot {
                    min: 0.0,
                    max: 100.0,
                    step: 1.0,
                    value: Some(SliderValue::Single(if muted() { 0.0 } else { gain() })),
                    disabled: muted(),
                    on_value_change: move |val| {
                        if let SliderValue::Single(v) = val {
                            gain.set(v);
                        }
                    },
                    class: "relative flex h-6 w-full touch-none select-none items-center data-[disabled]:opacity-40",
                    SliderTrack {
                        class: "relative h-2 w-full grow overflow-hidden rounded-full bg-muted",
                        SliderRange {
                            class: "absolute h-full rounded-full bg-dark",
                        }
                    }
                    SliderThumb {
                        index: 0,
                        aria_label: Some("Audio gain".to_string()),
                        class: "block h-4 w-4 rounded-full border border-border-input bg-background shadow-mini transition-transform focus-visible:outline-hidden focus-visible:ring-2 focus-visible:ring-foreground",
                    }
                }

                div {
                    class: "flex items-center justify-between rounded-card-sm border border-border-card bg-muted/40 px-3 py-2.5",
                    span { class: "text-xs font-medium text-foreground", "Mute output" }
                    SwitchRoot {
                        checked: Some(muted()),
                        on_checked_change: move |next| muted.set(next),
                        class: "inline-flex h-5 w-9 shrink-0 cursor-pointer items-center rounded-full border border-border-input bg-muted p-0.5 transition-colors data-[state=checked]:bg-dark",
                        SwitchThumb {
                            class: "pointer-events-none block h-3.5 w-3.5 rounded-full bg-background shadow-mini transition-transform data-[state=checked]:translate-x-4",
                        }
                    }
                }
            }

            p {
                class: "text-xs text-foreground-alt",
                "Pointer dragging, keyboard stepping, and accessible switch state."
            }
        }
    }
}

#[component]
fn QuickSettingsCard() -> Element {
    rsx! {
        div {
            class: "flex flex-col justify-between rounded-card border border-border-card bg-background-alt p-5 shadow-card",
            div {
                class: "space-y-1",
                span { class: "text-xs font-medium text-muted-foreground uppercase tracking-wider", "Tabs" }
                h3 { class: "text-base font-semibold text-foreground", "Target Runtime" }
            }

            div {
                class: "my-5",
                TabsRoot {
                    default_value: Some("web".to_string()),
                    class: "w-full",
                    TabsList {
                        class: "grid w-full grid-cols-2 rounded-button bg-muted p-1",
                        TabsTrigger {
                            value: "web".to_string(),
                            class: "rounded-[4px] py-1.5 text-xs font-medium text-foreground-alt transition-colors data-[state=active]:bg-background data-[state=active]:text-foreground data-[state=active]:shadow-mini",
                            "Web WASM"
                        }
                        TabsTrigger {
                            value: "native".to_string(),
                            class: "rounded-[4px] py-1.5 text-xs font-medium text-foreground-alt transition-colors data-[state=active]:bg-background data-[state=active]:text-foreground data-[state=active]:shadow-mini",
                            "Blitz Native"
                        }
                    }
                    TabsContent {
                        value: "web".to_string(),
                        class: "mt-3 rounded-card-sm border border-border-card bg-muted/30 p-3 text-xs leading-relaxed text-foreground-alt outline-hidden",
                        div { class: "font-mono font-semibold text-foreground", "wasm32-unknown-unknown" }
                        p { class: "mt-1", "DOM observers, Floating UI positioning, and native browser focus restoration." }
                    }
                    TabsContent {
                        value: "native".to_string(),
                        class: "mt-3 rounded-card-sm border border-border-card bg-muted/30 p-3 text-xs leading-relaxed text-foreground-alt outline-hidden",
                        div { class: "font-mono font-semibold text-foreground", "--features native" }
                        p { class: "mt-1", "Same Dioxus RSX and state machines backed by Blitz layout and Vello." }
                    }
                }
            }

            p {
                class: "text-xs text-foreground-alt",
                "Roving focus across triggers with automatic or manual activation."
            }
        }
    }
}

#[component]
fn ModalTooltipCard() -> Element {
    let mut modal_open = use_signal(|| false);

    rsx! {
        div {
            class: "flex flex-col justify-between rounded-card border border-border-card bg-background-alt p-5 shadow-card",
            div {
                class: "space-y-1",
                span { class: "text-xs font-medium text-muted-foreground uppercase tracking-wider", "Dialog + Tooltip" }
                h3 { class: "text-base font-semibold text-foreground", "Overlay Layers" }
            }

            div {
                class: "my-6 flex flex-col items-center justify-center gap-3 rounded-card-sm border border-dashed border-border-input bg-muted/25 p-5",
                TooltipRoot {
                    TooltipTrigger {
                        class: "rounded-button border border-border-input bg-background px-3 py-1.5 text-xs font-medium text-foreground-alt shadow-mini transition-colors hover:border-border-input-hover hover:text-foreground",
                        "Hover for hint"
                    }
                    TooltipPortal {
                        TooltipContent {
                            side: PlacementSide::Top,
                            side_offset: 6.0,
                            class: "relative z-50 rounded-button border border-border-card bg-dark px-2.5 py-1 text-xs font-medium text-background shadow-popover",
                            "Keyboard focus & escape dismiss"
                        }
                    }
                }

                DialogRoot {
                    open: Some(modal_open()),
                    on_open_change: move |next| modal_open.set(next),
                    DialogTrigger {
                        class: "inline-flex h-9 items-center justify-center rounded-button bg-dark px-4 text-xs font-semibold text-background shadow-btn transition-opacity hover:opacity-90",
                        "Open modal"
                    }
                    DialogPortal {
                        DialogOverlay {
                            class: "fixed inset-0 z-50 bg-black/50 backdrop-blur-[2px] animate-fade-in",
                        }
                        DialogContent {
                            class: "fixed left-1/2 top-1/2 z-50 w-[min(92vw,26rem)] -translate-x-1/2 -translate-y-1/2 rounded-card border border-border-card bg-background p-6 shadow-popover outline-hidden animate-scale-in",
                            div {
                                class: "flex items-start justify-between gap-4",
                                DialogTitle {
                                    class: "text-base font-semibold text-foreground",
                                    "Modal Focus Scope"
                                }
                                DialogClose {
                                    class: "inline-flex h-7 w-7 items-center justify-center rounded-button text-muted-foreground hover:bg-muted hover:text-foreground",
                                    span { class: "sr-only", "Close modal" }
                                    CloseIcon { class: "h-4 w-4" }
                                }
                            }
                            DialogDescription {
                                class: "mt-2 text-sm leading-relaxed text-foreground-alt",
                                "Focus stays trapped inside this dialog while open, and returns cleanly to the trigger on Escape or outside click."
                            }
                            div {
                                class: "mt-5 flex justify-end",
                                DialogClose {
                                    class: "inline-flex h-9 items-center rounded-button bg-dark px-4 text-xs font-semibold text-background hover:opacity-90",
                                    "Done"
                                }
                            }
                        }
                    }
                }
            }

            p {
                class: "text-xs text-foreground-alt",
                "Portal rendering, scroll locking, and focus restoration."
            }
        }
    }
}

#[component]
fn FaqAccordionCard() -> Element {
    rsx! {
        div {
            class: "flex flex-col justify-between rounded-card border border-border-card bg-background-alt p-5 shadow-card",
            div {
                class: "space-y-1",
                span { class: "text-xs font-medium text-muted-foreground uppercase tracking-wider", "Accordion" }
                h3 { class: "text-base font-semibold text-foreground", "Quick Answers" }
            }

            div {
                class: "my-4",
                AccordionRoot {
                    mode: AccordionMode::Single { collapsible: true },
                    default_value: Some("unstyled".to_string()),
                    class: "w-full divide-y divide-border-card rounded-card-sm border border-border-card bg-background",
                    AccordionItem {
                        value: "unstyled".to_string(),
                        class: "px-3",
                        AccordionHeader {
                            level: 4,
                            AccordionTrigger {
                                class: "group flex w-full items-center justify-between py-2.5 text-left text-xs font-semibold text-foreground",
                                span { "Is CSS included?" }
                                ChevronDownIcon { class: "h-3.5 w-3.5 text-muted-foreground transition-transform duration-150 group-data-[state=open]:rotate-180" }
                            }
                        }
                        AccordionContent {
                            class: "overflow-hidden text-xs leading-relaxed text-foreground-alt data-[state=open]:animate-accordion-down data-[state=closed]:animate-accordion-up",
                            p { class: "pb-2.5", "No. Bring Tailwind v4, CSS modules, or plain stylesheets." }
                        }
                    }
                    AccordionItem {
                        value: "a11y".to_string(),
                        class: "px-3",
                        AccordionHeader {
                            level: 4,
                            AccordionTrigger {
                                class: "group flex w-full items-center justify-between py-2.5 text-left text-xs font-semibold text-foreground",
                                span { "Keyboard support?" }
                                ChevronDownIcon { class: "h-3.5 w-3.5 text-muted-foreground transition-transform duration-150 group-data-[state=open]:rotate-180" }
                            }
                        }
                        AccordionContent {
                            class: "overflow-hidden text-xs leading-relaxed text-foreground-alt data-[state=open]:animate-accordion-down data-[state=closed]:animate-accordion-up",
                            p { class: "pb-2.5", "Full WAI-ARIA roles, arrow navigation, and CSS height variables." }
                        }
                    }
                }
            }

            p {
                class: "text-xs text-foreground-alt",
                "Smooth height keyframes via --monoxus-accordion-content-height."
            }
        }
    }
}

#[component]
fn ValuePropCard(title: String, description: String) -> Element {
    rsx! {
        div {
            class: "rounded-card border border-border-card bg-background-alt p-6 shadow-mini",
            h3 {
                class: "text-base font-semibold tracking-tight text-foreground",
                "{title}"
            }
            p {
                class: "mt-2 text-sm leading-6 text-foreground-alt",
                "{description}"
            }
        }
    }
}

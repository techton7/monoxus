use dioxus::prelude::*;
use monoxus::foundation::overlay::PlacementSide;
use monoxus::popover::{PopoverContent, PopoverPortal, PopoverRoot, PopoverTrigger};

use crate::api_schema::{CssVarDoc, DataAttrDoc, KeyboardDoc, PropDoc};
use crate::components::icons::InfoIcon;

#[derive(Props, Clone, PartialEq)]
pub struct PropsTableProps {
    pub part_name: &'static str,
    pub props: &'static [PropDoc],
}

/// Compact Radix-style Props reference table with expandable Popover details.
#[component]
pub fn PropsTable(props: PropsTableProps) -> Element {
    let _part_name = props.part_name;
    let props = props.props;
    if props.is_empty() {
        return rsx! {
            p { class: "text-sm text-muted-foreground", "No component-specific props." }
        };
    }

    rsx! {
        div {
            class: "my-4 overflow-x-auto rounded-card-sm border border-border-card",
            table {
                class: "w-full border-collapse text-left text-sm",
                thead {
                    class: "border-b border-border-card bg-muted/50",
                    tr {
                        th { class: "px-4 py-2.5 text-xs font-semibold tracking-wide text-foreground uppercase", "Prop" }
                        th { class: "px-4 py-2.5 text-xs font-semibold tracking-wide text-foreground uppercase", "Type" }
                        th { class: "px-4 py-2.5 text-xs font-semibold tracking-wide text-foreground uppercase", "Default" }
                    }
                }
                tbody {
                    for prop in props.iter() {
                        tr {
                            key: "{prop.name}",
                            class: "border-t border-border-card transition-colors hover:bg-muted/25",
                            td {
                                class: "whitespace-nowrap px-4 py-2.5 align-middle",
                                div {
                                    class: "inline-flex items-center gap-1.5",
                                    code {
                                        class: "rounded bg-muted px-1.5 py-0.5 font-mono text-xs font-semibold text-foreground",
                                        "{prop.name}"
                                        if prop.required {
                                            span { class: "ml-0.5 text-destructive", "*" }
                                        }
                                    }
                                    PopoverRoot {
                                        side: PlacementSide::Top,
                                        side_offset: 6.0,
                                        PopoverTrigger {
                                            class: "inline-flex h-5 w-5 items-center justify-center rounded-full text-muted-foreground transition-colors hover:bg-muted hover:text-foreground",
                                            span { class: "sr-only", "Details for {prop.name}" }
                                            InfoIcon { class: "h-3.5 w-3.5" }
                                        }
                                        PopoverPortal {
                                            PopoverContent {
                                                class: "z-50 w-72 rounded-card-sm border border-border-card bg-background p-3.5 text-xs leading-relaxed text-foreground shadow-popover outline-hidden",
                                                div {
                                                    class: "mb-1.5 font-mono text-[11px] font-semibold text-foreground",
                                                    "{prop.name}: {prop.type_full}"
                                                }
                                                p {
                                                    class: "text-foreground-alt",
                                                    "{prop.description}"
                                                }
                                            }
                                        }
                                    }
                                }
                            }
                            td {
                                class: "px-4 py-2.5 align-middle",
                                code {
                                    class: "rounded border border-border-card bg-background-alt px-1.5 py-0.5 font-mono text-xs text-foreground-alt",
                                    "{prop.type_simple}"
                                }
                            }
                            td {
                                class: "whitespace-nowrap px-4 py-2.5 align-middle font-mono text-xs text-foreground-alt",
                                if prop.required {
                                    "Required"
                                } else {
                                    "{prop.default.unwrap_or(\"—\")}"
                                }
                            }
                        }
                    }
                }
            }
        }
    }
}

/// Data attributes exposed on rendered DOM elements for styling hooks.
#[component]
pub fn DataAttributesTable(attrs: &'static [DataAttrDoc]) -> Element {
    if attrs.is_empty() {
        return rsx! {};
    }

    rsx! {
        div {
            class: "my-4 overflow-x-auto rounded-card-sm border border-border-card",
            table {
                class: "w-full border-collapse text-left text-sm",
                thead {
                    class: "border-b border-border-card bg-muted/50",
                    tr {
                        th { class: "w-[34%] px-4 py-2.5 text-xs font-semibold tracking-wide text-foreground uppercase", "Data attribute" }
                        th { class: "w-[26%] px-4 py-2.5 text-xs font-semibold tracking-wide text-foreground uppercase", "Values" }
                        th { class: "px-4 py-2.5 text-xs font-semibold tracking-wide text-foreground uppercase", "Description" }
                    }
                }
                tbody {
                    for item in attrs.iter() {
                        tr {
                            key: "{item.name}",
                            class: "border-t border-border-card transition-colors hover:bg-muted/25",
                            td {
                                class: "px-4 py-2.5 align-top font-mono text-xs font-medium text-foreground",
                                "{item.name}"
                            }
                            td {
                                class: "px-4 py-2.5 align-top",
                                code {
                                    class: "rounded bg-muted px-1.5 py-0.5 font-mono text-xs text-foreground-alt",
                                    "{item.values}"
                                }
                            }
                            td {
                                class: "px-4 py-2.5 align-top text-xs leading-relaxed text-foreground-alt",
                                "{item.description}"
                            }
                        }
                    }
                }
            }
        }
    }
}

/// CSS custom properties dynamically populated by a component part.
#[component]
pub fn CssVariablesTable(vars: &'static [CssVarDoc]) -> Element {
    if vars.is_empty() {
        return rsx! {};
    }

    rsx! {
        div {
            class: "my-4 overflow-x-auto rounded-card-sm border border-border-card",
            table {
                class: "w-full border-collapse text-left text-sm",
                thead {
                    class: "border-b border-border-card bg-muted/50",
                    tr {
                        th { class: "w-[42%] px-4 py-2.5 text-xs font-semibold tracking-wide text-foreground uppercase", "CSS Variable" }
                        th { class: "px-4 py-2.5 text-xs font-semibold tracking-wide text-foreground uppercase", "Description" }
                    }
                }
                tbody {
                    for var in vars.iter() {
                        tr {
                            key: "{var.name}",
                            class: "border-t border-border-card transition-colors hover:bg-muted/25",
                            td {
                                class: "px-4 py-2.5 align-top font-mono text-xs font-medium text-foreground",
                                "{var.name}"
                            }
                            td {
                                class: "px-4 py-2.5 align-top text-xs leading-relaxed text-foreground-alt",
                                "{var.description}"
                            }
                        }
                    }
                }
            }
        }
    }
}

/// WAI-ARIA keyboard interactions table with `<kbd>` key chips.
#[component]
pub fn KeyboardTable(keys: &'static [KeyboardDoc]) -> Element {
    if keys.is_empty() {
        return rsx! {};
    }

    rsx! {
        div {
            class: "my-4 overflow-x-auto rounded-card-sm border border-border-card",
            table {
                class: "w-full border-collapse text-left text-sm",
                thead {
                    class: "border-b border-border-card bg-muted/50",
                    tr {
                        th { class: "w-[32%] px-4 py-2.5 text-xs font-semibold tracking-wide text-foreground uppercase", "Key" }
                        th { class: "px-4 py-2.5 text-xs font-semibold tracking-wide text-foreground uppercase", "Description" }
                    }
                }
                tbody {
                    for item in keys.iter() {
                        tr {
                            key: "{item.key}",
                            class: "border-t border-border-card transition-colors hover:bg-muted/25",
                            td {
                                class: "px-4 py-2.5 align-top",
                                kbd {
                                    class: "inline-flex items-center rounded border border-border-input bg-muted/70 px-2 py-0.5 font-mono text-xs font-medium text-foreground shadow-kbd",
                                    "{item.key}"
                                }
                            }
                            td {
                                class: "px-4 py-2.5 align-top text-xs leading-relaxed text-foreground-alt",
                                "{item.description}"
                            }
                        }
                    }
                }
            }
        }
    }
}

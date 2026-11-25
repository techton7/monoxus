use dioxus::prelude::*;
use monoxus::select::{
    SelectContent, SelectIcon, SelectItem, SelectItemData, SelectItemIndicator, SelectItemText,
    SelectPortal, SelectRoot, SelectTrigger, SelectValue, SelectViewport,
};

const TARGETS: &[(&str, &str)] = &[
    ("web", "Dioxus Web"),
    ("blitz", "Blitz Native"),
    ("desktop", "Desktop"),
    ("fullstack", "Fullstack"),
];

#[component]
pub fn SelectDemo() -> Element {
    let mut selected = use_signal(|| Some(String::from("web")));
    let items: Vec<SelectItemData> = TARGETS
        .iter()
        .map(|(value, label)| SelectItemData::new(*value, *label, false))
        .collect();

    rsx! {
        div { class: "flex flex-col items-center gap-3",
            SelectRoot {
                value: selected(),
                items: Some(items),
                on_value_change: move |next| selected.set(next),

                SelectTrigger {
                    class: "inline-flex h-10 w-60 items-center justify-between rounded-input border border-border-input bg-background px-3.5 text-sm text-foreground shadow-mini transition-colors hover:bg-background-alt focus:outline-none",
                    SelectValue {
                        placeholder: "Select a runtime",
                        class: "truncate",
                    }
                    SelectIcon {
                        class: "ml-2 text-xs text-muted-foreground",
                        "▾"
                    }
                }

                SelectPortal {
                    SelectContent {
                        side_offset: 6.0,
                        same_width: true,
                        class: "z-50 overflow-hidden rounded-card-sm border border-border-card bg-background p-1 shadow-popover outline-none",

                        SelectViewport {
                            class: "max-h-60 space-y-0.5",
                            for (value, label) in TARGETS {
                                SelectItem {
                                    key: "{value}",
                                    value: value.to_string(),
                                    text: label.to_string(),
                                    class: "relative flex cursor-pointer select-none items-center justify-between rounded-button px-3 py-2 text-sm text-foreground outline-none transition-colors data-[highlighted=true]:bg-muted data-[state=checked]:font-medium",
                                    SelectItemText { "{label}" }
                                    if selected().as_deref() == Some(*value) {
                                        SelectItemIndicator {
                                            class: "text-xs text-foreground",
                                            "✓"
                                        }
                                    }
                                }
                            }
                        }
                    }
                }
            }

            p { class: "text-xs text-muted-foreground",
                "Target: "
                span { class: "font-medium text-foreground",
                    "{selected().unwrap_or_else(|| String::from(\"none\"))}"
                }
            }
        }
    }
}

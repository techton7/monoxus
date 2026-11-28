use dioxus::prelude::*;
use monoxus::combobox::{
    ComboboxContent, ComboboxFilterMode, ComboboxInput, ComboboxItem, ComboboxItemData,
    ComboboxPortal, ComboboxRoot, ComboboxTrigger, ComboboxViewport, matches_query,
};

const FRAMEWORKS: &[(&str, &str)] = &[
    ("dioxus", "Dioxus"),
    ("blitz", "Blitz Native"),
    ("oxidase", "Oxidase"),
    ("monoxus", "Monoxus"),
    ("taffy", "Taffy Layout"),
    ("stylo", "Stylo CSS"),
];

#[component]
pub fn ComboboxDemo() -> Element {
    let mut selected = use_signal(|| Some(String::from("monoxus")));
    let mut query = use_signal(|| String::from("Monoxus"));

    let items: Vec<ComboboxItemData> = FRAMEWORKS
        .iter()
        .map(|(value, label)| ComboboxItemData::new(*value, *label, false))
        .collect();

    let current_query = query();
    let selected_label = selected().and_then(|val| {
        FRAMEWORKS
            .iter()
            .find(|(v, _)| *v == val)
            .map(|(_, label)| *label)
    });
    let show_all = selected_label.is_some_and(|lbl| lbl.eq_ignore_ascii_case(current_query.trim()));
    let filtered: Vec<(&str, &str)> = FRAMEWORKS
        .iter()
        .copied()
        .filter(|(_, label)| {
            show_all || matches_query(label, &current_query, ComboboxFilterMode::Contains)
        })
        .collect();

    rsx! {
        div { class: "flex flex-col items-center gap-3",
            ComboboxRoot {
                value: Some(selected()),
                input_value: current_query.clone(),
                items,
                on_value_change: move |next: Option<String>| selected.set(next),
                on_input_value_change: move |next: String| query.set(next),

                div { class: "relative inline-flex w-64 items-center",
                    ComboboxInput {
                        placeholder: "Search a crate...",
                        class: "h-10 w-full rounded-input border border-border-input bg-background pl-3.5 pr-9 text-sm text-foreground shadow-mini outline-none transition-colors placeholder:text-muted-foreground focus:border-foreground",
                    }
                    ComboboxTrigger {
                        class: "absolute right-1.5 inline-flex h-7 w-7 items-center justify-center rounded-button text-xs text-muted-foreground transition-colors hover:bg-muted hover:text-foreground",
                        "▾"
                    }
                }

                ComboboxPortal {
                    ComboboxContent {
                        side_offset: 6.0,
                        same_width: true,
                        class: "z-50 overflow-hidden rounded-card-sm border border-border-card bg-background p-1 shadow-popover outline-none",

                        ComboboxViewport {
                            class: "max-h-56 space-y-0.5",
                            if filtered.is_empty() {
                                div { class: "px-3 py-2 text-center text-xs text-muted-foreground",
                                    "No matching crates."
                                }
                            } else {
                                for (value, label) in filtered {
                                    ComboboxItem {
                                        key: "{value}",
                                        value: value,
                                        label: label,
                                        class: "flex cursor-pointer select-none items-center justify-between rounded-button px-3 py-2 text-sm text-foreground outline-none transition-colors data-[highlighted]:bg-muted data-[selected]:font-medium",
                                        span { "{label}" }
                                        if selected().as_deref() == Some(value) {
                                            span { class: "text-xs text-foreground", "✓" }
                                        }
                                    }
                                }
                            }
                        }
                    }
                }
            }

            p { class: "text-xs text-muted-foreground",
                "Selected: "
                span { class: "font-medium text-foreground",
                    "{selected().unwrap_or_else(|| String::from(\"none\"))}"
                }
            }
        }
    }
}

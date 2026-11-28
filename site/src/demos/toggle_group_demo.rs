use dioxus::prelude::*;
use monoxus::toggle_group::{ToggleGroupItem, ToggleGroupMode, ToggleGroupRoot};

#[component]
pub fn ToggleGroupDemo() -> Element {
    let mut alignment = use_signal(|| "center".to_string());

    rsx! {
        div { class: "flex flex-col items-center gap-3",
            ToggleGroupRoot {
                r#type: ToggleGroupMode::Single,
                value: Some(alignment()),
                on_value_change: move |next: String| {
                    if !next.is_empty() {
                        alignment.set(next);
                    }
                },
                class: "inline-flex items-center gap-1 rounded-input border border-border-card bg-muted p-1 shadow-mini",

                for (val, label) in [("left", "Left"), ("center", "Center"), ("right", "Right")] {
                    ToggleGroupItem {
                        key: "{val}",
                        value: val.to_string(),
                        class: "rounded-button px-3 py-1.5 text-xs font-medium text-muted-foreground transition-colors hover:text-foreground data-[state=on]:bg-background data-[state=on]:text-foreground data-[state=on]:shadow-mini",
                        "{label}"
                    }
                }
            }

            span { class: "text-xs text-muted-foreground",
                "Alignment: "
                span { class: "font-medium text-foreground", "{alignment}" }
            }
        }
    }
}

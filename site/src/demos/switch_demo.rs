use dioxus::prelude::*;
use monoxus::label::Label;
use monoxus::switch::{SwitchRoot, SwitchThumb};

#[component]
pub fn SwitchDemo() -> Element {
    let mut airplane_mode = use_signal(|| false);

    rsx! {
        div { class: "flex items-center gap-3 rounded-card-sm border border-border-card bg-background px-4 py-3 shadow-mini",
            SwitchRoot {
                id: "airplane-mode",
                checked: Some(airplane_mode()),
                on_checked_change: move |next| airplane_mode.set(next),
                class: "peer inline-flex h-6 w-11 shrink-0 cursor-pointer items-center rounded-full border border-border-input bg-muted p-0.5 transition-colors data-[state=checked]:border-dark data-[state=checked]:bg-dark",
                SwitchThumb {
                    class: "pointer-events-none block h-4 w-4 rounded-full bg-background shadow-mini transition-transform data-[state=checked]:translate-x-5 data-[state=unchecked]:translate-x-0",
                }
            }
            div { class: "flex flex-col",
                Label {
                    r#for: "airplane-mode",
                    class: "text-sm font-medium text-foreground select-none",
                    "Airplane mode"
                }
                span { class: "text-xs text-muted-foreground",
                    if airplane_mode() { "Enabled" } else { "Disabled" }
                }
            }
        }
    }
}

use dioxus::prelude::*;
use monoxus::toggle::Toggle;

#[component]
pub fn ToggleDemo() -> Element {
    let mut muted = use_signal(|| false);

    rsx! {
        div { class: "flex items-center gap-3",
            Toggle {
                pressed: Some(muted()),
                on_pressed_change: move |next| muted.set(next),
                class: "inline-flex h-10 items-center justify-center gap-2 rounded-button border border-border-input bg-background px-4 text-sm font-medium text-foreground shadow-mini transition-colors hover:bg-muted data-[state=on]:border-dark data-[state=on]:bg-dark data-[state=on]:text-background",
                if muted() { "Muted" } else { "Mute audio" }
            }

            span { class: "text-xs text-muted-foreground",
                if muted() { "State: on" } else { "State: off" }
            }
        }
    }
}

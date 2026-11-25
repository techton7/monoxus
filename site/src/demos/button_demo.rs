use dioxus::prelude::*;
use monoxus::button::{Button, ButtonType};

#[component]
pub fn ButtonDemo() -> Element {
    let mut count = use_signal(|| 0usize);

    rsx! {
        div { class: "flex flex-wrap items-center justify-center gap-3",
            Button {
                r#type: ButtonType::Button,
                class: "inline-flex h-10 items-center justify-center rounded-button bg-dark px-4 text-sm font-medium text-background shadow-btn transition-colors hover:bg-dark/90 active:scale-[0.98] disabled:pointer-events-none disabled:opacity-50",
                on_click: move |_| count += 1,
                "Clicked {count} times"
            }

            Button {
                r#type: ButtonType::Button,
                disabled: count() == 0,
                class: "inline-flex h-10 items-center justify-center rounded-button border border-border-input bg-background px-4 text-sm font-medium text-foreground shadow-mini transition-colors hover:bg-muted active:scale-[0.98] disabled:pointer-events-none disabled:opacity-40",
                on_click: move |_| count.set(0),
                "Reset"
            }

            Button {
                href: "https://github.com/techton7/monoxus",
                target: "_blank",
                rel: "noopener noreferrer",
                class: "inline-flex h-10 items-center justify-center gap-1.5 rounded-button border border-border-card bg-background-alt px-4 text-sm font-medium text-foreground-alt shadow-mini transition-colors hover:bg-muted hover:text-foreground",
                "GitHub"
                span { class: "text-xs text-muted-foreground", "↗" }
            }
        }
    }
}

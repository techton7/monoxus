use dioxus::prelude::*;
use monoxus::aspect_ratio::AspectRatio;
use monoxus::button::{Button, ButtonType};

#[component]
pub fn AspectRatioDemo() -> Element {
    let mut ratio_label = use_signal(|| "16:9");
    let ratio = match ratio_label() {
        "4:3" => 4.0 / 3.0,
        "1:1" => 1.0,
        _ => 16.0 / 9.0,
    };

    rsx! {
        div { class: "flex w-full max-w-sm flex-col gap-3",
            div { class: "flex items-center justify-center gap-1.5",
                for preset in ["16:9", "4:3", "1:1"] {
                    Button {
                        key: "{preset}",
                        r#type: ButtonType::Button,
                        class: if ratio_label() == preset {
                            "rounded-button bg-dark px-3 py-1 text-xs font-medium text-background shadow-mini"
                        } else {
                            "rounded-button border border-border-input bg-background px-3 py-1 text-xs font-medium text-foreground-alt hover:bg-muted"
                        },
                        on_click: move |_| ratio_label.set(preset),
                        "{preset}"
                    }
                }
            }

            div { class: "overflow-hidden rounded-card-sm border border-border-card bg-background-alt shadow-mini",
                AspectRatio {
                    ratio,
                    class: "flex items-center justify-center bg-muted/60",
                    div { class: "flex flex-col items-center gap-1 text-center",
                        span { class: "text-lg font-semibold tracking-tight text-foreground", "{ratio_label}" }
                        span { class: "text-xs text-muted-foreground", "Fixed aspect ratio frame" }
                    }
                }
            }
        }
    }
}

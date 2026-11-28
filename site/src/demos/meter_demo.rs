use dioxus::prelude::*;
use monoxus::button::{Button, ButtonType};
use monoxus::meter::MeterRoot;

#[component]
pub fn MeterDemo() -> Element {
    let mut usage = use_signal(|| 68.0_f64);
    let pct = usage().clamp(0.0, 100.0);

    rsx! {
        div { class: "flex w-full max-w-sm flex-col gap-3",
            div { class: "flex items-center justify-between text-xs",
                span { class: "font-medium text-foreground", "Storage used" }
                span { class: "text-muted-foreground", "{pct:.0} / 100 GB" }
            }

            MeterRoot {
                value: pct,
                min: 0.0,
                max: 100.0,
                class: "h-2.5 w-full overflow-hidden rounded-full bg-muted",
                div {
                    class: "h-full rounded-full bg-dark transition-all duration-200",
                    style: "width: {pct}%",
                }
            }

            div { class: "flex items-center justify-end gap-2",
                Button {
                    r#type: ButtonType::Button,
                    class: "rounded-button border border-border-input bg-background px-2.5 py-1 text-xs font-medium text-foreground shadow-mini hover:bg-muted",
                    on_click: move |_| usage.set((usage() - 15.0).max(0.0)),
                    "-15 GB"
                }
                Button {
                    r#type: ButtonType::Button,
                    class: "rounded-button border border-border-input bg-background px-2.5 py-1 text-xs font-medium text-foreground shadow-mini hover:bg-muted",
                    on_click: move |_| usage.set((usage() + 15.0).min(100.0)),
                    "+15 GB"
                }
            }
        }
    }
}

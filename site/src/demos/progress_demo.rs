use dioxus::prelude::*;
use monoxus::button::{Button, ButtonType};
use monoxus::progress::{ProgressIndicator, ProgressRoot};

#[component]
pub fn ProgressDemo() -> Element {
    let mut progress = use_signal(|| Some(66.0_f64));
    let pct = progress().unwrap_or(0.0).clamp(0.0, 100.0);

    rsx! {
        div { class: "flex w-full max-w-sm flex-col gap-3",
            div { class: "flex items-center justify-between text-xs",
                span { class: "font-medium text-foreground", "Building WASM bundle" }
                span { class: "text-muted-foreground",
                    if let Some(val) = progress() {
                        "{val:.0}%"
                    } else {
                        "Indeterminate"
                    }
                }
            }

            ProgressRoot {
                value: progress(),
                max: 100.0,
                get_value_label: Some(Callback::new(|(val, max): (f64, f64)| {
                    format!("{:.0}% of {:.0}", val, max)
                })),
                class: "h-2.5 w-full overflow-hidden rounded-full bg-muted",
                ProgressIndicator {
                    class: "h-full rounded-full bg-dark transition-all duration-300 data-[state=indeterminate]:w-1/3 data-[state=indeterminate]:animate-pulse",
                    style: if progress().is_some() {
                        format!("width: {pct}%")
                    } else {
                        String::new()
                    },
                }
            }

            div { class: "flex items-center justify-end gap-2",
                Button {
                    r#type: ButtonType::Button,
                    class: "rounded-button border border-border-input bg-background px-2.5 py-1 text-xs font-medium text-foreground shadow-mini hover:bg-muted",
                    on_click: move |_| progress.set(None),
                    "Indeterminate"
                }
                Button {
                    r#type: ButtonType::Button,
                    class: "rounded-button border border-border-input bg-background px-2.5 py-1 text-xs font-medium text-foreground shadow-mini hover:bg-muted",
                    on_click: move |_| {
                        let next = match progress() {
                            Some(v) if v < 100.0 => (v + 20.0).min(100.0),
                            _ => 20.0,
                        };
                        progress.set(Some(next));
                    },
                    "+20%"
                }
            }
        }
    }
}

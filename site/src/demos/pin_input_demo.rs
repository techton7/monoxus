use dioxus::prelude::*;
use monoxus::pin_input::{PinInputCell, PinInputPattern, PinInputRoot};

#[component]
pub fn PinInputDemo() -> Element {
    let mut code = use_signal(String::new);
    let mut completed = use_signal(|| None::<String>);

    rsx! {
        div { class: "flex flex-col items-center gap-3",
            PinInputRoot {
                value: Some(code()),
                on_value_change: move |next| {
                    code.set(next);
                    if completed().is_some() {
                        completed.set(None);
                    }
                },
                on_complete: move |full| completed.set(Some(full)),
                max_length: 6,
                pattern: PinInputPattern::Digits,
                aria_label: "One-time verification code",
                class: "inline-flex items-center gap-2",

                for index in 0..6 {
                    PinInputCell {
                        key: "{index}",
                        index,
                        class: "flex h-11 w-10 items-center justify-center rounded-input border border-border-input bg-background-alt text-base font-semibold text-foreground shadow-mini transition-colors data-[active]:border-foreground data-[active]:ring-2 data-[active]:ring-foreground/20",
                    }
                }
            }

            p { class: "text-xs text-muted-foreground",
                if let Some(full) = completed() {
                    "Verified code: "
                    span { class: "font-mono font-medium text-foreground", "{full}" }
                } else {
                    "Enter 6 digits (supports paste & arrow keys)"
                }
            }
        }
    }
}

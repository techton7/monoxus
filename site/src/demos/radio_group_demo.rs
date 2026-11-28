use dioxus::prelude::*;
use monoxus::radio_group::{
    RadioGroupIndicator, RadioGroupItem, RadioGroupOrientation, RadioGroupRoot,
};

#[component]
pub fn RadioGroupDemo() -> Element {
    let mut plan = use_signal(|| String::from("pro"));

    let options = [
        ("hobby", "Hobby", "Personal projects and previews"),
        ("pro", "Pro", "Production deployments with team seats"),
        ("enterprise", "Enterprise", "Dedicated SLA and SSO governance"),
    ];

    rsx! {
        div { class: "w-full max-w-sm space-y-3",
            RadioGroupRoot {
                value: Some(plan()),
                on_value_change: move |next| plan.set(next),
                orientation: Some(RadioGroupOrientation::Vertical),
                class: "flex flex-col gap-2",

                for (value, title, subtitle) in options {
                    div {
                        key: "{value}",
                        class: "flex items-center gap-3 rounded-card-sm border border-border-card bg-background-alt px-3.5 py-2.5",
                        RadioGroupItem {
                            value,
                            class: "inline-flex h-5 w-5 shrink-0 items-center justify-center rounded-full border border-border-input bg-background shadow-mini transition-colors focus-visible:outline-none focus-visible:ring-2 focus-visible:ring-foreground data-[state=checked]:border-dark",
                            RadioGroupIndicator {
                                class: "h-2.5 w-2.5 rounded-full bg-dark",
                            }
                        }
                        div { class: "flex flex-col",
                            span { class: "text-sm font-medium text-foreground", "{title}" }
                            span { class: "text-xs text-muted-foreground", "{subtitle}" }
                        }
                    }
                }
            }

            p { class: "text-xs text-muted-foreground",
                "Selected tier: "
                span { class: "font-medium text-foreground", "{plan}" }
            }
        }
    }
}

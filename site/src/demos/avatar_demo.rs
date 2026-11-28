use dioxus::prelude::*;
use monoxus::avatar::{AvatarFallback, AvatarImage, AvatarImageLoadingStatus, AvatarRoot};

#[component]
pub fn AvatarDemo() -> Element {
    let mut status = use_signal(|| AvatarImageLoadingStatus::Idle);

    rsx! {
        div { class: "flex items-center gap-4",
            AvatarRoot {
                class: "relative inline-flex h-12 w-12 shrink-0 overflow-hidden rounded-full border border-border-card bg-muted shadow-mini",
                AvatarImage {
                    src: "data:image/svg+xml,%3Csvg xmlns='http://www.w3.org/2000/svg' viewBox='0 0 64 64'%3E%3Crect width='64' height='64' fill='%2318181b'/%3E%3Ctext x='32' y='40' text-anchor='middle' fill='%23fafafa' font-family='sans-serif' font-size='22' font-weight='600'%3EMX%3C/text%3E%3C/svg%3E",
                    alt: "Monoxus",
                    class: "h-full w-full object-cover",
                    on_loading_status_change: move |next| status.set(next),
                }
                AvatarFallback {
                    class: "flex h-full w-full items-center justify-center bg-muted text-sm font-medium text-foreground",
                    "MX"
                }
            }

            AvatarRoot {
                class: "relative inline-flex h-12 w-12 shrink-0 overflow-hidden rounded-full border border-border-card bg-background-alt shadow-mini",
                AvatarImage {
                    src: "",
                    alt: "Fallback example",
                    class: "h-full w-full object-cover",
                }
                AvatarFallback {
                    class: "flex h-full w-full items-center justify-center bg-background-alt text-sm font-medium text-foreground-alt",
                    "JD"
                }
            }

            span { class: "text-xs text-muted-foreground",
                "Image status: "
                span { class: "font-medium text-foreground", "{status().as_str()}" }
            }
        }
    }
}

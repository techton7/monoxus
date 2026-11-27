use dioxus::prelude::*;
use monoxus::button::Button;

use crate::components::icons::{CheckIcon, CopyIcon, ExternalLinkIcon};
use crate::theme::copy_to_clipboard;

/// Header section for markdown and component documentation pages.
#[component]
pub fn DocPageHeader(
    title: String,
    description: String,
    markdown_content: String,
    #[props(default)] source_url: Option<String>,
    #[props(default)] aria_url: Option<String>,
    #[props(default)] aria_label: Option<String>,
) -> Element {
    let mut copied = use_signal(|| false);

    let on_copy_md = move |_| {
        copy_to_clipboard(&markdown_content);
        copied.set(true);
        schedule_copied_reset(copied);
    };

    rsx! {
        header {
            class: "mb-8 space-y-4 border-b border-border-card pb-6",
            div {
                class: "space-y-2",
                h1 {
                    class: "text-3xl font-semibold tracking-tight text-foreground sm:text-4xl",
                    "{title}"
                }
                p {
                    class: "text-base leading-7 text-foreground-alt sm:text-lg",
                    "{description}"
                }
            }

            div {
                class: "flex flex-wrap items-center gap-2 pt-1",
                Button {
                    class: "inline-flex h-8 items-center gap-1.5 rounded-button border border-border-input bg-background-alt px-3 text-xs font-medium text-foreground shadow-mini transition-colors hover:border-border-input-hover hover:bg-muted/60",
                    on_click: on_copy_md,
                    if copied() {
                        CheckIcon { class: "h-3.5 w-3.5 text-foreground" }
                        span { "Copied" }
                    } else {
                        CopyIcon { class: "h-3.5 w-3.5 text-foreground-alt" }
                        span { "Copy Page" }
                    }
                }

                if let Some(url) = source_url {
                    a {
                        href: "{url}",
                        target: "_blank",
                        rel: "noreferrer",
                        class: "inline-flex h-8 items-center gap-1.5 rounded-button border border-border-card bg-background px-3 text-xs font-medium text-foreground-alt transition-colors hover:border-border-input-hover hover:text-foreground",
                        span { "View Source" }
                        ExternalLinkIcon { class: "h-3 w-3" }
                    }
                }

                if let Some(url) = aria_url {
                    a {
                        href: "{url}",
                        target: "_blank",
                        rel: "noreferrer",
                        class: "inline-flex h-8 items-center gap-1.5 rounded-button border border-border-card bg-background px-3 text-xs font-medium text-foreground-alt transition-colors hover:border-border-input-hover hover:text-foreground",
                        span { "{aria_label.as_deref().unwrap_or(\"WAI-ARIA Pattern\")}" }
                        ExternalLinkIcon { class: "h-3 w-3" }
                    }
                }
            }
        }
    }
}

pub(crate) fn schedule_copied_reset(copied: Signal<bool>) {
    #[cfg(target_arch = "wasm32")]
    {
        use wasm_bindgen::{JsCast, closure::Closure};

        if let Some(window) = web_sys::window() {
            let mut sig = copied;
            let cb = Closure::once_into_js(move || {
                let _ = sig.try_write().map(|mut s| *s = false);
            });
            let _ = window.set_timeout_with_callback_and_timeout_and_arguments_0(
                cb.as_ref().unchecked_ref(),
                1800,
            );
        }
    }
    #[cfg(not(target_arch = "wasm32"))]
    {
        let _ = copied;
    }
}

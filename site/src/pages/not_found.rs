use dioxus::prelude::*;

use crate::components::SiteHeader;
use crate::routes::Route;

/// Fallback 404 page for unmatched routes.
#[component]
pub fn NotFound(segments: Vec<String>) -> Element {
    let path = if segments.is_empty() {
        "/".to_string()
    } else {
        format!("/{}", segments.join("/"))
    };

    rsx! {
        document::Title { "404 — Monoxus" }
        div {
            class: "min-h-screen bg-background text-foreground",
            SiteHeader {}
            main {
                class: "mx-auto flex max-w-xl flex-col items-center justify-center px-4 py-28 text-center",
                span {
                    class: "rounded-full border border-border-card bg-muted px-3 py-1 font-mono text-xs font-medium text-foreground-alt",
                    "404"
                }
                h1 {
                    class: "mt-5 text-3xl font-semibold tracking-tight text-foreground sm:text-4xl",
                    "Page not found"
                }
                p {
                    class: "mt-3 text-sm leading-6 text-foreground-alt",
                    "No documentation or component route exists at "
                    code { class: "rounded bg-muted px-1.5 py-0.5 font-mono text-xs text-foreground", "{path}" }
                    "."
                }
                div {
                    class: "mt-7 flex flex-wrap items-center justify-center gap-3",
                    Link {
                        to: Route::Home {},
                        class: "inline-flex h-10 items-center rounded-input bg-dark px-4 text-xs font-semibold text-background shadow-btn transition-opacity hover:opacity-90",
                        "Back to Home"
                    }
                    Link {
                        to: Route::OverviewDoc { slug: "introduction".to_string() },
                        class: "inline-flex h-10 items-center rounded-input border border-border-input bg-background-alt px-4 text-xs font-medium text-foreground shadow-mini transition-colors hover:border-border-input-hover hover:bg-muted/50",
                        "Browse Docs"
                    }
                }
            }
        }
    }
}

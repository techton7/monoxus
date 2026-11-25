use dioxus::prelude::*;

use crate::api_schema::{component_to_markdown, find_component_doc};
use crate::components::icons::CheckIcon;
use crate::components::{
    ComponentPreview, CssVariablesTable, DataAttributesTable, DocPageHeader, DocPageShell,
    KeyboardTable, PropsTable,
};
use crate::demos::demo_source;
use crate::docs_data::{TocItem, highlight_rust_html};
use crate::routes::Route;

/// Component documentation page (`/docs/components/:slug`) following Radix/Bits UI cadence.
#[component]
pub fn ComponentDocPage(slug: String) -> Element {
    let current_path = format!("/docs/components/{slug}");
    let Some(doc) = find_component_doc(&slug) else {
        return rsx! {
            document::Title { "Component Not Found — Monoxus" }
            DocPageShell {
                toc: Vec::new(),
                current_path,
                div {
                    class: "rounded-card border border-border-card bg-background-alt p-8 text-center shadow-mini",
                    h1 { class: "text-2xl font-semibold text-foreground", "Component not found" }
                    p {
                        class: "mt-2 text-sm text-foreground-alt",
                        "No component reference page exists for \"{slug}\" yet."
                    }
                    div {
                        class: "mt-5 flex justify-center gap-3",
                        Link {
                            to: Route::ComponentDocPage { slug: "button".to_string() },
                            class: "inline-flex h-9 items-center rounded-button bg-dark px-4 text-xs font-semibold text-background hover:opacity-90",
                            "View Button"
                        }
                        Link {
                            to: Route::OverviewDoc { slug: "roadmap".to_string() },
                            class: "inline-flex h-9 items-center rounded-button border border-border-input bg-background px-4 text-xs font-medium text-foreground hover:bg-muted",
                            "Full Roadmap"
                        }
                    }
                }
            }
        };
    };

    let mut toc = vec![
        TocItem {
            id: "features",
            title: "Features",
            level: 2,
        },
        TocItem {
            id: "anatomy",
            title: "Anatomy",
            level: 2,
        },
        TocItem {
            id: "api-reference",
            title: "API Reference",
            level: 2,
        },
    ];
    for part in doc.parts.iter() {
        toc.push(TocItem {
            id: part.name,
            title: part.name,
            level: 3,
        });
    }
    if !doc.keyboard.is_empty() {
        toc.push(TocItem {
            id: "accessibility",
            title: "Accessibility",
            level: 2,
        });
    }

    let markdown_content = component_to_markdown(doc, demo_source(doc.slug));
    let source_url = format!(
        "https://github.com/techton7/monoxus/blob/main/{}",
        doc.source_path
    );
    let highlighted_anatomy = highlight_rust_html(doc.anatomy);

    rsx! {
        document::Title { "{doc.title} — Monoxus" }
        DocPageShell {
            toc,
            current_path,

            DocPageHeader {
                title: doc.title.to_string(),
                description: doc.description.to_string(),
                markdown_content,
                source_url: Some(source_url),
                aria_url: (!doc.aria_url.is_empty()).then(|| doc.aria_url.to_string()),
                aria_label: (!doc.aria_label.is_empty()).then(|| doc.aria_label.to_string()),
            }

            // Hero Demo + Collapsible Source
            ComponentPreview {
                slug: doc.slug.to_string(),
                default_open: true,
            }

            // Features
            section {
                id: "features",
                class: "mt-10 scroll-mt-24",
                h2 {
                    class: "border-b border-border-card pb-2 text-2xl font-semibold tracking-tight text-foreground",
                    "Features"
                }
                ul {
                    class: "mt-4 grid grid-cols-1 gap-2.5 sm:grid-cols-2",
                    for feat in doc.features.iter() {
                        li {
                            key: "{feat}",
                            class: "flex items-start gap-2.5 text-sm leading-6 text-foreground-alt",
                            span {
                                class: "mt-1 inline-flex h-4 w-4 shrink-0 items-center justify-center rounded-full border border-border-card bg-muted text-foreground",
                                CheckIcon { class: "h-2.5 w-2.5" }
                            }
                            span { "{feat}" }
                        }
                    }
                }
            }

            // Anatomy
            section {
                id: "anatomy",
                class: "mt-12 scroll-mt-24",
                h2 {
                    class: "border-b border-border-card pb-2 text-2xl font-semibold tracking-tight text-foreground",
                    "Anatomy"
                }
                p {
                    class: "mt-3 text-sm leading-6 text-foreground-alt",
                    "Import the primitive parts and compose them in your Dioxus component:"
                }
                pre {
                    class: "mt-4 overflow-x-auto rounded-card-sm border border-border-card bg-zinc-50 p-4 font-mono text-[13px] leading-relaxed text-foreground dark:bg-neutral-900/80",
                    code {
                        dangerous_inner_html: "{highlighted_anatomy}",
                    }
                }
            }

            // API Reference
            section {
                id: "api-reference",
                class: "mt-12 scroll-mt-24 space-y-8",
                h2 {
                    class: "border-b border-border-card pb-2 text-2xl font-semibold tracking-tight text-foreground",
                    "API Reference"
                }
                for part in doc.parts.iter() {
                    div {
                        key: "{part.name}",
                        id: "{part.name}",
                        class: "scroll-mt-24 space-y-3",
                        div {
                            class: "flex items-center gap-2.5",
                            h3 {
                                class: "inline-flex items-center rounded-button border border-border-card bg-muted/60 px-2.5 py-1 font-mono text-sm font-semibold text-foreground",
                                "{part.name}"
                            }
                        }
                        p {
                            class: "text-sm leading-6 text-foreground-alt",
                            "{part.description}"
                        }
                        PropsTable { part_name: part.name, props: part.props }
                        DataAttributesTable { attrs: part.data_attrs }
                        CssVariablesTable { vars: part.css_vars }
                    }
                }
            }

            // Accessibility / Keyboard Interactions
            if !doc.keyboard.is_empty() {
                section {
                    id: "accessibility",
                    class: "mt-12 scroll-mt-24 space-y-4",
                    h2 {
                        class: "border-b border-border-card pb-2 text-2xl font-semibold tracking-tight text-foreground",
                        "Accessibility"
                    }
                    h3 {
                        class: "text-base font-semibold text-foreground",
                        "Keyboard Interactions"
                    }
                    KeyboardTable { keys: doc.keyboard }
                }
            }
        }
    }
}

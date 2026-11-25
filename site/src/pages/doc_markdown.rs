use dioxus::prelude::*;

use crate::components::{DocPageHeader, DocPageShell};
use crate::docs_data::find_markdown_doc;
use crate::routes::Route;

/// Documentation page for `/docs/overview/:slug`.
#[component]
pub fn OverviewDoc(slug: String) -> Element {
    rsx! {
        MarkdownDocView {
            section: "overview".to_string(),
            slug,
        }
    }
}

/// Documentation page for `/docs/guides/:slug`.
#[component]
pub fn GuideDoc(slug: String) -> Element {
    rsx! {
        MarkdownDocView {
            section: "guides".to_string(),
            slug,
        }
    }
}

#[component]
fn MarkdownDocView(section: String, slug: String) -> Element {
    let current_path = format!("/docs/{section}/{slug}");
    let Some(doc) = find_markdown_doc(&section, &slug) else {
        return rsx! {
            document::Title { "Document Not Found — Monoxus" }
            DocPageShell {
                toc: Vec::new(),
                current_path,
                div {
                    class: "rounded-card border border-border-card bg-background-alt p-8 text-center shadow-mini",
                    h1 { class: "text-2xl font-semibold text-foreground", "Document not found" }
                    p {
                        class: "mt-2 text-sm text-foreground-alt",
                        "No documentation page matched /docs/{section}/{slug}."
                    }
                    Link {
                        to: Route::OverviewDoc { slug: "introduction".to_string() },
                        class: "mt-5 inline-flex h-9 items-center rounded-button bg-dark px-4 text-xs font-semibold text-background hover:opacity-90",
                        "Go to Introduction"
                    }
                }
            }
        };
    };

    rsx! {
        document::Title { "{doc.title} — Monoxus" }
        DocPageShell {
            toc: doc.toc.to_vec(),
            current_path,
            DocPageHeader {
                title: doc.title.to_string(),
                description: doc.description.to_string(),
                markdown_content: doc.raw_md.to_string(),
            }
            div {
                class: "prose-monoxus",
                dangerous_inner_html: "{doc.html}",
            }
        }
    }
}

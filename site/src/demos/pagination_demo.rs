use dioxus::prelude::*;
use monoxus::pagination::{
    PageData, PageItem, PaginationNextButton, PaginationPage, PaginationPrevButton,
    PaginationRoot, PaginationSnapshot,
};

#[component]
pub fn PaginationDemo() -> Element {
    let mut page = use_signal(|| 3usize);

    rsx! {
        div { class: "flex flex-col items-center gap-3",
            PaginationRoot {
                count: 100,
                per_page: 10,
                sibling_count: 1,
                page: Some(page()),
                on_page_change: move |next| page.set(next),
                class: "flex items-center gap-1.5",
                render_children: move |snap: PaginationSnapshot| rsx! {
                    PaginationPrevButton {
                        class: "inline-flex h-9 items-center justify-center rounded-button border border-border-input bg-background px-3 text-xs font-medium text-foreground shadow-mini transition-colors hover:bg-muted disabled:pointer-events-none disabled:opacity-40",
                        "Prev"
                    }

                    for item in snap.pages {
                        match item {
                            PageItem::Page { value, key } => rsx! {
                                PaginationPage {
                                    key: "{key}",
                                    page: PageData::new(value),
                                    class: "inline-flex h-9 w-9 items-center justify-center rounded-button border border-border-input bg-background text-xs font-medium text-foreground shadow-mini transition-colors hover:bg-muted data-[selected]:border-dark data-[selected]:bg-dark data-[selected]:text-background",
                                }
                            },
                            PageItem::Ellipsis { key } => rsx! {
                                span {
                                    key: "{key}",
                                    class: "inline-flex h-9 w-7 items-center justify-center text-xs text-muted-foreground select-none",
                                    "…"
                                }
                            },
                        }
                    }

                    PaginationNextButton {
                        class: "inline-flex h-9 items-center justify-center rounded-button border border-border-input bg-background px-3 text-xs font-medium text-foreground shadow-mini transition-colors hover:bg-muted disabled:pointer-events-none disabled:opacity-40",
                        "Next"
                    }
                },
            }

            p { class: "text-xs text-muted-foreground",
                "Page "
                span { class: "font-medium text-foreground", "{page}" }
                " of 10"
            }
        }
    }
}

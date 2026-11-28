use dioxus::prelude::*;
use monoxus::visually_hidden::VisuallyHidden;

#[component]
pub fn VisuallyHiddenDemo() -> Element {
    let mut bookmarked = use_signal(|| false);

    rsx! {
        div { class: "flex flex-col items-center gap-3",
            div { class: "flex items-center gap-2",
                button {
                    r#type: "button",
                    class: "inline-flex h-10 w-10 items-center justify-center rounded-button border border-border-input bg-background text-base text-foreground shadow-mini transition-colors hover:bg-muted",
                    onclick: move |_| bookmarked.toggle(),
                    if bookmarked() {
                        "★"
                    } else {
                        "☆"
                    }
                    VisuallyHidden {
                        if bookmarked() {
                            "Remove from bookmarks"
                        } else {
                            "Add to bookmarks"
                        }
                    }
                }

                span { class: "text-xs text-muted-foreground",
                    if bookmarked() {
                        "Bookmarked (screen reader text: \"Remove from bookmarks\")"
                    } else {
                        "Icon-only button (screen reader text: \"Add to bookmarks\")"
                    }
                }
            }
        }
    }
}

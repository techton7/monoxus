use dioxus::prelude::*;
use monoxus::scroll_area::{
    Orientation, ScrollAreaCorner, ScrollAreaRoot, ScrollAreaScrollbar, ScrollAreaThumb,
    ScrollAreaType, ScrollAreaViewport,
};

#[component]
pub fn ScrollAreaDemo() -> Element {
    let releases = [
        ("v0.2.0", "Exhaustive 28-primitive catalog & headless ui-sdk core"),
        ("v0.1.9", "Unified floating positioning & collision padding"),
        ("v0.1.8", "FieldRoot / FieldSet 1:N aria-describedby wiring"),
        ("v0.1.7", "Multi-thumb Slider collision & ResizeObserver track"),
        ("v0.1.6", "PinInput single-overlay OTP input & paste transform"),
        ("v0.1.5", "RatingGroup half-star pointer & decimal typing"),
        ("v0.1.4", "Toolbar unified roving focus & toggle groups"),
        ("v0.1.3", "ScrollArea custom thumbs & RTL corner sync"),
        ("v0.1.2", "Pagination ellipsis window & roving navigation"),
        ("v0.1.1", "Dialog transition status hooks & keep_mounted"),
    ];

    rsx! {
        ScrollAreaRoot {
            r#type: ScrollAreaType::Always,
            class: "h-56 w-full max-w-sm overflow-hidden rounded-card-sm border border-border-card bg-background-alt shadow-mini",

            ScrollAreaViewport {
                class: "h-full w-full p-4",
                div { class: "space-y-2.5 pr-3",
                    p { class: "text-xs font-semibold uppercase tracking-wider text-muted-foreground",
                        "Release Changelog"
                    }
                    for (version, summary) in releases {
                        div {
                            key: "{version}",
                            class: "rounded-button border border-border-card bg-background px-3 py-2",
                            div { class: "text-xs font-semibold text-foreground", "{version}" }
                            div { class: "text-xs text-muted-foreground", "{summary}" }
                        }
                    }
                }
            }

            ScrollAreaScrollbar {
                orientation: Orientation::Vertical,
                class: "flex w-2.5 touch-none select-none p-0.5 transition-colors",
                ScrollAreaThumb {
                    class: "relative flex-1 rounded-full bg-border-input transition-colors hover:bg-muted-foreground",
                }
            }

            ScrollAreaCorner {
                class: "bg-muted",
            }
        }
    }
}

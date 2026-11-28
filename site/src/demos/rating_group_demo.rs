use dioxus::prelude::*;
use monoxus::rating_group::{
    RatingGroupItem, RatingGroupItemData, RatingGroupItemState, RatingGroupRoot,
};

#[component]
pub fn RatingGroupDemo() -> Element {
    let mut rating = use_signal(|| 3.5_f64);

    rsx! {
        div { class: "flex flex-col items-center gap-3",
            RatingGroupRoot {
                value: Some(rating()),
                on_value_change: move |next| rating.set(next),
                max: 5,
                allow_half: true,
                class: "inline-flex items-center gap-1 rounded-button border border-border-card bg-background-alt px-3 py-2 outline-none focus-visible:ring-2 focus-visible:ring-foreground",
                render_items: move |items: Vec<RatingGroupItemData>| rsx! {
                    for item in items {
                        RatingGroupItem {
                            key: "{item.index}",
                            index: item.index,
                            class: "relative inline-flex h-7 w-7 cursor-pointer items-center justify-center text-xl leading-none select-none transition-transform hover:scale-105",
                            match item.state {
                                RatingGroupItemState::Active => rsx! {
                                    span { class: "text-foreground", "★" }
                                },
                                RatingGroupItemState::Partial => rsx! {
                                    span { class: "relative inline-block text-muted-foreground/35",
                                        "★"
                                        span {
                                            class: "absolute inset-0 overflow-hidden text-foreground",
                                            style: "width: 50%;",
                                            "★"
                                        }
                                    }
                                },
                                RatingGroupItemState::Inactive => rsx! {
                                    span { class: "text-muted-foreground/35", "★" }
                                },
                            }
                        }
                    }
                },
            }

            p { class: "text-xs text-muted-foreground",
                "Rating: "
                span { class: "font-medium text-foreground", "{rating():.1} / 5.0" }
            }
        }
    }
}

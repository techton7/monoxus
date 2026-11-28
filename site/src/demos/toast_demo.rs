use dioxus::prelude::*;
use monoxus::toast::{
    ToastActionData, ToastOptions, ToastPosition, ToastProvider, ToastViewport, toast,
};

#[component]
pub fn ToastDemo() -> Element {
    let mut undo_count = use_signal(|| 0u32);

    rsx! {
        ToastProvider {
            close_button: Some(true),

            div { class: "flex flex-col items-center gap-3",
                div { class: "flex flex-wrap items-center justify-center gap-2",
                    button {
                        r#type: "button",
                        class: "inline-flex h-9 items-center justify-center rounded-button bg-dark px-3.5 text-sm font-medium text-background shadow-btn transition-colors hover:bg-dark/90",
                        onclick: move |_| {
                            toast::message(
                                "Deployment scheduled",
                                Some(ToastOptions {
                                    description: Some("Release v0.2.0 queued for edge rollout.".into()),
                                    action: Some(
                                        ToastActionData::new("Undo", "Cancel scheduled deployment")
                                            .with_handler(move || {
                                                undo_count.set(undo_count() + 1);
                                            }),
                                    ),
                                    ..Default::default()
                                }),
                            );
                        },
                        "Schedule toast"
                    }

                    button {
                        r#type: "button",
                        class: "inline-flex h-9 items-center justify-center rounded-button border border-border-input bg-background px-3.5 text-sm font-medium text-foreground shadow-mini transition-colors hover:bg-muted",
                        onclick: move |_| {
                            toast::success("Build artifacts verified", None);
                        },
                        "Success"
                    }

                    button {
                        r#type: "button",
                        class: "inline-flex h-9 items-center justify-center rounded-button border border-border-input bg-background px-3.5 text-sm font-medium text-foreground shadow-mini transition-colors hover:bg-muted",
                        onclick: move |_| {
                            toast::warning("Rate limit approaching", None);
                        },
                        "Warning"
                    }

                    button {
                        r#type: "button",
                        class: "inline-flex h-9 items-center justify-center rounded-button border border-border-input bg-background px-3.5 text-sm font-medium text-foreground shadow-mini transition-colors hover:bg-muted",
                        onclick: move |_| {
                            toast::dismiss(None);
                        },
                        "Dismiss all"
                    }
                }

                p { class: "text-xs text-muted-foreground",
                    "Press F8 to focus the viewport · Undos triggered: "
                    span { class: "font-medium text-foreground", "{undo_count}" }
                }
            }

            ToastViewport {
                position: ToastPosition::BottomRight,
                class: "fixed bottom-4 right-4 z-50 flex w-[340px] max-w-[calc(100vw-2rem)] list-none flex-col gap-2 p-0 outline-none [&>li]:rounded-card-sm [&>li]:border [&>li]:border-border-card [&>li]:bg-background [&>li]:p-3.5 [&>li]:text-sm [&>li]:text-foreground [&>li]:shadow-popover [&_[data-title]]:font-medium [&_[data-description]]:mt-0.5 [&_[data-description]]:text-xs [&_[data-description]]:text-muted-foreground [&_[data-button]]:mt-2 [&_[data-button]]:inline-flex [&_[data-button]]:h-7 [&_[data-button]]:items-center [&_[data-button]]:rounded-input [&_[data-button]]:bg-dark [&_[data-button]]:px-2.5 [&_[data-button]]:text-xs [&_[data-button]]:font-medium [&_[data-button]]:text-background [&_[data-close-button]]:absolute [&_[data-close-button]]:right-2 [&_[data-close-button]]:top-2 [&_[data-close-button]]:text-xs [&_[data-close-button]]:text-muted-foreground",
            }
        }
    }
}

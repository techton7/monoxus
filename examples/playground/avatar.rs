use dioxus::prelude::*;
use monoxus::avatar::{Avatar, AvatarFallback, AvatarImage, AvatarImageLoadingStatus};

const AVATAR_STYLE: &str = "display: inline-flex; width: 4rem; height: 4rem; align-items: center; justify-content: center; overflow: hidden; border-radius: 9999px; background: #e2e8f0;";
const IMAGE_STYLE: &str = "width: 100%; height: 100%; object-fit: cover;";
const BUTTON_STYLE: &str = "padding: 0.375rem 0.75rem; border: 1px solid #cbd5e1; border-radius: 0.375rem; background: white; color: #334155; cursor: pointer;";
const CARD_STYLE: &str =
    "padding: 1.25rem; border: 1px solid #e2e8f0; border-radius: 0.75rem; background: white;";

#[component]
pub fn AvatarPlayground() -> Element {
    rsx! {
        div {
            class: "max-w-4xl mx-auto space-y-8",
            div {
                h2 {
                    class: "text-2xl font-bold text-slate-900",
                    "Avatar"
                }
                p {
                    class: "text-sm text-slate-500",
                    "SW2A-6..11 runtime scenarios. Image loading, policy forwarding, fallback timing, cancellation, and unsupported multiple-image behavior are owned by Avatar."
                }
            }
            ValidSourcesScenario {}
            SourceReplacementScenario {}
            DelayedFallbackScenario {}
            UnmountScenario {}
            PolicyScenario {}
            MultipleImagesScenario {}
        }
    }
}

#[component]
fn ValidSourcesScenario() -> Element {
    let png_src = asset!("/assets/icon.svg").to_string();
    let svg_src = asset!("/assets/icon.svg").to_string();
    let mut png_status = use_signal(|| AvatarImageLoadingStatus::Idle);
    let mut png_callbacks = use_signal(|| 0_u32);
    let mut svg_status = use_signal(|| AvatarImageLoadingStatus::Idle);
    let mut svg_callbacks = use_signal(|| 0_u32);

    rsx! {
        section {
            id: "avatar-valid-sources-scenario",
            style: CARD_STYLE,
            h3 { "Valid local raster and SVG (SW2A-6)" }
            p { "Both sources use real bundled assets; no consumer load or error handlers are attached." }
            div {
                style: "display: flex; flex-wrap: wrap; gap: 2rem; align-items: start;",
                div {
                    Avatar {
                        id: "avatar-valid-png-root".to_string(),
                        style: AVATAR_STYLE.to_string(),
                        "data-gui-test-id": "avatar-valid-png-root",
                        on_loading_status_change: move |status| {
                            png_status.set(status);
                            png_callbacks += 1;
                        },
                        AvatarImage {
                            id: "avatar-valid-png-image".to_string(),
                            style: IMAGE_STYLE.to_string(),
                            src: Some(png_src.clone()),
                            alt: "Local PNG avatar".to_string(),
                        }
                        AvatarFallback {
                            id: "avatar-valid-png-fallback".to_string(),
                            delay_ms: 0,
                            "PNG"
                        }
                    }
                    p {
                        id: "avatar-valid-png-status",
                        "status={png_status().as_str()}"
                    }
                    p {
                        id: "avatar-valid-png-callback-count",
                        "callbacks={png_callbacks()}"
                    }
                }
                div {
                    Avatar {
                        id: "avatar-valid-svg-root".to_string(),
                        style: AVATAR_STYLE.to_string(),
                        "data-gui-test-id": "avatar-valid-svg-root",
                        on_loading_status_change: move |status| {
                            svg_status.set(status);
                            svg_callbacks += 1;
                        },
                        AvatarImage {
                            id: "avatar-valid-svg-image".to_string(),
                            style: IMAGE_STYLE.to_string(),
                            src: Some(svg_src.clone()),
                            alt: "Local SVG avatar".to_string(),
                        }
                        AvatarFallback {
                            id: "avatar-valid-svg-fallback".to_string(),
                            delay_ms: 0,
                            "SVG"
                        }
                    }
                    p {
                        id: "avatar-valid-svg-status",
                        "status={svg_status().as_str()}"
                    }
                    p {
                        id: "avatar-valid-svg-callback-count",
                        "callbacks={svg_callbacks()}"
                    }
                }
            }
        }
    }
}

#[component]
fn SourceReplacementScenario() -> Element {
    let png_src = asset!("/assets/icon.svg").to_string();
    let svg_src = asset!("/assets/icon.svg").to_string();
    let mut src = use_signal(|| Some("data:image/png;base64,invalid".to_string()));
    let mut status = use_signal(|| AvatarImageLoadingStatus::Idle);
    let mut root_callbacks = use_signal(|| 0_u32);
    let mut image_callbacks = use_signal(|| 0_u32);

    rsx! {
        section {
            id: "avatar-source-replacement-scenario",
            style: CARD_STYLE,
            h3 { "Invalid, empty, absent, and replacement sources (SW2A-7..9)" }
            p { "Replace an in-flight or errored source by clicking the controls. Avatar's generation guard owns stale completion cancellation." }
            div {
                style: "display: flex; flex-wrap: wrap; gap: 0.5rem;",
                button {
                    id: "avatar-replace-png-button",
                    style: BUTTON_STYLE,
                    r#type: "button",
                    onclick: move |_| src.set(Some(png_src.clone())),
                    "Replace with PNG"
                }
                button {
                    id: "avatar-replace-svg-button",
                    style: BUTTON_STYLE,
                    r#type: "button",
                    onclick: move |_| src.set(Some(svg_src.clone())),
                    "Replace with SVG"
                }
                button {
                    id: "avatar-replace-invalid-button",
                    style: BUTTON_STYLE,
                    r#type: "button",
                    onclick: move |_| src.set(Some("data:image/png;base64,invalid".to_string())),
                    "Replace with invalid"
                }
                button {
                    id: "avatar-replace-empty-button",
                    style: BUTTON_STYLE,
                    r#type: "button",
                    onclick: move |_| src.set(Some(String::new())),
                    "Replace with empty"
                }
                button {
                    id: "avatar-replace-none-button",
                    style: BUTTON_STYLE,
                    r#type: "button",
                    onclick: move |_| src.set(None),
                    "Replace with absent"
                }
            }
            Avatar {
                id: "avatar-replacement-root".to_string(),
                style: AVATAR_STYLE.to_string(),
                on_loading_status_change: move |next| {
                    status.set(next);
                    root_callbacks += 1;
                },
                AvatarImage {
                    id: "avatar-replacement-image".to_string(),
                    style: IMAGE_STYLE.to_string(),
                    src: src(),
                    alt: "Replaceable avatar".to_string(),
                    on_loading_status_change: move |_| image_callbacks += 1,
                }
                AvatarFallback {
                    id: "avatar-replacement-fallback".to_string(),
                    delay_ms: 0,
                    "RPL"
                }
            }
            p {
                id: "avatar-replacement-status",
                "status={status().as_str()}"
            }
            p {
                id: "avatar-replacement-root-callback-count",
                "root_callbacks={root_callbacks()}"
            }
            p {
                id: "avatar-replacement-image-callback-count",
                "image_callbacks={image_callbacks()}"
            }
            code {
                id: "avatar-replacement-src",
                "{src().unwrap_or_else(|| \"<none>\".to_string())}"
            }
        }
    }
}

#[component]
fn DelayedFallbackScenario() -> Element {
    let png_src = asset!("/assets/icon.svg").to_string();
    let mut src = use_signal(|| Some("data:image/png;base64,invalid".to_string()));
    let mut delay_ms = use_signal(|| 800_u64);
    let mut status = use_signal(|| AvatarImageLoadingStatus::Idle);
    let mut callbacks = use_signal(|| 0_u32);

    rsx! {
        section {
            id: "avatar-delay-scenario",
            style: CARD_STYLE,
            h3 { "Fallback delay and rapid replacement (SW2A-8)" }
            p { "Use delay 0, 300, or 800 ms, then wait below and above the selected boundary. Status callbacks remain Avatar-owned and transition-idempotent." }
            div {
                style: "display: flex; flex-wrap: wrap; gap: 0.5rem;",
                button {
                    id: "avatar-delay-0-button",
                    style: BUTTON_STYLE,
                    r#type: "button",
                    onclick: move |_| delay_ms.set(0),
                    "Delay 0 ms"
                }
                button {
                    id: "avatar-delay-300-button",
                    style: BUTTON_STYLE,
                    r#type: "button",
                    onclick: move |_| delay_ms.set(300),
                    "Delay 300 ms"
                }
                button {
                    id: "avatar-delay-800-button",
                    style: BUTTON_STYLE,
                    r#type: "button",
                    onclick: move |_| delay_ms.set(800),
                    "Delay 800 ms"
                }
                button {
                    id: "avatar-delay-invalid-button",
                    style: BUTTON_STYLE,
                    r#type: "button",
                    onclick: move |_| src.set(Some("data:image/png;base64,invalid".to_string())),
                    "Set invalid"
                }
                button {
                    id: "avatar-delay-png-button",
                    style: BUTTON_STYLE,
                    r#type: "button",
                    onclick: move |_| src.set(Some(png_src.clone())),
                    "Replace rapidly with PNG"
                }
            }
            Avatar {
                id: "avatar-delay-root".to_string(),
                style: AVATAR_STYLE.to_string(),
                delay_ms: delay_ms(),
                on_loading_status_change: move |next| {
                    status.set(next);
                    callbacks += 1;
                },
                AvatarImage {
                    id: "avatar-delay-image".to_string(),
                    style: IMAGE_STYLE.to_string(),
                    src: src(),
                    alt: "Delayed avatar".to_string(),
                }
                AvatarFallback {
                    id: "avatar-delay-fallback".to_string(),
                    "DLY"
                }
            }
            p {
                id: "avatar-delay-status",
                "delay_ms={delay_ms()} status={status().as_str()}"
            }
            p {
                id: "avatar-delay-callback-count",
                "callbacks={callbacks()}"
            }
        }
    }
}

#[component]
fn UnmountScenario() -> Element {
    let png_src = asset!("/assets/icon.svg").to_string();
    let mut mounted = use_signal(|| true);
    let mut src = use_signal(|| Some(png_src.clone()));
    let mut callbacks = use_signal(|| 0_u32);
    let mut status = use_signal(|| AvatarImageLoadingStatus::Idle);

    rsx! {
        section {
            id: "avatar-unmount-scenario",
            style: CARD_STYLE,
            h3 { "Unmount during image work (SW2A-9)" }
            p { "Toggle the Avatar after replacement. After unmount, old image work must publish neither status nor callbacks." }
            button {
                id: "avatar-unmount-button",
                style: BUTTON_STYLE,
                r#type: "button",
                onclick: move |_| mounted.set(!mounted()),
                if mounted() { "Unmount Avatar" } else { "Remount Avatar" }
            }
            button {
                id: "avatar-unmount-png-button",
                style: BUTTON_STYLE,
                r#type: "button",
                onclick: move |_| src.set(Some(png_src.clone())),
                "Start PNG load"
            }
            button {
                id: "avatar-unmount-invalid-button",
                style: BUTTON_STYLE,
                r#type: "button",
                onclick: move |_| src.set(Some("/assets/avatar-unmount-missing.png".to_string())),
                "Start delayed invalid load"
            }
            if mounted() {
                Avatar {
                    id: "avatar-unmount-root".to_string(),
                    style: AVATAR_STYLE.to_string(),
                    delay_ms: 600,
                    on_loading_status_change: move |next| {
                        status.set(next);
                        callbacks += 1;
                    },
                    AvatarImage {
                        id: "avatar-unmount-image".to_string(),
                        style: IMAGE_STYLE.to_string(),
                        src: src(),
                        alt: "Unmounted avatar target".to_string(),
                    }
                    AvatarFallback {
                        id: "avatar-unmount-fallback".to_string(),
                        "UNM"
                    }
                }
            }
            p {
                id: "avatar-unmount-status",
                "mounted={mounted()} status={status().as_str()}"
            }
            p {
                id: "avatar-unmount-callback-count",
                "callbacks={callbacks()}"
            }
        }
    }
}

#[component]
fn PolicyScenario() -> Element {
    let svg_src = asset!("/assets/icon.svg").to_string();
    let mut status = use_signal(|| AvatarImageLoadingStatus::Idle);
    let mut callbacks = use_signal(|| 0_u32);

    rsx! {
        section {
            id: "avatar-policy-scenario",
            style: CARD_STYLE,
            h3 { "Image policy forwarding (SW2A-10)" }
            p { "The rendered image publishes crossorigin=\"anonymous\" and referrerpolicy=\"no-referrer\"; Avatar passes both to its preloader." }
            Avatar {
                id: "avatar-policy-root".to_string(),
                style: AVATAR_STYLE.to_string(),
                on_loading_status_change: move |next| {
                    status.set(next);
                    callbacks += 1;
                },
                AvatarImage {
                    id: "avatar-policy-image".to_string(),
                    style: IMAGE_STYLE.to_string(),
                    src: Some(svg_src.clone()),
                    alt: "Policy avatar".to_string(),
                    crossorigin: "anonymous".to_string(),
                    referrerpolicy: "no-referrer".to_string(),
                }
                AvatarFallback {
                    id: "avatar-policy-fallback".to_string(),
                    delay_ms: 0,
                    "POL"
                }
            }
            p {
                id: "avatar-policy-status",
                "status={status().as_str()}"
            }
            p {
                id: "avatar-policy-callback-count",
                "callbacks={callbacks()}"
            }
        }
    }
}

#[component]
fn MultipleImagesScenario() -> Element {
    let png_src = asset!("/assets/icon.svg").to_string();
    let svg_src = asset!("/assets/icon.svg").to_string();
    let mut status = use_signal(|| AvatarImageLoadingStatus::Idle);
    let mut callbacks = use_signal(|| 0_u32);

    rsx! {
        section {
            id: "avatar-two-images-scenario",
            style: CARD_STYLE,
            h3 { "Two images under one root (SW2A-11)" }
            p { "Debug builds emit the development warning. Multiple images are intentionally unsupported and may interfere through one shared status." }
            Avatar {
                id: "avatar-two-images-root".to_string(),
                style: AVATAR_STYLE.to_string(),
                on_loading_status_change: move |next| {
                    status.set(next);
                    callbacks += 1;
                },
                AvatarImage {
                    id: "avatar-two-images-png-image".to_string(),
                    style: IMAGE_STYLE.to_string(),
                    src: Some(png_src.clone()),
                    alt: "First unsupported avatar image".to_string(),
                }
                AvatarImage {
                    id: "avatar-two-images-svg-image".to_string(),
                    style: IMAGE_STYLE.to_string(),
                    src: Some(svg_src.clone()),
                    alt: "Second unsupported avatar image".to_string(),
                }
                AvatarFallback {
                    id: "avatar-two-images-fallback".to_string(),
                    delay_ms: 0,
                    "2X"
                }
            }
            p {
                id: "avatar-two-images-status",
                "status={status().as_str()}"
            }
            p {
                id: "avatar-two-images-callback-count",
                "callbacks={callbacks()}"
            }
        }
    }
}

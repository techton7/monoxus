use dioxus::prelude::*;
use monoxus::toast::*;
use std::time::Duration;

const SONNER_STYLE: &str = r#"
#monoxus-toast-viewport {
    position: fixed;
    width: 356px;
    max-width: calc(100vw - 32px);
    margin: 0;
    padding: 0;
    list-style: none;
    outline: none;
    z-index: 999999;
    pointer-events: none;
    box-sizing: border-box;
}

#monoxus-toast-viewport[data-x-position="right"] {
    right: 24px;
    left: auto;
}

#monoxus-toast-viewport[data-x-position="left"] {
    left: 24px;
    right: auto;
}

#monoxus-toast-viewport[data-x-position="center"] {
    left: 50%;
    right: auto;
    transform: translateX(-50%);
}

#monoxus-toast-viewport[data-y-position="top"] {
    top: 24px;
    bottom: auto;
}

#monoxus-toast-viewport[data-y-position="bottom"] {
    bottom: 24px;
    top: auto;
}

#monoxus-toast-viewport li[data-sonner-toast] {
    position: absolute;
    width: 100%;
    box-sizing: border-box;
    pointer-events: auto;
    touch-action: none;
    background-color: #ffffff;
    border: 1px solid #e2e8f0;
    border-radius: 10px;
    box-shadow: 0 10px 25px -5px rgba(0, 0, 0, 0.1), 0 8px 10px -6px rgba(0, 0, 0, 0.1);
    padding: 14px 16px;
    color: #0f172a;
    font-family: -apple-system, BlinkMacSystemFont, 'Segoe UI', Roboto, sans-serif;
    font-size: 0.875rem;
    line-height: 1.25rem;
    transition: transform 350ms cubic-bezier(0.16, 1, 0.3, 1), opacity 300ms ease, height 350ms ease;
    z-index: var(--z-index, 1);
    outline: none;
}

#monoxus-toast-viewport li[data-sonner-toast][data-y-position="bottom"] {
    bottom: 0;
    top: auto;
    --lift: -1;
}

#monoxus-toast-viewport li[data-sonner-toast][data-y-position="top"] {
    top: 0;
    bottom: auto;
    --lift: 1;
}

#monoxus-toast-viewport li[data-sonner-toast]:focus-visible {
    box-shadow: 0 0 0 2px #9333ea, 0 10px 25px -5px rgba(0, 0, 0, 0.1);
}

/* Stack Resting Transformations */
#monoxus-toast-viewport li[data-sonner-toast] {
    --y: translateY(calc(var(--lift, -1) * 14px * var(--toasts-before, 0))) scale(var(--scale, 1));
    transform: var(--y);
    height: var(--front-toast-height);
}

#monoxus-toast-viewport li[data-sonner-toast][data-expanded="false"][data-front="true"] {
    --y: translateY(0px) scale(1);
    transform: var(--y);
}

/* Expanded Stack: reveal items with measured offset */
#monoxus-toast-viewport li[data-sonner-toast][data-expanded="true"] {
    --y: translateY(calc(var(--lift, -1) * var(--offset, 0px)));
    transform: var(--y);
    height: var(--height, auto);
}

/* Sonner Gap Bridge: fills the empty gap between expanded cards so hover is continuous */
#monoxus-toast-viewport li[data-sonner-toast][data-expanded="true"][data-y-position="bottom"]::after {
    content: '';
    position: absolute;
    left: 0;
    height: calc(14px + 1px);
    bottom: 100%;
    width: 100%;
    pointer-events: auto;
}

#monoxus-toast-viewport li[data-sonner-toast][data-expanded="true"][data-y-position="top"]::after {
    content: '';
    position: absolute;
    left: 0;
    height: calc(14px + 1px);
    top: 100%;
    width: 100%;
    pointer-events: auto;
}

/* Active Swiping Gesture: transforms both X and Y components independently */
#monoxus-toast-viewport li[data-sonner-toast][data-swiping="true"] {
    transform: var(--y)
               translateY(var(--swipe-amount-y, 0px))
               translateX(var(--swipe-amount-x, 0px)) !important;
    transition: none !important;
}

/* Directional Swipe-Out Exit Animations (matching upstream Sonner / Svelte-Sonner) */
#monoxus-toast-viewport li[data-sonner-toast][data-swipe-out='true'] {
    animation-duration: 200ms;
    animation-timing-function: ease-out;
    animation-fill-mode: forwards;
    pointer-events: none;
}

#monoxus-toast-viewport li[data-sonner-toast][data-swipe-out='true'][data-swipe-direction='left'] {
    animation-name: swipe-out-left;
}

#monoxus-toast-viewport li[data-sonner-toast][data-swipe-out='true'][data-swipe-direction='right'] {
    animation-name: swipe-out-right;
}

#monoxus-toast-viewport li[data-sonner-toast][data-swipe-out='true'][data-swipe-direction='up'] {
    animation-name: swipe-out-up;
}

#monoxus-toast-viewport li[data-sonner-toast][data-swipe-out='true'][data-swipe-direction='down'] {
    animation-name: swipe-out-down;
}

@keyframes swipe-out-left {
    from {
        transform: var(--y) translateX(var(--swipe-amount-x, 0px));
        opacity: 1;
    }
    to {
        transform: var(--y) translateX(calc(var(--swipe-amount-x, 0px) - 100%));
        opacity: 0;
    }
}

@keyframes swipe-out-right {
    from {
        transform: var(--y) translateX(var(--swipe-amount-x, 0px));
        opacity: 1;
    }
    to {
        transform: var(--y) translateX(calc(var(--swipe-amount-x, 0px) + 100%));
        opacity: 0;
    }
}

@keyframes swipe-out-up {
    from {
        transform: var(--y) translateY(var(--swipe-amount-y, 0px));
        opacity: 1;
    }
    to {
        transform: var(--y) translateY(calc(var(--swipe-amount-y, 0px) - 100%));
        opacity: 0;
    }
}

@keyframes swipe-out-down {
    from {
        transform: var(--y) translateY(var(--swipe-amount-y, 0px));
        opacity: 1;
    }
    to {
        transform: var(--y) translateY(calc(var(--swipe-amount-y, 0px) + 100%));
        opacity: 0;
    }
}

/* Dismissing State without swipe (e.g. timeout / close button) */
#monoxus-toast-viewport li[data-sonner-toast]:not([data-swipe-out="true"])[data-y-position="bottom"][data-state="closed"] {
    opacity: 0;
    transform: translateY(100%) scale(0.95);
    pointer-events: none;
}

#monoxus-toast-viewport li[data-sonner-toast]:not([data-swipe-out="true"])[data-y-position="top"][data-state="closed"] {
    opacity: 0;
    transform: translateY(-100%) scale(0.95);
    pointer-events: none;
}

/* Hidden beyond limit */
#monoxus-toast-viewport li[data-sonner-toast][data-visible="false"] {
    opacity: 0;
    pointer-events: none;
}
"#;

#[component]
pub fn ToastPage() -> Element {
    let store = TOAST_STORE.read();
    let current_pos = store.config.position;
    let allowed_dirs = store.config.effective_swipe_directions();
    let allowed_dirs_str = allowed_dirs
        .iter()
        .map(|d| d.as_str().to_uppercase())
        .collect::<Vec<_>>()
        .join(", ");

    rsx! {
        style { "{SONNER_STYLE}" }

        div {
            style: "display: flex; flex-direction: column; gap: 2rem; max-width: 52rem; margin: 0 auto; padding-bottom: 5rem;",

            // Header section
            div {
                style: "display: flex; flex-direction: column; gap: 0.5rem; border-bottom: 1px solid #e2e8f0; padding-bottom: 1.5rem;",
                div {
                    style: "display: flex; align-items: center; gap: 0.75rem;",
                    h2 {
                        style: "margin: 0; font-size: 1.75rem; font-weight: 700; color: #0f172a; letter-spacing: -0.025em;",
                        "Toast & Stacked Notifications"
                    }
                    span {
                        style: "font-size: 0.75rem; font-weight: 600; padding: 0.2rem 0.5rem; border-radius: 9999px; background-color: #f3e8ff; color: #7e22ce;",
                        "Sonner Physics"
                    }
                }
                p {
                    style: "margin: 0; font-size: 0.9375rem; color: #64748b; line-height: 1.5;",
                    "Live demonstration of multi-position screen anchoring, card stacking geometry, DOM height measurement (ResizeObserver), hover expand/collapse, hotkey F8 landmark focus, and directional swipe gestures with spring damping."
                }
            }

            // Position Control Matrix
            div {
                style: "background-color: #ffffff; border: 1px solid #e2e8f0; border-radius: 0.75rem; padding: 1.25rem; display: flex; flex-direction: column; gap: 0.875rem;",
                div {
                    style: "display: flex; align-items: center; justify-content: space-between;",
                    span { style: "font-size: 0.8125rem; font-weight: 700; color: #475569; text-transform: uppercase; letter-spacing: 0.05em;", "Screen Placement Anchor" }
                    span {
                        id: "current-position-label",
                        style: "font-size: 0.75rem; font-weight: 600; padding: 0.15rem 0.5rem; border-radius: 0.25rem; background-color: #f3e8ff; color: #7e22ce;",
                        "Active: {current_pos.as_str()}"
                    }
                }
                div {
                    style: "display: grid; grid-template-columns: repeat(3, 1fr); gap: 0.5rem;",
                    for (pos, label) in [
                        (ToastPosition::TopLeft, "Top Left"),
                        (ToastPosition::TopCenter, "Top Center"),
                        (ToastPosition::TopRight, "Top Right"),
                        (ToastPosition::BottomLeft, "Bottom Left"),
                        (ToastPosition::BottomCenter, "Bottom Center"),
                        (ToastPosition::BottomRight, "Bottom Right"),
                    ] {
                        button {
                            key: "{pos.as_str()}",
                            id: "btn-pos-{pos.as_str()}",
                            style: if current_pos == pos {
                                "padding: 0.5rem; border-radius: 0.375rem; font-size: 0.8125rem; font-weight: 600; border: 1px solid #9333ea; background-color: #faf5ff; color: #7e22ce; cursor: pointer;"
                            } else {
                                "padding: 0.5rem; border-radius: 0.375rem; font-size: 0.8125rem; font-weight: 500; border: 1px solid #e2e8f0; background-color: #ffffff; color: #475569; cursor: pointer;"
                            },
                            onclick: move |_| {
                                toast::set_position(pos);
                            },
                            "{label}"
                        }
                    }
                }
                div {
                    style: "font-size: 0.8125rem; color: #64748b; background-color: #f8fafc; padding: 0.5rem 0.75rem; border-radius: 0.375rem; border: 1px solid #f1f5f9; display: flex; align-items: center; justify-content: space-between;",
                    span { "Allowed Dismiss Directions:" }
                    strong {
                        id: "allowed-swipe-directions",
                        style: "color: #0f172a;",
                        "{allowed_dirs_str}"
                    }
                }
            }

            // Controls Grid
            div {
                style: "display: grid; grid-template-columns: repeat(auto-fit, minmax(220px, 1fr)); gap: 1rem;",

                button {
                    id: "dispatch-stack-3",
                    style: "padding: 0.75rem 1rem; border-radius: 0.5rem; background-color: #7e22ce; color: white; font-weight: 600; font-size: 0.875rem; border: none; cursor: pointer; display: flex; align-items: center; justify-content: center; gap: 0.5rem; box-shadow: 0 4px 6px -1px rgba(126, 34, 206, 0.2);",
                    onclick: move |_| {
                        // Dispatch 3 stacked toasts with varying content lengths to exercise height measurement
                        toast::message(
                            "Base notification card (single line)",
                            Some(ToastOptions { duration: Some(Duration::from_millis(20000)), ..Default::default() })
                        );
                        toast::success(
                            "Record updated. Multiple changes committed to the remote data store successfully.",
                            Some(ToastOptions { duration: Some(Duration::from_millis(20000)), ..Default::default() })
                        );
                        toast::warning(
                            "Heads up! Your session will expire in 5 minutes. Please save any unsaved work before navigating away.",
                            Some(ToastOptions { duration: Some(Duration::from_millis(20000)), ..Default::default() })
                        );
                    },
                    "📚 Dispatch 3 Stacked Toasts"
                }

                button {
                    id: "dispatch-message",
                    style: "padding: 0.75rem 1rem; border-radius: 0.5rem; background-color: #ffffff; color: #334155; font-weight: 600; font-size: 0.875rem; border: 1px solid #cbd5e1; cursor: pointer;",
                    onclick: move |_| {
                        toast::message("Informational update ready", None);
                    },
                    "💬 Message Toast"
                }

                button {
                    id: "dispatch-success",
                    style: "padding: 0.75rem 1rem; border-radius: 0.5rem; background-color: #ffffff; color: #166534; font-weight: 600; font-size: 0.875rem; border: 1px solid #bbf7d0; cursor: pointer;",
                    onclick: move |_| {
                        toast::success("Operation completed successfully", None);
                    },
                    "✅ Success Toast"
                }

                button {
                    id: "dispatch-error",
                    style: "padding: 0.75rem 1rem; border-radius: 0.5rem; background-color: #ffffff; color: #991b1b; font-weight: 600; font-size: 0.875rem; border: 1px solid #fecaca; cursor: pointer;",
                    onclick: move |_| {
                        toast::error("An unexpected server error occurred", None);
                    },
                    "❌ Error Toast"
                }

                button {
                    id: "dispatch-promise",
                    style: "padding: 0.75rem 1rem; border-radius: 0.5rem; background-color: #ffffff; color: #1e40af; font-weight: 600; font-size: 0.875rem; border: 1px solid #bfdbfe; cursor: pointer;",
                    onclick: move |_| {
                        spawn(async move {
                            let _ = toast::promise(
                                async {
                                    futures_timer::Delay::new(Duration::from_millis(1200)).await;
                                    Ok::<&str, &str>("Payload loaded successfully")
                                },
                                "Fetching data from server...",
                                |res| format!("Done: {res}"),
                                |err| format!("Failed: {err}"),
                            ).await;
                        });
                    },
                    "⏳ Promise Toast"
                }

                button {
                    id: "clear-all",
                    style: "padding: 0.75rem 1rem; border-radius: 0.5rem; background-color: #f1f5f9; color: #475569; font-weight: 600; font-size: 0.875rem; border: 1px solid #e2e8f0; cursor: pointer;",
                    onclick: move |_| {
                        toast::dismiss(None);
                    },
                    "🗑 Clear All"
                }
            }

            // Live State Monitor Panel
            div {
                style: "background-color: #ffffff; border: 1px solid #e2e8f0; border-radius: 0.75rem; padding: 1.25rem; display: flex; flex-direction: column; gap: 1rem;",
                div {
                    style: "display: flex; align-items: center; justify-content: space-between; border-bottom: 1px solid #f1f5f9; padding-bottom: 0.75rem;",
                    span { style: "font-size: 0.8125rem; font-weight: 700; color: #475569; text-transform: uppercase; letter-spacing: 0.05em;", "Toaster Engine State" }
                    span {
                        id: "active-toasts-count",
                        style: "font-size: 0.75rem; font-weight: 600; padding: 0.125rem 0.5rem; border-radius: 9999px; background-color: #e2e8f0; color: #334155;",
                        "Active Toasts: {store.toasts.len()}"
                    }
                }
                div {
                    style: "display: grid; grid-template-columns: repeat(auto-fit, minmax(140px, 1fr)); gap: 0.75rem; font-size: 0.8125rem;",
                    div {
                        style: "padding: 0.5rem 0.75rem; background-color: #f8fafc; border-radius: 0.375rem; border: 1px solid #f1f5f9;",
                        span { style: "color: #64748b; display: block;", "Expanded:" }
                        strong {
                            id: "state-expanded",
                            style: if store.expanded { "color: #7e22ce;" } else { "color: #64748b;" },
                            "{store.expanded}"
                        }
                    }
                    div {
                        style: "padding: 0.5rem 0.75rem; background-color: #f8fafc; border-radius: 0.375rem; border: 1px solid #f1f5f9;",
                        span { style: "color: #64748b; display: block;", "Paused:" }
                        strong {
                            id: "state-paused",
                            style: if store.paused { "color: #c2410c;" } else { "color: #16a34a;" },
                            "{store.paused}"
                        }
                    }
                    div {
                        style: "padding: 0.5rem 0.75rem; background-color: #f8fafc; border-radius: 0.375rem; border: 1px solid #f1f5f9;",
                        span { style: "color: #64748b; display: block;", "Page Hidden:" }
                        strong { "{store.interaction_state.page_hidden}" }
                    }
                    div {
                        style: "padding: 0.5rem 0.75rem; background-color: #f8fafc; border-radius: 0.375rem; border: 1px solid #f1f5f9;",
                        span { style: "color: #64748b; display: block;", "Hovered:" }
                        strong { "{store.interaction_state.hovered}" }
                    }
                    div {
                        style: "padding: 0.5rem 0.75rem; background-color: #f8fafc; border-radius: 0.375rem; border: 1px solid #f1f5f9;",
                        span { style: "color: #64748b; display: block;", "Swiping:" }
                        strong { "{store.interaction_state.swiping}" }
                    }
                }
            }

            // Interactive Instructions
            div {
                style: "background-color: #faf5ff; border: 1px solid #e9d5ff; border-radius: 0.75rem; padding: 1.25rem; font-size: 0.875rem; color: #581c87; line-height: 1.5;",
                h4 { style: "margin: 0 0 0.5rem 0; font-size: 0.9375rem; font-weight: 700;", "Interaction Proof Guide" }
                ul {
                    style: "margin: 0; padding-left: 1.25rem; display: flex; flex-direction: column; gap: 0.375rem;",
                    li { "Position Anchor: Select any of the 6 screen placement positions. Cards dynamically pivot and stack towards the screen interior." }
                    li { "Directional Swipe: Dragging towards allowed directions dismisses the card; dragging against allowed directions encounters spring damping resistance." }
                    li { "Stacking & Hover Expand: Click 'Dispatch 3 Stacked Toasts', then hover over cards. Notice continuous cursor traversal without flicker." }
                    li { "Hotkey F8: Press F8 anywhere on the page to jump focus directly to the landmark notification region." }
                }
            }
        }

        // Landmark Viewport auto-mounting toasts
        ToastViewport {}
    }
}

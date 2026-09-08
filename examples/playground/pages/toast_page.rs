use dioxus::prelude::*;
use monoxus::toast::*;
use std::time::Duration;

const SONNER_STYLE: &str = r#"
#monoxus-toast-viewport {
    position: fixed;
    bottom: 24px;
    right: 24px;
    width: 356px;
    margin: 0;
    padding: 0;
    list-style: none;
    outline: none;
    z-index: 999999;
    pointer-events: none;
    box-sizing: border-box;
}

#monoxus-toast-viewport li[data-sonner-toast] {
    position: absolute;
    bottom: 0;
    right: 0;
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

#monoxus-toast-viewport li[data-sonner-toast]:focus-visible {
    box-shadow: 0 0 0 2px #9333ea, 0 10px 25px -5px rgba(0, 0, 0, 0.1);
}

/* Collapsed Stack: scale down and stagger behind front toast */
#monoxus-toast-viewport li[data-sonner-toast][data-expanded="false"][data-front="false"] {
    transform: translateY(calc(-14px * var(--toasts-before, 0))) scale(var(--scale, 1));
    height: var(--front-toast-height);
}

#monoxus-toast-viewport li[data-sonner-toast][data-expanded="false"][data-front="true"] {
    transform: translateY(0px) scale(1);
}

/* Expanded Stack: reveal items with measured offset */
#monoxus-toast-viewport li[data-sonner-toast][data-expanded="true"] {
    transform: translateY(calc(-1 * var(--offset, 0px)));
    height: var(--height, auto);
}

/* Sonner Gap Bridge: fills the empty gap between expanded cards so hover is continuous */
#monoxus-toast-viewport li[data-sonner-toast][data-expanded="true"]::after {
    content: '';
    position: absolute;
    left: 0;
    height: calc(14px + 1px);
    bottom: 100%;
    width: 100%;
    pointer-events: auto;
}

/* Active Swiping Gesture */
#monoxus-toast-viewport li[data-sonner-toast][data-swiping="true"] {
    transform: translateY(calc(-1 * var(--offset, 0px))) translateY(var(--drag-offset, 0px)) !important;
    transition: none !important;
}

/* Dismissing State */
#monoxus-toast-viewport li[data-sonner-toast][data-state="closed"] {
    opacity: 0;
    transform: translateY(100%) scale(0.95);
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
                    "Live demonstration of card stacking geometry, DOM height measurement (ResizeObserver), hover expand/collapse, hotkey F8 landmark focus, and pointer swipe-to-dismiss gestures."
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
                            Some(ToastOptions { duration: Some(Duration::from_millis(15000)), ..Default::default() })
                        );
                        toast::success(
                            "Record updated. Multiple changes committed to the remote data store successfully.",
                            Some(ToastOptions { duration: Some(Duration::from_millis(15000)), ..Default::default() })
                        );
                        toast::warning(
                            "Heads up! Your session will expire in 5 minutes. Please save any unsaved work before navigating away.",
                            Some(ToastOptions { duration: Some(Duration::from_millis(15000)), ..Default::default() })
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
                    li { "Stacking: Click 'Dispatch 3 Stacked Toasts' to see physical card stacking with dynamic scale and offset." }
                    li { "Hover Expand: Move pointer over the toast stack. Viewport switches to data-expanded='true' and timers pause." }
                    li { "Hotkey F8: Press F8 anywhere on the page. Focus immediately lands on the ToastViewport landmark." }
                    li { "Swipe to Dismiss: Click and drag any toast downwards (>45px) to trigger gesture dismissal." }
                }
            }
        }

        // Landmark Viewport auto-mounting toasts
        ToastViewport {}
    }
}

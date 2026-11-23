use dioxus::prelude::*;
use monoxus::{
    foundation::overlay::{PlacementAlign, PlacementSide},
    hover_card::{
        HoverCard, HoverCardArrow, HoverCardContent, HoverCardContentStatic, HoverCardPortal,
        HoverCardRoot, HoverCardTrigger,
    },
};

const CARD_STYLE: &str = "border-radius: 1rem; border: 1px solid #e2e8f0; background-color: #ffffff; padding: 1.5rem; box-shadow: 0 1px 3px rgba(15, 23, 42, 0.06); display: flex; flex-direction: column; gap: 1rem;";
const TRIGGER_STYLE: &str = "display: inline-flex; align-items: center; gap: 0.375rem; padding: 0.5rem 0.875rem; border-radius: 0.5rem; border: 1px solid #cbd5e1; background-color: #f8fafc; color: #5b21b6; font-size: 0.875rem; font-weight: 600; text-decoration: none; cursor: pointer;";
const CONTENT_STYLE: &str = "width: 280px; padding: 1rem; border-radius: 0.75rem; border: 1px solid #cbd5e1; background-color: #ffffff; color: #0f172a; box-shadow: 0 12px 32px rgba(15, 23, 42, 0.14); z-index: 40; display: flex; flex-direction: column; gap: 0.5rem;";

#[component]
pub fn HoverCardPlayground() -> Element {
    rsx! {
        div {
            class: "max-w-4xl mx-auto space-y-8",
            style: "max-width: 56rem; margin: 0 auto; display: flex; flex-direction: column; gap: 2rem;",

            div {
                style: "display: flex; flex-direction: column; gap: 0.375rem;",
                h2 {
                    style: "margin: 0; font-size: 1.5rem; font-weight: 700; color: #0f172a; letter-spacing: -0.025em;",
                    "Hover Card"
                }
                p {
                    style: "margin: 0; font-size: 0.875rem; color: #64748b;",
                    "Sighted-user hover preview overlay with configurable open/close delays, safe-polygon grace corridor, touch exclusion, :focus-visible open, non-tabbable content, and text selection persistence."
                }
            }

            StandardHoverCardScenario {}
            FocusAndSelectionScenario {}
            StaticHoverCardScenario {}
        }
    }
}

fn state_badge_style(is_open: bool) -> &'static str {
    if is_open {
        "display: inline-flex; align-items: center; border-radius: 9999px; padding: 0.125rem 0.625rem; font-size: 0.75rem; font-family: monospace; font-weight: 700; background-color: #ede9fe; color: #6d28d9; border: 1px solid #c4b5fd;"
    } else {
        "display: inline-flex; align-items: center; border-radius: 9999px; padding: 0.125rem 0.625rem; font-size: 0.75rem; font-family: monospace; font-weight: 600; background-color: #f1f5f9; color: #475569; border: 1px solid #e2e8f0;"
    }
}

// ---------------------------------------------------------------------------
// Scenario 1: Standard HoverCard (SW3A-1, SW3A-2, SW3A-6)
// ---------------------------------------------------------------------------
#[component]
fn StandardHoverCardScenario() -> Element {
    let mut fast_open = use_signal(|| false);
    let mut default_open = use_signal(|| false);

    rsx! {
        div {
            style: CARD_STYLE,
            div {
                style: "display: flex; flex-direction: column; gap: 0.25rem;",
                h3 {
                    style: "margin: 0; font-size: 1rem; font-weight: 600; color: #0f172a;",
                    "1. Delayed Open, Safe-Polygon Grace Corridor & Floating Geometry (SW3A-1, SW3A-2, SW3A-6)"
                }
                p {
                    style: "margin: 0; font-size: 0.75rem; color: #64748b;",
                    "Fast card uses open_delay: 300ms and close_delay: 200ms with bottom-center placement and HoverCardArrow. Second card uses default 700ms / 300ms delays."
                }
            }

            div {
                style: "display: flex; flex-wrap: wrap; align-items: center; gap: 2rem; padding: 1.25rem; border-radius: 0.75rem; border: 1px dashed #cbd5e1; background-color: #f8fafc;",

                // Fast timing HoverCard (300ms / 200ms)
                div {
                    style: "display: flex; align-items: center; gap: 0.75rem;",
                    HoverCardRoot {
                        id: "sw3a-hc-root",
                        open_delay: 300,
                        close_delay: 200,
                        side: PlacementSide::Bottom,
                        align: PlacementAlign::Center,
                        side_offset: 6.0,
                        on_open_change: move |next| fast_open.set(next),
                        HoverCardTrigger {
                            id: "sw3a-hc-trigger",
                            href: "https://dioxuslabs.com",
                            target: "_blank",
                            rel: "noreferrer",
                            style: TRIGGER_STYLE,
                            on_click: move |evt: MouseEvent| evt.prevent_default(),
                            "@monoxus/hover-card"
                        }
                        HoverCardPortal {
                            HoverCardContent {
                                id: "sw3a-hc-content",
                                style: CONTENT_STYLE,
                                div {
                                    style: "display: flex; align-items: center; justify-content: space-between; gap: 0.5rem;",
                                    strong {
                                        style: "font-size: 0.875rem; color: #0f172a;",
                                        "Monoxus HoverCard"
                                    }
                                    span {
                                        style: "font-size: 0.6875rem; font-family: monospace; color: #6d28d9; background: #f5f3ff; padding: 0.125rem 0.375rem; border-radius: 0.25rem;",
                                        "300ms / 200ms"
                                    }
                                }
                                p {
                                    style: "margin: 0; font-size: 0.8125rem; line-height: 1.45; color: #475569;",
                                    "Preview card positioned via FloatingLayer with safe-polygon hover corridor and --monoxus-hover-card-* geometry CSS vars."
                                }
                                HoverCardArrow {
                                    id: "sw3a-hc-arrow",
                                    width: 12.0,
                                    height: 6.0,
                                    style: "color: #7c3aed;",
                                }
                            }
                        }
                    }

                    span {
                        id: "sw3a-hc-state-badge",
                        style: state_badge_style(fast_open()),
                        if fast_open() { "open" } else { "closed" }
                    }
                }

                // Default timing HoverCard (700ms / 300ms)
                div {
                    style: "display: flex; align-items: center; gap: 0.75rem;",
                    HoverCard {
                        id: "sw3a-hc-default-root",
                        on_open_change: move |next| default_open.set(next),
                        HoverCardTrigger {
                            id: "sw3a-hc-default-trigger",
                            href: "#sw3a-hc-default",
                            style: TRIGGER_STYLE,
                            on_click: move |evt: MouseEvent| evt.prevent_default(),
                            "Default Timing (700ms / 300ms)"
                        }
                        HoverCardPortal {
                            HoverCardContent {
                                id: "sw3a-hc-default-content",
                                style: CONTENT_STYLE,
                                strong {
                                    style: "font-size: 0.875rem; color: #0f172a;",
                                    "Radix Default Timing"
                                }
                                p {
                                    style: "margin: 0; font-size: 0.8125rem; line-height: 1.45; color: #475569;",
                                    "Uses canonical 700ms open_delay and 300ms close_delay defaults."
                                }
                                HoverCardArrow {
                                    id: "sw3a-hc-default-arrow",
                                    width: 12.0,
                                    height: 6.0,
                                    style: "color: #7c3aed;",
                                }
                            }
                        }
                    }

                    span {
                        id: "sw3a-hc-default-state-badge",
                        style: state_badge_style(default_open()),
                        if default_open() { "open" } else { "closed" }
                    }
                }
            }
        }
    }
}

// ---------------------------------------------------------------------------
// Scenario 2: Focus-Visible, Non-Tabbable Content & Text Selection (SW3A-4, SW3A-5)
// ---------------------------------------------------------------------------
#[component]
fn FocusAndSelectionScenario() -> Element {
    let mut focus_open = use_signal(|| false);
    let mut inner_clicks = use_signal(|| 0usize);

    rsx! {
        div {
            style: CARD_STYLE,
            div {
                style: "display: flex; flex-direction: column; gap: 0.25rem;",
                h3 {
                    style: "margin: 0; font-size: 1rem; font-weight: 600; color: #0f172a;",
                    "2. Focus-Visible, Non-Tabbable Content & Text Selection Guard (SW3A-4, SW3A-5)"
                }
                p {
                    style: "margin: 0; font-size: 0.75rem; color: #64748b;",
                    "Tab onto the trigger to open via :focus-visible; content descendants receive tabindex=\"-1\" so Tab skips directly to the next control. Selecting text inside content keeps the card open until selection clears."
                }
            }

            div {
                style: "display: flex; flex-wrap: wrap; align-items: center; gap: 1rem; padding: 1.25rem; border-radius: 0.75rem; border: 1px dashed #cbd5e1; background-color: #f8fafc;",
                HoverCardRoot {
                    id: "sw3a-hc-focus-root",
                    open_delay: 150,
                    close_delay: 200,
                    ignore_non_keyboard_focus: true,
                    on_open_change: move |next| focus_open.set(next),
                    HoverCardTrigger {
                        id: "sw3a-hc-focus-trigger",
                        href: "#sw3a-hc-focus",
                        style: TRIGGER_STYLE,
                        on_click: move |evt: MouseEvent| evt.prevent_default(),
                        "Keyboard Focus & Selection Trigger"
                    }
                    HoverCardPortal {
                        HoverCardContent {
                            id: "sw3a-hc-focus-content",
                            style: CONTENT_STYLE,
                            p {
                                id: "sw3a-hc-selectable-text",
                                style: "margin: 0; font-size: 0.8125rem; line-height: 1.45; color: #334155;",
                                "Select this preview text and drag outside the card: HoverCard keeps content open while selection persists."
                            }
                            button {
                                id: "sw3a-hc-inner-btn",
                                r#type: "button",
                                style: "align-self: flex-start; padding: 0.375rem 0.75rem; border-radius: 0.375rem; border: 1px solid #7c3aed; background-color: #f5f3ff; color: #5b21b6; font-size: 0.75rem; font-weight: 600; cursor: pointer;",
                                onclick: move |_| inner_clicks += 1,
                                "Inner Action ({inner_clicks()})"
                            }
                        }
                    }
                }

                button {
                    id: "sw3a-hc-after-focus-btn",
                    r#type: "button",
                    style: "padding: 0.5rem 0.875rem; border-radius: 0.5rem; border: 1px solid #cbd5e1; background-color: #ffffff; color: #334155; font-size: 0.8125rem; font-weight: 600; cursor: pointer;",
                    "Next Tab Stop"
                }

                span {
                    id: "sw3a-hc-focus-state-badge",
                    style: state_badge_style(focus_open()),
                    if focus_open() { "open" } else { "closed" }
                }
            }

            div {
                style: "display: flex; flex-wrap: wrap; align-items: center; gap: 0.75rem; font-size: 0.8125rem; color: #475569;",
                span {
                    style: "font-weight: 600; color: #0f172a;",
                    "Blitz D-5 Dogfood Probe:"
                }
                span {
                    id: "sw3a-hc-explicit-user-select-text",
                    style: "user-select: text; display: inline-block; padding: 0.375rem 0.625rem; border-radius: 0.375rem; border: 1px dashed #94a3b8; background-color: #f8fafc; font-size: 0.8125rem; color: #334155;",
                    "Explicit user-select: text probe span for Blitz Native DragMode::Selecting verification"
                }
            }
        }
    }
}

// ---------------------------------------------------------------------------
// Scenario 3: HoverCardContentStatic Demo
// ---------------------------------------------------------------------------
#[component]
fn StaticHoverCardScenario() -> Element {
    let mut static_open = use_signal(|| true);

    rsx! {
        div {
            style: CARD_STYLE,
            div {
                style: "display: flex; flex-direction: column; gap: 0.25rem;",
                h3 {
                    style: "margin: 0; font-size: 1rem; font-weight: 600; color: #0f172a;",
                    "3. Static Flow Content (HoverCardContentStatic)"
                }
                p {
                    style: "margin: 0; font-size: 0.75rem; color: #64748b;",
                    "HoverCardContentStatic renders inline without FloatingLayer positioning while retaining hover delays, outside dismiss, and selection guard."
                }
            }

            div {
                style: "display: flex; flex-direction: column; align-items: flex-start; gap: 0.75rem; padding: 1.25rem; border-radius: 0.75rem; border: 1px dashed #cbd5e1; background-color: #f8fafc;",
                HoverCardRoot {
                    id: "sw3a-hc-static-root",
                    default_open: true,
                    open_delay: 150,
                    close_delay: 200,
                    on_open_change: move |next| static_open.set(next),
                    div {
                        style: "display: flex; align-items: center; gap: 0.75rem;",
                        HoverCardTrigger {
                            id: "sw3a-hc-static-trigger",
                            href: "#sw3a-hc-static",
                            style: TRIGGER_STYLE,
                            on_click: move |evt: MouseEvent| evt.prevent_default(),
                            "Static Preview Trigger"
                        }
                        span {
                            id: "sw3a-hc-static-state-badge",
                            style: state_badge_style(static_open()),
                            if static_open() { "open" } else { "closed" }
                        }
                    }
                    HoverCardContentStatic {
                        id: "sw3a-hc-static-content",
                        style: "margin-top: 0.5rem; width: 320px; padding: 1rem; border-radius: 0.75rem; border: 1px solid #cbd5e1; background-color: #ffffff; color: #0f172a; box-shadow: 0 4px 12px rgba(15, 23, 42, 0.08);",
                        strong {
                            style: "display: block; font-size: 0.875rem; color: #0f172a; margin-bottom: 0.25rem;",
                            "Static Flow Preview Card"
                        }
                        p {
                            style: "margin: 0; font-size: 0.8125rem; line-height: 1.45; color: #475569;",
                            "Rendered in normal document flow via HoverCardContentStatic."
                        }
                    }
                }
            }
        }
    }
}

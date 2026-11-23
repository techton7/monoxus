use dioxus::prelude::*;
use monoxus::foundation::shared::Direction;
use monoxus::navigation_menu::{
    NavigationMenuActivationMode, NavigationMenuContent, NavigationMenuIndicator,
    NavigationMenuItem, NavigationMenuLink, NavigationMenuLinkSelectEvent, NavigationMenuList,
    NavigationMenuRoot, NavigationMenuSub, NavigationMenuTrigger, NavigationMenuViewport,
};

const CARD_STYLE: &str = "border-radius: 1rem; border: 1px solid #e2e8f0; background-color: #ffffff; padding: 1.5rem; box-shadow: 0 1px 3px rgba(15, 23, 42, 0.06); display: flex; flex-direction: column; gap: 1rem;";
const TRIGGER_STYLE: &str = "box-sizing: border-box; display: inline-flex; align-items: center; justify-content: center; gap: 0.375rem; height: 36px; padding: 0 0.875rem; border-radius: 0.5rem; border: 1px solid #cbd5e1; background-color: #ffffff; color: #0f172a; font-size: 0.875rem; font-weight: 600; cursor: pointer; outline: none; transition: background-color 150ms ease, border-color 150ms ease, color 150ms ease;";
const DIRECT_LINK_STYLE: &str = "box-sizing: border-box; display: inline-flex; align-items: center; justify-content: center; height: 36px; padding: 0 0.875rem; border-radius: 0.5rem; border: 1px solid #cbd5e1; background-color: #ffffff; color: #334155; font-size: 0.875rem; font-weight: 600; text-decoration: none; cursor: pointer; outline: none;";
const CONTENT_LINK_STYLE: &str = "box-sizing: border-box; display: inline-flex; flex-direction: column; align-items: flex-start; justify-content: center; gap: 0.125rem; padding: 0.5rem 0.75rem; border-radius: 0.5rem; border: 1px solid #e2e8f0; background-color: #f8fafc; color: #0f172a; font-size: 0.8125rem; font-weight: 600; text-decoration: none; cursor: pointer; outline: none;";
const BADGE_STYLE: &str = "display: inline-flex; align-items: center; border-radius: 9999px; padding: 0.125rem 0.625rem; font-size: 0.75rem; font-family: monospace; font-weight: 600; background-color: #f1f5f9; color: #334155; border: 1px solid #e2e8f0;";
const TOGGLE_BTN_STYLE: &str = "display: inline-flex; align-items: center; gap: 0.375rem; padding: 0.375rem 0.75rem; border-radius: 0.5rem; border: 1px solid #cbd5e1; background-color: #ffffff; color: #334155; font-size: 0.75rem; font-weight: 600; cursor: pointer;";

const NAV_MENU_PLAYGROUND_CSS: &str = r#"
[data-monoxus-navigation-menu-trigger][data-state="open"] {
    background-color: #f5f3ff !important;
    border-color: #8b5cf6 !important;
    color: #5b21b6 !important;
}
[data-monoxus-navigation-menu-trigger][data-disabled] {
    background-color: #f1f5f9 !important;
    border-color: #e2e8f0 !important;
    color: #94a3b8 !important;
    cursor: not-allowed !important;
    opacity: 0.65;
}
[data-monoxus-navigation-menu-trigger]:focus-visible,
[data-monoxus-navigation-menu-link]:focus-visible {
    box-shadow: 0 0 0 2px #7c3aed !important;
}
[data-monoxus-navigation-menu-link][data-active] {
    background-color: #ede9fe !important;
    border-color: #a78bfa !important;
    color: #5b21b6 !important;
}
[data-monoxus-navigation-menu-link][data-focused] {
    border-color: #7c3aed !important;
}
[data-monoxus-navigation-menu-indicator] {
    bottom: -6px;
    height: 4px;
    border-radius: 9999px;
    background-color: #7c3aed;
    transition: transform 200ms ease, width 200ms ease, opacity 150ms ease;
    pointer-events: none;
}
[data-monoxus-navigation-menu-indicator][data-state="hidden"] {
    opacity: 0;
}
[data-monoxus-navigation-menu-viewport] {
    width: var(--monoxus-navigation-menu-viewport-width);
    height: var(--monoxus-navigation-menu-viewport-height);
    transition: width 220ms ease, height 220ms ease, opacity 180ms ease;
    overflow: hidden;
    border-radius: 0.75rem;
    border: 1px solid #cbd5e1;
    background-color: #ffffff;
    box-shadow: 0 12px 32px rgba(15, 23, 42, 0.14);
}
[data-monoxus-navigation-menu-content][data-motion="from-start"] {
    border-left: 3px solid #7c3aed !important;
}
[data-monoxus-navigation-menu-content][data-motion="from-end"] {
    border-right: 3px solid #7c3aed !important;
}
[data-monoxus-navigation-menu-content][data-motion="to-start"] {
    opacity: 0.45;
}
[data-monoxus-navigation-menu-content][data-motion="to-end"] {
    opacity: 0.45;
}
"#;

#[component]
pub fn NavigationMenuPlayground() -> Element {
    rsx! {
        div {
            class: "max-w-4xl mx-auto space-y-8",
            style: "max-width: 56rem; margin: 0 auto; display: flex; flex-direction: column; gap: 2rem;",
            style { "{NAV_MENU_PLAYGROUND_CSS}" }

            div {
                style: "display: flex; flex-direction: column; gap: 0.375rem;",
                h2 {
                    style: "margin: 0; font-size: 1.5rem; font-weight: 700; color: #0f172a; letter-spacing: -0.025em;",
                    "Navigation Menu"
                }
                p {
                    style: "margin: 0; font-size: 0.875rem; color: #64748b;",
                    "Headless site-navigation primitive with 3-timer hover intent (200ms open / 150ms grace / 300ms skip), directional motion states, shared ResizeObserver Viewport & Indicator geometry, FocusProxy tab order, Manual/open_on_hover overrides, and nested NavigationMenuSub."
                }
            }

            ViewportIndicatorAndFocusScenario {}
            ManualAndNestedSubScenario {}
            InlineAndPerItemHoverScenario {}
        }
    }
}

// ---------------------------------------------------------------------------
// Scenario 1: Shared Viewport, Hover Delay/Grace/Skip, Motion, Indicator, FocusProxy, RTL & Link
// ---------------------------------------------------------------------------
#[component]
fn ViewportIndicatorAndFocusScenario() -> Element {
    let mut active_val = use_signal(String::new);
    let mut last_link = use_signal(|| "none".to_string());
    let mut dir = use_signal(|| Direction::Ltr);
    let mut prevent_link_close = use_signal(|| false);
    let mut inner_clicks = use_signal(|| 0usize);

    let dir_label = if dir() == Direction::Rtl { "rtl" } else { "ltr" };

    rsx! {
        div {
            style: CARD_STYLE,

            div {
                style: "display: flex; flex-direction: column; gap: 0.25rem;",
                h3 {
                    style: "margin: 0; font-size: 1rem; font-weight: 600; color: #0f172a;",
                    "1. Shared Viewport, Indicator Tracking, Motion Direction & FocusProxy (SW3C-1, SW3C-2, SW3C-3, SW3C-5, SW3C-6)"
                }
                p {
                    style: "margin: 0; font-size: 0.75rem; color: #64748b;",
                    "Hover opens after 200ms with 150ms close grace and 300ms skip-delay across triggers. Arrows rove triggers; Down (or Right/Left in vertical) enters Content; Tab/Shift+Tab preserves DOM order across Viewport via FocusProxy."
                }
            }

            // Control strip & live readouts
            div {
                style: "display: flex; flex-wrap: wrap; align-items: center; gap: 0.625rem;",
                button {
                    id: "sw3c-nm-dir-toggle",
                    r#type: "button",
                    style: TOGGLE_BTN_STYLE,
                    onclick: move |_| {
                        let next = if dir() == Direction::Ltr {
                            Direction::Rtl
                        } else {
                            Direction::Ltr
                        };
                        dir.set(next);
                    },
                    "Toggle dir ({dir_label})"
                }
                button {
                    id: "sw3c-nm-prevent-toggle",
                    r#type: "button",
                    style: TOGGLE_BTN_STYLE,
                    onclick: move |_| prevent_link_close.set(!prevent_link_close()),
                    if prevent_link_close() {
                        "Link prevent_default: ON"
                    } else {
                        "Link prevent_default: OFF"
                    }
                }
                span {
                    id: "sw3c-nm-dir-badge",
                    style: BADGE_STYLE,
                    "dir={dir_label}"
                }
                span {
                    id: "sw3c-nm-active-val",
                    style: BADGE_STYLE,
                    if active_val().is_empty() { "active=none" } else { "active={active_val()}" }
                }
                span {
                    id: "sw3c-nm-last-link",
                    style: BADGE_STYLE,
                    "selected={last_link()}"
                }
            }

            div {
                style: "display: flex; flex-direction: column; align-items: flex-start; gap: 0.75rem; padding: 1.25rem; border-radius: 0.75rem; border: 1px dashed #cbd5e1; background-color: #f8fafc;",

                div {
                    style: "display: flex; flex-wrap: wrap; align-items: center; gap: 0.75rem;",
                    button {
                        id: "sw3c-nm-prev-btn",
                        r#type: "button",
                        style: "padding: 0.5rem 0.75rem; border-radius: 0.5rem; border: 1px solid #cbd5e1; background-color: #ffffff; color: #475569; font-size: 0.75rem; font-weight: 600; cursor: pointer;",
                        "Before Menu"
                    }

                    NavigationMenuRoot {
                        id: "sw3c-nm-viewport-root",
                        dir: dir(),
                        on_value_change: move |next: String| active_val.set(next),

                        NavigationMenuList {
                            id: "sw3c-nm-viewport-list",
                            style: "display: flex; flex-direction: row; align-items: center; gap: 0.5rem; list-style: none; padding: 0; margin: 0;",

                            NavigationMenuItem {
                                value: Some("overview".to_string()),
                                NavigationMenuTrigger {
                                    id: "sw3c-nm-trig-overview",
                                    style: "width: 112px; {TRIGGER_STYLE}",
                                    "Overview"
                                }
                                NavigationMenuContent {
                                    id: "sw3c-nm-content-overview",
                                    style: "box-sizing: border-box; width: 360px; height: 156px; padding: 14px; display: flex; flex-direction: column; gap: 8px; background-color: #ffffff;",
                                    div {
                                        style: "display: flex; gap: 8px;",
                                        NavigationMenuLink {
                                            id: "sw3c-nm-link-intro",
                                            href: "#intro",
                                            active: true,
                                            style: "flex: 1; {CONTENT_LINK_STYLE}",
                                            on_select: move |_| last_link.set("intro".to_string()),
                                            span { "Introduction" }
                                            span {
                                                style: "font-size: 0.6875rem; font-weight: 400; color: #64748b;",
                                                "aria-current=\"page\""
                                            }
                                        }
                                        NavigationMenuLink {
                                            id: "sw3c-nm-link-prevent",
                                            href: "#keep-open",
                                            style: "flex: 1; {CONTENT_LINK_STYLE}",
                                            on_select: move |evt: NavigationMenuLinkSelectEvent| {
                                                last_link.set("prevent-link".to_string());
                                                if *prevent_link_close.peek() {
                                                    evt.prevent_default();
                                                }
                                            },
                                            span { "Guarded Link" }
                                            span {
                                                style: "font-size: 0.6875rem; font-weight: 400; color: #64748b;",
                                                "Respects prevent_default"
                                            }
                                        }
                                    }
                                    button {
                                        id: "sw3c-nm-inner-btn",
                                        r#type: "button",
                                        style: "align-self: flex-start; padding: 0.375rem 0.75rem; border-radius: 0.375rem; border: 1px solid #7c3aed; background-color: #f5f3ff; color: #5b21b6; font-size: 0.75rem; font-weight: 600; cursor: pointer;",
                                        onclick: move |_| inner_clicks += 1,
                                        "Inside Action ({inner_clicks()})"
                                    }
                                }
                            }

                            NavigationMenuItem {
                                value: Some("components".to_string()),
                                NavigationMenuTrigger {
                                    id: "sw3c-nm-trig-components",
                                    style: "width: 132px; {TRIGGER_STYLE}",
                                    "Components"
                                }
                                NavigationMenuContent {
                                    id: "sw3c-nm-content-components",
                                    style: "box-sizing: border-box; width: 440px; height: 196px; padding: 14px; display: flex; flex-direction: column; gap: 8px; background-color: #ffffff;",
                                    div {
                                        style: "display: flex; gap: 8px;",
                                        NavigationMenuLink {
                                            id: "sw3c-nm-link-primitives",
                                            href: "#primitives",
                                            style: "flex: 1; {CONTENT_LINK_STYLE}",
                                            on_select: move |_| last_link.set("primitives".to_string()),
                                            "Primitives"
                                        }
                                        NavigationMenuLink {
                                            id: "sw3c-nm-link-patterns",
                                            href: "#patterns",
                                            style: "flex: 1; {CONTENT_LINK_STYLE}",
                                            on_select: move |_| last_link.set("patterns".to_string()),
                                            "Patterns"
                                        }
                                    }
                                    NavigationMenuLink {
                                        id: "sw3c-nm-link-tokens",
                                        href: "#tokens",
                                        style: "width: 100%; {CONTENT_LINK_STYLE}",
                                        on_select: move |_| last_link.set("tokens".to_string()),
                                        "Design Tokens & CSS Geometry Variables"
                                    }
                                }
                            }

                            NavigationMenuItem {
                                value: Some("disabled-item".to_string()),
                                NavigationMenuTrigger {
                                    id: "sw3c-nm-trig-disabled",
                                    disabled: true,
                                    style: "width: 96px; {TRIGGER_STYLE}",
                                    "Disabled"
                                }
                            }

                            NavigationMenuItem {
                                value: Some("docs".to_string()),
                                NavigationMenuLink {
                                    id: "sw3c-nm-link-docs",
                                    href: "#docs",
                                    style: "width: 80px; {DIRECT_LINK_STYLE}",
                                    on_select: move |_| last_link.set("docs".to_string()),
                                    "Docs"
                                }
                            }

                            NavigationMenuIndicator {
                                id: "sw3c-nm-indicator",
                            }
                        }

                        div {
                            style: "display: inline-flex; flex-direction: column; align-items: flex-start;",
                            NavigationMenuViewport {
                                id: "sw3c-nm-viewport",
                                style: "margin-top: 10px;",
                            }
                        }
                    }

                    button {
                        id: "sw3c-nm-next-btn",
                        r#type: "button",
                        style: "padding: 0.5rem 0.75rem; border-radius: 0.5rem; border: 1px solid #cbd5e1; background-color: #ffffff; color: #475569; font-size: 0.75rem; font-weight: 600; cursor: pointer;",
                        "After Menu"
                    }
                }
            }
        }
    }
}

// ---------------------------------------------------------------------------
// Scenario 2: Manual Activation Mode & Nested NavigationMenuSub (disable_toggle)
// ---------------------------------------------------------------------------
#[component]
fn ManualAndNestedSubScenario() -> Element {
    let mut manual_val = use_signal(String::new);
    let mut sub_val = use_signal(|| "guides".to_string());
    let mut sub_disable_toggle = use_signal(|| true);

    rsx! {
        div {
            style: CARD_STYLE,

            div {
                style: "display: flex; flex-direction: column; gap: 0.25rem;",
                h3 {
                    style: "margin: 0; font-size: 1rem; font-weight: 600; color: #0f172a;",
                    "2. Manual Activation Mode & Nested NavigationMenuSub (SW3C-4, SW3C-6)"
                }
                p {
                    style: "margin: 0; font-size: 0.75rem; color: #64748b;",
                    "activation_mode=Manual ignores pointer hover and opens only on click/Enter/Space. Click Workspace (Manual), then click the already-active Guides tab again to see Sub disable_toggle = true (stays open) vs false (closes sub-panel)."
                }
            }

            div {
                style: "display: flex; flex-wrap: wrap; align-items: center; gap: 0.625rem;",
                button {
                    id: "sw3c-nm-sub-disable-toggle-btn",
                    r#type: "button",
                    style: TOGGLE_BTN_STYLE,
                    onclick: move |_| sub_disable_toggle.set(!sub_disable_toggle()),
                    if sub_disable_toggle() {
                        "Sub disable_toggle: true"
                    } else {
                        "Sub disable_toggle: false"
                    }
                }
                span {
                    id: "sw3c-nm-manual-val",
                    style: BADGE_STYLE,
                    if manual_val().is_empty() { "manual=none" } else { "manual={manual_val()}" }
                }
                span {
                    id: "sw3c-nm-sub-val",
                    style: BADGE_STYLE,
                    if sub_val().is_empty() { "sub=none" } else { "sub={sub_val()}" }
                }
            }

            div {
                style: "padding: 1.25rem; border-radius: 0.75rem; border: 1px dashed #cbd5e1; background-color: #f8fafc;",
                NavigationMenuRoot {
                    id: "sw3c-nm-manual-root",
                    activation_mode: NavigationMenuActivationMode::Manual,
                    on_value_change: move |next: String| manual_val.set(next),

                    NavigationMenuList {
                        id: "sw3c-nm-manual-list",
                        style: "display: flex; flex-direction: row; align-items: center; gap: 0.5rem; list-style: none; padding: 0; margin: 0;",

                        NavigationMenuItem {
                            value: Some("workspace".to_string()),
                            NavigationMenuTrigger {
                                id: "sw3c-nm-manual-trig-ws",
                                style: TRIGGER_STYLE,
                                "Workspace (Manual)"
                            }
                            NavigationMenuContent {
                                id: "sw3c-nm-manual-content-ws",
                                style: "box-sizing: border-box; width: 420px; padding: 14px; background-color: #ffffff;",
                                NavigationMenuSub {
                                    id: "sw3c-nm-sub",
                                    default_value: Some("guides".to_string()),
                                    disable_toggle: sub_disable_toggle(),
                                    on_value_change: move |next: String| sub_val.set(next),

                                    NavigationMenuList {
                                        id: "sw3c-nm-sub-list",
                                        style: "display: flex; flex-direction: row; gap: 0.5rem; list-style: none; padding: 0; margin: 0 0 0.75rem 0;",
                                        NavigationMenuItem {
                                            value: Some("guides".to_string()),
                                            NavigationMenuTrigger {
                                                id: "sw3c-nm-sub-trig-guides",
                                                style: TRIGGER_STYLE,
                                                "Guides"
                                            }
                                            NavigationMenuContent {
                                                id: "sw3c-nm-sub-content-guides",
                                                style: "box-sizing: border-box; width: 380px; padding: 12px; display: flex; gap: 8px; background-color: #f8fafc;",
                                                NavigationMenuLink {
                                                    id: "sw3c-nm-sub-link-start",
                                                    href: "#sub-start",
                                                    style: "flex: 1; {CONTENT_LINK_STYLE}",
                                                    "Quick Start"
                                                }
                                                NavigationMenuLink {
                                                    id: "sw3c-nm-sub-link-theming",
                                                    href: "#sub-theming",
                                                    style: "flex: 1; {CONTENT_LINK_STYLE}",
                                                    "Theming"
                                                }
                                            }
                                        }
                                        NavigationMenuItem {
                                            value: Some("api".to_string()),
                                            NavigationMenuTrigger {
                                                id: "sw3c-nm-sub-trig-api",
                                                style: TRIGGER_STYLE,
                                                "API Reference"
                                            }
                                            NavigationMenuContent {
                                                id: "sw3c-nm-sub-content-api",
                                                style: "box-sizing: border-box; width: 380px; padding: 12px; display: flex; gap: 8px; background-color: #f8fafc;",
                                                NavigationMenuLink {
                                                    id: "sw3c-nm-sub-link-props",
                                                    href: "#sub-props",
                                                    style: "flex: 1; {CONTENT_LINK_STYLE}",
                                                    "Component Props"
                                                }
                                                NavigationMenuLink {
                                                    id: "sw3c-nm-sub-link-attrs",
                                                    href: "#sub-attrs",
                                                    style: "flex: 1; {CONTENT_LINK_STYLE}",
                                                    "Data Attributes"
                                                }
                                            }
                                        }
                                    }

                                    div {
                                        style: "display: inline-flex; flex-direction: column; align-items: flex-start;",
                                        NavigationMenuViewport {
                                            id: "sw3c-nm-sub-viewport",
                                            style: "box-shadow: none; border-color: #e2e8f0;",
                                        }
                                    }
                                }
                            }
                        }

                        NavigationMenuItem {
                            value: Some("resources".to_string()),
                            NavigationMenuLink {
                                id: "sw3c-nm-manual-link-res",
                                href: "#resources",
                                style: DIRECT_LINK_STYLE,
                                "Resources"
                            }
                        }
                    }

                    div {
                        style: "display: inline-flex; flex-direction: column; align-items: flex-start;",
                        NavigationMenuViewport {
                            id: "sw3c-nm-manual-viewport",
                            style: "margin-top: 10px;",
                        }
                    }
                }
            }
        }
    }
}

// ---------------------------------------------------------------------------
// Scenario 3: Inline Mode (No NavigationMenuViewport) & Per-Item open_on_hover = false
// ---------------------------------------------------------------------------
#[component]
fn InlineAndPerItemHoverScenario() -> Element {
    let mut inline_val = use_signal(String::new);

    rsx! {
        div {
            style: CARD_STYLE,

            div {
                style: "display: flex; flex-direction: column; gap: 0.25rem;",
                h3 {
                    style: "margin: 0; font-size: 1rem; font-weight: 600; color: #0f172a;",
                    "3. Inline Mode (No Viewport) & Per-Item open_on_hover = false (SW3C-4, SW3C-6)"
                }
                p {
                    style: "margin: 0; font-size: 0.75rem; color: #64748b;",
                    "Without NavigationMenuViewport, each NavigationMenuContent mounts inline inside its NavigationMenuItem. The first trigger opens on hover; the second sets open_on_hover=Some(false) and opens only via click or keyboard."
                }
            }

            div {
                style: "display: flex; align-items: center; gap: 0.5rem;",
                span {
                    id: "sw3c-nm-inline-val",
                    style: BADGE_STYLE,
                    if inline_val().is_empty() { "inline=none" } else { "inline={inline_val()}" }
                }
            }

            div {
                style: "padding: 1.25rem; border-radius: 0.75rem; border: 1px dashed #cbd5e1; background-color: #f8fafc;",
                NavigationMenuRoot {
                    id: "sw3c-nm-inline-root",
                    activation_mode: NavigationMenuActivationMode::Automatic,
                    on_value_change: move |next: String| inline_val.set(next),

                    NavigationMenuList {
                        id: "sw3c-nm-inline-list",
                        style: "display: flex; flex-direction: row; align-items: flex-start; gap: 0.75rem; list-style: none; padding: 0; margin: 0;",

                        NavigationMenuItem {
                            value: Some("auto-inline".to_string()),
                            style: "position: relative; display: flex; flex-direction: column; align-items: flex-start; gap: 0.5rem;",
                            NavigationMenuTrigger {
                                id: "sw3c-nm-inline-trig-auto",
                                style: TRIGGER_STYLE,
                                "Hover Inline"
                            }
                            NavigationMenuContent {
                                id: "sw3c-nm-inline-content-auto",
                                style: "box-sizing: border-box; width: 220px; padding: 12px; border-radius: 0.75rem; border: 1px solid #cbd5e1; background-color: #ffffff; box-shadow: 0 8px 20px rgba(15, 23, 42, 0.08); display: flex; flex-direction: column; gap: 6px;",
                                NavigationMenuLink {
                                    id: "sw3c-nm-inline-link-a",
                                    href: "#inline-a",
                                    style: CONTENT_LINK_STYLE,
                                    "Inline Hover Link A"
                                }
                            }
                        }

                        NavigationMenuItem {
                            value: Some("click-only-inline".to_string()),
                            open_on_hover: Some(false),
                            style: "position: relative; display: flex; flex-direction: column; align-items: flex-start; gap: 0.5rem;",
                            NavigationMenuTrigger {
                                id: "sw3c-nm-inline-trig-click",
                                style: TRIGGER_STYLE,
                                "Click-Only Inline"
                            }
                            NavigationMenuContent {
                                id: "sw3c-nm-inline-content-click",
                                style: "box-sizing: border-box; width: 220px; padding: 12px; border-radius: 0.75rem; border: 1px solid #cbd5e1; background-color: #ffffff; box-shadow: 0 8px 20px rgba(15, 23, 42, 0.08); display: flex; flex-direction: column; gap: 6px;",
                                NavigationMenuLink {
                                    id: "sw3c-nm-inline-link-b",
                                    href: "#inline-b",
                                    style: CONTENT_LINK_STYLE,
                                    "Click-Only Link B"
                                }
                            }
                        }
                    }
                }
            }
        }
    }
}

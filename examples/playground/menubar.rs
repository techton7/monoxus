use dioxus::prelude::*;
use monoxus::{
    foundation::{
        overlay::{PlacementAlign, PlacementSide},
        shared::Direction,
    },
    menubar::{
        CheckedState, MenubarArrow, MenubarCheckboxItem, MenubarContent, MenubarContentStatic,
        MenubarGroup, MenubarGroupHeading, MenubarItem, MenubarItemIndicator, MenubarMenu,
        MenubarPortal, MenubarRadioGroup, MenubarRadioItem, MenubarRoot, MenubarSeparator,
        MenubarSub, MenubarSubContent, MenubarSubTrigger, MenubarTrigger, MenuPreventableEvent,
    },
};

const CARD_STYLE: &str = "border-radius: 1rem; border: 1px solid #e2e8f0; background-color: #ffffff; padding: 1.5rem; box-shadow: 0 1px 3px rgba(15, 23, 42, 0.06); display: flex; flex-direction: column; gap: 1rem;";
const CTRL_BTN_STYLE: &str = "display: inline-flex; align-items: center; gap: 0.375rem; padding: 0.45rem 0.75rem; border-radius: 0.5rem; border: 1px solid #cbd5e1; background-color: #ffffff; color: #334155; font-size: 0.75rem; font-weight: 600; cursor: pointer;";
const MENUBAR_STYLE: &str = "display: inline-flex; align-items: center; gap: 0.25rem; padding: 0.25rem; border-radius: 0.625rem; border: 1px solid #cbd5e1; background-color: #f8fafc; box-shadow: 0 1px 2px rgba(15, 23, 42, 0.05);";
const TRIGGER_STYLE: &str = "display: inline-flex; align-items: center; padding: 0.375rem 0.75rem; border-radius: 0.375rem; border: 1px solid transparent; background-color: transparent; color: #1e293b; font-size: 0.875rem; font-weight: 600; cursor: pointer; box-sizing: border-box; outline: none;";
const CONTENT_STYLE: &str = "min-width: 220px; padding: 0.375rem; border-radius: 0.75rem; border: 1px solid #cbd5e1; background-color: #ffffff; color: #0f172a; box-shadow: 0 14px 34px rgba(15, 23, 42, 0.16); z-index: 60; display: flex; flex-direction: column; gap: 0.125rem;";
const ITEM_STYLE: &str = "display: flex; align-items: center; justify-content: space-between; gap: 0.5rem; padding: 0.4rem 0.625rem; border-radius: 0.375rem; font-size: 0.8125rem; font-weight: 500; color: #1e293b; cursor: pointer; user-select: none; outline: none;";
const HEADING_STYLE: &str = "padding: 0.3rem 0.625rem 0.15rem; font-size: 0.6875rem; font-weight: 700; text-transform: uppercase; letter-spacing: 0.05em; color: #64748b;";
const SEP_STYLE: &str = "height: 1px; margin: 0.25rem 0.375rem; background-color: #e2e8f0;";
const IND_STYLE: &str = "display: inline-flex; align-items: center; justify-content: center; width: 1rem; font-size: 0.75rem; font-weight: 700; color: #7c3aed;";

const MENUBAR_PLAYGROUND_CSS: &str = r#"
[data-monoxus-menubar-trigger]:focus-visible {
    outline: 2px solid #7c3aed;
    outline-offset: 1px;
}
[data-monoxus-menubar-trigger][data-highlighted],
[data-monoxus-menubar-trigger][data-state="open"] {
    background-color: #ede9fe !important;
    color: #5b21b6 !important;
}
[data-monoxus-menubar-trigger][data-disabled] {
    color: #94a3b8 !important;
    opacity: 0.5 !important;
    cursor: not-allowed !important;
}
[data-monoxus-menubar-item][data-highlighted],
[data-monoxus-menubar-checkbox-item][data-highlighted],
[data-monoxus-menubar-radio-item][data-highlighted],
[data-monoxus-menubar-sub-trigger][data-highlighted] {
    background-color: #f3e8ff !important;
    color: #5b21b6 !important;
}
[data-monoxus-menubar-item][data-disabled],
[data-monoxus-menubar-checkbox-item][data-disabled],
[data-monoxus-menubar-radio-item][data-disabled],
[data-monoxus-menubar-sub-trigger][data-disabled] {
    color: #94a3b8 !important;
    opacity: 0.5 !important;
    cursor: not-allowed !important;
}
"#;

fn badge_style(active: bool) -> &'static str {
    if active {
        "display: inline-flex; align-items: center; border-radius: 9999px; padding: 0.15rem 0.6rem; font-size: 0.75rem; font-family: monospace; font-weight: 700; background-color: #ede9fe; color: #6d28d9; border: 1px solid #c4b5fd;"
    } else {
        "display: inline-flex; align-items: center; border-radius: 9999px; padding: 0.15rem 0.6rem; font-size: 0.75rem; font-family: monospace; font-weight: 600; background-color: #f1f5f9; color: #475569; border: 1px solid #e2e8f0;"
    }
}

#[component]
pub fn MenubarPlayground() -> Element {
    rsx! {
        div {
            class: "max-w-4xl mx-auto space-y-8",
            style: "max-width: 56rem; margin: 0 auto; display: flex; flex-direction: column; gap: 2rem;",
            style { "{MENUBAR_PLAYGROUND_CSS}" }

            div {
                style: "display: flex; flex-direction: column; gap: 0.375rem;",
                h2 {
                    style: "margin: 0; font-size: 1.5rem; font-weight: 700; color: #0f172a; letter-spacing: -0.025em;",
                    "Menubar"
                }
                p {
                    style: "margin: 0; font-size: 0.875rem; color: #64748b;",
                    "Persistent horizontal role=\"menubar\" coordinating top-level roving focus, pointer-enter menu swapping, cross-menu ArrowLeft/ArrowRight switching from root and submenus, disabled menu skip, non-modal Tab out, and isolated RTL bars."
                }
            }

            DesktopMenubarScenario {}
            RtlAndStaticMenubarScenario {}
        }
    }
}

#[component]
fn DesktopMenubarScenario() -> Element {
    let mut active_menu = use_signal(String::new);
    let mut rtl_mode = use_signal(|| false);
    let mut loop_focus = use_signal(|| true);
    let mut last_action = use_signal(|| String::from("none"));
    let mut show_sidebar = use_signal(|| CheckedState::Checked);
    let mut full_urls = use_signal(|| CheckedState::Unchecked);
    let mut zoom_level = use_signal(|| String::from("100%"));
    let mut outside_clicks = use_signal(|| 0usize);

    let effective_dir = if rtl_mode() {
        Direction::Rtl
    } else {
        Direction::Ltr
    };

    rsx! {
        div {
            style: CARD_STYLE,
            div {
                style: "display: flex; flex-direction: column; gap: 0.25rem;",
                h3 {
                    style: "margin: 0; font-size: 1rem; font-weight: 600; color: #0f172a;",
                    "1. Desktop Menubar: Roving Focus, Hover Swap, Cross-Menu Arrows & Submenu Precedence (SW3B-15..SW3B-18)"
                }
                p {
                    style: "margin: 0; font-size: 0.75rem; color: #64748b;",
                    "Hovering View while File is open swaps menus immediately. Inside Share submenu, ArrowLeft closes only the submenu; ArrowRight on a leaf item switches across disabled Edit to View."
                }
            }

            div {
                style: "display: flex; flex-wrap: wrap; align-items: center; gap: 0.5rem;",
                button {
                    id: "sw3b-mb-toggle-dir",
                    r#type: "button",
                    style: CTRL_BTN_STYLE,
                    onclick: move |_| {
                        let cur = *rtl_mode.peek();
                        rtl_mode.set(!cur);
                    },
                    if rtl_mode() { "dir = rtl" } else { "dir = ltr" }
                }
                button {
                    id: "sw3b-mb-toggle-loop",
                    r#type: "button",
                    style: CTRL_BTN_STYLE,
                    onclick: move |_| {
                        let cur = *loop_focus.peek();
                        loop_focus.set(!cur);
                    },
                    if loop_focus() { "loop_focus = true" } else { "loop_focus = false" }
                }
            }

            div {
                style: "display: flex; flex-wrap: wrap; align-items: center; justify-content: space-between; gap: 1rem; padding: 1.25rem; border-radius: 0.75rem; border: 1px dashed #cbd5e1; background-color: #f8fafc;",
                MenubarRoot {
                    id: "sw3b-mb-main",
                    dir: Some(effective_dir),
                    loop_focus: loop_focus(),
                    style: MENUBAR_STYLE,
                    on_value_change: move |val| active_menu.set(val),

                    MenubarMenu {
                        value: Some("file".to_string()),
                        MenubarTrigger {
                            id: "sw3b-mb-trig-file",
                            style: TRIGGER_STYLE,
                            "File"
                        }
                        MenubarPortal {
                            MenubarContent {
                                id: "sw3b-mb-content-file",
                                style: CONTENT_STYLE,
                                side: PlacementSide::Bottom,
                                align: PlacementAlign::Start,
                                side_offset: 6.0,
                                MenubarItem {
                                    id: "sw3b-mb-file-new",
                                    style: ITEM_STYLE,
                                    on_select: move |_| last_action.set("file:new".to_string()),
                                    span { "New Tab" }
                                    span { style: "font-size: 0.7rem; color: #94a3b8;", "⌘T" }
                                }
                                MenubarSub {
                                    MenubarSubTrigger {
                                        id: "sw3b-mb-file-share",
                                        style: ITEM_STYLE,
                                        span { "Share" }
                                        span { style: "color: #64748b;", "▸" }
                                    }
                                    MenubarPortal {
                                        MenubarSubContent {
                                            id: "sw3b-mb-file-share-sub",
                                            style: "min-width: 165px; padding: 0.375rem; border-radius: 0.75rem; border: 1px solid #cbd5e1; background-color: #ffffff; color: #0f172a; box-shadow: 0 12px 28px rgba(15, 23, 42, 0.16); z-index: 70; display: flex; flex-direction: column; gap: 0.125rem;",
                                            MenubarItem {
                                                id: "sw3b-mb-share-email",
                                                style: ITEM_STYLE,
                                                on_select: move |_| last_action.set("share:email".to_string()),
                                                "Email Link"
                                            }
                                            MenubarItem {
                                                id: "sw3b-mb-share-notes",
                                                style: ITEM_STYLE,
                                                on_select: move |_| last_action.set("share:notes".to_string()),
                                                "Notes"
                                            }
                                        }
                                    }
                                }
                                MenubarItem {
                                    id: "sw3b-mb-file-open",
                                    close_on_select: false,
                                    style: ITEM_STYLE,
                                    on_select: move |_| last_action.set("file:open".to_string()),
                                    "Open Recent"
                                }
                                MenubarSeparator { id: "sw3b-mb-file-sep", style: SEP_STYLE }
                                MenubarItem {
                                    id: "sw3b-mb-file-print",
                                    style: ITEM_STYLE,
                                    on_select: move |_| last_action.set("file:print".to_string()),
                                    "Print..."
                                }
                                MenubarArrow {
                                    id: "sw3b-mb-file-arrow",
                                    width: 10.0,
                                    height: 5.0,
                                    style: "color: #cbd5e1;",
                                }
                            }
                        }
                    }

                    MenubarMenu {
                        value: Some("edit".to_string()),
                        MenubarTrigger {
                            id: "sw3b-mb-trig-edit",
                            disabled: true,
                            style: TRIGGER_STYLE,
                            "Edit"
                        }
                        MenubarPortal {
                            MenubarContent {
                                id: "sw3b-mb-content-edit",
                                style: CONTENT_STYLE,
                                MenubarItem { id: "sw3b-mb-edit-undo", "Undo" }
                            }
                        }
                    }

                    MenubarMenu {
                        value: Some("view".to_string()),
                        MenubarTrigger {
                            id: "sw3b-mb-trig-view",
                            style: TRIGGER_STYLE,
                            "View"
                        }
                        MenubarPortal {
                            MenubarContent {
                                id: "sw3b-mb-content-view",
                                style: CONTENT_STYLE,
                                side: PlacementSide::Bottom,
                                align: PlacementAlign::Start,
                                side_offset: 6.0,
                                MenubarCheckboxItem {
                                    id: "sw3b-mb-view-sidebar",
                                    checked: Some(show_sidebar()),
                                    close_on_select: false,
                                    style: ITEM_STYLE,
                                    on_checked_change: move |next| show_sidebar.set(next),
                                    span { "Show Sidebar" }
                                    MenubarItemIndicator {
                                        id: "sw3b-mb-view-sidebar-ind",
                                        style: IND_STYLE,
                                        "✓"
                                    }
                                }
                                MenubarCheckboxItem {
                                    id: "sw3b-mb-view-fullurls",
                                    checked: Some(full_urls()),
                                    close_on_select: false,
                                    style: ITEM_STYLE,
                                    on_checked_change: move |next| full_urls.set(next),
                                    span { "Always Show Full URLs" }
                                    MenubarItemIndicator {
                                        id: "sw3b-mb-view-fullurls-ind",
                                        style: IND_STYLE,
                                        "✓"
                                    }
                                }
                                MenubarSeparator { id: "sw3b-mb-view-sep", style: SEP_STYLE }
                                MenubarGroup {
                                    id: "sw3b-mb-view-grp",
                                    MenubarGroupHeading {
                                        id: "sw3b-mb-view-grp-heading",
                                        style: HEADING_STYLE,
                                        "Zoom"
                                    }
                                    MenubarRadioGroup {
                                        id: "sw3b-mb-view-rg",
                                        value: Some(zoom_level()),
                                        on_value_change: move |next| zoom_level.set(next),
                                        MenubarRadioItem {
                                            id: "sw3b-mb-view-zoom-100",
                                            value: "100%",
                                            close_on_select: false,
                                            style: ITEM_STYLE,
                                            span { "Actual Size (100%)" }
                                            MenubarItemIndicator { style: IND_STYLE, "●" }
                                        }
                                        MenubarRadioItem {
                                            id: "sw3b-mb-view-zoom-125",
                                            value: "125%",
                                            close_on_select: false,
                                            style: ITEM_STYLE,
                                            span { "Zoom In (125%)" }
                                            MenubarItemIndicator { style: IND_STYLE, "●" }
                                        }
                                    }
                                }
                            }
                        }
                    }

                    MenubarMenu {
                        value: Some("window".to_string()),
                        MenubarTrigger {
                            id: "sw3b-mb-trig-window",
                            style: TRIGGER_STYLE,
                            "Window"
                        }
                        MenubarPortal {
                            MenubarContent {
                                id: "sw3b-mb-content-window",
                                style: CONTENT_STYLE,
                                side: PlacementSide::Bottom,
                                align: PlacementAlign::Start,
                                side_offset: 6.0,
                                MenubarItem {
                                    id: "sw3b-mb-win-min",
                                    style: ITEM_STYLE,
                                    on_select: move |_| last_action.set("window:minimize".to_string()),
                                    "Minimize"
                                }
                                MenubarItem {
                                    id: "sw3b-mb-win-zoom",
                                    style: ITEM_STYLE,
                                    on_select: move |_| last_action.set("window:zoom".to_string()),
                                    "Zoom"
                                }
                            }
                        }
                    }
                }

                button {
                    id: "sw3b-mb-outside-btn",
                    r#type: "button",
                    style: CTRL_BTN_STYLE,
                    onclick: move |_| {
                        let next = *outside_clicks.peek() + 1;
                        outside_clicks.set(next);
                    },
                    "Outside / Tab Target ({outside_clicks()})"
                }
            }

            div {
                style: "display: flex; flex-wrap: wrap; align-items: center; gap: 0.5rem;",
                span {
                    id: "sw3b-mb-active-badge",
                    style: badge_style(!active_menu().is_empty()),
                    {if active_menu().is_empty() { "active=none".to_string() } else { format!("active={}", active_menu()) }}
                }
                span {
                    id: "sw3b-mb-last-action",
                    style: badge_style(last_action() != "none"),
                    "action={last_action()}"
                }
                span {
                    id: "sw3b-mb-zoom-badge",
                    style: badge_style(true),
                    "zoom={zoom_level()}"
                }
            }
        }
    }
}

#[component]
fn RtlAndStaticMenubarScenario() -> Element {
    let mut rtl_action = use_signal(|| String::from("idle"));

    rsx! {
        div {
            style: CARD_STYLE,
            div {
                style: "display: flex; flex-direction: column; gap: 0.25rem;",
                h3 {
                    style: "margin: 0; font-size: 1rem; font-weight: 600; color: #0f172a;",
                    "2. Isolated RTL & Clamped Menubar with Static Content (SW3B-15, MenubarContentStatic)"
                }
                p {
                    style: "margin: 0; font-size: 0.75rem; color: #64748b;",
                    "Second independent Menubar with dir=\"rtl\", loop_focus=false, and MenubarContentStatic."
                }
            }

            div {
                dir: "rtl",
                style: "display: flex; flex-direction: column; align-items: flex-start; justify-content: space-between; min-height: 8.5rem; gap: 0.75rem; padding: 1.25rem; border-radius: 0.75rem; border: 1px dashed #cbd5e1; background-color: #f8fafc;",
                MenubarRoot {
                    id: "sw3b-mb-rtl",
                    dir: Some(Direction::Rtl),
                    loop_focus: false,
                    style: MENUBAR_STYLE,
                    div {
                        style: "position: relative; display: inline-flex;",
                        MenubarMenu {
                            value: Some("tools".to_string()),
                            MenubarTrigger {
                                id: "sw3b-mb-rtl-tools",
                                style: TRIGGER_STYLE,
                                "Tools"
                            }
                            MenubarContentStatic {
                                id: "sw3b-mb-rtl-tools-static",
                                style: "position: absolute; top: 100%; right: 0; margin-top: 0.5rem; width: 180px; z-index: 20; padding: 0.375rem; border-radius: 0.75rem; border: 1px solid #cbd5e1; background-color: #ffffff; box-shadow: 0 4px 12px rgba(15, 23, 42, 0.08);",
                                on_open_auto_focus: move |evt: MenuPreventableEvent| evt.prevent_default(),
                                on_close_auto_focus: move |evt: MenuPreventableEvent| evt.prevent_default(),
                                MenubarItem {
                                    id: "sw3b-mb-rtl-tool-fmt",
                                    close_on_select: false,
                                    style: ITEM_STYLE,
                                    on_select: move |_| rtl_action.set("format".to_string()),
                                    "Format Document"
                                }
                            }
                        }
                    }
                    div {
                        style: "position: relative; display: inline-flex;",
                        MenubarMenu {
                            value: Some("help".to_string()),
                            MenubarTrigger {
                                id: "sw3b-mb-rtl-help",
                                style: TRIGGER_STYLE,
                                "Help"
                            }
                            MenubarContentStatic {
                                id: "sw3b-mb-rtl-help-static",
                                style: "position: absolute; top: 100%; right: 0; margin-top: 0.5rem; width: 180px; z-index: 20; padding: 0.375rem; border-radius: 0.75rem; border: 1px solid #cbd5e1; background-color: #ffffff; box-shadow: 0 4px 12px rgba(15, 23, 42, 0.08);",
                                on_open_auto_focus: move |evt: MenuPreventableEvent| evt.prevent_default(),
                                on_close_auto_focus: move |evt: MenuPreventableEvent| evt.prevent_default(),
                                MenubarItem {
                                    id: "sw3b-mb-rtl-help-docs",
                                    close_on_select: false,
                                    style: ITEM_STYLE,
                                    on_select: move |_| rtl_action.set("docs".to_string()),
                                    "Documentation"
                                }
                            }
                        }
                    }
                }
                span {
                    id: "sw3b-mb-rtl-badge",
                    dir: "ltr",
                    style: badge_style(rtl_action() != "idle"),
                    "rtl={rtl_action()}"
                }
            }
        }
    }
}

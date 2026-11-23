use dioxus::prelude::*;
use monoxus::{
    context_menu::{
        CheckedState, ContextMenuArrow, ContextMenuCheckboxGroup, ContextMenuCheckboxItem,
        ContextMenuContent, ContextMenuContentStatic, ContextMenuGroup,
        ContextMenuGroupHeading, ContextMenuItem, ContextMenuItemIndicator, ContextMenuLabel,
        ContextMenuPortal, ContextMenuRadioGroup, ContextMenuRadioItem, ContextMenuRoot,
        ContextMenuSeparator, ContextMenuSub, ContextMenuSubContent, ContextMenuSubContentStatic,
        ContextMenuSubTrigger, ContextMenuTrigger, ContextMenuTriggerRenderProps,
        MenuPreventableEvent, MenuSelectEvent,
    },
    foundation::shared::Direction,
};

const CARD_STYLE: &str = "border-radius: 1rem; border: 1px solid #e2e8f0; background-color: #ffffff; padding: 1.5rem; box-shadow: 0 1px 3px rgba(15, 23, 42, 0.06); display: flex; flex-direction: column; gap: 1rem;";
const CTRL_BTN_STYLE: &str = "display: inline-flex; align-items: center; gap: 0.375rem; padding: 0.45rem 0.75rem; border-radius: 0.5rem; border: 1px solid #cbd5e1; background-color: #ffffff; color: #334155; font-size: 0.75rem; font-weight: 600; cursor: pointer;";
const ZONE_A_STYLE: &str = "position: relative; flex: 1 1 260px; min-height: 170px; padding: 1.25rem; border-radius: 0.75rem; border: 2px dashed #c4b5fd; background-color: #faf5ff; color: #4c1d95; display: flex; flex-direction: column; justify-content: space-between; gap: 0.75rem; user-select: none;";
const ZONE_NESTED_STYLE: &str = "padding: 0.75rem 1rem; border-radius: 0.5rem; border: 1px solid #a78bfa; background-color: #ede9fe; color: #5b21b6; font-size: 0.8125rem; font-weight: 600;";
const ZONE_B_STYLE: &str = "flex: 1 1 200px; min-height: 170px; padding: 1.25rem; border-radius: 0.75rem; border: 2px dashed #93c5fd; background-color: #eff6ff; color: #1e3a8a; display: flex; align-items: center; justify-content: center; text-align: center; font-size: 0.875rem; font-weight: 600; user-select: none;";
const ZONE_DISABLED_STYLE: &str = "flex: 1 1 180px; min-height: 170px; padding: 1.25rem; border-radius: 0.75rem; border: 2px dashed #cbd5e1; background-color: #f1f5f9; color: #64748b; display: flex; align-items: center; justify-content: center; text-align: center; font-size: 0.8125rem; font-weight: 600; user-select: none; cursor: not-allowed;";
const CONTENT_STYLE: &str = "min-width: 225px; max-height: 300px; overflow-y: auto; padding: 0.375rem; border-radius: 0.75rem; border: 1px solid #cbd5e1; background-color: #ffffff; color: #0f172a; box-shadow: 0 14px 34px rgba(15, 23, 42, 0.16); z-index: 60; display: flex; flex-direction: column; gap: 0.125rem;";
const ITEM_STYLE: &str = "display: flex; align-items: center; justify-content: space-between; gap: 0.5rem; padding: 0.4rem 0.625rem; border-radius: 0.375rem; font-size: 0.8125rem; font-weight: 500; color: #1e293b; cursor: pointer; user-select: none; outline: none;";
const HEADING_STYLE: &str = "padding: 0.3rem 0.625rem 0.15rem; font-size: 0.6875rem; font-weight: 700; text-transform: uppercase; letter-spacing: 0.05em; color: #64748b;";
const SEP_STYLE: &str = "height: 1px; margin: 0.25rem 0.375rem; background-color: #e2e8f0;";
const IND_STYLE: &str = "display: inline-flex; align-items: center; justify-content: center; width: 1rem; font-size: 0.75rem; font-weight: 700; color: #7c3aed;";

const CONTEXT_MENU_PLAYGROUND_CSS: &str = r#"
[data-monoxus-context-menu-trigger][data-state="open"] {
    box-shadow: 0 0 0 3px rgba(124, 58, 237, 0.24);
}
[data-monoxus-context-menu-item][data-highlighted],
[data-monoxus-context-menu-checkbox-item][data-highlighted],
[data-monoxus-context-menu-radio-item][data-highlighted],
[data-monoxus-context-menu-sub-trigger][data-highlighted] {
    background-color: #f3e8ff !important;
    color: #5b21b6 !important;
}
[data-monoxus-context-menu-item][data-disabled],
[data-monoxus-context-menu-checkbox-item][data-disabled],
[data-monoxus-context-menu-radio-item][data-disabled],
[data-monoxus-context-menu-sub-trigger][data-disabled] {
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
pub fn ContextMenuPlayground() -> Element {
    rsx! {
        div {
            class: "max-w-4xl mx-auto space-y-8",
            style: "max-width: 56rem; margin: 0 auto; display: flex; flex-direction: column; gap: 2rem;",
            style { "{CONTEXT_MENU_PLAYGROUND_CSS}" }

            div {
                style: "display: flex; flex-direction: column; gap: 0.375rem;",
                h2 {
                    style: "margin: 0; font-size: 1.5rem; font-weight: 700; color: #0f172a; letter-spacing: -0.025em;",
                    "Context Menu"
                }
                p {
                    style: "margin: 0; font-size: 0.875rem; color: #64748b;",
                    "Virtual-point right-click (oncontextmenu) and 700ms touch/pen long-press floating menu with dynamic re-anchoring, nested innermost trigger claiming, disabled region suppression, and multi-trigger switching."
                }
            }

            VirtualAnchorAndMultiTriggerScenario {}
            StaticContextMenuScenario {}
        }
    }
}

#[component]
fn VirtualAnchorAndMultiTriggerScenario() -> Element {
    let mut open_a = use_signal(|| false);
    let mut open_nested = use_signal(|| false);
    let mut open_b = use_signal(|| false);
    let mut modal_mode = use_signal(|| true);
    let mut rtl_mode = use_signal(|| false);
    let mut last_action = use_signal(|| String::from("none"));
    let mut bookmarks_state = use_signal(|| CheckedState::Indeterminate);
    let mut group_vals = use_signal(|| vec![String::from("rulers")]);
    let mut theme_val = use_signal(|| String::from("light"));
    let mut open_count_a = use_signal(|| 0usize);

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
                    "1. Virtual Anchor Re-Anchoring, Nested Trigger & Multi-Trigger Switch (SW3B-12..SW3B-14)"
                }
                p {
                    style: "margin: 0; font-size: 0.75rem; color: #64748b;",
                    "Right-click Canvas A at two coordinates to see virtual-point re-anchoring (side=right, side_offset=2). Right-click Nested Inner Zone to claim innermost menu, or Canvas B while Canvas A is open to switch."
                }
            }

            div {
                style: "display: flex; flex-wrap: wrap; align-items: center; gap: 0.5rem;",
                button {
                    id: "sw3b-cm-open-a-btn",
                    r#type: "button",
                    style: CTRL_BTN_STYLE,
                    onclick: move |_| open_a.set(true),
                    "Open Menu A (Preview)"
                }
                button {
                    id: "sw3b-cm-toggle-modal",
                    r#type: "button",
                    style: CTRL_BTN_STYLE,
                    onclick: move |_| {
                        let cur = *modal_mode.peek();
                        modal_mode.set(!cur);
                    },
                    if modal_mode() { "modal = true" } else { "modal = false" }
                }
                button {
                    id: "sw3b-cm-toggle-dir",
                    r#type: "button",
                    style: CTRL_BTN_STYLE,
                    onclick: move |_| {
                        let cur = *rtl_mode.peek();
                        rtl_mode.set(!cur);
                    },
                    if rtl_mode() { "dir = rtl" } else { "dir = ltr" }
                }
            }

            div {
                style: "display: flex; flex-wrap: wrap; gap: 1rem;",

                // Canvas A with nested inner trigger
                ContextMenuRoot {
                    open: Some(open_a()),
                    modal: modal_mode(),
                    dir: Some(effective_dir),
                    on_open_change: move |next| {
                        open_a.set(next);
                        if next {
                            let c = *open_count_a.peek() + 1;
                            open_count_a.set(c);
                        }
                    },
                    ContextMenuTrigger {
                        id: "sw3b-cm-trig-a",
                        style: ZONE_A_STYLE,
                        div {
                            style: "font-size: 0.875rem; font-weight: 600;",
                            "Canvas A — Right-click or 700ms touch hold here"
                        }
                        ContextMenuRoot {
                            open: Some(open_nested()),
                            modal: modal_mode(),
                            dir: Some(effective_dir),
                            on_open_change: move |next| open_nested.set(next),
                            ContextMenuTrigger {
                                id: "sw3b-cm-trig-nested",
                                style: ZONE_NESTED_STYLE,
                                "Nested Inner Target (Right-click / Long-press claims innermost)"
                            }
                            ContextMenuPortal {
                                ContextMenuContent {
                                    id: "sw3b-cm-content-nested",
                                    style: CONTENT_STYLE,
                                    ContextMenuLabel {
                                        id: "sw3b-cm-nested-label",
                                        style: HEADING_STYLE,
                                        "Inner Region"
                                    }
                                    ContextMenuItem {
                                        id: "sw3b-cm-nested-inspect",
                                        style: ITEM_STYLE,
                                        on_select: move |_| last_action.set("nested:inspect".to_string()),
                                        "Inspect Inner Node"
                                    }
                                    ContextMenuItem {
                                        id: "sw3b-cm-nested-copy",
                                        close_on_select: false,
                                        style: ITEM_STYLE,
                                        on_select: move |_| last_action.set("nested:copy".to_string()),
                                        "Copy Selector"
                                    }
                                }
                            }
                        }
                        div {
                            style: "font-size: 0.75rem; color: #6d28d9;",
                            "Re-right-clicking moves menu to new pointer coordinates."
                        }
                    }
                    ContextMenuPortal {
                        ContextMenuContent {
                            id: "sw3b-cm-content-a",
                            style: CONTENT_STYLE,
                            ContextMenuGroup {
                                id: "sw3b-cm-grp-a",
                                ContextMenuGroupHeading {
                                    id: "sw3b-cm-grp-a-heading",
                                    style: HEADING_STYLE,
                                    "Canvas A"
                                }
                                ContextMenuItem {
                                    id: "sw3b-cm-a-back",
                                    style: ITEM_STYLE,
                                    on_select: move |_| last_action.set("a:back".to_string()),
                                    "Back"
                                }
                                ContextMenuItem {
                                    id: "sw3b-cm-a-forward",
                                    disabled: true,
                                    style: ITEM_STYLE,
                                    "Forward (Disabled)"
                                }
                                ContextMenuItem {
                                    id: "sw3b-cm-a-reload",
                                    close_on_select: false,
                                    style: ITEM_STYLE,
                                    on_select: move |evt: MenuSelectEvent| {
                                        evt.prevent_default();
                                        last_action.set("a:reload".to_string());
                                    },
                                    "Reload (Stays Open)"
                                }
                            }

                            ContextMenuSeparator { id: "sw3b-cm-sep-a", style: SEP_STYLE }

                            ContextMenuCheckboxItem {
                                id: "sw3b-cm-a-bookmarks",
                                checked: Some(bookmarks_state()),
                                close_on_select: false,
                                style: ITEM_STYLE,
                                on_checked_change: move |next| bookmarks_state.set(next),
                                span { "Show Bookmarks" }
                                ContextMenuItemIndicator {
                                    id: "sw3b-cm-a-bookmarks-ind",
                                    style: IND_STYLE,
                                    if bookmarks_state() == CheckedState::Indeterminate { "◐" } else { "✓" }
                                }
                            }

                            ContextMenuCheckboxGroup {
                                id: "sw3b-cm-a-cb-group",
                                value: Some(group_vals()),
                                on_value_change: move |next| group_vals.set(next),
                                ContextMenuCheckboxItem {
                                    id: "sw3b-cm-a-cb-rulers",
                                    value: Some("rulers".to_string()),
                                    close_on_select: false,
                                    style: ITEM_STYLE,
                                    span { "Rulers" }
                                    ContextMenuItemIndicator { style: IND_STYLE, "✓" }
                                }
                                ContextMenuCheckboxItem {
                                    id: "sw3b-cm-a-cb-grid",
                                    value: Some("grid".to_string()),
                                    close_on_select: false,
                                    style: ITEM_STYLE,
                                    span { "Snap Grid" }
                                    ContextMenuItemIndicator { style: IND_STYLE, "✓" }
                                }
                            }

                            ContextMenuSeparator { id: "sw3b-cm-sep-a2", style: SEP_STYLE }

                            ContextMenuRadioGroup {
                                id: "sw3b-cm-a-rg",
                                value: Some(theme_val()),
                                on_value_change: move |next| theme_val.set(next),
                                ContextMenuRadioItem {
                                    id: "sw3b-cm-a-rad-light",
                                    value: "light",
                                    close_on_select: false,
                                    style: ITEM_STYLE,
                                    span { "Theme: Light" }
                                    ContextMenuItemIndicator { style: IND_STYLE, "●" }
                                }
                                ContextMenuRadioItem {
                                    id: "sw3b-cm-a-rad-dark",
                                    value: "dark",
                                    close_on_select: false,
                                    style: ITEM_STYLE,
                                    span { "Theme: Dark" }
                                    ContextMenuItemIndicator { style: IND_STYLE, "●" }
                                }
                            }

                            ContextMenuSeparator { id: "sw3b-cm-sep-a3", style: SEP_STYLE }

                            ContextMenuSub {
                                ContextMenuSubTrigger {
                                    id: "sw3b-cm-a-sub-trig",
                                    style: ITEM_STYLE,
                                    span { "Developer Tools" }
                                    span { style: "color: #64748b;", "▸" }
                                }
                                ContextMenuPortal {
                                    ContextMenuSubContent {
                                        id: "sw3b-cm-a-sub-content",
                                        style: "min-width: 170px; padding: 0.375rem; border-radius: 0.75rem; border: 1px solid #cbd5e1; background-color: #ffffff; color: #0f172a; box-shadow: 0 12px 28px rgba(15, 23, 42, 0.16); z-index: 70; display: flex; flex-direction: column; gap: 0.125rem;",
                                        ContextMenuItem {
                                            id: "sw3b-cm-a-sub-dom",
                                            style: ITEM_STYLE,
                                            on_select: move |_| last_action.set("a:devtools-dom".to_string()),
                                            "Elements"
                                        }
                                        ContextMenuItem {
                                            id: "sw3b-cm-a-sub-console",
                                            style: ITEM_STYLE,
                                            on_select: move |_| last_action.set("a:devtools-console".to_string()),
                                            "Console"
                                        }
                                    }
                                }
                            }

                            ContextMenuArrow {
                                id: "sw3b-cm-a-arrow",
                                width: 10.0,
                                height: 5.0,
                                style: "color: #cbd5e1;",
                            }
                        }
                    }
                }

                // Canvas B (multi-trigger switch)
                ContextMenuRoot {
                    open: Some(open_b()),
                    modal: modal_mode(),
                    on_open_change: move |next| open_b.set(next),
                    ContextMenuTrigger {
                        id: "sw3b-cm-trig-b",
                        style: ZONE_B_STYLE,
                        "Canvas B — Right-click here to switch from A"
                    }
                    ContextMenuPortal {
                        ContextMenuContent {
                            id: "sw3b-cm-content-b",
                            style: CONTENT_STYLE,
                            ContextMenuItem {
                                id: "sw3b-cm-b-rename",
                                style: ITEM_STYLE,
                                on_select: move |_| last_action.set("b:rename".to_string()),
                                "Rename Layer"
                            }
                            ContextMenuItem {
                                id: "sw3b-cm-b-export",
                                style: ITEM_STYLE,
                                on_select: move |_| last_action.set("b:export".to_string()),
                                "Export Asset"
                            }
                        }
                    }
                }

                // Disabled Canvas C
                ContextMenuRoot {
                    ContextMenuTrigger {
                        id: "sw3b-cm-trig-disabled",
                        disabled: true,
                        style: ZONE_DISABLED_STYLE,
                        "Disabled Zone — Right-click / Hold Ignored"
                    }
                    ContextMenuPortal {
                        ContextMenuContent {
                            id: "sw3b-cm-content-disabled",
                            style: CONTENT_STYLE,
                            ContextMenuItem { "Never Opens" }
                        }
                    }
                }
            }

            div {
                style: "display: flex; flex-wrap: wrap; align-items: center; gap: 0.5rem;",
                span {
                    id: "sw3b-cm-state-a",
                    style: badge_style(open_a()),
                    if open_a() { "A:open" } else { "A:closed" }
                }
                span {
                    id: "sw3b-cm-state-nested",
                    style: badge_style(open_nested()),
                    if open_nested() { "Nested:open" } else { "Nested:closed" }
                }
                span {
                    id: "sw3b-cm-state-b",
                    style: badge_style(open_b()),
                    if open_b() { "B:open" } else { "B:closed" }
                }
                span {
                    id: "sw3b-cm-last-action",
                    style: badge_style(last_action() != "none"),
                    "action={last_action()}"
                }
                span {
                    id: "sw3b-cm-theme-badge",
                    style: badge_style(true),
                    "theme={theme_val()}"
                }
                span {
                    id: "sw3b-cm-open-count-a",
                    style: badge_style(open_count_a() > 0),
                    "opensA={open_count_a()}"
                }
            }
        }
    }
}

#[component]
fn StaticContextMenuScenario() -> Element {
    let mut static_action = use_signal(|| String::from("idle"));

    let trigger_render = Callback::new(|props: ContextMenuTriggerRenderProps| {
        rsx! {
            div {
                id: "{props.attrs.id}",
                class: props.class.as_deref().unwrap_or_default(),
                style: "{props.style}",
                "data-state": "{props.attrs.data_state.as_str()}",
                "data-disabled": props.attrs.data_disabled,
                oncontextmenu: move |evt| props.on_context_menu.call(evt),
                onpointerdown: move |evt| props.on_pointer_down.call(evt),
                onpointermove: move |evt| props.on_pointer_move.call(evt),
                onpointerup: move |evt| props.on_pointer_up.call(evt),
                onpointercancel: move |evt| props.on_pointer_cancel.call(evt),
                ..props.attributes,
                {props.children}
            }
        }
    });

    rsx! {
        div {
            style: CARD_STYLE,
            div {
                style: "display: flex; flex-direction: column; gap: 0.25rem;",
                h3 {
                    style: "margin: 0; font-size: 1rem; font-weight: 600; color: #0f172a;",
                    "2. Static Inline ContextMenu & as_child Trigger (ContextMenuContentStatic)"
                }
                p {
                    style: "margin: 0; font-size: 0.75rem; color: #64748b;",
                    "Non-modal static context surface retaining keyboard navigation and submenu parity."
                }
            }

            div {
                style: "display: flex; flex-direction: column; align-items: flex-start; gap: 0.75rem; padding: 1.25rem; border-radius: 0.75rem; border: 1px dashed #cbd5e1; background-color: #f8fafc;",
                ContextMenuRoot {
                    modal: false,
                    ContextMenuTrigger {
                        id: "sw3b-cm-static-trig",
                        style: Some("padding: 0.5rem 0.875rem; border-radius: 0.5rem; border: 1px solid #cbd5e1; background-color: #ffffff; font-size: 0.8125rem; font-weight: 600; color: #334155;".to_string()),
                        as_child: true,
                        render_child: Some(trigger_render),
                        "Static Context Trigger Region (as_child)"
                    }
                    ContextMenuContentStatic {
                        id: "sw3b-cm-static-content",
                        style: "width: 225px; padding: 0.375rem; border-radius: 0.75rem; border: 1px solid #cbd5e1; background-color: #ffffff; color: #0f172a; box-shadow: 0 4px 12px rgba(15, 23, 42, 0.08); display: flex; flex-direction: column; gap: 0.125rem;",
                        on_open_auto_focus: move |evt: MenuPreventableEvent| evt.prevent_default(),
                        on_close_auto_focus: move |evt: MenuPreventableEvent| evt.prevent_default(),
                        ContextMenuItem {
                            id: "sw3b-cm-static-item",
                            close_on_select: false,
                            style: ITEM_STYLE,
                            on_select: move |_| static_action.set("view-source".to_string()),
                            "View Page Source"
                        }
                        ContextMenuSub {
                            ContextMenuSubTrigger {
                                id: "sw3b-cm-static-sub-trig",
                                style: ITEM_STYLE,
                                span { "More Tools" }
                                span { style: "color: #64748b;", "▸" }
                            }
                            ContextMenuSubContentStatic {
                                id: "sw3b-cm-static-sub-content",
                                style: "margin-left: 0.5rem; padding: 0.25rem; border-radius: 0.5rem; border: 1px solid #e2e8f0; background-color: #f8fafc;",
                                ContextMenuItem {
                                    id: "sw3b-cm-static-sub-item",
                                    close_on_select: false,
                                    style: ITEM_STYLE,
                                    on_select: move |_| static_action.set("save-as".to_string()),
                                    "Save As..."
                                }
                            }
                        }
                    }
                }
                span {
                    id: "sw3b-cm-static-badge",
                    style: badge_style(static_action() != "idle"),
                    "static={static_action()}"
                }
            }
        }
    }
}

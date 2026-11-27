use dioxus::prelude::*;
use monoxus::{
    dropdown_menu::{
        CheckedState, DropdownMenuArrow, DropdownMenuCheckboxGroup, DropdownMenuCheckboxItem,
        DropdownMenuContent, DropdownMenuContentStatic,
        DropdownMenuGroup, DropdownMenuGroupHeading, DropdownMenuItem,
        DropdownMenuItemIndicator, DropdownMenuLabel, DropdownMenuPortal, DropdownMenuRadioGroup,
        DropdownMenuRadioItem, DropdownMenuRoot, DropdownMenuSeparator, DropdownMenuSub,
        DropdownMenuSubContent, DropdownMenuSubContentStatic, DropdownMenuSubTrigger,
        DropdownMenuTrigger, MenuPreventableEvent,
        MenuSelectEvent,
    },
    foundation::{
        overlay::{PlacementAlign, PlacementSide},
        shared::Direction,
    },
};

const CARD_STYLE: &str = "border-radius: 1rem; border: 1px solid #e2e8f0; background-color: #ffffff; padding: 1.5rem; box-shadow: 0 1px 3px rgba(15, 23, 42, 0.06); display: flex; flex-direction: column; gap: 1rem;";
const STAGE_STYLE: &str = "display: flex; flex-wrap: wrap; align-items: center; gap: 0.75rem; padding: 1.25rem; border-radius: 0.75rem; border: 1px dashed #cbd5e1; background-color: #f8fafc;";
const CTRL_BTN_STYLE: &str = "display: inline-flex; align-items: center; gap: 0.375rem; padding: 0.45rem 0.75rem; border-radius: 0.5rem; border: 1px solid #cbd5e1; background-color: #ffffff; color: #334155; font-size: 0.75rem; font-weight: 600; cursor: pointer;";
const TRIGGER_STYLE: &str = "display: inline-flex; align-items: center; gap: 0.4rem; padding: 0.5rem 0.9rem; border-radius: 0.5rem; border: 1px solid #7c3aed; background-color: #7c3aed; color: #ffffff; font-size: 0.875rem; font-weight: 600; cursor: pointer; box-sizing: border-box;";
const CONTENT_STYLE: &str = "min-width: 230px; max-height: 260px; overflow-y: auto; padding: 0.375rem; border-radius: 0.75rem; border: 1px solid #cbd5e1; background-color: #ffffff; color: #0f172a; box-shadow: 0 14px 34px rgba(15, 23, 42, 0.16); z-index: 60; display: flex; flex-direction: column; gap: 0.125rem;";
const ITEM_STYLE: &str = "display: flex; align-items: center; justify-content: space-between; gap: 0.5rem; padding: 0.4rem 0.625rem; border-radius: 0.375rem; font-size: 0.8125rem; font-weight: 500; color: #1e293b; cursor: pointer; user-select: none; outline: none;";
const GROUP_HEADING_STYLE: &str = "padding: 0.3rem 0.625rem 0.15rem; font-size: 0.6875rem; font-weight: 700; text-transform: uppercase; letter-spacing: 0.05em; color: #64748b;";
const SEP_STYLE: &str = "height: 1px; margin: 0.25rem 0.375rem; background-color: #e2e8f0;";
const INDICATOR_SLOT_STYLE: &str = "display: inline-flex; align-items: center; justify-content: center; width: 1rem; font-size: 0.75rem; font-weight: 700; color: #7c3aed;";

const DROPDOWN_MENU_PLAYGROUND_CSS: &str = r#"
[data-monoxus-dropdown-menu-trigger][data-state="open"] {
    background-color: #6d28d9 !important;
    border-color: #5b21b6 !important;
    box-shadow: 0 0 0 3px rgba(124, 58, 237, 0.22);
}
[data-monoxus-dropdown-menu-trigger]:focus-visible {
    outline: 2px solid #7c3aed;
    outline-offset: 2px;
}
[data-monoxus-dropdown-menu-item][data-highlighted],
[data-monoxus-dropdown-menu-checkbox-item][data-highlighted],
[data-monoxus-dropdown-menu-radio-item][data-highlighted],
[data-monoxus-dropdown-menu-sub-trigger][data-highlighted] {
    background-color: #f3e8ff !important;
    color: #5b21b6 !important;
}
[data-monoxus-dropdown-menu-sub-trigger][data-state="open"]:not([data-highlighted]) {
    background-color: #f8fafc !important;
    color: #6d28d9 !important;
}
[data-monoxus-dropdown-menu-item][data-disabled],
[data-monoxus-dropdown-menu-checkbox-item][data-disabled],
[data-monoxus-dropdown-menu-radio-item][data-disabled],
[data-monoxus-dropdown-menu-sub-trigger][data-disabled] {
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

fn checked_label(state: CheckedState) -> &'static str {
    match state {
        CheckedState::Unchecked => "unchecked",
        CheckedState::Checked => "checked",
        CheckedState::Indeterminate => "indeterminate",
    }
}

#[component]
pub fn DropdownMenuPlayground() -> Element {
    rsx! {
        div {
            class: "max-w-4xl mx-auto space-y-8",
            style: "max-width: 56rem; margin: 0 auto; display: flex; flex-direction: column; gap: 2rem;",
            style { "{DROPDOWN_MENU_PLAYGROUND_CSS}" }

            div {
                style: "display: flex; flex-direction: column; gap: 0.375rem;",
                h2 {
                    style: "margin: 0; font-size: 1.5rem; font-weight: 700; color: #0f172a; letter-spacing: -0.025em;",
                    "Dropdown Menu"
                }
                p {
                    style: "margin: 0; font-size: 0.875rem; color: #64748b;",
                    "Button-triggered floating menu supporting pointer vs keyboard open focus arbitration, disabled item skip, 1000ms typeahead cycling, tri-state CheckboxItem, CheckboxGroup, RadioGroup, nested submenus with triangle grace corridors, and modal/non-modal Tab policies."
                }
            }

            FullDropdownMenuScenario {}
            StaticAndAsChildDropdownScenario {}
        }
    }
}

#[component]
fn FullDropdownMenuScenario() -> Element {
    let mut is_open = use_signal(|| false);
    let mut modal_mode = use_signal(|| true);
    let mut loop_focus = use_signal(|| true);
    let mut rtl_mode = use_signal(|| false);
    let mut last_action = use_signal(|| String::from("none"));
    let mut tri_state = use_signal(|| CheckedState::Indeterminate);
    let mut cb_group = use_signal(|| vec![String::from("bold")]);
    let mut radio_val = use_signal(|| String::from("left"));
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
                    "1. Full Interactive Menu, Selection, Submenu Grace Corridor & Modality (SW3B-1..SW3B-11)"
                }
                p {
                    style: "margin: 0; font-size: 0.75rem; color: #64748b;",
                    "Click opens focusing content container; Enter/Space/ArrowDown opens focusing first enabled item. Type 'c'/'cl'/'co' for typeahead, toggle Indeterminate CheckboxItem, CheckboxGroup, RadioGroup, and hover Share for 100ms submenu + safe corridor."
                }
            }

            div {
                style: "display: flex; flex-wrap: wrap; align-items: center; gap: 0.5rem;",
                button {
                    id: "sw3b-dm-toggle-modal",
                    r#type: "button",
                    style: CTRL_BTN_STYLE,
                    onclick: move |_| {
                        let cur = *modal_mode.peek();
                        modal_mode.set(!cur);
                    },
                    if modal_mode() { "Mode: modal = true" } else { "Mode: modal = false" }
                }
                button {
                    id: "sw3b-dm-toggle-loop",
                    r#type: "button",
                    style: CTRL_BTN_STYLE,
                    onclick: move |_| {
                        let cur = *loop_focus.peek();
                        loop_focus.set(!cur);
                    },
                    if loop_focus() { "loop_focus = true" } else { "loop_focus = false" }
                }
                button {
                    id: "sw3b-dm-toggle-dir",
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
                style: STAGE_STYLE,
                button {
                    id: "sw3b-dm-prev-focus",
                    r#type: "button",
                    style: CTRL_BTN_STYLE,
                    "Prev Control"
                }

                DropdownMenuRoot {
                    modal: modal_mode(),
                    loop_focus: loop_focus(),
                    dir: Some(effective_dir),
                    on_open_change: move |next| is_open.set(next),
                    DropdownMenuTrigger {
                        id: "sw3b-dm-trigger",
                        style: TRIGGER_STYLE,
                        "Actions ▾"
                    }
                    DropdownMenuPortal {
                        DropdownMenuContent {
                            id: "sw3b-dm-content",
                            style: CONTENT_STYLE,
                            side: PlacementSide::Bottom,
                            align: PlacementAlign::Start,
                            side_offset: 6.0,
                            DropdownMenuGroup {
                                id: "sw3b-dm-grp-edit",
                                DropdownMenuGroupHeading {
                                    id: "sw3b-dm-grp-edit-heading",
                                    style: GROUP_HEADING_STYLE,
                                    "Edit Actions"
                                }
                                DropdownMenuItem {
                                    id: "sw3b-dm-item-cut",
                                    style: ITEM_STYLE,
                                    on_select: move |_| last_action.set("cut".to_string()),
                                    span { "Cut" }
                                    span { style: "font-size: 0.7rem; color: #94a3b8;", "⌘X" }
                                }
                                DropdownMenuItem {
                                    id: "sw3b-dm-item-disabled",
                                    disabled: true,
                                    style: ITEM_STYLE,
                                    on_select: move |_| last_action.set("disabled".to_string()),
                                    span { "Archive (Disabled)" }
                                }
                                DropdownMenuItem {
                                    id: "sw3b-dm-item-copy",
                                    style: ITEM_STYLE,
                                    on_select: move |_| last_action.set("copy".to_string()),
                                    span { "Copy" }
                                    span { style: "font-size: 0.7rem; color: #94a3b8;", "⌘C" }
                                }
                                DropdownMenuItem {
                                    id: "sw3b-dm-item-clone",
                                    close_on_select: false,
                                    style: ITEM_STYLE,
                                    on_select: move |_| last_action.set("clone".to_string()),
                                    span { "Clone (Stays Open)" }
                                }
                                DropdownMenuItem {
                                    id: "sw3b-dm-item-prevent",
                                    style: ITEM_STYLE,
                                    on_select: move |evt: MenuSelectEvent| {
                                        evt.prevent_default();
                                        last_action.set("prevent".to_string());
                                    },
                                    span { "Print (prevent_default)" }
                                }
                            }

                            DropdownMenuSeparator {
                                id: "sw3b-dm-sep-1",
                                style: SEP_STYLE,
                            }

                            DropdownMenuLabel {
                                id: "sw3b-dm-label-prefs",
                                style: GROUP_HEADING_STYLE,
                                "Formatting"
                            }

                            DropdownMenuCheckboxItem {
                                id: "sw3b-dm-cb-tri",
                                checked: Some(tri_state()),
                                close_on_select: false,
                                style: ITEM_STYLE,
                                on_checked_change: move |next| tri_state.set(next),
                                span { "Select Mixed" }
                                DropdownMenuItemIndicator {
                                    id: "sw3b-dm-cb-tri-ind",
                                    style: INDICATOR_SLOT_STYLE,
                                    if tri_state() == CheckedState::Indeterminate { "◐" } else { "✓" }
                                }
                            }

                            DropdownMenuCheckboxGroup {
                                id: "sw3b-dm-cb-group",
                                value: Some(cb_group()),
                                on_value_change: move |next| cb_group.set(next),
                                DropdownMenuCheckboxItem {
                                    id: "sw3b-dm-cb-bold",
                                    value: Some("bold".to_string()),
                                    close_on_select: false,
                                    style: ITEM_STYLE,
                                    span { "Bold" }
                                    DropdownMenuItemIndicator {
                                        id: "sw3b-dm-cb-bold-ind",
                                        style: INDICATOR_SLOT_STYLE,
                                        "✓"
                                    }
                                }
                                DropdownMenuCheckboxItem {
                                    id: "sw3b-dm-cb-italic",
                                    value: Some("italic".to_string()),
                                    close_on_select: false,
                                    style: ITEM_STYLE,
                                    span { "Italic" }
                                    DropdownMenuItemIndicator {
                                        id: "sw3b-dm-cb-italic-ind",
                                        force_mount: true,
                                        style: INDICATOR_SLOT_STYLE,
                                        if cb_group().iter().any(|v| v == "italic") { "✓" } else { "·" }
                                    }
                                }
                            }

                            DropdownMenuSeparator {
                                id: "sw3b-dm-sep-2",
                                style: SEP_STYLE,
                            }

                            DropdownMenuRadioGroup {
                                id: "sw3b-dm-rg",
                                value: Some(radio_val()),
                                on_value_change: move |next| radio_val.set(next),
                                DropdownMenuRadioItem {
                                    id: "sw3b-dm-rad-left",
                                    value: "left",
                                    close_on_select: false,
                                    style: ITEM_STYLE,
                                    span { "Align Left" }
                                    DropdownMenuItemIndicator {
                                        id: "sw3b-dm-rad-left-ind",
                                        style: INDICATOR_SLOT_STYLE,
                                        "●"
                                    }
                                }
                                DropdownMenuRadioItem {
                                    id: "sw3b-dm-rad-center",
                                    value: "center",
                                    close_on_select: false,
                                    style: ITEM_STYLE,
                                    span { "Align Center" }
                                    DropdownMenuItemIndicator {
                                        id: "sw3b-dm-rad-center-ind",
                                        style: INDICATOR_SLOT_STYLE,
                                        "●"
                                    }
                                }
                                DropdownMenuRadioItem {
                                    id: "sw3b-dm-rad-right",
                                    value: "right",
                                    close_on_select: false,
                                    style: ITEM_STYLE,
                                    span { "Align Right" }
                                    DropdownMenuItemIndicator {
                                        id: "sw3b-dm-rad-right-ind",
                                        style: INDICATOR_SLOT_STYLE,
                                        "●"
                                    }
                                }
                            }

                            DropdownMenuSeparator {
                                id: "sw3b-dm-sep-3",
                                style: SEP_STYLE,
                            }

                            DropdownMenuSub {
                                DropdownMenuSubTrigger {
                                    id: "sw3b-dm-sub-trigger",
                                    style: ITEM_STYLE,
                                    span { "Share" }
                                    span { style: "color: #64748b;", "▸" }
                                }
                                DropdownMenuPortal {
                                    DropdownMenuSubContent {
                                        id: "sw3b-dm-sub-content",
                                        style: "min-width: 170px; padding: 0.375rem; border-radius: 0.75rem; border: 1px solid #cbd5e1; background-color: #ffffff; color: #0f172a; box-shadow: 0 12px 28px rgba(15, 23, 42, 0.16); z-index: 70; display: flex; flex-direction: column; gap: 0.125rem;",
                                        DropdownMenuItem {
                                            id: "sw3b-dm-sub-email",
                                            style: ITEM_STYLE,
                                            on_select: move |_| last_action.set("share:email".to_string()),
                                            "Email Link"
                                        }
                                        DropdownMenuItem {
                                            id: "sw3b-dm-sub-slack",
                                            style: ITEM_STYLE,
                                            on_select: move |_| last_action.set("share:slack".to_string()),
                                            "Send to Slack"
                                        }
                                    }
                                }
                            }

                            DropdownMenuArrow {
                                id: "sw3b-dm-arrow",
                                width: 12.0,
                                height: 6.0,
                                style: "color: #cbd5e1;",
                            }
                        }
                    }
                }

                button {
                    id: "sw3b-dm-next-focus",
                    r#type: "button",
                    style: CTRL_BTN_STYLE,
                    onclick: move |_| {
                        let next = *outside_clicks.peek() + 1;
                        outside_clicks.set(next);
                    },
                    "Next Control ({outside_clicks()})"
                }
            }

            div {
                style: "display: flex; flex-wrap: wrap; align-items: center; gap: 0.5rem;",
                span {
                    id: "sw3b-dm-state-badge",
                    style: badge_style(is_open()),
                    if is_open() { "open" } else { "closed" }
                }
                span {
                    id: "sw3b-dm-last-action",
                    style: badge_style(last_action() != "none"),
                    "action={last_action()}"
                }
                span {
                    id: "sw3b-dm-tri-badge",
                    style: badge_style(tri_state() == CheckedState::Checked),
                    "tri={checked_label(tri_state())}"
                }
                span {
                    id: "sw3b-dm-group-badge",
                    style: badge_style(!cb_group().is_empty()),
                    "group={cb_group().join(\",\")}"
                }
                span {
                    id: "sw3b-dm-radio-badge",
                    style: badge_style(true),
                    "align={radio_val()}"
                }
                span {
                    id: "sw3b-dm-outside-count",
                    style: badge_style(outside_clicks() > 0),
                    "outside={outside_clicks()}"
                }
            }
        }
    }
}

#[component]
fn StaticAndAsChildDropdownScenario() -> Element {
    let mut static_action = use_signal(|| String::from("idle"));

    rsx! {
        div {
            style: CARD_STYLE,
            div {
                style: "display: flex; flex-direction: column; gap: 0.25rem;",
                h3 {
                    style: "margin: 0; font-size: 1rem; font-weight: 600; color: #0f172a;",
                    "2. Static Flow & as_child Composition (DropdownMenuContentStatic, SubContentStatic)"
                }
                p {
                    style: "margin: 0; font-size: 0.75rem; color: #64748b;",
                    "Non-modal static menu keeping inline flow positioning with custom as_child trigger/content projection."
                }
            }

            div {
                style: "display: flex; flex-direction: column; align-items: flex-start; gap: 0.75rem; padding: 1.25rem; border-radius: 0.75rem; border: 1px dashed #cbd5e1; background-color: #f8fafc;",
                DropdownMenuRoot {
                    modal: false,
                    DropdownMenuTrigger {
                        id: "sw3b-dm-static-trigger",
                        style: CTRL_BTN_STYLE,
                        child: move |props: Vec<Attribute>| rsx! {
                            button {
                                ..props,
                                "Static Menu Trigger (as_child)"
                            }
                        },
                        children: rsx! {},
                    }
                    DropdownMenuContentStatic {
                        id: "sw3b-dm-static-content",
                        style: "width: 240px; padding: 0.375rem; border-radius: 0.75rem; border: 1px solid #cbd5e1; background-color: #ffffff; color: #0f172a; box-shadow: 0 4px 12px rgba(15, 23, 42, 0.08); display: flex; flex-direction: column; gap: 0.125rem;",
                        on_open_auto_focus: move |evt: MenuPreventableEvent| evt.prevent_default(),
                        on_close_auto_focus: move |evt: MenuPreventableEvent| evt.prevent_default(),
                        child: move |props: Vec<Attribute>| rsx! {
                            div {
                                ..props,
                                DropdownMenuItem {
                                    id: "sw3b-dm-static-item-1",
                                    close_on_select: false,
                                    style: ITEM_STYLE,
                                    on_select: move |_| static_action.set("duplicate".to_string()),
                                    "Duplicate Inline"
                                }
                                DropdownMenuItem {
                                    id: "sw3b-dm-static-item-2",
                                    disabled: true,
                                    style: ITEM_STYLE,
                                    "Locked Action"
                                }
                                DropdownMenuSub {
                                    DropdownMenuSubTrigger {
                                        id: "sw3b-dm-static-sub-trig",
                                        style: ITEM_STYLE,
                                        span { "Nested Static" }
                                        span { style: "color: #64748b;", "▸" }
                                    }
                                    DropdownMenuSubContentStatic {
                                        id: "sw3b-dm-static-sub-content",
                                        style: "margin-left: 0.5rem; padding: 0.25rem; border-radius: 0.5rem; border: 1px solid #e2e8f0; background-color: #f8fafc;",
                                        DropdownMenuItem {
                                            id: "sw3b-dm-static-sub-item",
                                            close_on_select: false,
                                            style: ITEM_STYLE,
                                            on_select: move |_| static_action.set("nested-static".to_string()),
                                            "Static Sub Item"
                                        }
                                    }
                                }
                            }
                        },
                        children: rsx! {},
                    }
                }
                span {
                    id: "sw3b-dm-static-badge",
                    style: badge_style(static_action() != "idle"),
                    "static={static_action()}"
                }
            }
        }
    }
}

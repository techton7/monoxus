use dioxus::prelude::*;
use monoxus::{
    foundation::shared::{Direction, Orientation},
    toolbar::{
        Toolbar, ToolbarButton, ToolbarLink, ToolbarRoot, ToolbarSeparator, ToolbarToggleGroup,
        ToolbarToggleGroupMode, ToolbarToggleItem,
    },
};

const CARD_STYLE: &str = "border-radius: 1rem; border: 1px solid #e2e8f0; background-color: #ffffff; padding: 1.5rem; box-shadow: 0 1px 3px rgba(15, 23, 42, 0.06); display: flex; flex-direction: column; gap: 1rem;";
const TOOLBAR_ROW_STYLE: &str = "display: inline-flex; flex-wrap: wrap; align-items: center; gap: 0.375rem; padding: 0.375rem; border-radius: 0.75rem; border: 1px solid #cbd5e1; background-color: #f8fafc;";
const TOOLBAR_COL_STYLE: &str = "display: inline-flex; flex-direction: column; align-items: stretch; gap: 0.375rem; padding: 0.5rem; border-radius: 0.75rem; border: 1px solid #cbd5e1; background-color: #f8fafc; width: 11rem;";
const ITEM_BASE_STYLE: &str = "display: inline-flex; align-items: center; justify-content: center; gap: 0.375rem; padding: 0.375rem 0.75rem; border-radius: 0.5rem; border: 1px solid #cbd5e1; background-color: #ffffff; color: #1e293b; font-size: 0.8125rem; font-weight: 600; text-decoration: none; cursor: pointer;";

const TOOLBAR_PLAYGROUND_CSS: &str = r#"
[data-toolbar-item]:focus-visible {
    outline: 2px solid #7c3aed;
    outline-offset: 2px;
}
[data-toolbar-item]:hover:not([data-disabled]):not(:disabled) {
    background-color: #f5f3ff !important;
    border-color: #c4b5fd !important;
    color: #5b21b6 !important;
}
[data-toolbar-item][data-disabled],
[data-toolbar-item]:disabled {
    background-color: #f1f5f9 !important;
    color: #94a3b8 !important;
    border-color: #e2e8f0 !important;
    opacity: 0.6;
    cursor: not-allowed !important;
}
[data-toolbar-group-item][data-state="on"] {
    background-color: #7c3aed !important;
    color: #ffffff !important;
    border-color: #6d28d9 !important;
}
[data-toolbar-group-item][data-state="on"]:hover:not([data-disabled]) {
    background-color: #6d28d9 !important;
    color: #ffffff !important;
}
"#;

#[component]
pub fn ToolbarPlayground() -> Element {
    rsx! {
        div {
            class: "max-w-4xl mx-auto space-y-8",
            style: "max-width: 56rem; margin: 0 auto; display: flex; flex-direction: column; gap: 2rem;",
            style { "{TOOLBAR_PLAYGROUND_CSS}" }

            div {
                style: "display: flex; flex-direction: column; gap: 0.375rem;",
                h2 {
                    style: "margin: 0; font-size: 1.5rem; font-weight: 700; color: #0f172a; letter-spacing: -0.025em;",
                    "Toolbar"
                }
                p {
                    style: "margin: 0; font-size: 0.875rem; color: #64748b;",
                    "Composite toolbar with unified roving focus across heterogeneous buttons, links, and single/multiple toggle groups, separator skipping, vertical & RTL navigation, Space link activation, and nested toolbar isolation."
                }
            }

            MainToolbarScenario {}
            OrientationDirectionAndClampScenario {}
            NestedToolbarScenario {}
        }
    }
}

// ---------------------------------------------------------------------------
// Scenario 1: Horizontal Main Toolbar (SW3A-7, SW3A-10, SW3A-11)
// ---------------------------------------------------------------------------
#[component]
fn MainToolbarScenario() -> Element {
    let mut cut_clicks = use_signal(|| 0usize);
    let mut link_clicks = use_signal(|| 0usize);
    let mut align_val = use_signal(|| String::from("center"));
    let mut fmt_vals = use_signal(|| vec![String::from("bold")]);

    let fmt_summary = if fmt_vals().is_empty() {
        String::from("none")
    } else {
        fmt_vals().join(", ")
    };

    rsx! {
        div {
            style: CARD_STYLE,
            div {
                style: "display: flex; flex-direction: column; gap: 0.25rem;",
                h3 {
                    style: "margin: 0; font-size: 1rem; font-weight: 600; color: #0f172a;",
                    "1. Unified Roving Focus, Toggle Groups & Link Space Guard (SW3A-7, SW3A-10, SW3A-11)"
                }
                p {
                    style: "margin: 0; font-size: 0.75rem; color: #64748b;",
                    "ArrowLeft/Right and Home/End traverse enabled items across groups while skipping ToolbarSeparator and disabled buttons. Space activates ToolbarLink when focused directly, but is guarded when focused inside #sw3a-tb-nested-input (#3232)."
                }
            }

            ToolbarRoot {
                id: "sw3a-toolbar-main",
                orientation: Orientation::Horizontal,
                loop_focus: true,
                style: TOOLBAR_ROW_STYLE,
                ToolbarButton {
                    id: "sw3a-tb-btn-cut",
                    value: "cut".to_string(),
                    style: ITEM_BASE_STYLE,
                    on_click: move |_| cut_clicks += 1,
                    "Cut"
                }
                ToolbarButton {
                    id: "sw3a-tb-btn-disabled",
                    value: "copy-disabled".to_string(),
                    disabled: true,
                    style: ITEM_BASE_STYLE,
                    "Copy (Disabled)"
                }
                ToolbarSeparator {
                    id: "sw3a-tb-sep-1",
                    style: "width: 2px; height: 22px; background-color: #cbd5e1; margin: 0 0.25rem;",
                }
                ToolbarToggleGroup {
                    id: "sw3a-tb-tg-align",
                    r#type: ToolbarToggleGroupMode::Single,
                    default_value: "center".to_string(),
                    on_value_change: move |val| align_val.set(val),
                    style: "display: inline-flex; align-items: center; gap: 0.25rem;",
                    ToolbarToggleItem {
                        id: "sw3a-tb-align-left",
                        value: "left".to_string(),
                        style: ITEM_BASE_STYLE,
                        "Left"
                    }
                    ToolbarToggleItem {
                        id: "sw3a-tb-align-center",
                        value: "center".to_string(),
                        style: ITEM_BASE_STYLE,
                        "Center"
                    }
                    ToolbarToggleItem {
                        id: "sw3a-tb-align-right",
                        value: "right".to_string(),
                        style: ITEM_BASE_STYLE,
                        "Right"
                    }
                }
                ToolbarSeparator {
                    id: "sw3a-tb-sep-2",
                    style: "width: 2px; height: 22px; background-color: #cbd5e1; margin: 0 0.25rem;",
                }
                ToolbarToggleGroup {
                    id: "sw3a-tb-tg-fmt",
                    r#type: ToolbarToggleGroupMode::Multiple,
                    default_values: vec!["bold".to_string()],
                    on_values_change: move |vals| fmt_vals.set(vals),
                    style: "display: inline-flex; align-items: center; gap: 0.25rem;",
                    ToolbarToggleItem {
                        id: "sw3a-tb-fmt-bold",
                        value: "bold".to_string(),
                        style: ITEM_BASE_STYLE,
                        "Bold"
                    }
                    ToolbarToggleItem {
                        id: "sw3a-tb-fmt-italic",
                        value: "italic".to_string(),
                        style: ITEM_BASE_STYLE,
                        "Italic"
                    }
                }
                ToolbarSeparator {
                    id: "sw3a-tb-sep-3",
                    decorative: true,
                    style: "width: 2px; height: 22px; background-color: #cbd5e1; margin: 0 0.25rem;",
                }
                ToolbarLink {
                    id: "sw3a-tb-link",
                    href: "#sw3a-tb-link-target",
                    value: "share".to_string(),
                    style: "display: inline-flex; align-items: center; gap: 0.5rem; padding: 0.25rem 0.625rem; border-radius: 0.5rem; border: 1px solid #cbd5e1; background-color: #ffffff; color: #5b21b6; font-size: 0.8125rem; font-weight: 600; text-decoration: none; cursor: pointer;",
                    on_click: move |evt: MouseEvent| {
                        evt.prevent_default();
                        link_clicks += 1;
                    },
                    span { "Docs Link" }
                    input {
                        id: "sw3a-tb-nested-input",
                        r#type: "text",
                        "aria-label": "Nested input space guard",
                        placeholder: "Nested input",
                        style: "width: 6.5rem; padding: 0.125rem 0.375rem; border-radius: 0.25rem; border: 1px solid #94a3b8; font-size: 0.75rem; color: #0f172a; background-color: #f8fafc;",
                        onclick: move |evt: MouseEvent| evt.stop_propagation(),
                    }
                }
            }

            div {
                style: "display: flex; flex-wrap: wrap; align-items: center; gap: 1rem; font-size: 0.75rem; font-family: monospace; color: #475569;",
                span {
                    "Cut clicks: "
                    strong { id: "sw3a-tb-cut-clicks", style: "color: #0f172a;", "{cut_clicks()}" }
                }
                span {
                    "Align (Single): "
                    strong {
                        id: "sw3a-tb-align-value",
                        style: "color: #6d28d9;",
                        if align_val().is_empty() { "(none)" } else { "{align_val()}" }
                    }
                }
                span {
                    "Format (Multiple): "
                    strong { id: "sw3a-tb-fmt-value", style: "color: #6d28d9;", "{fmt_summary}" }
                }
                span {
                    "Link clicks: "
                    strong { id: "sw3a-tb-link-clicks", style: "color: #0f172a;", "{link_clicks()}" }
                }
            }
        }
    }
}

// ---------------------------------------------------------------------------
// Scenario 2: Vertical, Boundary Clamp (loop_focus: false) & RTL (SW3A-8, SW3A-9)
// ---------------------------------------------------------------------------
#[component]
fn OrientationDirectionAndClampScenario() -> Element {
    rsx! {
        div {
            style: CARD_STYLE,
            div {
                style: "display: flex; flex-direction: column; gap: 0.25rem;",
                h3 {
                    style: "margin: 0; font-size: 1rem; font-weight: 600; color: #0f172a;",
                    "2. Vertical Orientation, Loop Clamp (loop_focus: false) & RTL Direction (SW3A-8, SW3A-9)"
                }
                p {
                    style: "margin: 0; font-size: 0.75rem; color: #64748b;",
                    "Vertical toolbar navigates via ArrowUp/ArrowDown and inverts ToolbarSeparator to horizontal. Clamp toolbar stops at ends when loop_focus is false. RTL toolbar mirrors ArrowLeft/ArrowRight."
                }
            }

            div {
                style: "display: flex; flex-wrap: wrap; align-items: flex-start; gap: 2rem;",

                // Vertical Toolbar
                div {
                    style: "display: flex; flex-direction: column; gap: 0.5rem;",
                    span {
                        style: "font-size: 0.75rem; font-weight: 600; color: #334155;",
                        "Vertical (ArrowUp / ArrowDown)"
                    }
                    ToolbarRoot {
                        id: "sw3a-toolbar-vertical",
                        orientation: Orientation::Vertical,
                        loop_focus: true,
                        style: TOOLBAR_COL_STYLE,
                        ToolbarButton {
                            id: "sw3a-tb-vert-up",
                            value: "up".to_string(),
                            style: ITEM_BASE_STYLE,
                            "Move Up"
                        }
                        ToolbarSeparator {
                            id: "sw3a-tb-vert-sep",
                            style: "height: 2px; width: 100%; background-color: #cbd5e1; margin: 0.125rem 0;",
                        }
                        ToolbarButton {
                            id: "sw3a-tb-vert-mid",
                            value: "mid".to_string(),
                            style: ITEM_BASE_STYLE,
                            "Center"
                        }
                        ToolbarToggleGroup {
                            id: "sw3a-tb-vert-tg",
                            r#type: ToolbarToggleGroupMode::Single,
                            default_value: "grid".to_string(),
                            style: "display: flex; flex-direction: column; gap: 0.25rem;",
                            ToolbarToggleItem {
                                id: "sw3a-tb-vert-grid",
                                value: "grid".to_string(),
                                style: ITEM_BASE_STYLE,
                                "Grid"
                            }
                            ToolbarToggleItem {
                                id: "sw3a-tb-vert-list",
                                value: "list".to_string(),
                                style: ITEM_BASE_STYLE,
                                "List"
                            }
                        }
                    }
                }

                // Boundary Clamping & RTL Toolbars
                div {
                    style: "display: flex; flex-direction: column; gap: 1.25rem;",

                    div {
                        style: "display: flex; flex-direction: column; gap: 0.5rem;",
                        span {
                            style: "font-size: 0.75rem; font-weight: 600; color: #334155;",
                            "Boundary Clamping (loop_focus: false)"
                        }
                        Toolbar {
                            id: "sw3a-toolbar-clamp",
                            orientation: Orientation::Horizontal,
                            loop_focus: false,
                            style: TOOLBAR_ROW_STYLE,
                            ToolbarButton {
                                id: "sw3a-tb-clamp-first",
                                value: "first".to_string(),
                                style: ITEM_BASE_STYLE,
                                "First"
                            }
                            ToolbarButton {
                                id: "sw3a-tb-clamp-mid",
                                value: "mid".to_string(),
                                style: ITEM_BASE_STYLE,
                                "Middle"
                            }
                            ToolbarButton {
                                id: "sw3a-tb-clamp-last",
                                value: "last".to_string(),
                                style: ITEM_BASE_STYLE,
                                "Last"
                            }
                        }
                    }

                    div {
                        style: "display: flex; flex-direction: column; gap: 0.5rem;",
                        span {
                            style: "font-size: 0.75rem; font-weight: 600; color: #334155;",
                            "RTL Mirroring (dir: Direction::Rtl — ArrowLeft advances next)"
                        }
                        ToolbarRoot {
                            id: "sw3a-toolbar-rtl",
                            orientation: Orientation::Horizontal,
                            dir: Direction::Rtl,
                            loop_focus: true,
                            style: TOOLBAR_ROW_STYLE,
                            ToolbarButton {
                                id: "sw3a-tb-rtl-1",
                                value: "rtl-1".to_string(),
                                style: ITEM_BASE_STYLE,
                                "الأول (1)"
                            }
                            ToolbarSeparator {
                                id: "sw3a-tb-rtl-sep",
                                style: "width: 2px; height: 22px; background-color: #cbd5e1; margin: 0 0.25rem;",
                            }
                            ToolbarButton {
                                id: "sw3a-tb-rtl-2",
                                value: "rtl-2".to_string(),
                                style: ITEM_BASE_STYLE,
                                "الثاني (2)"
                            }
                            ToolbarLink {
                                id: "sw3a-tb-rtl-3",
                                href: "#sw3a-rtl-link",
                                value: "rtl-3".to_string(),
                                style: ITEM_BASE_STYLE,
                                on_click: move |evt: MouseEvent| evt.prevent_default(),
                                "الثالث (3)"
                            }
                        }
                    }
                }
            }
        }
    }
}

// ---------------------------------------------------------------------------
// Scenario 3: Nested Toolbar Isolation (SW3A-12)
// ---------------------------------------------------------------------------
#[component]
fn NestedToolbarScenario() -> Element {
    rsx! {
        div {
            style: CARD_STYLE,
            div {
                style: "display: flex; flex-direction: column; gap: 0.25rem;",
                h3 {
                    style: "margin: 0; font-size: 1rem; font-weight: 600; color: #0f172a;",
                    "3. Nested Toolbar Isolation (SW3A-12)"
                }
                p {
                    style: "margin: 0; font-size: 0.75rem; color: #64748b;",
                    "Outer toolbar arrow navigation skips the inner toolbar items completely; inner toolbar arrow navigation stays scoped to the inner toolbar."
                }
            }

            ToolbarRoot {
                id: "sw3a-toolbar-outer",
                orientation: Orientation::Horizontal,
                loop_focus: true,
                style: "display: inline-flex; flex-wrap: wrap; align-items: center; gap: 0.625rem; padding: 0.5rem 0.75rem; border-radius: 0.75rem; border: 1px solid #cbd5e1; background-color: #f8fafc;",
                ToolbarButton {
                    id: "sw3a-tb-outer-1",
                    value: "outer-1".to_string(),
                    style: ITEM_BASE_STYLE,
                    "Outer 1"
                }

                div {
                    style: "padding: 0.25rem; border-radius: 0.625rem; border: 1px dashed #7c3aed; background-color: #f5f3ff;",
                    ToolbarRoot {
                        id: "sw3a-toolbar-inner",
                        orientation: Orientation::Horizontal,
                        loop_focus: true,
                        style: "display: inline-flex; align-items: center; gap: 0.25rem;",
                        ToolbarButton {
                            id: "sw3a-tb-inner-1",
                            value: "inner-1".to_string(),
                            style: ITEM_BASE_STYLE,
                            "Inner 1"
                        }
                        ToolbarSeparator {
                            id: "sw3a-tb-inner-sep",
                            style: "width: 2px; height: 20px; background-color: #c4b5fd; margin: 0 0.125rem;",
                        }
                        ToolbarButton {
                            id: "sw3a-tb-inner-2",
                            value: "inner-2".to_string(),
                            style: ITEM_BASE_STYLE,
                            "Inner 2"
                        }
                    }
                }

                ToolbarButton {
                    id: "sw3a-tb-outer-2",
                    value: "outer-2".to_string(),
                    style: ITEM_BASE_STYLE,
                    "Outer 2"
                }
            }
        }
    }
}

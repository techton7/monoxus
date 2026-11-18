use dioxus::prelude::*;
use monoxus::{
    foundation::shared::{Direction, DirectionProvider},
    toggle_group::{ToggleGroupItem, ToggleGroupMode, ToggleGroupOrientation, ToggleGroupRoot},
};

#[component]
pub fn ToggleGroupPlayground() -> Element {
    let mut single_val = use_signal(|| String::from("center"));
    let mut multi_vals = use_signal(|| vec![String::from("bold")]);
    let mut vertical_val = use_signal(|| String::from("top"));
    let mut vertical_focus = use_signal(|| String::from("none"));
    let mut show_removable = use_signal(|| true);

    let multi_summary = if multi_vals().is_empty() {
        String::from("none")
    } else {
        multi_vals().join(", ")
    };

    let is_bold = multi_vals().contains(&"bold".to_string());
    let is_italic = multi_vals().contains(&"italic".to_string());
    let has_underline = multi_vals().contains(&"underline".to_string());
    let has_strike = multi_vals().contains(&"strike".to_string());
    let text_decoration = match (has_underline, has_strike) {
        (true, true) => "underline line-through",
        (true, false) => "underline",
        (false, true) => "line-through",
        (false, false) => "none",
    };

    rsx! {
        div {
            class: "max-w-4xl mx-auto space-y-8",

            div {
                class: "space-y-1.5",
                h2 {
                    class: "text-2xl font-bold tracking-tight text-slate-900",
                    "Toggle Group"
                }
                p {
                    class: "text-sm text-slate-500",
                    "A set of two-state buttons with Single (radiogroup/radio + aria-checked) vs Multiple (group + aria-pressed) ARIA pairing, roving tabindex, RTL/Vertical navigation, and unmount recovery."
                }
            }

            // SW1A-6: Single Mode (Text Alignment)
            div {
                class: "rounded-2xl border border-slate-200 bg-white p-6 shadow-sm space-y-5",
                div {
                    class: "space-y-1",
                    h3 {
                        class: "text-base font-semibold text-slate-900",
                        "1. Single Selection Mode (Alignment) (SW1A-6)"
                    }
                    p {
                        class: "text-xs text-slate-500",
                        "Single mode emits role=\"radiogroup\" + item role=\"radio\" & aria-checked. Re-clicking active item deselects to empty."
                    }
                }

                div {
                    class: "space-y-3",
                    ToggleGroupRoot {
                        id: "tg-single".to_string(),
                        r#type: ToggleGroupMode::Single,
                        default_value: "center".to_string(),
                        on_value_change: move |v| single_val.set(v),
                        class: "inline-flex items-center gap-1.5 rounded-lg bg-slate-100 p-1".to_string(),
                        ToggleGroupItem {
                            id: "tg-align-left".to_string(),
                            value: "left".to_string(),
                            "Left"
                        }
                        ToggleGroupItem {
                            id: "tg-align-center".to_string(),
                            value: "center".to_string(),
                            "Center"
                        }
                        ToggleGroupItem {
                            id: "tg-align-right".to_string(),
                            value: "right".to_string(),
                            "Right"
                        }
                    }

                    div {
                        id: "tg-align-preview",
                        style: format!(
                            "padding: 1rem 1.25rem; border-radius: 0.5rem; background-color: #f8fafc; border: 1px solid #e2e8f0; font-size: 0.9375rem; color: #0f172a; text-align: {}; transition: text-align 0.15s ease;",
                            match single_val().as_str() {
                                "left" => "left",
                                "center" => "center",
                                "right" => "right",
                                _ => "left",
                            }
                        ),
                        "Live Text Alignment Preview Card"
                    }

                    div {
                        class: "text-xs font-mono text-slate-500",
                        "Single active value: "
                        strong {
                            id: "tg-single-value",
                            if single_val().is_empty() { "(empty)" } else { "{single_val()}" }
                        }
                    }
                }
            }

            // SW1A-6: Multiple Mode (Formatting)
            div {
                class: "rounded-2xl border border-slate-200 bg-white p-6 shadow-sm space-y-5",
                div {
                    class: "space-y-1",
                    h3 {
                        class: "text-base font-semibold text-slate-900",
                        "2. Multiple Selection Mode (Text Formatting) (SW1A-6)"
                    }
                    p {
                        class: "text-xs text-slate-500",
                        "Multiple mode emits role=\"group\" + item aria-pressed. Any combination of items can be toggled on or off independently."
                    }
                }

                div {
                    class: "space-y-3",
                    ToggleGroupRoot {
                        id: "tg-multiple".to_string(),
                        r#type: ToggleGroupMode::Multiple,
                        default_values: vec!["bold".to_string()],
                        on_values_change: move |vals| multi_vals.set(vals),
                        class: "inline-flex items-center gap-1.5 rounded-lg bg-slate-100 p-1".to_string(),
                        ToggleGroupItem {
                            id: "tg-fmt-bold".to_string(),
                            value: "bold".to_string(),
                            "Bold"
                        }
                        ToggleGroupItem {
                            id: "tg-fmt-italic".to_string(),
                            value: "italic".to_string(),
                            "Italic"
                        }
                        ToggleGroupItem {
                            id: "tg-fmt-underline".to_string(),
                            value: "underline".to_string(),
                            "Underline"
                        }
                        ToggleGroupItem {
                            id: "tg-fmt-strike".to_string(),
                            value: "strike".to_string(),
                            "Strike"
                        }
                    }

                    div {
                        id: "tg-format-preview",
                        style: format!(
                            "padding: 1rem 1.25rem; border-radius: 0.5rem; background-color: #f8fafc; border: 1px solid #e2e8f0; font-size: 1rem; color: #0f172a; font-weight: {}; font-style: {}; text-decoration: {}; transition: all 0.15s ease;",
                            if is_bold { "700" } else { "400" },
                            if is_italic { "italic" } else { "normal" },
                            text_decoration
                        ),
                        "The quick brown fox jumps over the lazy dog."
                    }

                    div {
                        class: "text-xs font-mono text-slate-500",
                        "Multiple values: "
                        strong { id: "tg-multiple-value", "{multi_summary}" }
                    }
                }
            }

            // SW1A-7: Vertical Orientation & Boundary Clamping
            div {
                class: "rounded-2xl border border-slate-200 bg-white p-6 shadow-sm space-y-5",
                div {
                    class: "space-y-1",
                    h3 {
                        class: "text-base font-semibold text-slate-900",
                        "3. Vertical Orientation & Roving Focus Clamping (SW1A-7)"
                    }
                    p {
                        class: "text-xs text-slate-500",
                        "Vertical roving tabindex navigates via ArrowUp and ArrowDown. Boundary clamping (loop_focus: false) stops navigation at extremities."
                    }
                }

                div {
                    class: "space-y-3",
                    ToggleGroupRoot {
                        id: "tg-vertical".to_string(),
                        r#type: ToggleGroupMode::Single,
                        orientation: ToggleGroupOrientation::Vertical,
                        loop_focus: false,
                        default_value: "top".to_string(),
                        on_value_change: move |v| vertical_val.set(v),
                        class: "inline-flex flex-col items-stretch gap-1.5 rounded-lg bg-slate-100 p-1.5 sw1a-tg-vertical".to_string(),
                        ToggleGroupItem {
                            id: "tg-vert-top".to_string(),
                            value: "top".to_string(),
                            onfocus: move |_| vertical_focus.set(String::from("Top")),
                            "Top View"
                        }
                        ToggleGroupItem {
                            id: "tg-vert-mid".to_string(),
                            value: "mid".to_string(),
                            onfocus: move |_| vertical_focus.set(String::from("Middle")),
                            "Middle View"
                        }
                        ToggleGroupItem {
                            id: "tg-vert-bot".to_string(),
                            value: "bot".to_string(),
                            onfocus: move |_| vertical_focus.set(String::from("Bottom")),
                            "Bottom View"
                        }
                    }

                    div {
                        class: "flex items-center gap-3 text-xs font-mono text-slate-600",
                        span {
                            "Focused vertical item: "
                            strong { id: "tg-vertical-focused-item", "{vertical_focus()}" }
                        }
                        span {
                            "Selected: "
                            strong { id: "tg-vertical-selected", "{vertical_val()}" }
                        }
                    }
                }
            }

            // SW1A-7: Disabled Root & Disabled Items
            div {
                class: "rounded-2xl border border-slate-200 bg-white p-6 shadow-sm space-y-4",
                div {
                    class: "space-y-1",
                    h3 {
                        class: "text-base font-semibold text-slate-900",
                        "4. Disabled Root & Inactive Roving (SW1A-7)"
                    }
                    p {
                        class: "text-xs text-slate-500",
                        "A disabled ToggleGroupRoot applies data-disabled to the container and disables all child items."
                    }
                }

                ToggleGroupRoot {
                    id: "tg-disabled-root".to_string(),
                    r#type: ToggleGroupMode::Single,
                    disabled: true,
                    default_value: "a".to_string(),
                    class: "inline-flex items-center gap-1.5 rounded-lg bg-slate-100 p-1".to_string(),
                    ToggleGroupItem {
                        id: "tg-dis-a".to_string(),
                        value: "a".to_string(),
                        "Option A"
                    }
                    ToggleGroupItem {
                        id: "tg-dis-b".to_string(),
                        value: "b".to_string(),
                        "Option B"
                    }
                }
            }

            // SW1A-8: Dynamic Active Tab-Stop Unmount Recovery
            div {
                class: "rounded-2xl border border-slate-200 bg-white p-6 shadow-sm space-y-4",
                div {
                    class: "space-y-1",
                    h3 {
                        class: "text-base font-semibold text-slate-900",
                        "5. Dynamic Active Tab-Stop Unmount Recovery (SW1A-8)"
                    }
                    p {
                        class: "text-xs text-slate-500",
                        "When the item holding tabindex=\"0\" is removed from the DOM, use_drop unregisters it so the first remaining enabled item recovers tabindex=\"0\" automatically."
                    }
                }

                div {
                    class: "flex flex-wrap items-center gap-3",
                    button {
                        id: "tg-dynamic-toggle-remove",
                        r#type: "button",
                        class: "sw1a-btn-secondary text-xs",
                        onclick: move |_| {
                            let cur = *show_removable.peek();
                            show_removable.set(!cur);
                        },
                        if show_removable() { "Remove Active Item" } else { "Restore Removed Item" }
                    }
                    span {
                        class: "text-xs font-mono text-slate-500",
                        if show_removable() {
                            "Tab-stop anchor: #tg-dyn-removable"
                        } else {
                            "Tab-stop recovered: #tg-dyn-survivor-1"
                        }
                    }
                }

                ToggleGroupRoot {
                    id: "tg-dynamic".to_string(),
                    r#type: ToggleGroupMode::Single,
                    default_value: "removable".to_string(),
                    class: "inline-flex items-center gap-1.5 rounded-lg bg-slate-100 p-1".to_string(),
                    if show_removable() {
                        ToggleGroupItem {
                            id: "tg-dyn-removable".to_string(),
                            value: "removable".to_string(),
                            "Removable Active"
                        }
                    }
                    ToggleGroupItem {
                        id: "tg-dyn-survivor-1".to_string(),
                        value: "survivor-1".to_string(),
                        "Survivor 1"
                    }
                    ToggleGroupItem {
                        id: "tg-dyn-survivor-2".to_string(),
                        value: "survivor-2".to_string(),
                        "Survivor 2"
                    }
                }
            }

            // SW1A-7 Additional Configurations: RTL & Sequential Tab Order
            div {
                class: "rounded-2xl border border-slate-200 bg-white p-6 shadow-sm space-y-5",
                div {
                    class: "space-y-1",
                    h3 {
                        class: "text-base font-semibold text-slate-900",
                        "6. RTL Direction & Non-Roving Sequential Tab Order"
                    }
                    p {
                        class: "text-xs text-slate-500",
                        "DirectionProvider inverts horizontal arrow navigation; roving_focus: false preserves standard document tab order across all items."
                    }
                }

                div {
                    class: "space-y-2",
                    div { class: "text-xs font-semibold text-slate-600", "RTL Direction (ArrowLeft moves next)" }
                    DirectionProvider {
                        dir: Direction::Rtl,
                        ToggleGroupRoot {
                            id: "tg-rtl".to_string(),
                            r#type: ToggleGroupMode::Single,
                            default_value: "rtl-1".to_string(),
                            class: "inline-flex items-center gap-1.5 rounded-lg bg-slate-100 p-1".to_string(),
                            ToggleGroupItem { id: "tg-rtl-1".to_string(), value: "rtl-1".to_string(), "الأول (1)" }
                            ToggleGroupItem { id: "tg-rtl-2".to_string(), value: "rtl-2".to_string(), disabled: true, "معطل (2)" }
                            ToggleGroupItem { id: "tg-rtl-3".to_string(), value: "rtl-3".to_string(), "الثالث (3)" }
                        }
                    }
                }

                div {
                    class: "space-y-2 pt-2 border-t border-slate-100",
                    div { class: "text-xs font-semibold text-slate-600", "Sequential Tab Order (roving_focus: false)" }
                    ToggleGroupRoot {
                        id: "tg-no-roving".to_string(),
                        r#type: ToggleGroupMode::Multiple,
                        roving_focus: false,
                        class: "inline-flex items-center gap-1.5 rounded-lg bg-slate-100 p-1".to_string(),
                        ToggleGroupItem { id: "tg-nr-1".to_string(), value: "nr-1".to_string(), "Tab Stop A" }
                        ToggleGroupItem { id: "tg-nr-2".to_string(), value: "nr-2".to_string(), "Tab Stop B" }
                        ToggleGroupItem { id: "tg-nr-3".to_string(), value: "nr-3".to_string(), "Tab Stop C" }
                    }
                }
            }
        }
    }
}

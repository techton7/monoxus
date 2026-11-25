use dioxus::prelude::*;
use monoxus::combobox::{
    ComboboxContent, ComboboxContentStatic, ComboboxGroup, ComboboxGroupHeading, ComboboxInput,
    ComboboxItem, ComboboxItemData, ComboboxMode, ComboboxPortal, ComboboxRoot,
    ComboboxScrollAlignment, ComboboxScrollDownButton, ComboboxScrollUpButton, ComboboxTrigger,
    ComboboxViewport,
};

const CARD_STYLE: &str = "border-radius: 1rem; border: 1px solid #e2e8f0; background-color: #ffffff; padding: 1.5rem; box-shadow: 0 1px 3px rgba(15, 23, 42, 0.06); display: flex; flex-direction: column; gap: 1rem;";
const STAGE_STYLE: &str = "display: flex; flex-direction: column; gap: 0.75rem; padding: 1.25rem; border-radius: 0.75rem; border: 1px dashed #cbd5e1; background-color: #f8fafc;";
const CTRL_BTN_STYLE: &str = "display: inline-flex; align-items: center; gap: 0.375rem; padding: 0.45rem 0.75rem; border-radius: 0.5rem; border: 1px solid #cbd5e1; background-color: #ffffff; color: #334155; font-size: 0.75rem; font-weight: 600; cursor: pointer;";
const INPUT_ROW_STYLE: &str = "display: inline-flex; align-items: stretch; width: 100%; max-width: 320px; border-radius: 0.5rem; border: 1px solid #cbd5e1; background-color: #ffffff; overflow: hidden;";
const INPUT_STYLE: &str = "flex: 1; min-width: 0; padding: 0.5rem 0.75rem; border: none; font-size: 0.875rem; color: #0f172a; background-color: transparent; outline: none;";
const TRIGGER_STYLE: &str = "display: inline-flex; align-items: center; justify-content: center; padding: 0 0.75rem; border: none; border-left: 1px solid #e2e8f0; background-color: #f8fafc; color: #334155; font-size: 0.75rem; font-weight: 700; cursor: pointer;";
const CONTENT_STYLE: &str = "min-width: 260px; padding: 0.375rem; border-radius: 0.75rem; border: 1px solid #cbd5e1; background-color: #ffffff; color: #0f172a; box-shadow: 0 14px 34px rgba(15, 23, 42, 0.16); z-index: 60;";
const STATIC_CONTENT_STYLE: &str = "width: 100%; max-width: 320px; margin-top: 0.375rem; padding: 0.375rem; border-radius: 0.75rem; border: 1px solid #cbd5e1; background-color: #ffffff; color: #0f172a; box-shadow: 0 6px 18px rgba(15, 23, 42, 0.08);";
const GROUP_HEADING_STYLE: &str = "padding: 0.3rem 0.5rem 0.15rem; font-size: 0.6875rem; font-weight: 700; text-transform: uppercase; letter-spacing: 0.05em; color: #64748b;";
const SCROLL_BTN_STYLE: &str = "display: flex; align-items: center; justify-content: center; padding: 0.25rem; font-size: 0.75rem; font-weight: 700; color: #6d28d9; background-color: #f5f3ff; border-radius: 0.375rem; cursor: pointer; user-select: none;";

const COMBOBOX_PLAYGROUND_CSS: &str = r#"
[data-combobox-item] {
    display: flex;
    align-items: center;
    justify-content: space-between;
    gap: 0.5rem;
    padding: 0.425rem 0.625rem;
    border-radius: 0.375rem;
    font-size: 0.8125rem;
    font-weight: 500;
    color: #1e293b;
    cursor: pointer;
    user-select: none;
}
[data-combobox-item][data-highlighted] {
    background-color: #ede9fe;
    color: #5b21b6;
}
[data-combobox-item][data-selected] {
    font-weight: 700;
}
[data-combobox-item][data-disabled] {
    color: #94a3b8;
    opacity: 0.55;
    cursor: not-allowed;
}
[data-combobox-input]:focus-visible {
    box-shadow: inset 0 -2px 0 #7c3aed;
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
pub fn ComboboxShowcase() -> Element {
    rsx! {
        div {
            class: "max-w-4xl mx-auto space-y-8",
            style: "max-width: 56rem; margin: 0 auto; display: flex; flex-direction: column; gap: 2rem;",
            style { "{COMBOBOX_PLAYGROUND_CSS}" }

            div {
                style: "display: flex; flex-direction: column; gap: 0.375rem;",
                h2 {
                    style: "margin: 0; font-size: 1.5rem; font-weight: 700; color: #0f172a; letter-spacing: -0.025em;",
                    "Combobox"
                }
                p {
                    style: "margin: 0; font-size: 0.875rem; color: #64748b;",
                    "Editable filterable combobox supporting single and multiple selection, non-blurring trigger/item interaction, virtual focus (aria-activedescendant), form reset sync, and scrollable viewports with PageUp/PageDown."
                }
            }

            SingleComboboxScenario {}
            MultipleComboboxScenario {}
            ScrollableComboboxScenario {}
        }
    }
}

#[component]
fn SingleComboboxScenario() -> Element {
    let options = [
        ("dioxus", "Dioxus", false),
        ("leptos", "Leptos", false),
        ("yew", "Yew", false),
        ("sycamore", "Sycamore", true),
        ("egui", "egui", false),
        ("iced", "Iced", false),
    ];

    let items_data: Vec<ComboboxItemData> = options
        .iter()
        .map(|(v, l, d)| ComboboxItemData {
            value: (*v).to_string(),
            label: (*l).to_string(),
            disabled: *d,
        })
        .collect();

    let mut selected = use_signal(|| Some("dioxus".to_string()));
    let mut input_val = use_signal(|| "Dioxus".to_string());
    let mut open = use_signal(|| false);

    let query = input_val().trim().to_lowercase();
    let selected_label = selected()
        .as_deref()
        .and_then(|val| options.iter().find(|(v, _, _)| *v == val).map(|(_, l, _)| *l))
        .unwrap_or("");
    let show_all = query.is_empty() || query == selected_label.to_lowercase();

    let filtered: Vec<(&str, &str, bool)> = options
        .iter()
        .copied()
        .filter(|(v, l, _)| {
            show_all || l.to_lowercase().contains(&query) || v.to_lowercase().contains(&query)
        })
        .collect();

    let value_readout = selected().unwrap_or_else(|| "none".to_string());

    rsx! {
        section {
            id: "sw4b-cbx-single-card",
            style: "{CARD_STYLE}",
            div {
                h3 {
                    style: "margin: 0; font-size: 1.0625rem; font-weight: 700; color: #0f172a;",
                    "1. Single-Select Combobox + Form Reset (SW4B-5, SW4B-7, SW4B-9)"
                }
                p {
                    style: "margin: 0.2rem 0 0; font-size: 0.8125rem; color: #64748b;",
                    "Consumer-filtered framework list with disabled option (aria-disabled=\"true\"), allow_deselect, clear_on_deselect, HiddenFormInput, and ancestor <form> reset sync."
                }
            }

            div {
                style: "display: flex; flex-wrap: wrap; gap: 0.5rem;",
                span {
                    id: "sw4b-cbx-single-value",
                    style: "{badge_style(selected().is_some())}",
                    "value: {value_readout}"
                }
                span {
                    id: "sw4b-cbx-single-open",
                    style: "{badge_style(open())}",
                    if open() { "state: open" } else { "state: closed" }
                }
            }

            div {
                style: "{STAGE_STYLE}",
                form {
                    id: "sw4b-cbx-single-form",
                    style: "display: flex; flex-wrap: wrap; align-items: center; gap: 0.75rem;",
                    onsubmit: move |evt| evt.prevent_default(),

                    ComboboxRoot {
                        id: Some("sw4b-cbx-single".to_string()),
                        mode: ComboboxMode::Single,
                        value: Some(selected),
                        default_value: Some("dioxus".to_string()),
                        on_value_change: move |v: Option<String>| selected.set(v),
                        input_value: Some(input_val),
                        default_input_value: Some("Dioxus".to_string()),
                        on_input_value_change: move |iv: String| input_val.set(iv),
                        open: Some(open),
                        on_open_change: move |next: bool| open.set(next),
                        name: Some("framework".to_string()),
                        items: items_data,
                        allow_deselect: true,
                        clear_on_deselect: true,

                        div {
                            style: "{INPUT_ROW_STYLE}",
                            ComboboxInput {
                                id: Some("sw4b-cbx-single-input".to_string()),
                                clear_on_deselect: true,
                                placeholder: Some("Select a Rust UI framework...".to_string()),
                                style: Some(INPUT_STYLE.to_string()),
                            }
                            ComboboxTrigger {
                                id: Some("sw4b-cbx-single-trigger".to_string()),
                                style: Some(TRIGGER_STYLE.to_string()),
                                "v"
                            }
                        }

                        ComboboxPortal {
                            ComboboxContent {
                                id: Some("sw4b-cbx-single-content".to_string()),
                                same_width: true,
                                side_offset: 6.0,
                                style: Some(CONTENT_STYLE.to_string()),
                                ComboboxViewport {
                                    id: Some("sw4b-cbx-single-viewport".to_string()),
                                    style: Some("max-height: 200px; overflow-y: auto;".to_string()),
                                    ComboboxGroup {
                                        ComboboxGroupHeading {
                                            style: Some(GROUP_HEADING_STYLE.to_string()),
                                            "Rust UI Frameworks"
                                        }
                                        if filtered.is_empty() {
                                            div {
                                                style: "padding: 0.75rem; text-align: center; font-size: 0.8125rem; color: #64748b;",
                                                "No frameworks found."
                                            }
                                        }
                                        for (val, label, disabled) in filtered {
                                            ComboboxItem {
                                                key: "{val}",
                                                id: Some(format!("sw4b-cbx-single-item-{val}")),
                                                value: val.to_string(),
                                                label: Some(label.to_string()),
                                                disabled: disabled,
                                                span { "{label}" }
                                                if selected().as_deref() == Some(val) {
                                                    span { style: "color: #7c3aed;", "✓" }
                                                } else if disabled {
                                                    span { style: "font-size: 0.7rem; color: #94a3b8;", "disabled" }
                                                }
                                            }
                                        }
                                    }
                                }
                            }
                        }
                    }

                    button {
                        id: "sw4b-cbx-single-reset",
                        r#type: "reset",
                        style: "{CTRL_BTN_STYLE}",
                        "Reset Form"
                    }
                }
            }
        }
    }
}

#[component]
fn MultipleComboboxScenario() -> Element {
    let fruits = [
        ("apple", "Apple"),
        ("banana", "Banana"),
        ("cherry", "Cherry"),
        ("mango", "Mango"),
        ("peach", "Peach"),
        ("strawberry", "Strawberry"),
    ];

    let items_data: Vec<ComboboxItemData> = fruits
        .iter()
        .map(|(v, l)| ComboboxItemData {
            value: (*v).to_string(),
            label: (*l).to_string(),
            disabled: false,
        })
        .collect();

    let mut selected_values = use_signal(|| vec!["apple".to_string(), "mango".to_string()]);
    let mut input_val = use_signal(String::new);
    let mut open = use_signal(|| false);

    let query = input_val().trim().to_lowercase();
    let filtered: Vec<(&str, &str)> = fruits
        .iter()
        .copied()
        .filter(|(v, l)| {
            query.is_empty()
                || l.to_lowercase().contains(&query)
                || v.to_lowercase().contains(&query)
        })
        .collect();

    let current_vals = selected_values();

    rsx! {
        section {
            id: "sw4b-cbx-multi-card",
            style: "{CARD_STYLE}",
            div {
                h3 {
                    style: "margin: 0; font-size: 1.0625rem; font-weight: 700; color: #0f172a;",
                    "2. Multiple-Select Combobox (SW4B-6)"
                }
                p {
                    style: "margin: 0.2rem 0 0; font-size: 0.8125rem; color: #64748b;",
                    "ComboboxMode::Multiple with aria-multiselectable=\"true\", persistent open menu across selections, selected pills, and ComboboxContentStatic."
                }
            }

            div {
                id: "sw4b-cbx-multi-pills",
                style: "display: flex; flex-wrap: wrap; gap: 0.375rem;",
                if current_vals.is_empty() {
                    span {
                        style: "{badge_style(false)}",
                        "selected: none"
                    }
                } else {
                    for val in current_vals.iter() {
                        span {
                            key: "{val}",
                            style: "{badge_style(true)}",
                            "{val}"
                        }
                    }
                }
            }

            div {
                style: "{STAGE_STYLE}",
                ComboboxRoot {
                    id: Some("sw4b-cbx-multi".to_string()),
                    mode: ComboboxMode::Multiple,
                    values: Some(selected_values),
                    on_values_change: move |vals: Vec<String>| selected_values.set(vals),
                    input_value: Some(input_val),
                    on_input_value_change: move |iv: String| input_val.set(iv),
                    open: Some(open),
                    on_open_change: move |next: bool| open.set(next),
                    name: Some("fruits".to_string()),
                    items: items_data,

                    div {
                        style: "{INPUT_ROW_STYLE}",
                        ComboboxInput {
                            id: Some("sw4b-cbx-multi-input".to_string()),
                            placeholder: Some("Filter fruits (multi-select)...".to_string()),
                            style: Some(INPUT_STYLE.to_string()),
                        }
                        ComboboxTrigger {
                            id: Some("sw4b-cbx-multi-trigger".to_string()),
                            style: Some(TRIGGER_STYLE.to_string()),
                            "v"
                        }
                    }

                    ComboboxContentStatic {
                        id: Some("sw4b-cbx-multi-content".to_string()),
                        style: Some(STATIC_CONTENT_STYLE.to_string()),
                        ComboboxViewport {
                            id: Some("sw4b-cbx-multi-viewport".to_string()),
                            style: Some("max-height: 180px; overflow-y: auto;".to_string()),
                            for (val, label) in filtered {
                                ComboboxItem {
                                    key: "{val}",
                                    id: Some(format!("sw4b-cbx-multi-item-{val}")),
                                    value: val.to_string(),
                                    label: Some(label.to_string()),
                                    span { "{label}" }
                                    if current_vals.iter().any(|v| v == val) {
                                        span { style: "color: #7c3aed;", "✓" }
                                    }
                                }
                            }
                        }
                    }
                }
            }
        }
    }
}

#[component]
fn ScrollableComboboxScenario() -> Element {
    let mut selected = use_signal(|| None::<String>);
    let mut input_val = use_signal(String::new);
    let mut open = use_signal(|| false);
    let mut alignment = use_signal(|| ComboboxScrollAlignment::Nearest);

    let all_items: Vec<(String, String)> = (1..=24)
        .map(|i| (format!("item-{i:02}"), format!("Option {i:02} — Scrollable Entry")))
        .collect();

    let items_data: Vec<ComboboxItemData> = all_items
        .iter()
        .map(|(v, l)| ComboboxItemData {
            value: v.clone(),
            label: l.clone(),
            disabled: false,
        })
        .collect();

    let query = input_val().trim().to_lowercase();
    let filtered: Vec<(String, String)> = all_items
        .into_iter()
        .filter(|(v, l)| {
            query.is_empty()
                || l.to_lowercase().contains(&query)
                || v.to_lowercase().contains(&query)
        })
        .collect();

    let align_label = match alignment() {
        ComboboxScrollAlignment::Nearest => "Nearest",
        ComboboxScrollAlignment::Center => "Center",
    };
    let selected_readout = selected().unwrap_or_else(|| "none".to_string());

    rsx! {
        section {
            id: "sw4b-cbx-scroll-card",
            style: "{CARD_STYLE}",
            div {
                style: "display: flex; align-items: center; justify-content: space-between; flex-wrap: wrap; gap: 0.5rem;",
                div {
                    h3 {
                        style: "margin: 0; font-size: 1.0625rem; font-weight: 700; color: #0f172a;",
                        "3. Long-List Scrollable Combobox (SW4B-8)"
                    }
                    p {
                        style: "margin: 0.2rem 0 0; font-size: 0.8125rem; color: #64748b;",
                        "24 items in a 180px ComboboxViewport with ComboboxScrollUpButton/ComboboxScrollDownButton, PageDown/PageUp (±10), and scroll_alignment toggle."
                    }
                }
                button {
                    id: "sw4b-cbx-scroll-align-toggle",
                    r#type: "button",
                    style: "{CTRL_BTN_STYLE}",
                    onclick: move |_| {
                        let next = match alignment() {
                            ComboboxScrollAlignment::Nearest => ComboboxScrollAlignment::Center,
                            ComboboxScrollAlignment::Center => ComboboxScrollAlignment::Nearest,
                        };
                        alignment.set(next);
                    },
                    "scroll_alignment: {align_label}"
                }
            }

            div {
                style: "display: flex; flex-wrap: wrap; gap: 0.5rem;",
                span {
                    id: "sw4b-cbx-scroll-value",
                    style: "{badge_style(selected().is_some())}",
                    "value: {selected_readout}"
                }
            }

            div {
                style: "{STAGE_STYLE}",
                ComboboxRoot {
                    id: Some("sw4b-cbx-scroll".to_string()),
                    mode: ComboboxMode::Single,
                    value: Some(selected),
                    on_value_change: move |v: Option<String>| selected.set(v),
                    input_value: Some(input_val),
                    on_input_value_change: move |iv: String| input_val.set(iv),
                    open: Some(open),
                    on_open_change: move |next: bool| open.set(next),
                    loop_focus: true,
                    scroll_alignment: alignment(),
                    items: items_data,

                    div {
                        style: "{INPUT_ROW_STYLE}",
                        ComboboxInput {
                            id: Some("sw4b-cbx-scroll-input".to_string()),
                            placeholder: Some("Type or press PageDown / PageUp...".to_string()),
                            style: Some(INPUT_STYLE.to_string()),
                        }
                        ComboboxTrigger {
                            id: Some("sw4b-cbx-scroll-trigger".to_string()),
                            style: Some(TRIGGER_STYLE.to_string()),
                            "v"
                        }
                    }

                    ComboboxPortal {
                        ComboboxContent {
                            id: Some("sw4b-cbx-scroll-content".to_string()),
                            same_width: true,
                            side_offset: 6.0,
                            style: Some(CONTENT_STYLE.to_string()),
                            ComboboxScrollUpButton {
                                style: Some(SCROLL_BTN_STYLE.to_string()),
                                "▲ Scroll Up"
                            }
                            ComboboxViewport {
                                id: Some("sw4b-cbx-scroll-viewport".to_string()),
                                style: Some("max-height: 180px; overflow-y: auto;".to_string()),
                                for (val, label) in filtered {
                                    ComboboxItem {
                                        key: "{val}",
                                        id: Some(format!("sw4b-cbx-scroll-{val}")),
                                        value: val.clone(),
                                        label: Some(label.clone()),
                                        span { "{label}" }
                                        if selected().as_deref() == Some(val.as_str()) {
                                            span { style: "color: #7c3aed;", "✓" }
                                        }
                                    }
                                }
                            }
                            ComboboxScrollDownButton {
                                style: Some(SCROLL_BTN_STYLE.to_string()),
                                "▼ Scroll Down"
                            }
                        }
                    }
                }
            }
        }
    }
}

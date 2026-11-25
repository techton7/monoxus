use dioxus::prelude::*;
use monoxus::{
    calendar::CalendarSnapshot,
    date_field::DateFieldSegmentItem,
    date_picker::*,
    foundation::date_time::{CalendarDate, CalendarLocale, DateValue},
    select::{
        SelectContent, SelectItem, SelectItemIndicator, SelectItemText, SelectPortal, SelectRoot,
        SelectTrigger, SelectValue, SelectViewport,
    },
};

const DATE_PICKER_PLAYGROUND_CSS: &str = r#"
[data-date-field-input] {
    display: inline-flex;
    align-items: center;
    gap: 2px;
    min-height: 38px;
    padding: 0 8px 0 12px;
    border-radius: 0.5rem;
    border: 1px solid #cbd5e1;
    background-color: #ffffff;
    color: #0f172a;
    font-size: 0.875rem;
    font-family: ui-monospace, SFMono-Regular, Menlo, monospace;
}
[data-date-field-label] {
    font-size: 0.8125rem;
    font-weight: 600;
    color: #334155;
    cursor: pointer;
    user-select: none;
}
[data-date-field-segment] {
    padding: 2px 3px;
    border-radius: 0.25rem;
    outline: none;
    user-select: none;
}
[data-date-field-segment][data-placeholder] {
    color: #94a3b8;
}
[data-date-field-segment]:focus {
    background-color: #0f172a;
    color: #ffffff;
}
[data-segment="trigger"] {
    display: inline-flex;
    align-items: center;
    justify-content: center;
    height: 28px;
    padding: 0 8px;
    margin-left: 6px;
    border-radius: 0.375rem;
    border: 1px solid #cbd5e1;
    background-color: #f8fafc;
    color: #334155;
    font-size: 0.75rem;
    font-weight: 600;
    font-family: inherit;
    cursor: pointer;
}
[data-segment="trigger"]:focus-visible {
    outline: 2px solid #7c3aed;
    outline-offset: 1px;
}
[data-date-picker-content] {
    z-index: 40;
    border-radius: 0.75rem;
    border: 1px solid #cbd5e1;
    background-color: #ffffff;
    box-shadow: 0 10px 25px -5px rgba(15, 23, 42, 0.14);
    padding: 1rem;
    outline: none;
}
[data-calendar-day] {
    width: 36px;
    height: 36px;
    display: flex;
    align-items: center;
    justify-content: center;
    border-radius: 0.5rem;
    font-size: 0.875rem;
    font-weight: 500;
    color: #0f172a;
    background-color: transparent;
    cursor: pointer;
    user-select: none;
    box-sizing: border-box;
    border: 1px solid transparent;
}
[data-calendar-day]:hover:not([data-disabled]):not([data-selected]) {
    background-color: #f1f5f9;
}
[data-calendar-day]:focus-visible {
    outline: 2px solid #7c3aed;
    outline-offset: 1px;
}
[data-calendar-day][data-outside-month] {
    color: #94a3b8;
}
[data-calendar-day][data-selected] {
    background-color: #0f172a;
    color: #ffffff;
    font-weight: 600;
}
[data-calendar-day][data-disabled] {
    color: #cbd5e1;
    opacity: 0.65;
    cursor: not-allowed;
}
[data-calendar-prev-button],
[data-calendar-next-button] {
    display: inline-flex;
    align-items: center;
    justify-content: center;
    height: 30px;
    min-width: 30px;
    padding: 0 8px;
    border-radius: 0.5rem;
    border: 1px solid #cbd5e1;
    background-color: #ffffff;
    color: #334155;
    font-size: 0.8125rem;
    cursor: pointer;
}
.dp-select-item {
    display: flex;
    align-items: center;
    justify-content: space-between;
    gap: 0.5rem;
    padding: 0.375rem 0.5rem;
    border-radius: 0.375rem;
    font-size: 0.8125rem;
    color: #0f172a;
    cursor: pointer;
    user-select: none;
}
.dp-select-item[data-highlighted="true"] {
    background-color: #f1f5f9;
}
.dp-select-item[data-selected="true"] {
    font-weight: 600;
}
"#;

const CARD_STYLE: &str = "border-radius: 1rem; border: 1px solid #e2e8f0; background-color: #ffffff; padding: 1.5rem; display: flex; flex-direction: column; gap: 1rem;";
const HEADER_STYLE: &str = "display: flex; align-items: center; justify-content: space-between; gap: 0.5rem; flex-wrap: wrap; margin-bottom: 0.5rem;";
const TABLE_STYLE: &str = "border-collapse: collapse; table-layout: fixed; width: 252px;";
const TH_STYLE: &str = "width: 36px; height: 28px; text-align: center; font-size: 0.75rem; font-weight: 600; color: #64748b; padding: 0;";
const TD_STYLE: &str = "width: 36px; height: 36px; padding: 2px; text-align: center;";
const BADGE_STYLE: &str = "display: inline-flex; align-items: center; border-radius: 0.25rem; padding: 0.125rem 0.5rem; background-color: #f1f5f9; color: #334155; font-size: 0.75rem; font-weight: 700; font-family: monospace;";

fn dp_date(year: i32, month: u8, day: u8) -> DateValue {
    DateValue::Date(CalendarDate::new(year, month, day).expect("valid playground date"))
}

fn render_picker_select(slot: DatePickerSelectRenderProps, min_width: &str) -> Element {
    let current_val = slot.value_string.clone();
    let select_items = slot.select_items();
    let items = slot.items.clone();
    let trigger_style = format!(
        "display: inline-flex; align-items: center; justify-content: space-between; gap: 0.375rem; height: 30px; padding: 0 8px; border-radius: 0.5rem; border: 1px solid #cbd5e1; background-color: #ffffff; color: #0f172a; font-size: 0.8125rem; font-weight: 500; cursor: pointer; {min_width}"
    );
    rsx! {
        SelectRoot {
            id: slot.attrs.id.clone(),
            items: Some(select_items),
            value: Some(current_val.clone()),
            disabled: slot.disabled,
            on_value_change: slot.on_value_change,
            SelectTrigger {
                class: "dp-select-trigger".to_string(),
                style: trigger_style,
                SelectValue {}
                span { style: "font-size: 0.6875rem; color: #64748b;", "▾" }
            }
            SelectPortal {
                SelectContent {
                    class: "dp-select-content".to_string(),
                    style: "z-index: 60; background-color: #ffffff; border: 1px solid #cbd5e1; border-radius: 0.5rem; box-shadow: 0 10px 15px -3px rgba(15, 23, 42, 0.12); padding: 0.25rem; outline: none;".to_string(),
                    SelectViewport {
                        style: "max-height: 220px;".to_string(),
                        for opt in items {
                            {
                                let opt_val = opt.value_string();
                                let is_selected = opt_val == current_val;
                                rsx! {
                                    SelectItem {
                                        key: "{opt.value}",
                                        value: opt_val,
                                        text: opt.label.clone(),
                                        class: "dp-select-item".to_string(),
                                        SelectItemText { "{opt.label}" }
                                        SelectItemIndicator {
                                            if is_selected {
                                                span { style: "color: #0f172a; font-weight: 700; margin-left: 0.5rem;", "✓" }
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
    }
}

#[component]
pub fn DatePickerShowcase() -> Element {
    rsx! {
        style { "{DATE_PICKER_PLAYGROUND_CSS}" }
        div {
            class: "max-w-4xl mx-auto space-y-8",
            style: "max-width: 56rem; margin: 0 auto; display: flex; flex-direction: column; gap: 2rem;",

            div {
                class: "space-y-1.5",
                style: "display: flex; flex-direction: column; gap: 0.375rem;",
                h2 {
                    class: "text-2xl font-bold tracking-tight text-slate-900",
                    style: "margin: 0; font-size: 1.5rem; font-weight: 700; color: #0f172a;",
                    "DatePicker"
                }
                p {
                    class: "text-sm text-slate-500",
                    style: "margin: 0; font-size: 0.875rem; color: #64748b;",
                    "Anchored DateField + Popover + Calendar composition with pickerOpenFocus, close_on_date_select, nested monoxus::Select Month/Year dropdowns (OVL-1/OVL-2), and multi-picker stable ID isolation."
                }
            }

            StandardDatePickerScenario {}
            MultiPickerIsolationScenario {}
        }
    }
}

#[component]
fn StandardDatePickerScenario() -> Element {
    let value = use_signal(|| Some(dp_date(2024, 6, 15)));
    let open = use_signal(|| false);

    rsx! {
        div {
            class: "rounded-2xl border border-slate-200 bg-white p-6 shadow-sm space-y-4",
            style: "{CARD_STYLE}",

            div {
                style: "display: flex; flex-direction: column; gap: 0.25rem;",
                h3 {
                    style: "margin: 0; font-size: 1rem; font-weight: 600; color: #0f172a;",
                    "1. Standard DatePicker with Nested Month/Year Select (as_child) (SW5C-4, SW5C-5)"
                }
                p {
                    style: "margin: 0; font-size: 0.75rem; color: #64748b;",
                    "Open popover to verify focus lands on the focused calendar day. Selecting a Month/Year via nested monoxus::Select keeps the picker open (OVL-1); Escape closes Select first, then DatePicker (OVL-2)."
                }
            }

            div {
                style: "display: flex; flex-wrap: wrap; align-items: flex-end; gap: 1.5rem;",
                DatePickerRoot {
                    id: "sw5c-dp-main",
                    value: Some(value),
                    default_placeholder: Some(dp_date(2024, 6, 15)),
                    open: Some(open),
                    close_on_date_select: true,
                    locale: CalendarLocale::EnUs,
                    div {
                        style: "display: flex; flex-direction: column; gap: 0.375rem;",
                        DatePickerLabel {
                            id: "sw5c-dp-main-label",
                            "Schedule Date"
                        }
                        DatePickerInput {
                            id: "sw5c-dp-main-input",
                            name: "schedule_date",
                            render_children: move |segments: Vec<DateFieldSegmentItem>| rsx! {
                                for seg in segments {
                                    DatePickerSegment {
                                        key: "{seg.index}",
                                        part: seg.kind(),
                                        index: Some(seg.index),
                                        id: format!("sw5c-dp-main-seg-{}", seg.index),
                                    }
                                }
                                DatePickerTrigger {
                                    id: "sw5c-dp-main-trigger",
                                    "Pick"
                                }
                            }
                        }
                    }
                    DatePickerPortal {
                        DatePickerContent {
                            id: "sw5c-dp-main-content",
                            DatePickerCalendar {
                                id: "sw5c-dp-main-calendar",
                                render_children: move |snap: CalendarSnapshot| rsx! {
                                    DatePickerHeader {
                                        style: "{HEADER_STYLE}",
                                        DatePickerPrevButton { id: "sw5c-dp-main-prev", "‹" }
                                        div {
                                            style: "display: flex; align-items: center; gap: 0.375rem;",
                                            DatePickerMonthSelect {
                                                id: "sw5c-dp-main-month-select",
                                                as_child: true,
                                                render_child: move |slot: DatePickerSelectRenderProps| {
                                                    render_picker_select(slot, "min-width: 112px;")
                                                },
                                            }
                                            DatePickerYearSelect {
                                                id: "sw5c-dp-main-year-select",
                                                as_child: true,
                                                render_child: move |slot: DatePickerSelectRenderProps| {
                                                    render_picker_select(slot, "min-width: 82px;")
                                                },
                                            }
                                        }
                                        DatePickerNextButton { id: "sw5c-dp-main-next", "›" }
                                    }
                                    DatePickerHeading {
                                        id: "sw5c-dp-main-heading",
                                        style: "text-align: center; font-size: 0.875rem; font-weight: 600; color: #0f172a; margin-bottom: 0.5rem;",
                                    }
                                    DatePickerGrid {
                                        id: "sw5c-dp-main-grid",
                                        style: "{TABLE_STYLE}",
                                        DatePickerGridHead {
                                            DatePickerGridRow {
                                                for (w_idx, wd) in snap.weekdays.iter().enumerate() {
                                                    DatePickerHeadCell { key: "{w_idx}", style: "{TH_STYLE}", "{wd}" }
                                                }
                                            }
                                        }
                                        DatePickerGridBody {
                                            for (r_idx, week) in snap.months[0].weeks.iter().enumerate() {
                                                DatePickerGridRow {
                                                    key: "{r_idx}",
                                                    for date in week.iter().copied() {
                                                        DatePickerCell {
                                                            key: "{date}",
                                                            date,
                                                            month: snap.months[0].value,
                                                            style: "{TD_STYLE}",
                                                            DatePickerDay {
                                                                id: "sw5c-dp-main-day-{date}",
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
                    }
                }

                div {
                    style: "display: flex; flex-wrap: wrap; gap: 1rem; font-size: 0.75rem;",
                    div {
                        style: "display: flex; align-items: center; gap: 0.5rem;",
                        span { style: "color: #64748b; font-weight: 600;", "Value:" }
                        span {
                            id: "sw5c-dp-main-value",
                            style: "{BADGE_STYLE}",
                            {value().map_or_else(|| String::from("none"), |v| v.to_iso_string())}
                        }
                    }
                    div {
                        style: "display: flex; align-items: center; gap: 0.5rem;",
                        span { style: "color: #64748b; font-weight: 600;", "Open:" }
                        span {
                            id: "sw5c-dp-main-open",
                            style: "{BADGE_STYLE}",
                            "{open}"
                        }
                    }
                }
            }
        }
    }
}

#[component]
fn MultiPickerIsolationScenario() -> Element {
    let left_val = use_signal(|| Some(dp_date(2024, 7, 4)));
    let right_val = use_signal(|| Some(dp_date(2024, 8, 12)));

    rsx! {
        div {
            class: "rounded-2xl border border-slate-200 bg-white p-6 shadow-sm space-y-4",
            style: "{CARD_STYLE}",

            div {
                style: "display: flex; flex-direction: column; gap: 0.25rem;",
                h3 {
                    style: "margin: 0; font-size: 1rem; font-weight: 600; color: #0f172a;",
                    "2. Multi-Picker Stable ID Isolation (id: None) (SW5C-7)"
                }
                p {
                    style: "margin: 0; font-size: 0.75rem; color: #64748b;",
                    "Two side-by-side DatePickers omitting root id generate distinct aria-controls / popover content IDs with independent open/close state."
                }
            }

            div {
                id: "sw5c-dp-multi-wrapper",
                style: "display: flex; flex-wrap: wrap; gap: 1.5rem;",
                DatePickerRoot {
                    value: Some(left_val),
                    default_placeholder: Some(dp_date(2024, 7, 4)),
                    div {
                        style: "display: flex; flex-direction: column; gap: 0.375rem;",
                        DatePickerLabel { "Picker A (id: None)" }
                        DatePickerInput {
                            render_children: move |segments: Vec<DateFieldSegmentItem>| rsx! {
                                for seg in segments {
                                    DatePickerSegment {
                                        key: "{seg.index}",
                                        part: seg.kind(),
                                        index: Some(seg.index),
                                    }
                                }
                                DatePickerTrigger {
                                    id: "sw5c-dp-iso-a-trigger",
                                    "Open A"
                                }
                            }
                        }
                    }
                    DatePickerPortal {
                        DatePickerContent {
                            DatePickerCalendar {
                                render_children: move |snap: CalendarSnapshot| rsx! {
                                    DatePickerHeader {
                                        style: "{HEADER_STYLE}",
                                        DatePickerPrevButton { "‹" }
                                        DatePickerHeading { style: "font-size: 0.875rem; font-weight: 600;" }
                                        DatePickerNextButton { "›" }
                                    }
                                    DatePickerGrid {
                                        style: "{TABLE_STYLE}",
                                        DatePickerGridHead {
                                            DatePickerGridRow {
                                                for (w_idx, wd) in snap.weekdays.iter().enumerate() {
                                                    DatePickerHeadCell { key: "{w_idx}", style: "{TH_STYLE}", "{wd}" }
                                                }
                                            }
                                        }
                                        DatePickerGridBody {
                                            for (r_idx, week) in snap.months[0].weeks.iter().enumerate() {
                                                DatePickerGridRow {
                                                    key: "{r_idx}",
                                                    for date in week.iter().copied() {
                                                        DatePickerCell {
                                                            key: "{date}",
                                                            date,
                                                            month: snap.months[0].value,
                                                            style: "{TD_STYLE}",
                                                            DatePickerDay {}
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
                }

                DatePickerRoot {
                    value: Some(right_val),
                    default_placeholder: Some(dp_date(2024, 8, 12)),
                    div {
                        style: "display: flex; flex-direction: column; gap: 0.375rem;",
                        DatePickerLabel { "Picker B (id: None)" }
                        DatePickerInput {
                            render_children: move |segments: Vec<DateFieldSegmentItem>| rsx! {
                                for seg in segments {
                                    DatePickerSegment {
                                        key: "{seg.index}",
                                        part: seg.kind(),
                                        index: Some(seg.index),
                                    }
                                }
                                DatePickerTrigger {
                                    id: "sw5c-dp-iso-b-trigger",
                                    "Open B"
                                }
                            }
                        }
                    }
                    DatePickerPortal {
                        DatePickerContent {
                            DatePickerCalendar {
                                render_children: move |snap: CalendarSnapshot| rsx! {
                                    DatePickerHeader {
                                        style: "{HEADER_STYLE}",
                                        DatePickerPrevButton { "‹" }
                                        DatePickerHeading { style: "font-size: 0.875rem; font-weight: 600;" }
                                        DatePickerNextButton { "›" }
                                    }
                                    DatePickerGrid {
                                        style: "{TABLE_STYLE}",
                                        DatePickerGridHead {
                                            DatePickerGridRow {
                                                for (w_idx, wd) in snap.weekdays.iter().enumerate() {
                                                    DatePickerHeadCell { key: "{w_idx}", style: "{TH_STYLE}", "{wd}" }
                                                }
                                            }
                                        }
                                        DatePickerGridBody {
                                            for (r_idx, week) in snap.months[0].weeks.iter().enumerate() {
                                                DatePickerGridRow {
                                                    key: "{r_idx}",
                                                    for date in week.iter().copied() {
                                                        DatePickerCell {
                                                            key: "{date}",
                                                            date,
                                                            month: snap.months[0].value,
                                                            style: "{TD_STYLE}",
                                                            DatePickerDay {}
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
                }
            }
        }
    }
}

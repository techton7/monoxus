use dioxus::prelude::*;
use monoxus::{
    date_field::DateFieldSegmentItem,
    date_range_field::RangeFieldInputKind,
    date_range_picker::*,
    foundation::date_time::{CalendarDate, CalendarLocale, DateValue, DateValueRange},
    range_calendar::RangeCalendarSnapshot,
};

const DATE_RANGE_PICKER_PLAYGROUND_CSS: &str = r#"
[data-date-range-field-root] {
    display: inline-flex;
    flex-direction: column;
    gap: 0.375rem;
}
[data-date-range-field-label] {
    font-size: 0.8125rem;
    font-weight: 600;
    color: #334155;
    cursor: pointer;
    user-select: none;
}
.drp-inputs-row {
    display: inline-flex;
    align-items: center;
    gap: 0.5rem;
    min-height: 38px;
    padding: 0 8px 0 12px;
    border-radius: 0.5rem;
    border: 1px solid #cbd5e1;
    background-color: #ffffff;
    color: #0f172a;
    font-size: 0.875rem;
    font-family: ui-monospace, SFMono-Regular, Menlo, monospace;
}
[data-date-field-input] {
    display: inline-flex;
    align-items: center;
    gap: 2px;
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
[data-date-range-picker-content] {
    z-index: 40;
    border-radius: 0.75rem;
    border: 1px solid #cbd5e1;
    background-color: #ffffff;
    box-shadow: 0 10px 25px -5px rgba(15, 23, 42, 0.14);
    padding: 1rem;
    outline: none;
}
[data-range-calendar-day] {
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
[data-range-calendar-day]:hover:not([data-disabled]):not([data-selected]) {
    background-color: #f1f5f9;
}
[data-range-calendar-day]:focus-visible {
    outline: 2px solid #7c3aed;
    outline-offset: 1px;
}
[data-range-calendar-day][data-highlighted]:not([data-selection-start]):not([data-selection-end]) {
    background-color: #ede9fe;
    color: #5b21b6;
    border-radius: 0;
}
[data-range-calendar-day][data-range-middle] {
    background-color: #f1f5f9;
    color: #0f172a;
    border-radius: 0;
}
[data-range-calendar-day][data-range-start],
[data-range-calendar-day][data-selection-start] {
    background-color: #0f172a;
    color: #ffffff;
    font-weight: 600;
    border-top-right-radius: 0;
    border-bottom-right-radius: 0;
}
[data-range-calendar-day][data-range-end],
[data-range-calendar-day][data-selection-end] {
    background-color: #0f172a;
    color: #ffffff;
    font-weight: 600;
    border-top-left-radius: 0;
    border-bottom-left-radius: 0;
}
[data-range-calendar-day][data-range-start][data-range-end] {
    border-radius: 0.5rem;
}
[data-range-calendar-day][data-disabled] {
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
"#;

const CARD_STYLE: &str = "border-radius: 1rem; border: 1px solid #e2e8f0; background-color: #ffffff; padding: 1.5rem; display: flex; flex-direction: column; gap: 1rem;";
const HEADER_STYLE: &str = "display: flex; align-items: center; justify-content: space-between; gap: 0.5rem; margin-bottom: 0.5rem;";
const TABLE_STYLE: &str = "border-collapse: collapse; table-layout: fixed; width: 252px;";
const TH_STYLE: &str = "width: 36px; height: 28px; text-align: center; font-size: 0.75rem; font-weight: 600; color: #64748b; padding: 0;";
const TD_STYLE: &str = "width: 36px; height: 36px; padding: 2px 0; text-align: center;";
const BADGE_STYLE: &str = "display: inline-flex; align-items: center; border-radius: 0.25rem; padding: 0.125rem 0.5rem; background-color: #f1f5f9; color: #334155; font-size: 0.75rem; font-weight: 700; font-family: monospace;";

fn drp_date(year: i32, month: u8, day: u8) -> DateValue {
    DateValue::Date(CalendarDate::new(year, month, day).expect("valid playground range date"))
}

#[component]
pub fn DateRangePickerShowcase() -> Element {
    let range = use_signal(|| {
        DateValueRange::new(Some(drp_date(2024, 6, 10)), Some(drp_date(2024, 6, 15)))
    });
    let open = use_signal(|| false);

    rsx! {
        style { "{DATE_RANGE_PICKER_PLAYGROUND_CSS}" }
        div {
            class: "max-w-4xl mx-auto space-y-8",
            style: "max-width: 56rem; margin: 0 auto; display: flex; flex-direction: column; gap: 2rem;",

            div {
                class: "space-y-1.5",
                style: "display: flex; flex-direction: column; gap: 0.375rem;",
                h2 {
                    class: "text-2xl font-bold tracking-tight text-slate-900",
                    style: "margin: 0; font-size: 1.5rem; font-weight: 700; color: #0f172a;",
                    "DateRangePicker"
                }
                p {
                    class: "text-sm text-slate-500",
                    style: "margin: 0; font-size: 0.875rem; color: #64748b;",
                    "Anchored DateRangeField + Popover + 2-month RangeCalendar composition with close_on_range_select and LIFO Escape clearing partial start before closing the popover."
                }
            }

            div {
                class: "rounded-2xl border border-slate-200 bg-white p-6 shadow-sm space-y-4",
                style: "{CARD_STYLE}",

                div {
                    style: "display: flex; flex-direction: column; gap: 0.25rem;",
                    h3 {
                        style: "margin: 0; font-size: 1rem; font-weight: 600; color: #0f172a;",
                        "1. 2-Month DateRangePicker with Partial-Range Escape & close_on_range_select (SW5C-6)"
                    }
                    p {
                        style: "margin: 0; font-size: 0.75rem; color: #64748b;",
                        "Open popover, click a start date, press Escape once to clear partial start while keeping popover open, press Escape again to close and restore focus to trigger, or select a full range to auto-close."
                    }
                }

                DateRangePickerRoot {
                    id: "sw5c-drp-main",
                    value: Some(range),
                    default_placeholder: Some(drp_date(2024, 6, 1)),
                    open: Some(open),
                    close_on_range_select: true,
                    number_of_months: 2,
                    fixed_weeks: true,
                    locale: CalendarLocale::EnUs,
                    DateRangePickerLabel {
                        id: "sw5c-drp-main-label",
                        "Trip Dates"
                    }
                    div {
                        class: "drp-inputs-row",
                        DateRangePickerInput {
                            kind: RangeFieldInputKind::Start,
                            id: "sw5c-drp-start-input",
                            name: "trip_start",
                            render_children: move |segments: Vec<DateFieldSegmentItem>| rsx! {
                                for seg in segments {
                                    DateRangePickerSegment {
                                        key: "{seg.index}",
                                        part: seg.kind(),
                                        index: Some(seg.index),
                                        id: format!("sw5c-drp-start-seg-{}", seg.index),
                                    }
                                }
                            }
                        }
                        span {
                            style: "color: #64748b; user-select: none;",
                            "–"
                        }
                        DateRangePickerInput {
                            kind: RangeFieldInputKind::End,
                            id: "sw5c-drp-end-input",
                            name: "trip_end",
                            render_children: move |segments: Vec<DateFieldSegmentItem>| rsx! {
                                for seg in segments {
                                    DateRangePickerSegment {
                                        key: "{seg.index}",
                                        part: seg.kind(),
                                        index: Some(seg.index),
                                        id: format!("sw5c-drp-end-seg-{}", seg.index),
                                    }
                                }
                            }
                        }
                        DateRangePickerTrigger {
                            id: "sw5c-drp-main-trigger",
                            "Pick Range"
                        }
                    }
                    DateRangePickerPortal {
                        DateRangePickerContent {
                            id: "sw5c-drp-main-content",
                            DateRangePickerCalendar {
                                id: "sw5c-drp-main-calendar",
                                render_children: move |snap: RangeCalendarSnapshot| rsx! {
                                    DateRangePickerHeader {
                                        style: "{HEADER_STYLE}",
                                        DateRangePickerPrevButton { id: "sw5c-drp-main-prev", "‹ Prev" }
                                        DateRangePickerHeading {
                                            id: "sw5c-drp-main-heading",
                                            style: "font-size: 0.875rem; font-weight: 600; color: #0f172a;",
                                        }
                                        DateRangePickerNextButton { id: "sw5c-drp-main-next", "Next ›" }
                                    }
                                    div {
                                        style: "display: flex; flex-wrap: wrap; gap: 1.25rem;",
                                        for (m_idx, month) in snap.months.iter().enumerate() {
                                            DateRangePickerGrid {
                                                key: "{m_idx}",
                                                id: "sw5c-drp-main-grid-{m_idx}",
                                                month_idx: Some(m_idx),
                                                style: "{TABLE_STYLE}",
                                                DateRangePickerGridHead {
                                                    DateRangePickerGridRow {
                                                        for (w_idx, wd) in snap.weekdays.iter().enumerate() {
                                                            DateRangePickerHeadCell { key: "{w_idx}", style: "{TH_STYLE}", "{wd}" }
                                                        }
                                                    }
                                                }
                                                DateRangePickerGridBody {
                                                    for (r_idx, week) in month.weeks.iter().enumerate() {
                                                        DateRangePickerGridRow {
                                                            key: "{r_idx}",
                                                            for date in week.iter().copied() {
                                                                DateRangePickerCell {
                                                                    key: "{date}",
                                                                    date,
                                                                    month: month.value,
                                                                    style: "{TD_STYLE}",
                                                                    DateRangePickerDay {
                                                                        id: "sw5c-drp-day-{m_idx}-{date}",
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

                div {
                    style: "display: flex; flex-wrap: wrap; gap: 1rem; font-size: 0.75rem;",
                    div {
                        style: "display: flex; align-items: center; gap: 0.5rem;",
                        span { style: "color: #64748b; font-weight: 600;", "Start:" }
                        span {
                            id: "sw5c-drp-start-value",
                            style: "{BADGE_STYLE}",
                            {range().start.map_or_else(|| String::from("none"), |v| v.to_iso_string())}
                        }
                    }
                    div {
                        style: "display: flex; align-items: center; gap: 0.5rem;",
                        span { style: "color: #64748b; font-weight: 600;", "End:" }
                        span {
                            id: "sw5c-drp-end-value",
                            style: "{BADGE_STYLE}",
                            {range().end.map_or_else(|| String::from("none"), |v| v.to_iso_string())}
                        }
                    }
                    div {
                        style: "display: flex; align-items: center; gap: 0.5rem;",
                        span { style: "color: #64748b; font-weight: 600;", "Open:" }
                        span {
                            id: "sw5c-drp-main-open",
                            style: "{BADGE_STYLE}",
                            "{open}"
                        }
                    }
                }
            }
        }
    }
}

use dioxus::prelude::*;
use monoxus::{
    foundation::date_time::{CalendarDate, DateRange},
    range_calendar::*,
};

const RANGE_CALENDAR_PLAYGROUND_CSS: &str = r#"
[data-range-calendar-root] {
    font-family: inherit;
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
    transition: background-color 0.12s ease, color 0.12s ease;
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
[data-range-calendar-prev-button],
[data-range-calendar-next-button] {
    display: inline-flex;
    align-items: center;
    justify-content: center;
    height: 32px;
    min-width: 32px;
    padding: 0 10px;
    border-radius: 0.5rem;
    border: 1px solid #cbd5e1;
    background-color: #ffffff;
    color: #334155;
    font-size: 0.8125rem;
    font-weight: 500;
    cursor: pointer;
}
[data-range-calendar-month-select],
[data-range-calendar-year-select] {
    height: 32px;
    padding: 0 8px;
    border-radius: 0.5rem;
    border: 1px solid #cbd5e1;
    background-color: #ffffff;
    color: #0f172a;
    font-size: 0.8125rem;
}
"#;

const CARD_STYLE: &str = "border-radius: 1rem; border: 1px solid #e2e8f0; background-color: #ffffff; padding: 1.5rem; display: flex; flex-direction: column; gap: 1rem;";
const CAL_CARD_STYLE: &str = "display: inline-flex; flex-direction: column; gap: 0.75rem; padding: 1rem; border-radius: 0.75rem; border: 1px solid #e2e8f0; background-color: #ffffff;";
const HEADER_STYLE: &str = "display: flex; align-items: center; justify-content: space-between; gap: 0.5rem; flex-wrap: wrap;";
const TABLE_STYLE: &str = "border-collapse: collapse; table-layout: fixed;";
const TH_STYLE: &str = "width: 36px; height: 28px; text-align: center; font-size: 0.75rem; font-weight: 600; color: #64748b; padding: 0;";
const TD_STYLE: &str = "width: 36px; height: 36px; padding: 2px 0; text-align: center;";
const BADGE_STYLE: &str = "display: inline-flex; align-items: center; border-radius: 0.25rem; padding: 0.125rem 0.5rem; background-color: #f1f5f9; color: #334155; font-size: 0.75rem; font-weight: 700; font-family: monospace;";

fn rc_date(year: i32, month: u8, day: u8) -> CalendarDate {
    CalendarDate::new(year, month, day).expect("valid playground range date")
}

#[component]
pub fn RangeCalendarPlayground() -> Element {
    rsx! {
        style { "{RANGE_CALENDAR_PLAYGROUND_CSS}" }
        div {
            class: "max-w-4xl mx-auto space-y-8",
            style: "max-width: 56rem; margin: 0 auto; display: flex; flex-direction: column; gap: 2rem;",

            div {
                class: "space-y-1.5",
                style: "display: flex; flex-direction: column; gap: 0.375rem;",
                h2 {
                    class: "text-2xl font-bold tracking-tight text-slate-900",
                    style: "margin: 0; font-size: 1.5rem; font-weight: 700; color: #0f172a;",
                    "RangeCalendar"
                }
                p {
                    class: "text-sm text-slate-500",
                    style: "margin: 0; font-size: 0.875rem; color: #64748b;",
                    "Contiguous date-range selection primitive with forward/backward ordering, live hover/focus highlighted_range preview, min_days/max_days/exclude_disabled validation, read_only guard, and Escape partial-range clearing."
                }
            }

            BasicRangeCalendarScenario {}
            ConstrainedRangeCalendarScenario {}
            ReadOnlyAndMultiMonthRangeScenario {}
        }
    }
}

#[component]
fn BasicRangeCalendarScenario() -> Element {
    let range = use_signal(|| DateRange::new(Some(rc_date(2024, 1, 8)), Some(rc_date(2024, 1, 14))));
    let mut commits = use_signal(|| 0usize);

    rsx! {
        div {
            class: "rounded-2xl border border-slate-200 bg-white p-6 shadow-sm space-y-4",
            style: "{CARD_STYLE}",

            div {
                style: "display: flex; flex-direction: column; gap: 0.25rem;",
                h3 {
                    class: "text-base font-semibold text-slate-900",
                    style: "margin: 0; font-size: 1rem; font-weight: 600; color: #0f172a;",
                    "1. Contiguous Range Selection, Hover Preview & Escape Partial Clear (SW5B-6)"
                }
                p {
                    class: "text-xs text-slate-500",
                    style: "margin: 0; font-size: 0.75rem; color: #64748b;",
                    "Click a start date, hover/focus backward or forward to preview data-highlighted, press Escape to clear a partial selection, or click an end date to commit."
                }
            }

            div {
                style: "display: flex; flex-wrap: wrap; align-items: flex-start; gap: 1.5rem;",
                RangeCalendarRoot {
                    id: "sw5b-rc-basic",
                    value: Some(range),
                    default_placeholder: rc_date(2024, 1, 1),
                    on_range_select: move |_| commits += 1,
                    style: "{CAL_CARD_STYLE}",
                    render_children: move |snap: RangeCalendarSnapshot| rsx! {
                        RangeCalendarHeader {
                            style: "{HEADER_STYLE}",
                            RangeCalendarPrevButton { id: "sw5b-rc-basic-prev", "‹" }
                            div {
                                style: "display: flex; align-items: center; gap: 0.375rem;",
                                RangeCalendarMonthSelect { id: "sw5b-rc-basic-month-select" }
                                RangeCalendarYearSelect { id: "sw5b-rc-basic-year-select" }
                            }
                            RangeCalendarNextButton { id: "sw5b-rc-basic-next", "›" }
                        }
                        RangeCalendarHeading {
                            id: "sw5b-rc-basic-heading",
                            style: "text-align: center; font-size: 0.875rem; font-weight: 600; color: #0f172a;",
                        }
                        RangeCalendarGrid {
                            id: "sw5b-rc-basic-grid",
                            style: "{TABLE_STYLE}",
                            RangeCalendarGridHead {
                                RangeCalendarGridRow {
                                    for (w_idx, wd) in snap.weekdays.iter().enumerate() {
                                        RangeCalendarHeadCell { key: "{w_idx}", style: "{TH_STYLE}", "{wd}" }
                                    }
                                }
                            }
                            RangeCalendarGridBody {
                                for (r_idx, week) in snap.months[0].weeks.iter().enumerate() {
                                    RangeCalendarGridRow {
                                        key: "{r_idx}",
                                        for date in week.iter().copied() {
                                            RangeCalendarCell {
                                                key: "{date}",
                                                date,
                                                month: snap.months[0].value,
                                                style: "{TD_STYLE}",
                                                RangeCalendarDay {
                                                    id: "sw5b-rc-basic-day-{date}",
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
                    style: "display: flex; flex-direction: column; gap: 0.5rem; font-size: 0.75rem;",
                    div {
                        style: "display: flex; align-items: center; gap: 0.5rem;",
                        span { style: "color: #64748b; font-weight: 600;", "Start:" }
                        span {
                            id: "sw5b-rc-basic-start",
                            style: "{BADGE_STYLE}",
                            {range().start.map_or_else(|| String::from("none"), |d| d.to_iso_string())}
                        }
                    }
                    div {
                        style: "display: flex; align-items: center; gap: 0.5rem;",
                        span { style: "color: #64748b; font-weight: 600;", "End:" }
                        span {
                            id: "sw5b-rc-basic-end",
                            style: "{BADGE_STYLE}",
                            {range().end.map_or_else(|| String::from("none"), |d| d.to_iso_string())}
                        }
                    }
                    div {
                        style: "display: flex; align-items: center; gap: 0.5rem;",
                        span { style: "color: #64748b; font-weight: 600;", "on_range_select:" }
                        span {
                            id: "sw5b-rc-basic-commits",
                            style: "{BADGE_STYLE}",
                            "{commits}"
                        }
                    }
                }
            }
        }
    }
}

#[component]
fn ConstrainedRangeCalendarScenario() -> Element {
    let range = use_signal(DateRange::default);

    rsx! {
        div {
            class: "rounded-2xl border border-slate-200 bg-white p-6 shadow-sm space-y-4",
            style: "{CARD_STYLE}",

            div {
                style: "display: flex; flex-direction: column; gap: 0.25rem;",
                h3 {
                    class: "text-base font-semibold text-slate-900",
                    style: "margin: 0; font-size: 1rem; font-weight: 600; color: #0f172a;",
                    "2. Constraints: min_days = 3, max_days = 6, exclude_disabled = true (SW5B-7)"
                }
                p {
                    class: "text-xs text-slate-500",
                    style: "margin: 0; font-size: 0.75rem; color: #64748b;",
                    "Jan 15 and Jan 22 are disabled. Selecting across a disabled date or outside 3..=6 days resets start to the clicked date."
                }
            }

            div {
                style: "display: flex; flex-wrap: wrap; align-items: flex-start; gap: 1.5rem;",
                RangeCalendarRoot {
                    id: "sw5b-rc-constrained",
                    value: Some(range),
                    default_placeholder: rc_date(2024, 1, 1),
                    min_days: Some(3),
                    max_days: Some(6),
                    exclude_disabled: true,
                    is_date_disabled: Callback::new(|date: CalendarDate| date.day == 15 || date.day == 22),
                    style: "{CAL_CARD_STYLE}",
                    render_children: move |snap: RangeCalendarSnapshot| rsx! {
                        RangeCalendarHeader {
                            style: "{HEADER_STYLE}",
                            RangeCalendarPrevButton { id: "sw5b-rc-const-prev", "‹" }
                            RangeCalendarHeading {
                                id: "sw5b-rc-const-heading",
                                style: "font-size: 0.875rem; font-weight: 600; color: #0f172a;",
                            }
                            RangeCalendarNextButton { id: "sw5b-rc-const-next", "›" }
                        }
                        RangeCalendarGrid {
                            style: "{TABLE_STYLE}",
                            RangeCalendarGridHead {
                                RangeCalendarGridRow {
                                    for (w_idx, wd) in snap.weekdays.iter().enumerate() {
                                        RangeCalendarHeadCell { key: "{w_idx}", style: "{TH_STYLE}", "{wd}" }
                                    }
                                }
                            }
                            RangeCalendarGridBody {
                                for (r_idx, week) in snap.months[0].weeks.iter().enumerate() {
                                    RangeCalendarGridRow {
                                        key: "{r_idx}",
                                        for date in week.iter().copied() {
                                            RangeCalendarCell {
                                                key: "{date}",
                                                date,
                                                month: snap.months[0].value,
                                                style: "{TD_STYLE}",
                                                RangeCalendarDay {
                                                    id: "sw5b-rc-const-day-{date}",
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
                    style: "display: flex; flex-direction: column; gap: 0.5rem; font-size: 0.75rem;",
                    div {
                        style: "display: flex; align-items: center; gap: 0.5rem;",
                        span { style: "color: #64748b; font-weight: 600;", "Start:" }
                        span {
                            id: "sw5b-rc-const-start",
                            style: "{BADGE_STYLE}",
                            {range().start.map_or_else(|| String::from("none"), |d| d.to_iso_string())}
                        }
                    }
                    div {
                        style: "display: flex; align-items: center; gap: 0.5rem;",
                        span { style: "color: #64748b; font-weight: 600;", "End:" }
                        span {
                            id: "sw5b-rc-const-end",
                            style: "{BADGE_STYLE}",
                            {range().end.map_or_else(|| String::from("none"), |d| d.to_iso_string())}
                        }
                    }
                }
            }
        }
    }
}

#[component]
fn ReadOnlyAndMultiMonthRangeScenario() -> Element {
    rsx! {
        div {
            class: "rounded-2xl border border-slate-200 bg-white p-6 shadow-sm space-y-4",
            style: "{CARD_STYLE}",

            div {
                style: "display: flex; flex-direction: column; gap: 0.25rem;",
                h3 {
                    class: "text-base font-semibold text-slate-900",
                    style: "margin: 0; font-size: 1rem; font-weight: 600; color: #0f172a;",
                    "3. Read-Only Selection Guard (5B D-3) & Multi-Month Paged RangeCalendar"
                }
                p {
                    class: "text-xs text-slate-500",
                    style: "margin: 0; font-size: 0.75rem; color: #64748b;",
                    "Read-only RangeCalendar preserves focus/keyboard navigation while blocking range mutations; multi-month renders two 6-week tables."
                }
            }

            div {
                style: "display: flex; flex-wrap: wrap; align-items: flex-start; gap: 1.5rem;",
                RangeCalendarRoot {
                    id: "sw5b-rc-readonly",
                    default_placeholder: rc_date(2024, 1, 1),
                    default_value: DateRange::new(Some(rc_date(2024, 1, 5)), Some(rc_date(2024, 1, 9))),
                    read_only: true,
                    style: "{CAL_CARD_STYLE}",
                    render_children: move |snap: RangeCalendarSnapshot| rsx! {
                        RangeCalendarHeader {
                            style: "{HEADER_STYLE}",
                            RangeCalendarHeading {
                                id: "sw5b-rc-ro-heading",
                                style: "font-size: 0.875rem; font-weight: 600; color: #0f172a;",
                            }
                        }
                        RangeCalendarGrid {
                            style: "{TABLE_STYLE}",
                            RangeCalendarGridHead {
                                RangeCalendarGridRow {
                                    for (w_idx, wd) in snap.weekdays.iter().enumerate() {
                                        RangeCalendarHeadCell { key: "{w_idx}", style: "{TH_STYLE}", "{wd}" }
                                    }
                                }
                            }
                            RangeCalendarGridBody {
                                for (r_idx, week) in snap.months[0].weeks.iter().enumerate() {
                                    RangeCalendarGridRow {
                                        key: "{r_idx}",
                                        for date in week.iter().copied() {
                                            RangeCalendarCell {
                                                key: "{date}",
                                                date,
                                                month: snap.months[0].value,
                                                style: "{TD_STYLE}",
                                                RangeCalendarDay {
                                                    id: "sw5b-rc-ro-day-{date}",
                                                }
                                            }
                                        }
                                    }
                                }
                            }
                        }
                    }
                }

                RangeCalendarRoot {
                    id: "sw5b-rc-multi-month",
                    default_placeholder: rc_date(2024, 1, 1),
                    number_of_months: 2,
                    fixed_weeks: true,
                    paged_navigation: true,
                    style: "{CAL_CARD_STYLE}",
                    render_children: move |snap: RangeCalendarSnapshot| rsx! {
                        RangeCalendarHeader {
                            style: "{HEADER_STYLE}",
                            RangeCalendarPrevButton { id: "sw5b-rc-mm-prev", "‹ Prev" }
                            RangeCalendarHeading {
                                id: "sw5b-rc-mm-heading",
                                style: "font-size: 0.875rem; font-weight: 600; color: #0f172a;",
                            }
                            RangeCalendarNextButton { id: "sw5b-rc-mm-next", "Next ›" }
                        }
                        div {
                            style: "display: flex; flex-wrap: wrap; gap: 1.25rem;",
                            for (m_idx, month) in snap.months.iter().enumerate() {
                                RangeCalendarGrid {
                                    key: "{m_idx}",
                                    month_idx: Some(m_idx),
                                    style: "{TABLE_STYLE}",
                                    RangeCalendarGridHead {
                                        RangeCalendarGridRow {
                                            for (w_idx, wd) in snap.weekdays.iter().enumerate() {
                                                RangeCalendarHeadCell { key: "{w_idx}", style: "{TH_STYLE}", "{wd}" }
                                            }
                                        }
                                    }
                                    RangeCalendarGridBody {
                                        for (r_idx, week) in month.weeks.iter().enumerate() {
                                            RangeCalendarGridRow {
                                                key: "{r_idx}",
                                                for date in week.iter().copied() {
                                                    RangeCalendarCell {
                                                        key: "{date}",
                                                        date,
                                                        month: month.value,
                                                        style: "{TD_STYLE}",
                                                        RangeCalendarDay {
                                                            id: "sw5b-rc-mm-day-{m_idx}-{date}",
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

use dioxus::prelude::*;
use monoxus::{
    calendar::*,
    foundation::{
        date_time::{CalendarDate, CalendarLocale, MonthFormat, WeekdayFormat, YearFormat},
        shared::Direction,
    },
    select::{
        SelectContent, SelectItem, SelectItemData, SelectItemIndicator, SelectItemText,
        SelectPortal, SelectRoot, SelectTrigger, SelectValue, SelectViewport,
    },
};

const CALENDAR_PLAYGROUND_CSS: &str = r#"
[data-calendar-root] {
    font-family: inherit;
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
[data-calendar-day][data-today]:not([data-selected]) {
    background-color: #f5f3ff;
    color: #6d28d9;
    font-weight: 600;
}
[data-calendar-day][data-outside-month] {
    color: #94a3b8;
}
[data-calendar-day][data-selected] {
    background-color: #0f172a;
    color: #ffffff;
    font-weight: 600;
}
[data-calendar-day][data-unavailable] {
    color: #ef4444;
    text-decoration: line-through;
    cursor: not-allowed;
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
[data-calendar-prev-button][data-disabled],
[data-calendar-next-button][data-disabled] {
    background-color: #f1f5f9;
    color: #94a3b8;
    border-color: #e2e8f0;
    cursor: not-allowed;
    opacity: 0.65;
}
[data-calendar-month-select],
[data-calendar-year-select] {
    height: 32px;
    padding: 0 8px;
    border-radius: 0.5rem;
    border: 1px solid #cbd5e1;
    background-color: #ffffff;
    color: #0f172a;
    font-size: 0.8125rem;
}
.cal-select-trigger[data-disabled] {
    background-color: #f1f5f9;
    color: #94a3b8;
    border-color: #e2e8f0;
    cursor: not-allowed;
    opacity: 0.65;
}
.cal-select-item {
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
.cal-select-item[data-highlighted="true"] {
    background-color: #f1f5f9;
}
.cal-select-item[data-selected="true"] {
    font-weight: 600;
}
"#;

const CARD_STYLE: &str = "border-radius: 1rem; border: 1px solid #e2e8f0; background-color: #ffffff; padding: 1.5rem; display: flex; flex-direction: column; gap: 1rem;";
const CAL_CARD_STYLE: &str = "display: inline-flex; flex-direction: column; gap: 0.75rem; padding: 1rem; border-radius: 0.75rem; border: 1px solid #e2e8f0; background-color: #ffffff;";
const HEADER_STYLE: &str = "display: flex; align-items: center; justify-content: space-between; gap: 0.5rem; flex-wrap: wrap;";
const TABLE_STYLE: &str = "border-collapse: collapse; table-layout: fixed; width: 280px; align-self: center;";
const TH_STYLE: &str = "width: 40px; height: 28px; text-align: center; font-size: 0.75rem; font-weight: 600; color: #64748b; padding: 0;";
const TD_STYLE: &str = "width: 40px; height: 40px; padding: 2px; text-align: center;";
const BADGE_STYLE: &str = "display: inline-flex; align-items: center; border-radius: 0.25rem; padding: 0.125rem 0.5rem; background-color: #f1f5f9; color: #334155; font-size: 0.75rem; font-weight: 700; font-family: monospace;";
const TOGGLE_BTN_STYLE: &str = "display: inline-flex; align-items: center; height: 30px; padding: 0 10px; border-radius: 0.5rem; border: 1px solid #cbd5e1; background-color: #f8fafc; color: #334155; font-size: 0.75rem; font-weight: 600; cursor: pointer;";

fn cal_date(year: i32, month: u8, day: u8) -> CalendarDate {
    CalendarDate::new(year, month, day).expect("valid playground calendar date")
}

#[component]
pub fn CalendarPlayground() -> Element {
    rsx! {
        style { "{CALENDAR_PLAYGROUND_CSS}" }
        div {
            class: "max-w-4xl mx-auto space-y-8",
            style: "max-width: 56rem; margin: 0 auto; display: flex; flex-direction: column; gap: 2rem;",

            div {
                class: "space-y-1.5",
                style: "display: flex; flex-direction: column; gap: 0.375rem;",
                h2 {
                    class: "text-2xl font-bold tracking-tight text-slate-900",
                    style: "margin: 0; font-size: 1.5rem; font-weight: 700; color: #0f172a;",
                    "Calendar"
                }
                p {
                    class: "text-sm text-slate-500",
                    style: "margin: 0; font-size: 0.875rem; color: #64748b;",
                    "Headless WAI-ARIA role=\"grid\" calendar supporting single/multiple selection, bounds & unavailable dates, multi-month paged navigation, pure-Rust en-US/ko-KR locales, RTL, and full 2D APG keyboard navigation."
                }
            }

            SingleCalendarScenario {}
            MultipleCalendarScenario {}
            MultiMonthFixedWeeksScenario {}
            LocaleRtlInitialFocusScenario {}
            CustomSelectCalendarScenario {}
        }
    }
}

#[component]
fn SingleCalendarScenario() -> Element {
    let mut selected = use_signal(|| Some(cal_date(2024, 1, 15)));
    let mut select_calls = use_signal(|| 0usize);

    rsx! {
        div {
            class: "rounded-2xl border border-slate-200 bg-white p-6 shadow-sm space-y-4",
            style: "{CARD_STYLE}",

            div {
                style: "display: flex; flex-direction: column; gap: 0.25rem;",
                h3 {
                    class: "text-base font-semibold text-slate-900",
                    style: "margin: 0; font-size: 1rem; font-weight: 600; color: #0f172a;",
                    "1. Single Selection, Bounds, Unavailable Dates, Month/Year Selects & 2D Keyboard (SW5B-1, SW5B-2, SW5B-4, SW5B-8)"
                }
                p {
                    class: "text-xs text-slate-500",
                    style: "margin: 0; font-size: 0.75rem; color: #64748b;",
                    "Bounded to 2024-01-03..=2026-12-20. Day 18 is unavailable; Day 31 is disabled. Try Arrow keys, PageUp/PageDown, Shift+PageUp/Down, Home/End, Enter/Space, or Month/Year selects."
                }
            }

            div {
                style: "display: flex; flex-wrap: wrap; align-items: flex-start; gap: 1.5rem;",
                CalendarRoot {
                    id: "sw5b-cal-single",
                    value: Some(selected()),
                    on_value_change: move |next: Option<CalendarDate>| selected.set(next),
                    default_placeholder: cal_date(2024, 1, 15),
                    min_value: cal_date(2024, 1, 3),
                    max_value: cal_date(2026, 12, 20),
                    is_date_unavailable: Callback::new(|date: CalendarDate| date.day == 18),
                    is_date_disabled: Callback::new(|date: CalendarDate| date.day == 31),
                    on_date_select: move |_| select_calls += 1,
                    style: "{CAL_CARD_STYLE}",
                    render_children: move |snap: CalendarSnapshot| rsx! {
                        CalendarHeader {
                            id: "sw5b-cal-single-header",
                            style: "{HEADER_STYLE}",
                            CalendarPrevButton { id: "sw5b-cal-single-prev", "‹" }
                            div {
                                style: "display: flex; align-items: center; gap: 0.375rem;",
                                CalendarMonthSelect { id: "sw5b-cal-single-month-select" }
                                CalendarYearSelect { id: "sw5b-cal-single-year-select" }
                            }
                            CalendarNextButton { id: "sw5b-cal-single-next", "›" }
                        }
                        CalendarHeading {
                            id: "sw5b-cal-single-heading",
                            style: "text-align: center; font-size: 0.875rem; font-weight: 600; color: #0f172a;",
                        }
                        for (m_idx, month) in snap.months.iter().enumerate() {
                            CalendarGrid {
                                key: "{m_idx}",
                                id: "sw5b-cal-single-grid",
                                month_idx: Some(m_idx),
                                style: "{TABLE_STYLE}",
                                CalendarGridHead {
                                    CalendarGridRow {
                                        for (w_idx, wd) in snap.weekdays.iter().enumerate() {
                                            CalendarHeadCell {
                                                key: "{w_idx}",
                                                style: "{TH_STYLE}",
                                                "{wd}"
                                            }
                                        }
                                    }
                                }
                                CalendarGridBody {
                                    for (r_idx, week) in month.weeks.iter().enumerate() {
                                        CalendarGridRow {
                                            key: "{r_idx}",
                                            for date in week.iter().copied() {
                                                CalendarCell {
                                                    key: "{date}",
                                                    id: "sw5b-cal-single-cell-{date}",
                                                    date,
                                                    month: month.value,
                                                    style: "{TD_STYLE}",
                                                    CalendarDay {
                                                        id: "sw5b-cal-single-day-{date}",
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
                    style: "display: flex; flex-direction: column; gap: 0.5rem; font-size: 0.75rem;",
                    div {
                        style: "display: flex; align-items: center; gap: 0.5rem;",
                        span { style: "color: #64748b; font-weight: 600;", "Selected:" }
                        span {
                            id: "sw5b-cal-single-selected",
                            style: "{BADGE_STYLE}",
                            {selected().map_or_else(|| String::from("none"), |d| d.to_iso_string())}
                        }
                    }
                    div {
                        style: "display: flex; align-items: center; gap: 0.5rem;",
                        span { style: "color: #64748b; font-weight: 600;", "on_date_select:" }
                        span {
                            id: "sw5b-cal-single-select-count",
                            style: "{BADGE_STYLE}",
                            "{select_calls}"
                        }
                    }
                }
            }
        }
    }
}

#[component]
fn MultipleCalendarScenario() -> Element {
    let mut values = use_signal(|| vec![cal_date(2024, 1, 10), cal_date(2024, 1, 12)]);
    let mut prevent_deselect = use_signal(|| false);
    let mut disable_outside = use_signal(|| true);

    let values_text = {
        let current = values();
        if current.is_empty() {
            String::from("empty")
        } else {
            current
                .iter()
                .map(CalendarDate::to_iso_string)
                .collect::<Vec<_>>()
                .join(", ")
        }
    };

    rsx! {
        div {
            class: "rounded-2xl border border-slate-200 bg-white p-6 shadow-sm space-y-4",
            style: "{CARD_STYLE}",

            div {
                style: "display: flex; flex-direction: column; gap: 0.25rem;",
                h3 {
                    class: "text-base font-semibold text-slate-900",
                    style: "margin: 0; font-size: 1rem; font-weight: 600; color: #0f172a;",
                    "2. Multiple Selection (max_days = 3), prevent_deselect & Outside-Month Policy (SW5B-1, SW5B-2)"
                }
                p {
                    class: "text-xs text-slate-500",
                    style: "margin: 0; font-size: 0.75rem; color: #64748b;",
                    "Selecting a 4th date resets to just the newly clicked date when max_days = 3."
                }
            }

            div {
                style: "display: flex; flex-wrap: wrap; gap: 0.5rem;",
                button {
                    id: "sw5b-cal-multi-prevent-deselect-btn",
                    r#type: "button",
                    style: "{TOGGLE_BTN_STYLE}",
                    onclick: move |_| prevent_deselect.set(!prevent_deselect()),
                    "prevent_deselect: {prevent_deselect}"
                }
                button {
                    id: "sw5b-cal-multi-outside-btn",
                    r#type: "button",
                    style: "{TOGGLE_BTN_STYLE}",
                    onclick: move |_| disable_outside.set(!disable_outside()),
                    "disable_days_outside_month: {disable_outside}"
                }
            }

            div {
                style: "display: flex; flex-wrap: wrap; align-items: flex-start; gap: 1.5rem;",
                CalendarRoot {
                    id: "sw5b-cal-multiple",
                    mode: CalendarMode::Multiple,
                    values: Some(values()),
                    on_values_change: move |next: Vec<CalendarDate>| values.set(next),
                    default_placeholder: cal_date(2024, 1, 15),
                    max_days: Some(3),
                    prevent_deselect: prevent_deselect(),
                    disable_days_outside_month: disable_outside(),
                    style: "{CAL_CARD_STYLE}",
                    render_children: move |snap: CalendarSnapshot| rsx! {
                        CalendarHeader {
                            style: "{HEADER_STYLE}",
                            CalendarPrevButton { id: "sw5b-cal-multi-prev", "‹" }
                            CalendarHeading {
                                id: "sw5b-cal-multi-heading",
                                style: "font-size: 0.875rem; font-weight: 600; color: #0f172a;",
                            }
                            CalendarNextButton { id: "sw5b-cal-multi-next", "›" }
                        }
                        CalendarGrid {
                            id: "sw5b-cal-multi-grid",
                            style: "{TABLE_STYLE}",
                            CalendarGridHead {
                                CalendarGridRow {
                                    for (w_idx, wd) in snap.weekdays.iter().enumerate() {
                                        CalendarHeadCell { key: "{w_idx}", style: "{TH_STYLE}", "{wd}" }
                                    }
                                }
                            }
                            CalendarGridBody {
                                for (r_idx, week) in snap.months[0].weeks.iter().enumerate() {
                                    CalendarGridRow {
                                        key: "{r_idx}",
                                        for date in week.iter().copied() {
                                            CalendarCell {
                                                key: "{date}",
                                                date,
                                                month: snap.months[0].value,
                                                style: "{TD_STYLE}",
                                                CalendarDay {
                                                    id: "sw5b-cal-multi-day-{date}",
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
                    style: "display: flex; align-items: center; gap: 0.5rem;",
                    span { style: "color: #64748b; font-size: 0.75rem; font-weight: 600;", "Selected:" }
                    span {
                        id: "sw5b-cal-multi-values",
                        style: "{BADGE_STYLE}",
                        "{values_text}"
                    }
                }
            }
        }
    }
}

#[component]
fn MultiMonthFixedWeeksScenario() -> Element {
    let mut paged_nav = use_signal(|| true);

    rsx! {
        div {
            class: "rounded-2xl border border-slate-200 bg-white p-6 shadow-sm space-y-4",
            style: "{CARD_STYLE}",

            div {
                style: "display: flex; flex-direction: column; gap: 0.25rem;",
                h3 {
                    class: "text-base font-semibold text-slate-900",
                    style: "margin: 0; font-size: 1rem; font-weight: 600; color: #0f172a;",
                    "3. Multi-Month (number_of_months = 2), fixed_weeks & paged_navigation (SW5B-3)"
                }
                p {
                    class: "text-xs text-slate-500",
                    style: "margin: 0; font-size: 0.75rem; color: #64748b;",
                    "Two consecutive 6-week (42-cell) grids advancing by 2 months when paged_navigation = true, or 1 month when false."
                }
            }

            div {
                button {
                    id: "sw5b-cal-paged-toggle",
                    r#type: "button",
                    style: "{TOGGLE_BTN_STYLE}",
                    onclick: move |_| paged_nav.set(!paged_nav()),
                    "paged_navigation: {paged_nav}"
                }
            }

            CalendarRoot {
                id: "sw5b-cal-multi-month",
                default_placeholder: cal_date(2024, 1, 1),
                number_of_months: 2,
                fixed_weeks: true,
                paged_navigation: paged_nav(),
                style: "{CAL_CARD_STYLE}",
                render_children: move |snap: CalendarSnapshot| rsx! {
                    CalendarHeader {
                        style: "{HEADER_STYLE}",
                        CalendarPrevButton { id: "sw5b-cal-mm-prev", "‹ Prev" }
                        CalendarHeading {
                            id: "sw5b-cal-mm-heading",
                            style: "font-size: 0.875rem; font-weight: 600; color: #0f172a;",
                        }
                        CalendarNextButton { id: "sw5b-cal-mm-next", "Next ›" }
                    }
                    div {
                        style: "display: flex; flex-wrap: wrap; gap: 1.25rem;",
                        for (m_idx, month) in snap.months.iter().enumerate() {
                            CalendarGrid {
                                key: "{m_idx}",
                                id: "sw5b-cal-mm-grid-{m_idx}",
                                month_idx: Some(m_idx),
                                style: "{TABLE_STYLE}",
                                CalendarGridHead {
                                    CalendarGridRow {
                                        for (w_idx, wd) in snap.weekdays.iter().enumerate() {
                                            CalendarHeadCell { key: "{w_idx}", style: "{TH_STYLE}", "{wd}" }
                                        }
                                    }
                                }
                                CalendarGridBody {
                                    for (r_idx, week) in month.weeks.iter().enumerate() {
                                        CalendarGridRow {
                                            key: "{r_idx}",
                                            for date in week.iter().copied() {
                                                CalendarCell {
                                                    key: "{date}",
                                                    date,
                                                    month: month.value,
                                                    style: "{TD_STYLE}",
                                                    CalendarDay {
                                                        id: "sw5b-cal-mm-day-{m_idx}-{date}",
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

#[component]
fn LocaleRtlInitialFocusScenario() -> Element {
    let mut use_korean = use_signal(|| true);
    let mut rtl = use_signal(|| false);
    let mut mount_initial_focus = use_signal(|| false);

    let locale = if use_korean() {
        CalendarLocale::KoKr
    } else {
        CalendarLocale::EnUs
    };
    let dir = if rtl() { Direction::Rtl } else { Direction::Ltr };

    rsx! {
        div {
            class: "rounded-2xl border border-slate-200 bg-white p-6 shadow-sm space-y-4",
            style: "{CARD_STYLE}",

            div {
                style: "display: flex; flex-direction: column; gap: 0.25rem;",
                h3 {
                    class: "text-base font-semibold text-slate-900",
                    style: "margin: 0; font-size: 1rem; font-weight: 600; color: #0f172a;",
                    "4. Pure-Rust Locale (ko-KR / en-US), RTL & initial_focus (SW5B-4, SW5B-5)"
                }
                p {
                    class: "text-xs text-slate-500",
                    style: "margin: 0; font-size: 0.75rem; color: #64748b;",
                    "Toggle ko-KR/en-US labels and RTL horizontal arrow direction, or mount a calendar with initial_focus = true."
                }
            }

            div {
                style: "display: flex; flex-wrap: wrap; gap: 0.5rem;",
                button {
                    id: "sw5b-cal-locale-toggle",
                    r#type: "button",
                    style: "{TOGGLE_BTN_STYLE}",
                    onclick: move |_| use_korean.set(!use_korean()),
                    if use_korean() { "Locale: ko-KR" } else { "Locale: en-US" }
                }
                button {
                    id: "sw5b-cal-dir-toggle",
                    r#type: "button",
                    style: "{TOGGLE_BTN_STYLE}",
                    onclick: move |_| rtl.set(!rtl()),
                    if rtl() { "Direction: RTL" } else { "Direction: LTR" }
                }
                button {
                    id: "sw5b-cal-init-focus-mount",
                    r#type: "button",
                    style: "{TOGGLE_BTN_STYLE}",
                    onclick: move |_| mount_initial_focus.set(!mount_initial_focus()),
                    if mount_initial_focus() { "Unmount initial_focus" } else { "Mount initial_focus" }
                }
            }

            div {
                style: "display: flex; flex-wrap: wrap; align-items: flex-start; gap: 1.5rem;",
                CalendarRoot {
                    id: "sw5b-cal-i18n",
                    default_placeholder: cal_date(2024, 1, 15),
                    locale,
                    dir: Some(dir),
                    weekday_format: WeekdayFormat::Short,
                    month_format: MonthFormat::Long,
                    year_format: YearFormat::Numeric,
                    style: "{CAL_CARD_STYLE}",
                    render_children: move |snap: CalendarSnapshot| rsx! {
                        CalendarHeader {
                            style: "{HEADER_STYLE}",
                            CalendarPrevButton { id: "sw5b-cal-i18n-prev", "‹" }
                            CalendarHeading {
                                id: "sw5b-cal-i18n-heading",
                                style: "font-size: 0.875rem; font-weight: 600; color: #0f172a;",
                            }
                            CalendarNextButton { id: "sw5b-cal-i18n-next", "›" }
                        }
                        CalendarGrid {
                            style: "{TABLE_STYLE}",
                            CalendarGridHead {
                                CalendarGridRow {
                                    for (w_idx, wd) in snap.weekdays.iter().enumerate() {
                                        CalendarHeadCell { key: "{w_idx}", style: "{TH_STYLE}", "{wd}" }
                                    }
                                }
                            }
                            CalendarGridBody {
                                for (r_idx, week) in snap.months[0].weeks.iter().enumerate() {
                                    CalendarGridRow {
                                        key: "{r_idx}",
                                        for date in week.iter().copied() {
                                            CalendarCell {
                                                key: "{date}",
                                                date,
                                                month: snap.months[0].value,
                                                style: "{TD_STYLE}",
                                                CalendarDay {
                                                    id: "sw5b-cal-i18n-day-{date}",
                                                }
                                            }
                                        }
                                    }
                                }
                            }
                        }
                    }
                }

                if mount_initial_focus() {
                    CalendarRoot {
                        id: "sw5b-cal-init-focus",
                        default_placeholder: cal_date(2024, 1, 1),
                        initial_focus: true,
                        is_date_disabled: Callback::new(|date: CalendarDate| date == cal_date(2024, 1, 1)),
                        style: "{CAL_CARD_STYLE}",
                        render_children: move |snap: CalendarSnapshot| rsx! {
                            CalendarHeader {
                                style: "{HEADER_STYLE}",
                                CalendarHeading { style: "font-size: 0.875rem; font-weight: 600; color: #0f172a;" }
                            }
                            CalendarGrid {
                                style: "{TABLE_STYLE}",
                                CalendarGridHead {
                                    CalendarGridRow {
                                        for (w_idx, wd) in snap.weekdays.iter().enumerate() {
                                            CalendarHeadCell { key: "{w_idx}", style: "{TH_STYLE}", "{wd}" }
                                        }
                                    }
                                }
                                CalendarGridBody {
                                    for (r_idx, week) in snap.months[0].weeks.iter().enumerate() {
                                        CalendarGridRow {
                                            key: "{r_idx}",
                                            for date in week.iter().copied() {
                                                CalendarCell {
                                                    key: "{date}",
                                                    date,
                                                    month: snap.months[0].value,
                                                    style: "{TD_STYLE}",
                                                    CalendarDay {
                                                        id: "sw5b-cal-init-day-{date}",
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

fn render_custom_calendar_select(
    id: &str,
    current_val: String,
    items: Vec<CalendarSelectOption>,
    on_value_change: EventHandler<Option<String>>,
    min_width: &str,
) -> Element {
    let select_items: Vec<SelectItemData> = items
        .iter()
        .map(|opt| SelectItemData::new(opt.value.to_string(), opt.label.clone(), false))
        .collect();
    let trigger_style = format!(
        "display: inline-flex; align-items: center; justify-content: space-between; gap: 0.375rem; height: 32px; padding: 0 10px; border-radius: 0.5rem; border: 1px solid #cbd5e1; background-color: #ffffff; color: #0f172a; font-size: 0.8125rem; font-weight: 500; cursor: pointer; {min_width}"
    );
    rsx! {
        SelectRoot {
            id: id.to_string(),
            items: Some(select_items),
            value: Some(current_val.clone()),
            on_value_change,
            SelectTrigger {
                class: "cal-select-trigger".to_string(),
                style: trigger_style,
                SelectValue {}
                span {
                    style: "font-size: 0.6875rem; color: #64748b;",
                    "▾"
                }
            }
            SelectPortal {
                SelectContent {
                    class: "cal-select-content".to_string(),
                    style: "z-index: 50; background-color: #ffffff; border: 1px solid #cbd5e1; border-radius: 0.5rem; box-shadow: 0 10px 15px -3px rgba(15, 23, 42, 0.12); padding: 0.25rem; outline: none;".to_string(),
                    SelectViewport {
                        style: "max-height: 220px;".to_string(),
                        for opt in items {
                            {
                                let opt_val = opt.value.to_string();
                                let is_selected = opt_val == current_val;
                                rsx! {
                                    SelectItem {
                                        key: "{opt.value}",
                                        value: opt_val,
                                        text: opt.label.clone(),
                                        class: "cal-select-item".to_string(),
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
fn CustomSelectCalendarScenario() -> Element {
    let mut selected = use_signal(|| Some(cal_date(2024, 1, 15)));
    let mut placeholder = use_signal(|| cal_date(2024, 1, 15));
    let mut select_calls = use_signal(|| 0usize);

    rsx! {
        div {
            class: "rounded-2xl border border-slate-200 bg-white p-6 shadow-sm space-y-4",
            style: "{CARD_STYLE}",

            div {
                style: "display: flex; flex-direction: column; gap: 0.25rem;",
                h3 {
                    class: "text-base font-semibold text-slate-900",
                    style: "margin: 0; font-size: 1rem; font-weight: 600; color: #0f172a;",
                    "5. Composed Month/Year Select (monoxus::Select)"
                }
                p {
                    class: "text-xs text-slate-500",
                    style: "margin: 0; font-size: 0.75rem; color: #64748b;",
                    "Same configuration as Scenario 1 (bounded to 2024-01-03..=2026-12-20; Day 18 unavailable; Day 31 disabled), replacing native <select> with composed monoxus::SelectRoot."
                }
            }

            div {
                style: "display: flex; flex-wrap: wrap; align-items: flex-start; gap: 1.5rem;",
                CalendarRoot {
                    id: "sw5b-cal-custom-select",
                    value: Some(selected()),
                    on_value_change: move |next: Option<CalendarDate>| selected.set(next),
                    placeholder: Some(placeholder()),
                    on_placeholder_change: move |next: CalendarDate| placeholder.set(next),
                    min_value: cal_date(2024, 1, 3),
                    max_value: cal_date(2026, 12, 20),
                    is_date_unavailable: Callback::new(|date: CalendarDate| date.day == 18),
                    is_date_disabled: Callback::new(|date: CalendarDate| date.day == 31),
                    on_date_select: move |_| select_calls += 1,
                    style: "{CAL_CARD_STYLE}",
                    render_children: move |snap: CalendarSnapshot| {
                        let month_items = build_calendar_month_options(
                            None,
                            &CalendarLocale::EnUs,
                            MonthFormat::Long,
                        );
                        let year_items = build_calendar_year_options(
                            &[2024, 2025, 2026],
                            &CalendarLocale::EnUs,
                            YearFormat::Numeric,
                        );
                        let cur_month = snap.placeholder.month.to_string();
                        let cur_year = snap.placeholder.year.to_string();
                        rsx! {
                        CalendarHeader {
                            id: "sw5b-cal-custom-header",
                            style: "{HEADER_STYLE}",
                            CalendarPrevButton { id: "sw5b-cal-custom-prev", "‹" }
                            div {
                                style: "display: flex; align-items: center; gap: 0.375rem;",
                                {render_custom_calendar_select(
                                    "sw5b-cal-custom-month-select",
                                    cur_month,
                                    month_items,
                                    EventHandler::new(move |val: Option<String>| {
                                        if let Some(m) = val.and_then(|s| s.parse::<u8>().ok()) {
                                            let next = placeholder.peek().with_month_clamped(m);
                                            placeholder.set(next);
                                        }
                                    }),
                                    "min-width: 120px;",
                                )}
                                {render_custom_calendar_select(
                                    "sw5b-cal-custom-year-select",
                                    cur_year,
                                    year_items,
                                    EventHandler::new(move |val: Option<String>| {
                                        if let Some(y) = val.and_then(|s| s.parse::<i32>().ok()) {
                                            let next = placeholder.peek().with_year_clamped(y);
                                            placeholder.set(next);
                                        }
                                    }),
                                    "min-width: 88px;",
                                )}
                            }
                            CalendarNextButton { id: "sw5b-cal-custom-next", "›" }
                        }
                        CalendarHeading {
                            id: "sw5b-cal-custom-heading",
                            style: "text-align: center; font-size: 0.875rem; font-weight: 600; color: #0f172a;",
                        }
                        for (m_idx, month) in snap.months.iter().enumerate() {
                            CalendarGrid {
                                key: "{m_idx}",
                                id: "sw5b-cal-custom-grid",
                                month_idx: Some(m_idx),
                                style: "{TABLE_STYLE}",
                                CalendarGridHead {
                                    CalendarGridRow {
                                        for (w_idx, wd) in snap.weekdays.iter().enumerate() {
                                            CalendarHeadCell {
                                                key: "{w_idx}",
                                                style: "{TH_STYLE}",
                                                "{wd}"
                                            }
                                        }
                                    }
                                }
                                CalendarGridBody {
                                    for (r_idx, week) in month.weeks.iter().enumerate() {
                                        CalendarGridRow {
                                            key: "{r_idx}",
                                            for date in week.iter().copied() {
                                                CalendarCell {
                                                    key: "{date}",
                                                    id: "sw5b-cal-custom-cell-{date}",
                                                    date,
                                                    month: month.value,
                                                    style: "{TD_STYLE}",
                                                    CalendarDay {
                                                        id: "sw5b-cal-custom-day-{date}",
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
                    style: "display: flex; flex-direction: column; gap: 0.5rem; font-size: 0.75rem;",
                    div {
                        style: "display: flex; align-items: center; gap: 0.5rem;",
                        span { style: "color: #64748b; font-weight: 600;", "Selected:" }
                        span {
                            id: "sw5b-cal-custom-selected",
                            style: "{BADGE_STYLE}",
                            {selected().map_or_else(|| String::from("none"), |d| d.to_iso_string())}
                        }
                    }
                    div {
                        style: "display: flex; align-items: center; gap: 0.5rem;",
                        span { style: "color: #64748b; font-weight: 600;", "on_date_select:" }
                        span {
                            id: "sw5b-cal-custom-select-count",
                            style: "{BADGE_STYLE}",
                            "{select_calls}"
                        }
                    }
                }
            }
        }
    }
}

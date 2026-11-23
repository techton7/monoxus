use dioxus::prelude::*;
use monoxus::{
    calendar::{
        CALENDAR_PARTS, Calendar, CalendarCell, CalendarDay, CalendarGrid, CalendarGridBody,
        CalendarGridHead, CalendarGridRow, CalendarHeadCell, CalendarHeader, CalendarHeading,
        CalendarMode, CalendarMonthSelect, CalendarNextButton, CalendarPart, CalendarPrevButton,
        CalendarRelationships, CalendarRoot, CalendarSnapshot, CalendarYearSelect,
        use_calendar_context,
    },
    foundation::{
        compose::Slottable,
        date_time::{
            CalendarDate, CalendarDateTime, CalendarLocale, CalendarTime, MonthFormat, Weekday,
            WeekdayFormat, YearFormat,
        },
        shared::{Direction, ScopeHandle},
    },
};

#[cfg(feature = "native")]
use crate::common;

fn d(y: i32, m: u8, day: u8) -> CalendarDate {
    CalendarDate::new(y, m, day).expect("valid test date")
}

#[test]
fn calendar_public_contracts_and_date_time_types() {
    assert_eq!(
        CALENDAR_PARTS
            .iter()
            .map(CalendarPart::as_str)
            .collect::<Vec<_>>(),
        [
            "root",
            "header",
            "heading",
            "prev-button",
            "next-button",
            "grid",
            "grid-head",
            "grid-body",
            "grid-row",
            "head-cell",
            "cell",
            "day",
            "month-select",
            "year-select",
        ]
    );

    let scope = ScopeHandle::root("calendar").child("contract");
    let rels = CalendarRelationships::new(scope.clone());
    assert_eq!(rels.root_id(), scope.token());
    assert_eq!(rels.grid_id(0), scope.qualify("grid-0"));
    assert_eq!(
        rels.day_id(0, d(2024, 1, 15)),
        scope.qualify("day-0-2024-01-15")
    );

    let dt = CalendarDateTime::new(d(2024, 2, 29), CalendarTime::new(9, 15, 0, 0).unwrap());
    assert_eq!(dt.to_iso_string(), "2024-02-29T09:15:00");
    assert_eq!(
        CalendarLocale::KoKr.cell_aria_label(d(2024, 1, 1)),
        "2024년 1월 1일 월요일"
    );

    let root_attrs = monoxus::calendar::CalendarRootAttrs::new(
        rels.root_id().to_string(),
        monoxus::calendar::CalendarKind::Calendar,
        String::from("Event January 2024"),
        Direction::Ltr,
        false,
        false,
        false,
    );
    let (_, proj_root, _) =
        monoxus::calendar::project_calendar_root_as_child(&root_attrs, "div", Slottable::new("cal"));
    assert_eq!(proj_root, root_attrs);
}

#[component]
fn CalendarContextProbe() -> Element {
    let ctx = use_calendar_context();
    let snap = ctx.snapshot();
    rsx! {
        CalendarHeader {
            CalendarPrevButton {}
            CalendarHeading {}
            CalendarNextButton {}
            CalendarMonthSelect {}
            CalendarYearSelect {}
        }
        for (m_idx, month) in snap.months.iter().enumerate() {
            CalendarGrid {
                key: "{m_idx}",
                month_idx: Some(m_idx),
                CalendarGridHead {
                    CalendarGridRow {
                        for (w_idx, wd) in snap.weekdays.iter().enumerate() {
                            CalendarHeadCell {
                                key: "{w_idx}",
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
                                    date,
                                    month: month.value,
                                    CalendarDay {}
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
fn CalendarVdomSmokeApp() -> Element {
    rsx! {
        Calendar {
            id: "vdom-cal",
            default_placeholder: d(2024, 1, 15),
            default_value: d(2024, 1, 10),
            fixed_weeks: true,
            number_of_months: 2,
            paged_navigation: true,
            weekday_format: WeekdayFormat::Short,
            month_format: MonthFormat::Long,
            year_format: YearFormat::Numeric,
            CalendarContextProbe {}
        }
    }
}

#[test]
fn calendar_vdom_renders_single_multiple_and_selects() {
    let mut dom = VirtualDom::new(CalendarVdomSmokeApp);
    dom.rebuild_in_place();

    #[component]
    fn MultipleRenderChildrenApp() -> Element {
        rsx! {
            CalendarRoot {
                id: "vdom-multi",
                mode: CalendarMode::Multiple,
                default_placeholder: d(2024, 3, 1),
                default_values: vec![d(2024, 3, 2), d(2024, 3, 5)],
                max_days: Some(3),
                week_starts_on: Some(Weekday::Monday),
                render_children: move |snap: CalendarSnapshot| rsx! {
                    CalendarHeader {
                        CalendarHeading {}
                    }
                    CalendarGrid {
                        CalendarGridBody {
                            for (r_idx, week) in snap.months[0].weeks.iter().enumerate() {
                                CalendarGridRow {
                                    key: "{r_idx}",
                                    for date in week.iter().copied() {
                                        CalendarCell {
                                            key: "{date}",
                                            date,
                                            month: snap.months[0].value,
                                            CalendarDay {}
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

    let mut multi_dom = VirtualDom::new(MultipleRenderChildrenApp);
    multi_dom.rebuild_in_place();
}

#[cfg(feature = "native")]
mod native_tests {
    use super::*;
    use common::native_fixture::*;
    use keyboard_types::{Code, Key, NamedKey};

    #[component]
    fn NativeCalendarMainScenario() -> Element {
        let selected_count = use_signal(|| 0usize);
        let mut selected_cb_count = selected_count;

        rsx! {
            div {
                style: "padding: 16px;",
                CalendarRoot {
                    id: "cal-main",
                    default_placeholder: d(2024, 1, 15),
                    min_value: d(2024, 1, 3),
                    max_value: d(2024, 2, 20),
                    is_date_unavailable: Callback::new(|date: CalendarDate| date == d(2024, 1, 18)),
                    is_date_disabled: Callback::new(|date: CalendarDate| date == d(2024, 1, 31)),
                    on_date_select: move |_| selected_cb_count += 1,
                    render_children: move |snap: CalendarSnapshot| rsx! {
                        div {
                            id: "cal-probe",
                            "data-placeholder": "{snap.placeholder}",
                            "data-heading": "{snap.heading}",
                            "data-select-calls": "{selected_count}",
                        }
                        CalendarHeader {
                            id: "cal-header",
                            CalendarPrevButton { id: "cal-prev", "Prev" }
                            CalendarHeading { id: "cal-heading" }
                            CalendarNextButton { id: "cal-next", "Next" }
                            CalendarMonthSelect { id: "cal-month-sel" }
                            CalendarYearSelect { id: "cal-year-sel" }
                        }
                        for (m_idx, month) in snap.months.iter().enumerate() {
                            CalendarGrid {
                                key: "{m_idx}",
                                id: "cal-grid-{m_idx}",
                                month_idx: Some(m_idx),
                                CalendarGridHead {
                                    CalendarGridRow {
                                        for (w_idx, wd) in snap.weekdays.iter().enumerate() {
                                            CalendarHeadCell {
                                                key: "{w_idx}",
                                                id: "cal-wh-{m_idx}-{w_idx}",
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
                                                    id: "cal-cell-{m_idx}-{date}",
                                                    date,
                                                    month: month.value,
                                                    CalendarDay {
                                                        id: "cal-day-{m_idx}-{date}",
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

    #[test]
    fn native_calendar_selection_bounds_live_region_and_keyboard_2d_nav() {
        let _lock = NATIVE_TEST_LOCK.lock().unwrap_or_else(|e| e.into_inner());
        let mut harness = mount_native_scenario(NativeCalendarMainScenario);

        // Strict table layout & non-empty geometry across all 7 columns
        assert_rect_non_empty(&harness, "#cal-grid-0");
        for col in 0..7 {
            assert_rect_non_empty(&harness, &format!("#cal-wh-0-{col}"));
        }

        // Declarative VisuallyHidden accessible heading & live region (5B D-5)
        let heading_nodes = harness.query_all("#cal-main [role='heading'][aria-level='2']");
        assert_eq!(heading_nodes.len(), 1);
        assert_eq!(
            harness.text_content("#cal-main [role='heading'][aria-level='2']"),
            "Event January 2024"
        );
        assert_eq!(
            harness.attr("#cal-main [role='status']", "aria-live").as_deref(),
            Some("polite")
        );
        assert_eq!(
            harness.attr("#cal-main [role='status']", "aria-atomic").as_deref(),
            Some("true")
        );

        // PrevButton is disabled because min_value is 2024-01-03 (Dec 2023 < min_value)
        assert_eq!(
            harness.attr("#cal-prev", "aria-disabled").as_deref(),
            Some("true")
        );
        assert_eq!(
            harness.attr("#cal-prev", "data-disabled").as_deref(),
            Some("")
        );
        assert_eq!(
            harness.attr("#cal-next", "aria-disabled").as_deref(),
            Some("false")
        );
        assert_eq!(harness.attr("#cal-next", "data-disabled"), None);

        // Bounds & unavailable & outside-month enforcement
        assert_eq!(
            harness.attr("#cal-day-0-2024-01-02", "data-disabled").as_deref(),
            Some("")
        );
        assert_eq!(
            harness.attr("#cal-day-0-2024-01-02", "tabindex"),
            None
        );
        assert_eq!(
            harness.attr("#cal-day-0-2024-01-18", "data-unavailable").as_deref(),
            Some("")
        );
        assert_eq!(
            harness.attr("#cal-day-0-2024-01-18", "aria-disabled").as_deref(),
            Some("true")
        );
        // Outside-month cell (2023-12-31) disabled by default (5B D-2)
        assert_eq!(
            harness.attr("#cal-day-0-2023-12-31", "data-outside-month").as_deref(),
            Some("")
        );
        assert_eq!(
            harness.attr("#cal-day-0-2023-12-31", "data-disabled").as_deref(),
            Some("")
        );

        // Clicking unavailable day does not select
        click_and_pump(&mut harness, "#cal-day-0-2024-01-18");
        assert_eq!(harness.attr("#cal-day-0-2024-01-18", "data-selected"), None);
        assert_eq!(
            harness.attr("#cal-probe", "data-select-calls").as_deref(),
            Some("0")
        );

        // Clicking Jan 15 selects it, fires on_date_select, and updates live announcement
        click_and_pump(&mut harness, "#cal-day-0-2024-01-15");
        assert_eq!(
            harness.attr("#cal-day-0-2024-01-15", "data-selected").as_deref(),
            Some("")
        );
        assert_eq!(
            harness.attr("#cal-cell-0-2024-01-15", "aria-selected").as_deref(),
            Some("true")
        );
        assert_eq!(
            harness.attr("#cal-probe", "data-select-calls").as_deref(),
            Some("1")
        );
        assert_eq!(
            harness.text_content("#cal-main [role='status']"),
            "Selected Date: January 15, 2024"
        );

        // Clicking Jan 15 again deselects it and announces empty
        click_and_pump(&mut harness, "#cal-day-0-2024-01-15");
        assert_eq!(harness.attr("#cal-day-0-2024-01-15", "data-selected"), None);
        assert_eq!(
            harness.text_content("#cal-main [role='status']"),
            "Selected date is now empty."
        );

        // 2D APG keyboard navigation (5B D-1):
        // Focus Jan 15 -> ArrowRight -> Jan 16 -> Space selects Jan 16
        focus_node(&mut harness, "#cal-day-0-2024-01-15");
        send_key_and_pump(&mut harness, Key::Named(NamedKey::ArrowRight), Code::ArrowRight);
        assert_native_focus_id(&harness, "cal-day-0-2024-01-16");
        send_key_and_pump(&mut harness, Key::Character(String::from(" ")), Code::Space);
        assert_eq!(
            harness.attr("#cal-day-0-2024-01-16", "data-selected").as_deref(),
            Some("")
        );

        // Home -> Jan 14 (Sunday), End -> Jan 20 (Saturday)
        send_key_and_pump(&mut harness, Key::Named(NamedKey::Home), Code::Home);
        assert_native_focus_id(&harness, "cal-day-0-2024-01-14");
        send_key_and_pump(&mut harness, Key::Named(NamedKey::End), Code::End);
        assert_native_focus_id(&harness, "cal-day-0-2024-01-20");

        // Down from Jan 24 skips disabled Jan 31 and pages into Feb 7
        focus_node(&mut harness, "#cal-day-0-2024-01-24");
        send_key_and_pump(&mut harness, Key::Named(NamedKey::ArrowDown), Code::ArrowDown);
        assert_eq!(
            harness.attr("#cal-probe", "data-heading").as_deref(),
            Some("February 2024")
        );
        assert_native_focus_id(&harness, "cal-day-0-2024-02-07");

        // PageUp returns to January (2024-01-07)
        send_key_and_pump(&mut harness, Key::Named(NamedKey::PageUp), Code::PageUp);
        assert_eq!(
            harness.attr("#cal-probe", "data-heading").as_deref(),
            Some("January 2024")
        );
        assert_native_focus_id(&harness, "cal-day-0-2024-01-07");

        // NextButton advances to February and disables NextButton (March > max_value 2024-02-20)
        click_and_pump(&mut harness, "#cal-next");
        assert_eq!(
            harness.attr("#cal-probe", "data-heading").as_deref(),
            Some("February 2024")
        );
        assert_eq!(
            harness.attr("#cal-next", "aria-disabled").as_deref(),
            Some("true")
        );

        // MonthSelect / YearSelect DOM contract present (ENG D-2 Blitz popup tracked separately)
        assert_eq!(
            harness.attr("#cal-month-sel", "aria-label").as_deref(),
            Some("Select a month")
        );
        assert_eq!(
            harness.attr("#cal-year-sel", "aria-label").as_deref(),
            Some("Select a year")
        );
        assert_eq!(
            harness.attr("#cal-year-sel", "data-calendar-year-select").as_deref(),
            Some("")
        );
    }

    #[component]
    fn NativeCalendarMultiMonthRtlInitialFocusScenario() -> Element {
        rsx! {
            div {
                style: "padding: 16px;",
                // 1. Multiple mode + max_days + fixed_weeks + 2 months + paged_navigation
                CalendarRoot {
                    id: "cal-multi",
                    mode: CalendarMode::Multiple,
                    default_placeholder: d(2024, 1, 1),
                    max_days: Some(2),
                    fixed_weeks: true,
                    number_of_months: 2,
                    paged_navigation: true,
                    render_children: move |snap: CalendarSnapshot| rsx! {
                        div { id: "cal-multi-heading", "{snap.heading}" }
                        CalendarPrevButton { id: "cal-multi-prev", "Prev" }
                        CalendarNextButton { id: "cal-multi-next", "Next" }
                        for (m_idx, month) in snap.months.iter().enumerate() {
                            CalendarGrid {
                                key: "{m_idx}",
                                month_idx: Some(m_idx),
                                CalendarGridBody {
                                    for (r_idx, week) in month.weeks.iter().enumerate() {
                                        CalendarGridRow {
                                            key: "{r_idx}",
                                            for date in week.iter().copied() {
                                                CalendarCell {
                                                    key: "{date}",
                                                    date,
                                                    month: month.value,
                                                    CalendarDay {
                                                        id: "multi-day-{m_idx}-{date}",
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

                // 2. RTL + initial_focus (default placeholder Jan 1 is disabled -> focuses Jan 2)
                CalendarRoot {
                    id: "cal-rtl",
                    default_placeholder: d(2024, 1, 1),
                    dir: Some(Direction::Rtl),
                    initial_focus: true,
                    is_date_disabled: Callback::new(|date: CalendarDate| date == d(2024, 1, 1)),
                    render_children: move |snap: CalendarSnapshot| rsx! {
                        CalendarGrid {
                            CalendarGridBody {
                                for (r_idx, week) in snap.months[0].weeks.iter().enumerate() {
                                    CalendarGridRow {
                                        key: "{r_idx}",
                                        for date in week.iter().copied() {
                                            CalendarCell {
                                                key: "{date}",
                                                date,
                                                month: snap.months[0].value,
                                                CalendarDay {
                                                    id: "rtl-day-{date}",
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

    #[test]
    fn native_calendar_multiple_fixed_weeks_paged_and_rtl_initial_focus() {
        let _lock = NATIVE_TEST_LOCK.lock().unwrap_or_else(|e| e.into_inner());
        let mut harness =
            mount_native_scenario(NativeCalendarMultiMonthRtlInitialFocusScenario);
        advance_frame_and_pump(&mut harness, 0.02);

        // initial_focus skipped disabled Jan 1 and landed on Jan 2
        assert_native_focus_id(&harness, "rtl-day-2024-01-02");

        // RTL: ArrowLeft advances to Jan 3; ArrowRight returns to Jan 2
        send_key_and_pump(&mut harness, Key::Named(NamedKey::ArrowLeft), Code::ArrowLeft);
        assert_native_focus_id(&harness, "rtl-day-2024-01-03");
        send_key_and_pump(&mut harness, Key::Named(NamedKey::ArrowRight), Code::ArrowRight);
        assert_native_focus_id(&harness, "rtl-day-2024-01-02");

        // Multi-month heading & fixed_weeks = 42 cells per month
        assert_eq!(
            harness.text_content("#cal-multi-heading"),
            "January - February 2024"
        );
        assert_eq!(harness.query_all("#cal-multi [data-calendar-grid]").len(), 2);
        assert_eq!(harness.query_all("#cal-multi [data-calendar-day]").len(), 84);

        // Multiple selection max_days = 2 resets to 3rd clicked day
        click_and_pump(&mut harness, "#multi-day-0-2024-01-05");
        click_and_pump(&mut harness, "#multi-day-0-2024-01-06");
        assert_eq!(
            harness.attr("#multi-day-0-2024-01-05", "data-selected").as_deref(),
            Some("")
        );
        assert_eq!(
            harness.attr("#multi-day-0-2024-01-06", "data-selected").as_deref(),
            Some("")
        );
        click_and_pump(&mut harness, "#multi-day-0-2024-01-07");
        assert_eq!(harness.attr("#multi-day-0-2024-01-05", "data-selected"), None);
        assert_eq!(harness.attr("#multi-day-0-2024-01-06", "data-selected"), None);
        assert_eq!(
            harness.attr("#multi-day-0-2024-01-07", "data-selected").as_deref(),
            Some("")
        );

        // paged_navigation advances by 2 months (Jan-Feb -> Mar-Apr)
        click_and_pump(&mut harness, "#cal-multi-next");
        assert_eq!(
            harness.text_content("#cal-multi-heading"),
            "March - April 2024"
        );
        click_and_pump(&mut harness, "#cal-multi-prev");
        assert_eq!(
            harness.text_content("#cal-multi-heading"),
            "January - February 2024"
        );
    }
}

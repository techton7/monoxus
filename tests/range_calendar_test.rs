use dioxus::prelude::*;
use monoxus::{
    foundation::{
        compose::Slottable,
        date_time::{CalendarDate, CalendarLocale, DateRange},
        shared::ScopeHandle,
    },
    range_calendar::{
        RANGE_CALENDAR_PARTS, RangeCalendar, RangeCalendarCell, RangeCalendarDay,
        RangeCalendarGrid, RangeCalendarGridBody, RangeCalendarGridHead, RangeCalendarGridRow,
        RangeCalendarHeadCell, RangeCalendarHeader, RangeCalendarHeading,
        RangeCalendarMonthSelect, RangeCalendarNextButton, RangeCalendarPart,
        RangeCalendarPrevButton, RangeCalendarRelationships, RangeCalendarRoot,
        RangeCalendarSnapshot, RangeCalendarYearSelect, use_range_calendar_context,
    },
};

#[cfg(feature = "native")]
use crate::common;

fn d(y: i32, m: u8, day: u8) -> CalendarDate {
    CalendarDate::new(y, m, day).expect("valid test date")
}

#[test]
fn range_calendar_public_contracts_and_as_child() {
    assert_eq!(
        RANGE_CALENDAR_PARTS
            .iter()
            .map(RangeCalendarPart::as_str)
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

    let scope = ScopeHandle::root("range-calendar").child("contract");
    let rels = RangeCalendarRelationships::new(scope.clone());
    assert_eq!(rels.root_id(), scope.token());
    assert_eq!(
        rels.day_id(0, d(2024, 1, 10)),
        scope.qualify("day-0-2024-01-10")
    );

    let cell = monoxus::range_calendar::RangeCalendarCellAttrs::new(
        rels.cell_id(0, d(2024, 1, 10)),
        d(2024, 1, 10),
        true,
        false,
        false,
        false,
        false,
        false,
        true,
        true,
        false,
        true,
        true,
        false,
        true,
        true,
    );
    let day = monoxus::range_calendar::RangeCalendarDayAttrs::from_cell(
        rels.day_id(0, d(2024, 1, 10)),
        d(2024, 1, 10),
        &CalendarLocale::EnUs,
        &cell,
        true,
    );
    let (_, proj, _) = monoxus::range_calendar::project_range_calendar_day_as_child(
        &day,
        "div",
        Slottable::new("10"),
    );
    assert_eq!(proj, day);
}

#[component]
fn RangeCalendarContextProbe() -> Element {
    let ctx = use_range_calendar_context();
    let snap = ctx.snapshot();
    rsx! {
        RangeCalendarHeader {
            RangeCalendarPrevButton {}
            RangeCalendarHeading {}
            RangeCalendarNextButton {}
            RangeCalendarMonthSelect {}
            RangeCalendarYearSelect {}
        }
        for (m_idx, month) in snap.months.iter().enumerate() {
            RangeCalendarGrid {
                key: "{m_idx}",
                month_idx: Some(m_idx),
                RangeCalendarGridHead {
                    RangeCalendarGridRow {
                        for (w_idx, wd) in snap.weekdays.iter().enumerate() {
                            RangeCalendarHeadCell {
                                key: "{w_idx}",
                                "{wd}"
                            }
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
                                    RangeCalendarDay {}
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
fn RangeCalendarVdomSmokeApp() -> Element {
    rsx! {
        RangeCalendar {
            id: "vdom-rc",
            default_placeholder: d(2024, 1, 1),
            default_value: DateRange::new(Some(d(2024, 1, 5)), Some(d(2024, 1, 10))),
            min_days: Some(2),
            max_days: Some(14),
            exclude_disabled: true,
            RangeCalendarContextProbe {}
        }
    }
}

#[test]
fn range_calendar_vdom_mounts_cleanly() {
    let mut dom = VirtualDom::new(RangeCalendarVdomSmokeApp);
    dom.rebuild_in_place();
}

#[cfg(feature = "native")]
mod native_tests {
    use super::*;
    use common::native_fixture::*;
    use keyboard_types::{Code, Key, NamedKey};

    #[component]
    fn NativeRangeCalendarScenario() -> Element {
        let range_commits = use_signal(|| 0usize);
        let mut commits_sig = range_commits;

        rsx! {
            div {
                style: "padding: 16px;",
                RangeCalendarRoot {
                    id: "rc-main",
                    default_placeholder: d(2024, 1, 1),
                    on_range_select: move |_| commits_sig += 1,
                    render_children: move |snap: RangeCalendarSnapshot| rsx! {
                        div {
                            id: "rc-probe",
                            "data-start": snap.start_value.map(|x| x.to_iso_string()).unwrap_or_default(),
                            "data-end": snap.end_value.map(|x| x.to_iso_string()).unwrap_or_default(),
                            "data-commits": "{range_commits}",
                        }
                        RangeCalendarHeader {
                            id: "rc-header",
                            RangeCalendarPrevButton { id: "rc-prev", "Prev" }
                            RangeCalendarHeading { id: "rc-heading" }
                            RangeCalendarNextButton { id: "rc-next", "Next" }
                            RangeCalendarMonthSelect { id: "rc-month-sel" }
                        }
                        RangeCalendarGrid {
                            id: "rc-grid",
                            RangeCalendarGridHead {
                                RangeCalendarGridRow {
                                    for (w_idx, wd) in snap.weekdays.iter().enumerate() {
                                        RangeCalendarHeadCell { key: "{w_idx}", "{wd}" }
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
                                                id: "rc-cell-{date}",
                                                date,
                                                month: snap.months[0].value,
                                                RangeCalendarDay {
                                                    id: "rc-day-{date}",
                                                }
                                            }
                                        }
                                    }
                                }
                            }
                        }
                    }
                }

                // Constrained + readonly scenario
                RangeCalendarRoot {
                    id: "rc-constrained",
                    default_placeholder: d(2024, 1, 1),
                    min_days: Some(3),
                    max_days: Some(5),
                    exclude_disabled: true,
                    is_date_disabled: Callback::new(|date: CalendarDate| date == d(2024, 1, 15)),
                    render_children: move |snap: RangeCalendarSnapshot| rsx! {
                        div {
                            id: "rc-const-probe",
                            "data-start": snap.start_value.map(|x| x.to_iso_string()).unwrap_or_default(),
                            "data-end": snap.end_value.map(|x| x.to_iso_string()).unwrap_or_default(),
                        }
                        RangeCalendarGrid {
                            RangeCalendarGridBody {
                                for (r_idx, week) in snap.months[0].weeks.iter().enumerate() {
                                    RangeCalendarGridRow {
                                        key: "{r_idx}",
                                        for date in week.iter().copied() {
                                            RangeCalendarCell {
                                                key: "{date}",
                                                date,
                                                month: snap.months[0].value,
                                                RangeCalendarDay {
                                                    id: "rc2-day-{date}",
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
                    id: "rc-readonly",
                    default_placeholder: d(2024, 1, 1),
                    default_value: DateRange::new(Some(d(2024, 1, 5)), Some(d(2024, 1, 8))),
                    read_only: true,
                    render_children: move |snap: RangeCalendarSnapshot| rsx! {
                        div {
                            id: "rc-ro-probe",
                            "data-start": snap.start_value.map(|x| x.to_iso_string()).unwrap_or_default(),
                            "data-end": snap.end_value.map(|x| x.to_iso_string()).unwrap_or_default(),
                        }
                        RangeCalendarGrid {
                            RangeCalendarGridBody {
                                for (r_idx, week) in snap.months[0].weeks.iter().enumerate() {
                                    RangeCalendarGridRow {
                                        key: "{r_idx}",
                                        for date in week.iter().copied() {
                                            RangeCalendarCell {
                                                key: "{date}",
                                                date,
                                                month: snap.months[0].value,
                                                RangeCalendarDay {
                                                    id: "rc-ro-day-{date}",
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
    fn native_range_calendar_selection_highlight_constraints_escape_and_readonly() {
        let _lock = NATIVE_TEST_LOCK.lock().unwrap_or_else(|e| e.into_inner());
        let mut harness = mount_native_scenario(NativeRangeCalendarScenario);

        // Shared parts emit data-range-calendar-* attributes
        assert_eq!(
            harness.attr("#rc-header", "data-range-calendar-header").as_deref(),
            Some("")
        );
        assert_eq!(harness.attr("#rc-header", "data-calendar-header"), None);
        assert_eq!(
            harness.attr("#rc-month-sel", "data-range-calendar-month-select").as_deref(),
            Some("")
        );

        // Click Jan 10 -> start set, focus Jan 6 -> backward highlighted preview (Jan 6..Jan 10)
        click_and_pump(&mut harness, "#rc-day-2024-01-10");
        assert_eq!(
            harness.attr("#rc-probe", "data-start").as_deref(),
            Some("2024-01-10")
        );
        assert_eq!(
            harness.text_content("#rc-main [role='status']"),
            "Selected Date: January 10, 2024"
        );

        focus_node(&mut harness, "#rc-day-2024-01-06");
        for day in 6..=10 {
            let sel = format!("#rc-day-2024-01-{day:02}");
            assert_eq!(harness.attr(&sel, "data-highlighted").as_deref(), Some(""));
        }
        assert_eq!(harness.attr("#rc-day-2024-01-05", "data-highlighted"), None);

        // Escape clears partial start selection (5B D-1)
        send_key_and_pump(&mut harness, Key::Named(NamedKey::Escape), Code::Escape);
        assert_eq!(harness.attr("#rc-probe", "data-start").as_deref(), Some(""));
        assert_eq!(
            harness.text_content("#rc-main [role='status']"),
            "Selected date is now empty."
        );

        // Backward selection commits ordered range (Jan 10 -> Jan 6 => 2024-01-06..2024-01-10)
        click_and_pump(&mut harness, "#rc-day-2024-01-10");
        click_and_pump(&mut harness, "#rc-day-2024-01-06");
        assert_eq!(
            harness.attr("#rc-probe", "data-start").as_deref(),
            Some("2024-01-06")
        );
        assert_eq!(
            harness.attr("#rc-probe", "data-end").as_deref(),
            Some("2024-01-10")
        );
        assert_eq!(harness.attr("#rc-probe", "data-commits").as_deref(), Some("1"));
        assert_eq!(
            harness.attr("#rc-day-2024-01-06", "data-selection-start").as_deref(),
            Some("")
        );
        assert_eq!(
            harness.attr("#rc-day-2024-01-08", "data-range-middle").as_deref(),
            Some("")
        );
        assert_eq!(
            harness.attr("#rc-day-2024-01-10", "data-selection-end").as_deref(),
            Some("")
        );
        assert_eq!(
            harness.text_content("#rc-main [role='status']"),
            "Selected Dates: January 6, 2024 to January 10, 2024"
        );

        // Constrained calendar: min_days=3, max_days=5, exclude_disabled (Jan 15 disabled)
        click_and_pump(&mut harness, "#rc2-day-2024-01-05");
        // Jan 6 is only 2 days -> violates min_days=3, resets start to Jan 6
        click_and_pump(&mut harness, "#rc2-day-2024-01-06");
        assert_eq!(
            harness.attr("#rc-const-probe", "data-start").as_deref(),
            Some("2024-01-06")
        );
        assert_eq!(
            harness.attr("#rc-const-probe", "data-end").as_deref(),
            Some("")
        );
        // Jan 12 is 7 days -> violates max_days=5, resets start to Jan 12
        click_and_pump(&mut harness, "#rc2-day-2024-01-12");
        assert_eq!(
            harness.attr("#rc-const-probe", "data-start").as_deref(),
            Some("2024-01-12")
        );
        // Jan 16 spans disabled Jan 15 with exclude_disabled=true -> resets start to Jan 16
        click_and_pump(&mut harness, "#rc2-day-2024-01-16");
        assert_eq!(
            harness.attr("#rc-const-probe", "data-start").as_deref(),
            Some("2024-01-16")
        );
        assert_eq!(
            harness.attr("#rc-const-probe", "data-end").as_deref(),
            Some("")
        );
        // Jan 18 (3 days: 16,17,18) is valid!
        click_and_pump(&mut harness, "#rc2-day-2024-01-18");
        assert_eq!(
            harness.attr("#rc-const-probe", "data-start").as_deref(),
            Some("2024-01-16")
        );
        assert_eq!(
            harness.attr("#rc-const-probe", "data-end").as_deref(),
            Some("2024-01-18")
        );

        // Read-only guard (5B D-3): click & keyboard Enter do not mutate range, while ArrowRight moves focus
        click_and_pump(&mut harness, "#rc-ro-day-2024-01-12");
        assert_eq!(
            harness.attr("#rc-ro-probe", "data-start").as_deref(),
            Some("2024-01-05")
        );
        assert_eq!(
            harness.attr("#rc-ro-probe", "data-end").as_deref(),
            Some("2024-01-08")
        );
        focus_node(&mut harness, "#rc-ro-day-2024-01-05");
        send_key_and_pump(&mut harness, Key::Named(NamedKey::ArrowRight), Code::ArrowRight);
        assert_native_focus_id(&harness, "rc-ro-day-2024-01-06");
        send_key_and_pump(&mut harness, Key::Named(NamedKey::Enter), Code::Enter);
        assert_eq!(
            harness.attr("#rc-ro-probe", "data-start").as_deref(),
            Some("2024-01-05")
        );
        assert_eq!(
            harness.attr("#rc-ro-probe", "data-end").as_deref(),
            Some("2024-01-08")
        );
    }
}

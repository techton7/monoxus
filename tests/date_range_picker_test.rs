use std::{cell::RefCell, rc::Rc};

use dioxus::prelude::*;
use monoxus::{
    date_field::DateFieldSegmentItem,
    date_range_field::RangeFieldInputKind,
    date_range_picker::{
        DATE_RANGE_PICKER_OPEN_FOCUS_SELECTOR, DATE_RANGE_PICKER_PARTS, DateRangePicker,
        DateRangePickerArrow, DateRangePickerCalendar, DateRangePickerCell, DateRangePickerClose,
        DateRangePickerContent, DateRangePickerContentStatic, DateRangePickerDay,
        DateRangePickerGrid, DateRangePickerGridBody, DateRangePickerGridHead,
        DateRangePickerGridRow, DateRangePickerHeadCell, DateRangePickerHeader,
        DateRangePickerHeading, DateRangePickerInput, DateRangePickerLabel,
        DateRangePickerMonthSelect, DateRangePickerNextButton, DateRangePickerPart,
        DateRangePickerPortal, DateRangePickerPrevButton, DateRangePickerRelationships,
        DateRangePickerRoot, DateRangePickerSegment, DateRangePickerTrigger,
        DateRangePickerYearSelect, use_date_range_picker_context,
    },
    foundation::{
        date_time::{CalendarDate, CalendarLocale, CalendarMonth, DateValue, DateValueRange},
        overlay::{active_overlay_layers_snapshot, unregister_active_overlay_layer},
        shared::ScopeHandle,
    },
    popover::PopoverContext,
    range_calendar::RangeCalendarSnapshot,
};

fn d(y: i32, m: u8, day: u8) -> DateValue {
    DateValue::Date(CalendarDate::new(y, m, day).expect("valid date"))
}

fn clear_active_overlay_stack() {
    for layer_id in active_overlay_layers_snapshot() {
        unregister_active_overlay_layer(&layer_id);
    }
}

#[test]
fn date_range_picker_public_contracts_all_24_parts_and_vdom_smoke() {
    clear_active_overlay_stack();
    assert_eq!(DATE_RANGE_PICKER_PARTS.len(), 11);
    assert_eq!(DateRangePickerPart::Portal.as_str(), "portal");
    assert_eq!(DateRangePickerPart::ContentStatic.as_str(), "content-static");
    assert_eq!(
        DATE_RANGE_PICKER_OPEN_FOCUS_SELECTOR,
        "[data-range-calendar-day][data-focused]"
    );

    let scope = ScopeHandle::root("date-range-picker").child("contract");
    let rels = DateRangePickerRelationships::new(scope.clone());
    assert_eq!(rels.root_id(), scope.token());
    assert_eq!(
        rels.open_focus_selector(),
        DATE_RANGE_PICKER_OPEN_FOCUS_SELECTOR
    );

    #[component]
    fn DateRangePickerVdomApp() -> Element {
        rsx! {
            DateRangePicker {
                id: "vdom-drp",
                default_value: DateValueRange::new(Some(d(2026, 4, 10)), Some(d(2026, 4, 15))),
                default_open: true,
                DateRangePickerLabel { "Stay" }
                DateRangePickerInput {
                    kind: RangeFieldInputKind::Start,
                    name: Some("check_in".to_string()),
                    render_children: move |segments: Vec<DateFieldSegmentItem>| rsx! {
                        for seg in segments {
                            DateRangePickerSegment {
                                key: "{seg.index}",
                                part: seg.kind(),
                                index: Some(seg.index),
                            }
                        }
                    }
                }
                DateRangePickerInput {
                    kind: RangeFieldInputKind::End,
                    name: Some("check_out".to_string()),
                    render_children: move |segments: Vec<DateFieldSegmentItem>| rsx! {
                        for seg in segments {
                            DateRangePickerSegment {
                                key: "{seg.index}",
                                part: seg.kind(),
                                index: Some(seg.index),
                            }
                        }
                    }
                }
                DateRangePickerTrigger { "Open" }
                DateRangePickerPortal {
                    DateRangePickerContent {
                        DateRangePickerArrow {}
                        DateRangePickerClose { "Close" }
                        DateRangePickerCalendar {
                            render_children: move |snap: RangeCalendarSnapshot| rsx! {
                                DateRangePickerHeader {
                                    DateRangePickerPrevButton { "<" }
                                    DateRangePickerHeading {}
                                    DateRangePickerMonthSelect {}
                                    DateRangePickerYearSelect {}
                                    DateRangePickerNextButton { ">" }
                                }
                                for month in snap.months {
                                    DateRangePickerGrid {
                                        key: "{month.value}",
                                        DateRangePickerGridHead {
                                            DateRangePickerGridRow {
                                                for (w_idx, weekday) in snap.weekdays.iter().enumerate() {
                                                    DateRangePickerHeadCell { key: "{w_idx}", "{weekday}" }
                                                }
                                            }
                                        }
                                        DateRangePickerGridBody {
                                            for (w_idx, week) in month.weeks.iter().enumerate() {
                                                DateRangePickerGridRow {
                                                    key: "{w_idx}",
                                                    for date in week.iter().copied() {
                                                        DateRangePickerCell {
                                                            key: "{date}",
                                                            date,
                                                            month: month.value,
                                                            DateRangePickerDay {}
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
            DateRangePickerRoot {
                id: "vdom-drp-static",
                default_open: true,
                DateRangePickerTrigger { "Static" }
                DateRangePickerContentStatic { "Static range calendar host" }
            }
        }
    }

    let mut dom = VirtualDom::new(DateRangePickerVdomApp);
    dom.rebuild_in_place();
    drop(dom);
    clear_active_overlay_stack();
}

#[derive(Clone, Default, PartialEq)]
struct RangePickerIdProbe {
    root_id: String,
    popover_content_id: String,
}

#[component]
fn CaptureRangePickerIds(out: Rc<RefCell<RangePickerIdProbe>>) -> Element {
    let picker = use_date_range_picker_context();
    let popover = use_context::<PopoverContext>();
    *out.borrow_mut() = RangePickerIdProbe {
        root_id: picker.rels.root_id().to_string(),
        popover_content_id: popover.relationships.content_id().to_string(),
    };
    rsx! {}
}

#[component]
fn MultiRangePickerIdsHarness(
    first: Rc<RefCell<RangePickerIdProbe>>,
    second: Rc<RefCell<RangePickerIdProbe>>,
) -> Element {
    rsx! {
        DateRangePickerRoot {
            DateRangePickerTrigger { "First" }
            CaptureRangePickerIds { out: first }
        }
        DateRangePickerRoot {
            DateRangePickerTrigger { "Second" }
            CaptureRangePickerIds { out: second }
        }
    }
}

#[test]
fn multi_date_range_picker_generates_unique_stable_ids() {
    clear_active_overlay_stack();
    let first = Rc::new(RefCell::new(RangePickerIdProbe::default()));
    let second = Rc::new(RefCell::new(RangePickerIdProbe::default()));

    let mut dom = VirtualDom::new_with_props(
        MultiRangePickerIdsHarness,
        MultiRangePickerIdsHarnessProps {
            first: first.clone(),
            second: second.clone(),
        },
    );
    dom.rebuild_in_place();

    assert_ne!(first.borrow().root_id, second.borrow().root_id);
    assert_ne!(
        first.borrow().popover_content_id,
        second.borrow().popover_content_id
    );

    drop(dom);
    clear_active_overlay_stack();
}

#[cfg(feature = "native")]
mod native_tests {
    use keyboard_types::{Code, Key, NamedKey};

    use super::*;
    use crate::common::native_fixture::{
        NATIVE_TEST_LOCK, assert_native_focus_id, click_and_pump, mount_native_scenario,
        send_key_and_pump,
    };

    #[component]
    fn NativeDateRangePickerScenario() -> Element {
        rsx! {
            div {
                style: "padding: 16px;",
                DateRangePickerRoot {
                    id: "native-drp",
                    locale: CalendarLocale::EnUs,
                    default_placeholder: d(2026, 4, 15),
                    close_on_range_select: true,
                    DateRangePickerLabel {
                        id: "drp-label",
                        style: "display: block; width: 120px; height: 24px;",
                        "Booking Dates"
                    }
                    div {
                        style: "display: flex; align-items: center; gap: 8px;",
                        DateRangePickerInput {
                            kind: RangeFieldInputKind::Start,
                            id: "drp-start-input",
                            name: Some("start_date".to_string()),
                            style: "display: flex; gap: 4px;",
                            render_children: move |segments: Vec<DateFieldSegmentItem>| rsx! {
                                for seg in segments {
                                    DateRangePickerSegment {
                                        key: "{seg.index}",
                                        id: format!("drp-start-seg-{}", seg.index),
                                        part: seg.kind(),
                                        index: Some(seg.index),
                                        style: "display: inline-block; min-width: 24px; height: 24px;",
                                    }
                                }
                            }
                        }
                        DateRangePickerInput {
                            kind: RangeFieldInputKind::End,
                            id: "drp-end-input",
                            name: Some("end_date".to_string()),
                            style: "display: flex; gap: 4px;",
                            render_children: move |segments: Vec<DateFieldSegmentItem>| rsx! {
                                for seg in segments {
                                    DateRangePickerSegment {
                                        key: "{seg.index}",
                                        id: format!("drp-end-seg-{}", seg.index),
                                        part: seg.kind(),
                                        index: Some(seg.index),
                                        style: "display: inline-block; min-width: 24px; height: 24px;",
                                    }
                                }
                            }
                        }
                        DateRangePickerTrigger {
                            style: "width: 80px; height: 28px;",
                            "Calendar"
                        }
                    }
                    DateRangePickerPortal {
                        DateRangePickerContent {
                            id: "native-drp-custom-content".to_string(),
                            style: "width: 280px; padding: 8px; background: white;",
                            DateRangePickerCalendar {
                                render_children: move |snap: RangeCalendarSnapshot| rsx! {
                                    DateRangePickerHeader {
                                        DateRangePickerHeading {}
                                    }
                                    for month in snap.months {
                                        DateRangePickerMonthGrid { month, weekdays: snap.weekdays.clone() }
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
    fn DateRangePickerMonthGrid(month: CalendarMonth, weekdays: Vec<String>) -> Element {
        rsx! {
            DateRangePickerGrid {
                DateRangePickerGridHead {
                    DateRangePickerGridRow {
                        for (w_idx, weekday) in weekdays.iter().enumerate() {
                            DateRangePickerHeadCell { key: "{w_idx}", "{weekday}" }
                        }
                    }
                }
                DateRangePickerGridBody {
                    for (w_idx, week) in month.weeks.iter().enumerate() {
                        DateRangePickerGridRow {
                            key: "{w_idx}",
                            for date in week.iter().copied() {
                                DateRangePickerCell {
                                    key: "{date}",
                                    date,
                                    month: month.value,
                                    DateRangePickerDay {
                                        style: "display: inline-block; width: 28px; height: 28px;",
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
    fn native_date_range_picker_partial_escape_keeps_open_and_complete_range_closes_and_syncs() {
        let _lock = NATIVE_TEST_LOCK.lock().unwrap_or_else(|e| e.into_inner());
        clear_active_overlay_stack();
        let mut harness = mount_native_scenario(NativeDateRangePickerScenario);

        let trigger_id = harness
            .attr("[data-segment='trigger']", "id")
            .expect("trigger must have an id");

        // Open DateRangePicker popover
        click_and_pump(&mut harness, "[data-segment='trigger']");
        assert_eq!(
            harness
                .attr("[data-date-range-picker-content]", "data-state")
                .as_deref(),
            Some("open")
        );

        // Click partial start date (2026-04-10) -> popover stays OPEN!
        click_and_pump(
            &mut harness,
            "[data-range-calendar-day][data-value='2026-04-10']",
        );
        assert_eq!(
            harness
                .attr("[data-date-range-picker-content]", "data-state")
                .as_deref(),
            Some("open")
        );
        assert_eq!(
            harness
                .attr(
                    "[data-range-calendar-day][data-value='2026-04-10']",
                    "data-selection-start"
                )
                .as_deref(),
            Some("")
        );

        // First Escape clears partial selection while keeping DateRangePickerContent OPEN!
        send_key_and_pump(&mut harness, Key::Named(NamedKey::Escape), Code::Escape);
        assert_eq!(
            harness
                .attr("[data-date-range-picker-content]", "data-state")
                .as_deref(),
            Some("open")
        );
        assert_eq!(
            harness.attr(
                "[data-range-calendar-day][data-value='2026-04-10']",
                "data-selection-start"
            ),
            None
        );

        // Second Escape closes the popover and restores focus to DateRangePickerTrigger!
        send_key_and_pump(&mut harness, Key::Named(NamedKey::Escape), Code::Escape);
        assert_eq!(
            harness
                .attr("[data-segment='trigger']", "aria-expanded")
                .as_deref(),
            Some("false")
        );
        assert_native_focus_id(&harness, &trigger_id);

        // Re-open and select complete range: 2026-04-10 -> 2026-04-16
        click_and_pump(&mut harness, "[data-segment='trigger']");
        click_and_pump(
            &mut harness,
            "[data-range-calendar-day][data-value='2026-04-10']",
        );
        assert_eq!(
            harness
                .attr("[data-date-range-picker-content]", "data-state")
                .as_deref(),
            Some("open")
        );
        click_and_pump(
            &mut harness,
            "[data-range-calendar-day][data-value='2026-04-16']",
        );

        // Complete range closes popover, syncs Start & End fields + hidden inputs, restores focus
        assert_eq!(
            harness
                .attr("[data-segment='trigger']", "aria-expanded")
                .as_deref(),
            Some("false")
        );
        assert_eq!(
            harness.attr("input[name='start_date']", "value").as_deref(),
            Some("2026-04-10")
        );
        assert_eq!(
            harness.attr("input[name='end_date']", "value").as_deref(),
            Some("2026-04-16")
        );
        assert_eq!(harness.text_content("#drp-start-seg-0"), "04");
        assert_eq!(harness.text_content("#drp-start-seg-2"), "10");
        assert_eq!(harness.text_content("#drp-end-seg-0"), "04");
        assert_eq!(harness.text_content("#drp-end-seg-2"), "16");
        assert_native_focus_id(&harness, &trigger_id);

        clear_active_overlay_stack();
    }
}

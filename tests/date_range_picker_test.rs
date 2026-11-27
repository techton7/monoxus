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
        date_time::{
            CalendarDate, CalendarLocale, CalendarMonth, DateValue, DateValueRange,
            FieldInvalidEvent, FieldInvalidReason, MonthFormat,
        },
        overlay::{active_overlay_layers_snapshot, unregister_active_overlay_layer},
        shared::ScopeHandle,
    },
    popover::PopoverContext,
    range_calendar::{RangeCalendarSnapshot, use_range_calendar_context},
    select::{
        SelectContent, SelectItem, SelectItemData, SelectPortal, SelectRoot, SelectTrigger,
        SelectValue,
    },
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

    #[component]
    fn NativeDateRangePickerRemediationScenario() -> Element {
        rsx! {
            div {
                style: "padding: 280px 16px 16px 16px;",
                form {
                    id: "drp-remediation-form",
                    DateRangePickerRoot {
                        id: "drp-remediation",
                        locale: CalendarLocale::EnUs,
                        default_value: DateValueRange::new(Some(d(2026, 4, 10)), Some(d(2026, 4, 15))),
                        default_placeholder: d(2026, 4, 10),
                        is_date_unavailable: move |date: CalendarDate| date.day == 22,
                        div {
                            style: "display: flex; align-items: center; gap: 8px;",
                            DateRangePickerInput {
                                kind: RangeFieldInputKind::Start,
                                id: "drp-rem-start-input",
                                name: Some("rem_start".to_string()),
                                style: "display: flex; gap: 4px;",
                                render_children: move |segments: Vec<DateFieldSegmentItem>| rsx! {
                                    for seg in segments {
                                        DateRangePickerSegment {
                                            key: "{seg.index}",
                                            id: format!("drp-rem-start-seg-{}", seg.index),
                                            part: seg.kind(),
                                            index: Some(seg.index),
                                            style: "display: inline-block; min-width: 24px; height: 24px;",
                                        }
                                    }
                                }
                            }
                            DateRangePickerInput {
                                kind: RangeFieldInputKind::End,
                                id: "drp-rem-end-input",
                                name: Some("rem_end".to_string()),
                                style: "display: flex; gap: 4px;",
                                render_children: move |segments: Vec<DateFieldSegmentItem>| rsx! {
                                    for seg in segments {
                                        DateRangePickerSegment {
                                            key: "{seg.index}",
                                            id: format!("drp-rem-end-seg-{}", seg.index),
                                            part: seg.kind(),
                                            index: Some(seg.index),
                                            style: "display: inline-block; min-width: 24px; height: 24px;",
                                        }
                                    }
                                }
                            }
                            DateRangePickerTrigger {
                                id: "drp-rem-trigger",
                                style: "width: 80px; height: 38px;",
                                "Open"
                            }
                            button {
                                id: "drp-rem-reset",
                                r#type: "reset",
                                style: "width: 60px; height: 30px;",
                                "Reset"
                            }
                        }
                        DateRangePickerPortal {
                            DateRangePickerContent {
                                id: "drp-rem-content",
                                collision_padding: 8.0,
                                avoid_collisions: true,
                                style: "width: 280px; height: 340px; padding: 8px; background: white;",
                                DateRangePickerCalendar {
                                    render_children: move |snap: RangeCalendarSnapshot| rsx! {
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
    }

    #[test]
    fn native_date_range_picker_clamps_viewport_validates_unavailable_and_resets_form() {
        let _lock = NATIVE_TEST_LOCK.lock().unwrap_or_else(|e| e.into_inner());
        clear_active_overlay_stack();
        let mut harness = mount_native_scenario(NativeDateRangePickerRemediationScenario);

        // 1. Open mid-viewport (y=280) 340px-tall DateRangePickerContent on 800x600 -> clamped y >= 0.0 (DEF-5C-04)
        click_and_pump(&mut harness, "#drp-rem-trigger");
        assert_eq!(
            harness.attr("#drp-rem-content", "data-state").as_deref(),
            Some("open")
        );
        let content_rect = harness.layout_rect("#drp-rem-content");
        assert!(
            content_rect.y >= 0.0 && content_rect.y + content_rect.height <= 600.0,
            "DateRangePickerContent must clamp inside 800x600 viewport, got rect={content_rect:?}"
        );
        send_key_and_pump(&mut harness, Key::Named(NamedKey::Escape), Code::Escape);

        // 2. Typing an unavailable end date (day 22) marks DateRangeFieldRoot invalid (DEF-5C-11)
        crate::common::native_fixture::focus_node(&mut harness, "#drp-rem-end-seg-2");
        send_key_and_pump(&mut harness, Key::Character("2".to_string()), Code::Digit2);
        send_key_and_pump(&mut harness, Key::Character("2".to_string()), Code::Digit2);
        assert_eq!(
            harness.attr("input[name='rem_end']", "value").as_deref(),
            Some("2026-04-22")
        );
        assert_eq!(
            harness.attr("[data-date-range-field-root]", "data-invalid").as_deref(),
            Some("")
        );

        // 3. Native <form> reset restores default_value (2026-04-10..2026-04-15) (DEF-5C-08)
        click_and_pump(&mut harness, "#drp-rem-reset");
        assert_eq!(
            harness.attr("input[name='rem_start']", "value").as_deref(),
            Some("2026-04-10")
        );
        assert_eq!(
            harness.attr("input[name='rem_end']", "value").as_deref(),
            Some("2026-04-15")
        );
        assert_eq!(harness.attr("[data-date-range-field-root]", "data-invalid"), None);

        clear_active_overlay_stack();
    }

    #[component]
    fn NativeDateRangePickerPartialUnavailableScenario() -> Element {
        let mut last_invalid_reason = use_signal(|| Option::<FieldInvalidReason>::None);

        rsx! {
            div {
                style: "padding: 16px;",
                DateRangePickerRoot {
                    id: "drp-partial-unavail",
                    locale: CalendarLocale::EnUs,
                    default_placeholder: d(2026, 4, 15),
                    is_date_unavailable: move |date: CalendarDate| date.day == 22,
                    on_invalid: move |evt: FieldInvalidEvent| {
                        last_invalid_reason.set(Some(evt.reason));
                    },
                    div {
                        style: "display: flex; align-items: center; gap: 8px;",
                        DateRangePickerInput {
                            kind: RangeFieldInputKind::Start,
                            id: "drp-pu-start-input",
                            name: Some("pu_start".to_string()),
                            style: "display: flex; gap: 4px;",
                            render_children: move |segments: Vec<DateFieldSegmentItem>| rsx! {
                                for seg in segments {
                                    DateRangePickerSegment {
                                        key: "{seg.index}",
                                        id: format!("drp-pu-start-seg-{}", seg.index),
                                        part: seg.kind(),
                                        index: Some(seg.index),
                                        style: "display: inline-block; min-width: 24px; height: 24px;",
                                    }
                                }
                            }
                        }
                        DateRangePickerInput {
                            kind: RangeFieldInputKind::End,
                            id: "drp-pu-end-input",
                            name: Some("pu_end".to_string()),
                            style: "display: flex; gap: 4px;",
                            render_children: move |segments: Vec<DateFieldSegmentItem>| rsx! {
                                for seg in segments {
                                    DateRangePickerSegment {
                                        key: "{seg.index}",
                                        id: format!("drp-pu-end-seg-{}", seg.index),
                                        part: seg.kind(),
                                        index: Some(seg.index),
                                        style: "display: inline-block; min-width: 24px; height: 24px;",
                                    }
                                }
                            }
                        }
                    }
                }
                span {
                    id: "drp-pu-invalid-reason",
                    "{last_invalid_reason.read():?}"
                }
            }
        }
    }

    #[test]
    fn native_date_range_picker_partial_unavailable_start_emits_unavailable() {
        let _lock = NATIVE_TEST_LOCK.lock().unwrap_or_else(|e| e.into_inner());
        clear_active_overlay_stack();
        let mut harness = mount_native_scenario(NativeDateRangePickerPartialUnavailableScenario);

        // Type unavailable date 2026-04-22 into Start while End remains empty
        crate::common::native_fixture::focus_node(&mut harness, "#drp-pu-start-seg-0");
        send_key_and_pump(&mut harness, Key::Character("4".to_string()), Code::Digit4);
        send_key_and_pump(&mut harness, Key::Character("2".to_string()), Code::Digit2);
        send_key_and_pump(&mut harness, Key::Character("2".to_string()), Code::Digit2);
        send_key_and_pump(&mut harness, Key::Character("2".to_string()), Code::Digit2);
        send_key_and_pump(&mut harness, Key::Character("0".to_string()), Code::Digit0);
        send_key_and_pump(&mut harness, Key::Character("2".to_string()), Code::Digit2);
        send_key_and_pump(&mut harness, Key::Character("6".to_string()), Code::Digit6);

        assert_eq!(
            harness.attr("input[name='pu_start']", "value").as_deref(),
            Some("2026-04-22")
        );
        assert_eq!(
            harness.attr("input[name='pu_end']", "value").as_deref(),
            Some("")
        );
        assert_eq!(
            harness
                .attr("[data-date-range-field-root]", "data-invalid")
                .as_deref(),
            Some("")
        );
        assert_eq!(
            harness.attr("#drp-pu-start-input", "data-invalid").as_deref(),
            Some("")
        );
        assert_eq!(harness.attr("#drp-pu-end-input", "data-invalid"), None);
        assert_eq!(
            harness.text_content("#drp-pu-invalid-reason"),
            "Some(Unavailable)"
        );

        clear_active_overlay_stack();
    }

    #[component]
    fn NativeDateRangePickerThreeStageEscapeScenario() -> Element {
        rsx! {
            div {
                style: "padding: 16px;",
                DateRangePickerRoot {
                    id: "drp-3esc",
                    locale: CalendarLocale::EnUs,
                    default_placeholder: d(2026, 4, 15),
                    close_on_range_select: true,
                    div {
                        style: "display: flex; align-items: center; gap: 8px;",
                        DateRangePickerInput {
                            kind: RangeFieldInputKind::Start,
                            id: "drp-3esc-start",
                            name: Some("esc3_start".to_string()),
                            style: "display: flex; gap: 4px;",
                            render_children: move |segments: Vec<DateFieldSegmentItem>| rsx! {
                                for seg in segments {
                                    DateRangePickerSegment {
                                        key: "{seg.index}",
                                        part: seg.kind(),
                                        index: Some(seg.index),
                                        style: "display: inline-block; min-width: 24px; height: 24px;",
                                    }
                                }
                            }
                        }
                        DateRangePickerInput {
                            kind: RangeFieldInputKind::End,
                            id: "drp-3esc-end",
                            name: Some("esc3_end".to_string()),
                            style: "display: flex; gap: 4px;",
                            render_children: move |segments: Vec<DateFieldSegmentItem>| rsx! {
                                for seg in segments {
                                    DateRangePickerSegment {
                                        key: "{seg.index}",
                                        part: seg.kind(),
                                        index: Some(seg.index),
                                        style: "display: inline-block; min-width: 24px; height: 24px;",
                                    }
                                }
                            }
                        }
                        DateRangePickerTrigger {
                            id: "drp-3esc-trigger",
                            style: "width: 80px; height: 28px;",
                            "Calendar"
                        }
                    }
                    DateRangePickerPortal {
                        DateRangePickerContent {
                            id: "drp-3esc-content",
                            style: "width: 280px; padding: 8px; background: white;",
                            DateRangePickerCalendar {
                                render_children: move |snap: RangeCalendarSnapshot| rsx! {
                                    DateRangePickerHeader {
                                        style: "display: flex; gap: 4px; margin-bottom: 8px;",
                                        DateRangePickerMonthSelect {
                                            child: move |_props: Vec<Attribute>| rsx! {
                                                DateRangePickerCustomMonthSelect {}
                                            },
                                        }
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
    fn DateRangePickerCustomMonthSelect() -> Element {
        let mut ctx = use_range_calendar_context();
        let current_val = ctx.current_month().to_string();
        let month_opts = ctx.month_options(MonthFormat::Long);
        let items: Vec<SelectItemData> = month_opts
            .iter()
            .map(|o| SelectItemData::new(o.value_string(), o.label.clone(), false))
            .collect();
        let selected_label = month_opts
            .iter()
            .find(|o| o.value_string() == current_val)
            .map(|o| o.label.clone())
            .unwrap_or_default();
        rsx! {
            SelectRoot {
                id: "drp-3esc-month-select",
                value: Some(current_val),
                items: Some(items),
                on_value_change: move |next: Option<String>| {
                    if let Some(s) = next
                        && let Ok(m) = s.parse::<u8>()
                    {
                        ctx.set_month(m);
                    }
                },
                SelectTrigger {
                    style: "width: 100px; height: 28px;",
                    SelectValue { placeholder: selected_label }
                }
                SelectPortal {
                    SelectContent {
                        style: "width: 120px; background: white;",
                        for opt in month_opts {
                            SelectItem {
                                key: "{opt.value}",
                                value: opt.value_string(),
                                style: "display: block; height: 24px;",
                                "{opt.label}"
                            }
                        }
                    }
                }
            }
        }
    }

    #[test]
    fn native_date_range_picker_three_stage_escape_with_nested_month_select() {
        let _lock = NATIVE_TEST_LOCK.lock().unwrap_or_else(|e| e.into_inner());
        clear_active_overlay_stack();
        let mut harness = mount_native_scenario(NativeDateRangePickerThreeStageEscapeScenario);

        // Open picker -> pick partial start (2026-04-10) -> open MonthSelect
        click_and_pump(&mut harness, "#drp-3esc-trigger");
        assert_eq!(
            harness.attr("#drp-3esc-content", "data-state").as_deref(),
            Some("open")
        );
        click_and_pump(
            &mut harness,
            "[data-range-calendar-day][data-value='2026-04-10']",
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
        click_and_pump(&mut harness, "[role='combobox']");
        assert_eq!(
            harness.attr("[role='listbox']", "data-state").as_deref(),
            Some("open")
        );

        // 1st Escape: closes MonthSelect ONLY (picker open, partial start intact)
        send_key_and_pump(&mut harness, Key::Named(NamedKey::Escape), Code::Escape);
        assert_eq!(
            harness.attr("[role='combobox']", "aria-expanded").as_deref(),
            Some("false")
        );
        assert_eq!(
            harness.attr("#drp-3esc-content", "data-state").as_deref(),
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

        // 2nd Escape: clears partial start (picker still open)
        send_key_and_pump(&mut harness, Key::Named(NamedKey::Escape), Code::Escape);
        assert_eq!(
            harness.attr("#drp-3esc-content", "data-state").as_deref(),
            Some("open")
        );
        assert_eq!(
            harness.attr(
                "[data-range-calendar-day][data-value='2026-04-10']",
                "data-selection-start"
            ),
            None
        );

        // 3rd Escape: closes picker and restores focus to trigger
        send_key_and_pump(&mut harness, Key::Named(NamedKey::Escape), Code::Escape);
        assert_eq!(
            harness.attr("#drp-3esc-trigger", "aria-expanded").as_deref(),
            Some("false")
        );
        assert_native_focus_id(&harness, "drp-3esc-trigger");

        clear_active_overlay_stack();
    }

    #[component]
    fn NativeDateRangePickerConstrainedStayOpenScenario() -> Element {
        rsx! {
            div {
                style: "padding: 16px;",
                DateRangePickerRoot {
                    id: "drp-constrained",
                    locale: CalendarLocale::EnUs,
                    default_placeholder: d(2026, 4, 15),
                    close_on_range_select: false,
                    min_days: Some(3),
                    max_days: Some(5),
                    exclude_disabled: true,
                    is_date_disabled: move |date: CalendarDate| date.day == 15,
                    div {
                        style: "display: flex; align-items: center; gap: 8px;",
                        DateRangePickerInput {
                            kind: RangeFieldInputKind::Start,
                            id: "drp-con-start",
                            name: Some("con_start".to_string()),
                            style: "display: flex; gap: 4px;",
                            render_children: move |segments: Vec<DateFieldSegmentItem>| rsx! {
                                for seg in segments {
                                    DateRangePickerSegment {
                                        key: "{seg.index}",
                                        part: seg.kind(),
                                        index: Some(seg.index),
                                        style: "display: inline-block; min-width: 24px; height: 24px;",
                                    }
                                }
                            }
                        }
                        DateRangePickerInput {
                            kind: RangeFieldInputKind::End,
                            id: "drp-con-end",
                            name: Some("con_end".to_string()),
                            style: "display: flex; gap: 4px;",
                            render_children: move |segments: Vec<DateFieldSegmentItem>| rsx! {
                                for seg in segments {
                                    DateRangePickerSegment {
                                        key: "{seg.index}",
                                        part: seg.kind(),
                                        index: Some(seg.index),
                                        style: "display: inline-block; min-width: 24px; height: 24px;",
                                    }
                                }
                            }
                        }
                        DateRangePickerTrigger {
                            id: "drp-con-trigger",
                            style: "width: 80px; height: 28px;",
                            "Calendar"
                        }
                    }
                    DateRangePickerPortal {
                        DateRangePickerContent {
                            id: "drp-con-content",
                            style: "width: 280px; padding: 8px; background: white;",
                            DateRangePickerCalendar {
                                render_children: move |snap: RangeCalendarSnapshot| rsx! {
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

    #[test]
    fn native_date_range_picker_close_on_range_select_false_and_constraints() {
        let _lock = NATIVE_TEST_LOCK.lock().unwrap_or_else(|e| e.into_inner());
        clear_active_overlay_stack();
        let mut harness = mount_native_scenario(NativeDateRangePickerConstrainedStayOpenScenario);

        click_and_pump(&mut harness, "#drp-con-trigger");
        assert_eq!(
            harness.attr("#drp-con-content", "data-state").as_deref(),
            Some("open")
        );

        // min_days: 3 rejects 2-day span (Apr 10 -> Apr 11); anchor resets to Apr 11
        click_and_pump(
            &mut harness,
            "[data-range-calendar-day][data-value='2026-04-10']",
        );
        click_and_pump(
            &mut harness,
            "[data-range-calendar-day][data-value='2026-04-11']",
        );
        assert_eq!(
            harness.attr("input[name='con_end']", "value").as_deref(),
            Some("")
        );

        // max_days: 5 rejects 7-day span (Apr 11 -> Apr 17); anchor resets to Apr 17
        click_and_pump(
            &mut harness,
            "[data-range-calendar-day][data-value='2026-04-17']",
        );
        assert_eq!(
            harness.attr("input[name='con_end']", "value").as_deref(),
            Some("")
        );

        // exclude_disabled: true rejects 4-day span crossing disabled Apr 15 (Apr 17 -> Apr 14); anchor resets to Apr 14
        click_and_pump(
            &mut harness,
            "[data-range-calendar-day][data-value='2026-04-14']",
        );
        assert_eq!(
            harness.attr("input[name='con_end']", "value").as_deref(),
            Some("")
        );

        // Valid 5-day span without disabled dates (Apr 14 -> Apr 10) completes range AND stays open (close_on_range_select: false)
        click_and_pump(
            &mut harness,
            "[data-range-calendar-day][data-value='2026-04-10']",
        );
        assert_eq!(
            harness.attr("input[name='con_start']", "value").as_deref(),
            Some("2026-04-10")
        );
        assert_eq!(
            harness.attr("input[name='con_end']", "value").as_deref(),
            Some("2026-04-14")
        );
        assert_eq!(
            harness.attr("#drp-con-content", "data-state").as_deref(),
            Some("open")
        );
        assert_eq!(
            harness.attr("#drp-con-trigger", "aria-expanded").as_deref(),
            Some("true")
        );

        clear_active_overlay_stack();
    }

    #[component]
    fn NativeDateRangePickerStaticAndOpenFocusScenario() -> Element {
        rsx! {
            div {
                style: "padding: 16px;",
                DateRangePickerRoot {
                    id: "drp-fm-static-root",
                    locale: CalendarLocale::EnUs,
                    default_placeholder: d(2026, 4, 15),
                    close_on_range_select: true,
                    div {
                        style: "display: flex; align-items: center; gap: 8px;",
                        DateRangePickerInput {
                            kind: RangeFieldInputKind::Start,
                            id: "drp-fm-static-start",
                            name: Some("fm_static_start".to_string()),
                            style: "display: flex; gap: 4px;",
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
                            id: "drp-fm-static-end",
                            name: Some("fm_static_end".to_string()),
                            style: "display: flex; gap: 4px;",
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
                        DateRangePickerTrigger {
                            id: "drp-fm-static-trigger",
                            style: "width: 80px; height: 28px;",
                            "Static Range"
                        }
                    }
                    DateRangePickerContentStatic {
                        id: "drp-fm-static-content",
                        style: "width: 280px; padding: 8px; background: white;",
                        on_open_auto_focus: move |evt: monoxus::date_range_picker::OpenAutoFocusEvent| {
                            evt.prevent_default();
                        },
                        DateRangePickerCalendar {
                            render_children: move |snap: RangeCalendarSnapshot| rsx! {
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

    #[test]
    fn native_date_range_picker_fm_dt1_static_and_fm_dt2_open_focus_and_complete_close() {
        let _lock = NATIVE_TEST_LOCK.lock().unwrap_or_else(|e| e.into_inner());
        clear_active_overlay_stack();
        let mut harness = mount_native_scenario(NativeDateRangePickerStaticAndOpenFocusScenario);

        // FM-DT-1 + FM-DT-2 (bits#2219): static content has no floating wrapper, emits data-date-range-picker-content,
        // and honors on_open_auto_focus.prevent_default()
        crate::common::native_fixture::focus_node(&mut harness, "#drp-fm-static-trigger");
        click_and_pump(&mut harness, "#drp-fm-static-trigger");
        assert_eq!(
            harness
                .attr("#drp-fm-static-content", "data-date-range-picker-content")
                .as_deref(),
            Some("")
        );
        assert!(harness.query("#drp-fm-static-content-wrapper").is_none());
        assert_native_focus_id(&harness, "drp-fm-static-trigger");

        // Partial range selection leaves popover open; 1st Escape clears partial start, 2nd Escape closes
        click_and_pump(
            &mut harness,
            "[data-range-calendar-day][data-value='2026-04-10']",
        );
        assert_eq!(
            harness
                .attr("#drp-fm-static-content", "data-state")
                .as_deref(),
            Some("open")
        );
        send_key_and_pump(&mut harness, Key::Named(NamedKey::Escape), Code::Escape);
        assert_eq!(
            harness
                .attr("#drp-fm-static-content", "data-state")
                .as_deref(),
            Some("open")
        );
        send_key_and_pump(&mut harness, Key::Named(NamedKey::Escape), Code::Escape);
        assert_eq!(
            harness
                .attr("#drp-fm-static-trigger", "aria-expanded")
                .as_deref(),
            Some("false")
        );

        // Complete range closes popover when close_on_range_select = true
        click_and_pump(&mut harness, "#drp-fm-static-trigger");
        click_and_pump(
            &mut harness,
            "[data-range-calendar-day][data-value='2026-04-10']",
        );
        click_and_pump(
            &mut harness,
            "[data-range-calendar-day][data-value='2026-04-14']",
        );
        assert_eq!(
            harness
                .attr("input[name='fm_static_start']", "value")
                .as_deref(),
            Some("2026-04-10")
        );
        assert_eq!(
            harness
                .attr("input[name='fm_static_end']", "value")
                .as_deref(),
            Some("2026-04-14")
        );
        assert_eq!(
            harness
                .attr("#drp-fm-static-trigger", "aria-expanded")
                .as_deref(),
            Some("false")
        );

        clear_active_overlay_stack();
    }

    #[component]
    fn NativeDateRangePickerReactivePredicateScenario() -> Element {
        let mut pred_stage = use_signal(|| 0_u8);
        let mut last_invalid_reason = use_signal(|| Option::<FieldInvalidReason>::None);
        let stage = *pred_stage.read();
        let is_date_unavailable = if stage == 0 {
            Some(Callback::new(|date: CalendarDate| date.day == 10))
        } else {
            Some(Callback::new(|date: CalendarDate| date.day == 22))
        };

        rsx! {
            div {
                style: "padding: 16px;",
                button {
                    id: "drp-pred-swap",
                    r#type: "button",
                    style: "width: 80px; height: 28px;",
                    onclick: move |_| {
                        let next = 1 - *pred_stage.read();
                        pred_stage.set(next);
                    },
                    "Swap"
                }
                DateRangePickerRoot {
                    id: "drp-pred-root",
                    locale: CalendarLocale::EnUs,
                    default_placeholder: d(2026, 4, 15),
                    is_date_unavailable,
                    on_invalid: move |evt: FieldInvalidEvent| {
                        last_invalid_reason.set(Some(evt.reason));
                    },
                    div {
                        style: "display: flex; align-items: center; gap: 8px;",
                        DateRangePickerInput {
                            kind: RangeFieldInputKind::Start,
                            id: "drp-pred-start-input",
                            name: Some("drp_pred_start".to_string()),
                            style: "display: flex; gap: 4px;",
                            render_children: move |segments: Vec<DateFieldSegmentItem>| rsx! {
                                for seg in segments {
                                    DateRangePickerSegment {
                                        key: "{seg.index}",
                                        id: format!("drp-pred-start-seg-{}", seg.index),
                                        part: seg.kind(),
                                        index: Some(seg.index),
                                        style: "display: inline-block; min-width: 24px; height: 24px;",
                                    }
                                }
                            }
                        }
                        DateRangePickerInput {
                            kind: RangeFieldInputKind::End,
                            id: "drp-pred-end-input",
                            name: Some("drp_pred_end".to_string()),
                            style: "display: flex; gap: 4px;",
                            render_children: move |segments: Vec<DateFieldSegmentItem>| rsx! {
                                for seg in segments {
                                    DateRangePickerSegment {
                                        key: "{seg.index}",
                                        id: format!("drp-pred-end-seg-{}", seg.index),
                                        part: seg.kind(),
                                        index: Some(seg.index),
                                        style: "display: inline-block; min-width: 24px; height: 24px;",
                                    }
                                }
                            }
                        }
                    }
                }
                span {
                    id: "drp-pred-invalid-reason",
                    "{last_invalid_reason.read():?}"
                }
            }
        }
    }

    #[test]
    fn native_date_range_picker_reactive_predicate_swap_updates_field_invalid_from_none() {
        let _lock = NATIVE_TEST_LOCK.lock().unwrap_or_else(|e| e.into_inner());
        clear_active_overlay_stack();
        let mut harness = mount_native_scenario(NativeDateRangePickerReactivePredicateScenario);

        // Mounted with empty range -> enter 2026-04-22 into Start (valid under cb1 which blocks day 10)
        crate::common::native_fixture::focus_node(&mut harness, "#drp-pred-start-seg-0");
        send_key_and_pump(&mut harness, Key::Character("4".to_string()), Code::Digit4);
        send_key_and_pump(&mut harness, Key::Character("2".to_string()), Code::Digit2);
        send_key_and_pump(&mut harness, Key::Character("2".to_string()), Code::Digit2);
        send_key_and_pump(&mut harness, Key::Character("2".to_string()), Code::Digit2);
        send_key_and_pump(&mut harness, Key::Character("0".to_string()), Code::Digit0);
        send_key_and_pump(&mut harness, Key::Character("2".to_string()), Code::Digit2);
        send_key_and_pump(&mut harness, Key::Character("6".to_string()), Code::Digit6);

        assert_eq!(
            harness.attr("input[name='drp_pred_start']", "value").as_deref(),
            Some("2026-04-22")
        );
        assert_eq!(harness.attr("[data-date-range-field-root]", "data-invalid"), None);
        assert_eq!(harness.attr("#drp-pred-start-input", "data-invalid"), None);

        // Swap is_date_unavailable Some(cb1) -> Some(cb2) (blocks day 22) without changing value/placeholder
        click_and_pump(&mut harness, "#drp-pred-swap");
        assert_eq!(
            harness
                .attr("[data-date-range-field-root]", "data-invalid")
                .as_deref(),
            Some("")
        );
        assert_eq!(
            harness.attr("#drp-pred-start-input", "data-invalid").as_deref(),
            Some("")
        );
        assert_eq!(harness.attr("#drp-pred-end-input", "data-invalid"), None);
        assert_eq!(
            harness.text_content("#drp-pred-invalid-reason"),
            "Some(Unavailable)"
        );

        // Swap back Some(cb2) -> Some(cb1) -> invalid state clears immediately
        click_and_pump(&mut harness, "#drp-pred-swap");
        assert_eq!(harness.attr("[data-date-range-field-root]", "data-invalid"), None);
        assert_eq!(harness.attr("#drp-pred-start-input", "data-invalid"), None);

        clear_active_overlay_stack();
    }
}

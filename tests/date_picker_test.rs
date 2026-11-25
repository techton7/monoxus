use std::{cell::RefCell, rc::Rc};

use dioxus::prelude::*;
use monoxus::{
    calendar::CalendarSnapshot,
    date_field::DateFieldSegmentItem,
    date_picker::{
        DATE_PICKER_OPEN_FOCUS_SELECTOR, DATE_PICKER_PARTS, DatePicker, DatePickerArrow,
        DatePickerCalendar, DatePickerCell, DatePickerClose, DatePickerContent,
        DatePickerContentStatic, DatePickerDay, DatePickerGrid, DatePickerGridBody,
        DatePickerGridHead, DatePickerGridRow, DatePickerHeadCell, DatePickerHeader,
        DatePickerHeading, DatePickerInput, DatePickerLabel, DatePickerMonthSelect,
        DatePickerNextButton, DatePickerPart, DatePickerPortal, DatePickerPrevButton,
        DatePickerRelationships, DatePickerRoot, DatePickerSegment, DatePickerSelectRenderProps,
        DatePickerTrigger, DatePickerYearSelect, use_date_picker_context,
    },
    foundation::{
        date_time::{
            CalendarDate, CalendarLocale, CalendarMonth, DateValue, FieldInvalidEvent,
            FieldInvalidReason,
        },
        overlay::{active_overlay_layers_snapshot, unregister_active_overlay_layer},
        shared::ScopeHandle,
    },
    popover::PopoverContext,
    select::{
        SelectContent, SelectItem, SelectPortal, SelectRoot, SelectTrigger, SelectValue,
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
fn date_picker_public_contracts_all_24_parts_and_vdom_smoke() {
    clear_active_overlay_stack();
    assert_eq!(DATE_PICKER_PARTS.len(), 11);
    assert_eq!(DatePickerPart::Trigger.as_str(), "trigger");
    assert_eq!(
        DATE_PICKER_OPEN_FOCUS_SELECTOR,
        "[data-calendar-day][data-focused]"
    );

    let scope = ScopeHandle::root("date-picker").child("contract");
    let rels = DatePickerRelationships::new(scope.clone());
    assert_eq!(rels.root_id(), scope.token());
    assert_eq!(rels.open_focus_selector(), DATE_PICKER_OPEN_FOCUS_SELECTOR);

    #[component]
    fn DatePickerVdomApp() -> Element {
        rsx! {
            DatePicker {
                id: "vdom-dp",
                default_value: d(2026, 4, 15),
                default_open: true,
                DatePickerLabel { "Event Date" }
                DatePickerInput {
                    name: Some("event_date".to_string()),
                    render_children: move |segments: Vec<DateFieldSegmentItem>| rsx! {
                        for seg in segments {
                            DatePickerSegment {
                                key: "{seg.index}",
                                part: seg.kind(),
                                index: Some(seg.index),
                            }
                        }
                    }
                }
                DatePickerTrigger { "Open" }
                DatePickerPortal {
                    DatePickerContent {
                        DatePickerArrow {}
                        DatePickerClose { "Close" }
                        DatePickerCalendar {
                            render_children: move |snap: CalendarSnapshot| rsx! {
                                DatePickerHeader {
                                    DatePickerPrevButton { "<" }
                                    DatePickerHeading {}
                                    DatePickerMonthSelect {}
                                    DatePickerYearSelect {}
                                    DatePickerNextButton { ">" }
                                }
                                for month in snap.months {
                                    DatePickerGrid {
                                        key: "{month.value}",
                                        DatePickerGridHead {
                                            DatePickerGridRow {
                                                for (w_idx, weekday) in snap.weekdays.iter().enumerate() {
                                                    DatePickerHeadCell { key: "{w_idx}", "{weekday}" }
                                                }
                                            }
                                        }
                                        DatePickerGridBody {
                                            for (w_idx, week) in month.weeks.iter().enumerate() {
                                                DatePickerGridRow {
                                                    key: "{w_idx}",
                                                    for date in week.iter().copied() {
                                                        DatePickerCell {
                                                            key: "{date}",
                                                            date,
                                                            month: month.value,
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
            DatePickerRoot {
                id: "vdom-dp-static",
                default_open: true,
                DatePickerTrigger { "Static" }
                DatePickerContentStatic { "Static calendar host" }
            }
        }
    }

    let mut dom = VirtualDom::new(DatePickerVdomApp);
    dom.rebuild_in_place();
    drop(dom);
    clear_active_overlay_stack();
}

#[derive(Clone, Default, PartialEq)]
struct PickerIdProbe {
    root_id: String,
    popover_content_id: String,
    popover_trigger_id: String,
}

#[component]
fn CapturePickerIds(out: Rc<RefCell<PickerIdProbe>>) -> Element {
    let picker = use_date_picker_context();
    let popover = use_context::<PopoverContext>();
    *out.borrow_mut() = PickerIdProbe {
        root_id: picker.rels.root_id().to_string(),
        popover_content_id: popover.relationships.content_id().to_string(),
        popover_trigger_id: popover.relationships.trigger_id().to_string(),
    };
    rsx! {}
}

#[component]
fn MultiPickerIdsHarness(
    first: Rc<RefCell<PickerIdProbe>>,
    second: Rc<RefCell<PickerIdProbe>>,
) -> Element {
    rsx! {
        DatePickerRoot {
            DatePickerTrigger { "First" }
            CapturePickerIds { out: first }
        }
        DatePickerRoot {
            DatePickerTrigger { "Second" }
            CapturePickerIds { out: second }
        }
    }
}

#[test]
fn multi_date_picker_generates_unique_stable_ids() {
    clear_active_overlay_stack();
    let first = Rc::new(RefCell::new(PickerIdProbe::default()));
    let second = Rc::new(RefCell::new(PickerIdProbe::default()));

    let mut dom = VirtualDom::new_with_props(
        MultiPickerIdsHarness,
        MultiPickerIdsHarnessProps {
            first: first.clone(),
            second: second.clone(),
        },
    );
    dom.rebuild_in_place();

    let a = first.borrow().clone();
    let b = second.borrow().clone();
    assert_ne!(a.root_id, b.root_id);
    assert_ne!(a.popover_content_id, b.popover_content_id);
    assert_ne!(a.popover_trigger_id, b.popover_trigger_id);

    drop(dom);
    clear_active_overlay_stack();
}

#[cfg(feature = "native")]
mod native_tests {
    use keyboard_types::{Code, Key, NamedKey};

    use super::*;
    use crate::common::native_fixture::{
        NATIVE_TEST_LOCK, assert_native_focus_id, click_and_pump, focus_node,
        mount_native_scenario, send_key_and_pump,
    };

    #[component]
    fn NativeDatePickerScenario() -> Element {
        rsx! {
            div {
                style: "padding: 16px;",
                DatePickerRoot {
                    id: "native-dp",
                    locale: CalendarLocale::EnUs,
                    default_placeholder: d(2026, 4, 15),
                    close_on_date_select: true,
                    DatePickerLabel {
                        id: "dp-label",
                        style: "display: block; width: 120px; height: 24px;",
                        "Appointment"
                    }
                    div {
                        style: "display: flex; align-items: center; gap: 8px;",
                        DatePickerInput {
                            id: "dp-input",
                            name: Some("appt_date".to_string()),
                            style: "display: flex; gap: 4px;",
                            render_children: move |segments: Vec<DateFieldSegmentItem>| rsx! {
                                for seg in segments {
                                    DatePickerSegment {
                                        key: "{seg.index}",
                                        id: format!("dp-seg-{}", seg.index),
                                        part: seg.kind(),
                                        index: Some(seg.index),
                                        style: "display: inline-block; min-width: 24px; height: 24px;",
                                    }
                                }
                            }
                        }
                        DatePickerTrigger {
                            style: "width: 80px; height: 28px;",
                            "Calendar"
                        }
                    }
                    DatePickerPortal {
                        DatePickerContent {
                            style: "width: 280px; padding: 8px; background: white;",
                            DatePickerCalendar {
                                render_children: move |snap: CalendarSnapshot| rsx! {
                                    DatePickerHeader {
                                        style: "display: flex; gap: 4px; margin-bottom: 8px;",
                                        DatePickerMonthSelect {
                                            as_child: true,
                                            render_child: move |rp: DatePickerSelectRenderProps| {
                                                let items = rp.select_items();
                                                let selected_label = rp.selected_label();
                                                rsx! {
                                                    SelectRoot {
                                                        id: "dp-month-select",
                                                        value: Some(rp.value_string.clone()),
                                                        items,
                                                        on_value_change: move |next| rp.on_value_change.call(next),
                                                        SelectTrigger {
                                                            style: "width: 100px; height: 28px;",
                                                            SelectValue { placeholder: selected_label }
                                                        }
                                                        SelectPortal {
                                                            SelectContent {
                                                                style: "width: 120px; background: white;",
                                                                for opt in rp.items.clone() {
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
                                        }
                                        DatePickerHeading {
                                            id: "dp-heading",
                                            style: "display: inline-block; min-width: 80px;",
                                        }
                                    }
                                    for month in snap.months {
                                        DatePickerMonthGrid { month, weekdays: snap.weekdays.clone() }
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
    fn DatePickerMonthGrid(month: CalendarMonth, weekdays: Vec<String>) -> Element {
        rsx! {
            DatePickerGrid {
                DatePickerGridHead {
                    DatePickerGridRow {
                        for (w_idx, weekday) in weekdays.iter().enumerate() {
                            DatePickerHeadCell { key: "{w_idx}", "{weekday}" }
                        }
                    }
                }
                DatePickerGridBody {
                    for (w_idx, week) in month.weeks.iter().enumerate() {
                        DatePickerGridRow {
                            key: "{w_idx}",
                            for date in week.iter().copied() {
                                DatePickerCell {
                                    key: "{date}",
                                    date,
                                    month: month.value,
                                    DatePickerDay {
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
    fn native_date_picker_select_closes_restores_focus_and_supports_nested_month_select() {
        let _lock = NATIVE_TEST_LOCK.lock().unwrap_or_else(|e| e.into_inner());
        clear_active_overlay_stack();
        let mut harness = mount_native_scenario(NativeDatePickerScenario);

        let trigger_id = harness
            .attr("[data-segment='trigger']", "id")
            .expect("trigger must have an id");

        // ArrowRight from last field segment (year = index 4) focuses DatePickerTrigger
        focus_node(&mut harness, "#dp-seg-4");
        send_key_and_pump(&mut harness, Key::Named(NamedKey::ArrowRight), Code::ArrowRight);
        assert_native_focus_id(&harness, &trigger_id);

        // ArrowLeft from DatePickerTrigger focuses last field segment (year = index 4)
        send_key_and_pump(&mut harness, Key::Named(NamedKey::ArrowLeft), Code::ArrowLeft);
        assert_native_focus_id(&harness, "dp-seg-4");

        // Open DatePicker popover by clicking trigger
        click_and_pump(&mut harness, "[data-segment='trigger']");
        assert_eq!(
            harness.attr("[data-date-picker-content]", "data-state").as_deref(),
            Some("open")
        );

        // Open nested MonthSelect (as_child with monoxus::Select) and select May (month 5)
        click_and_pump(&mut harness, "[role='combobox']");
        assert_eq!(
            harness.attr("[role='listbox']", "data-state").as_deref(),
            Some("open")
        );
        click_and_pump(&mut harness, "[role='option'][data-value='5']");

        // OVL-1: Option click updates month to May and does NOT close DatePicker popover!
        assert_eq!(
            harness.attr("[data-date-picker-content]", "data-state").as_deref(),
            Some("open")
        );
        assert!(harness.text_content("#dp-heading").contains("May"));

        // OVL-2: Re-open nested Select, press Escape -> closes Select ONLY, keeping DatePicker open!
        click_and_pump(&mut harness, "[role='combobox']");
        assert_eq!(
            harness.attr("[role='listbox']", "data-state").as_deref(),
            Some("open")
        );
        send_key_and_pump(&mut harness, Key::Named(NamedKey::Escape), Code::Escape);
        assert_eq!(
            harness.attr("[role='combobox']", "aria-expanded").as_deref(),
            Some("false")
        );
        assert_eq!(
            harness.attr("[data-date-picker-content]", "data-state").as_deref(),
            Some("open")
        );

        // Select 2026-05-20 -> updates field + hidden input, closes popover, restores trigger focus!
        click_and_pump(&mut harness, "[data-calendar-day][data-value='2026-05-20']");
        assert_eq!(
            harness.attr("input[name='appt_date']", "value").as_deref(),
            Some("2026-05-20")
        );
        assert_eq!(harness.text_content("#dp-seg-0"), "05");
        assert_eq!(harness.text_content("#dp-seg-2"), "20");
        assert_eq!(harness.text_content("#dp-seg-4"), "2026");
        assert_eq!(
            harness.attr("[data-segment='trigger']", "aria-expanded").as_deref(),
            Some("false")
        );
        assert_native_focus_id(&harness, &trigger_id);

        clear_active_overlay_stack();
    }

    #[component]
    fn NativeDatePickerRemediationScenario() -> Element {
        let mut complete_log = use_signal(Vec::<bool>::new);
        let mut escape_blocked = use_signal(|| true);
        let mut last_invalid_reason = use_signal(|| Option::<FieldInvalidReason>::None);

        rsx! {
            div {
                style: "padding: 280px 16px 16px 16px;",
                form {
                    id: "dp-remediation-form",
                    DatePickerRoot {
                        id: "dp-remediation",
                        locale: CalendarLocale::EnUs,
                        default_value: d(2026, 4, 15),
                        default_placeholder: d(2026, 4, 15),
                        is_date_disabled: move |date: CalendarDate| date.day == 20,
                        on_invalid: move |evt: FieldInvalidEvent| {
                            last_invalid_reason.set(Some(evt.reason));
                        },
                        on_open_change_complete: move |open| {
                            complete_log.write().push(open);
                        },
                        div {
                            style: "display: flex; align-items: center; gap: 8px;",
                            DatePickerInput {
                                id: "dp-rem-input",
                                name: Some("rem_date".to_string()),
                                style: "display: flex; gap: 4px;",
                                render_children: move |segments: Vec<DateFieldSegmentItem>| rsx! {
                                    for seg in segments {
                                        DatePickerSegment {
                                            key: "{seg.index}",
                                            id: format!("dp-rem-seg-{}", seg.index),
                                            part: seg.kind(),
                                            index: Some(seg.index),
                                            style: "display: inline-block; min-width: 24px; height: 24px;",
                                        }
                                    }
                                }
                            }
                            DatePickerTrigger {
                                id: "dp-rem-trigger",
                                style: "width: 80px; height: 38px;",
                                "Open"
                            }
                            button {
                                id: "dp-rem-reset",
                                r#type: "reset",
                                style: "width: 60px; height: 30px;",
                                "Reset"
                            }
                            button {
                                id: "dp-rem-allow-esc",
                                r#type: "button",
                                style: "width: 60px; height: 30px;",
                                onclick: move |_| escape_blocked.set(false),
                                "AllowEsc"
                            }
                        }
                        DatePickerPortal {
                            DatePickerContent {
                                id: "dp-rem-content",
                                collision_padding: 8.0,
                                avoid_collisions: true,
                                on_open_auto_focus: move |evt: monoxus::date_picker::OpenAutoFocusEvent| {
                                    evt.prevent_default();
                                },
                                on_escape_key_down: move |evt: monoxus::date_picker::EscapeKeyDownEvent| {
                                    if *escape_blocked.read() {
                                        evt.prevent_default();
                                    }
                                },
                                as_child: true,
                                render_child: move |rp: monoxus::date_picker::DatePickerContentRenderProps| {
                                    let wid = rp.wrapper_id.unwrap_or_default();
                                    let wstyle = rp.wrapper_style.unwrap_or_default();
                                    rsx! {
                                        div {
                                            id: "{wid}",
                                            "data-monoxus-floating-content-wrapper": "",
                                            style: "{wstyle}",
                                            section {
                                                id: "{rp.attrs.id}",
                                                role: "{rp.role}",
                                                "data-as-child-picker": "true",
                                                "data-state": rp.attrs.data_state,
                                                "data-date-picker-content": rp.attrs.data_date_picker_content,
                                                style: "width: 280px; height: 340px; padding: 8px; background: white; {rp.style}",
                                                tabindex: rp.attrs.tabindex,
                                                onmounted: move |evt| rp.on_mounted.call(evt),
                                                onkeydown: move |evt| rp.on_key_down.call(evt),
                                                onclick: move |evt| rp.on_click.call(evt),
                                                {rp.children}
                                            }
                                        }
                                    }
                                },
                                DatePickerCalendar {
                                    render_children: move |snap: CalendarSnapshot| rsx! {
                                        for month in snap.months {
                                            DatePickerMonthGrid { month, weekdays: snap.weekdays.clone() }
                                        }
                                    }
                                }
                            }
                        }
                    }
                }
                span {
                    id: "dp-rem-complete-count",
                    "{complete_log.read().len()}"
                }
                span {
                    id: "dp-rem-invalid-reason",
                    "{last_invalid_reason.read():?}"
                }
            }
        }
    }

    #[test]
    fn native_date_picker_clamps_viewport_validates_disabled_resets_form_and_honors_hooks() {
        let _lock = NATIVE_TEST_LOCK.lock().unwrap_or_else(|e| e.into_inner());
        clear_active_overlay_stack();
        let mut harness = mount_native_scenario(NativeDatePickerRemediationScenario);

        // 1. Open mid-viewport (y=280) 340px-tall DatePickerContent on 800x600 -> clamped y >= 0.0 (DEF-5C-04)
        focus_node(&mut harness, "#dp-rem-trigger");
        click_and_pump(&mut harness, "#dp-rem-trigger");
        assert_eq!(
            harness.attr("#dp-rem-content", "data-state").as_deref(),
            Some("open")
        );
        assert_eq!(
            harness.attr("#dp-rem-content", "data-as-child-picker").as_deref(),
            Some("true")
        );
        let content_rect = harness.layout_rect("#dp-rem-content");
        assert!(
            content_rect.y >= 0.0 && content_rect.y + content_rect.height <= 600.0,
            "DatePickerContent must clamp inside 800x600 viewport, got rect={content_rect:?}"
        );

        // 2. on_open_auto_focus.prevent_default() keeps focus on trigger instead of calendar day (DEF-5C-10)
        assert_native_focus_id(&harness, "dp-rem-trigger");

        // 3. on_escape_key_down.prevent_default() keeps popover open (DEF-5C-10)
        send_key_and_pump(&mut harness, Key::Named(NamedKey::Escape), Code::Escape);
        assert_eq!(
            harness.attr("#dp-rem-content", "data-state").as_deref(),
            Some("open")
        );

        // Allow escape: clicking outside button closes popover; reopen and verify Escape now closes
        click_and_pump(&mut harness, "#dp-rem-allow-esc");
        assert_eq!(
            harness.attr("#dp-rem-trigger", "aria-expanded").as_deref(),
            Some("false")
        );
        click_and_pump(&mut harness, "#dp-rem-trigger");
        assert_eq!(
            harness.attr("#dp-rem-trigger", "aria-expanded").as_deref(),
            Some("true")
        );
        send_key_and_pump(&mut harness, Key::Named(NamedKey::Escape), Code::Escape);
        assert_eq!(
            harness.attr("#dp-rem-trigger", "aria-expanded").as_deref(),
            Some("false")
        );

        // 4. Typing a disabled date (day 20) marks DatePickerInput invalid with FieldInvalidReason::Unavailable (DEF-5C-11)
        focus_node(&mut harness, "#dp-rem-seg-2");
        send_key_and_pump(&mut harness, Key::Character("2".to_string()), Code::Digit2);
        send_key_and_pump(&mut harness, Key::Character("0".to_string()), Code::Digit0);
        assert_eq!(
            harness.attr("input[name='rem_date']", "value").as_deref(),
            Some("2026-04-20")
        );
        assert_eq!(
            harness.attr("#dp-rem-input", "data-invalid").as_deref(),
            Some("")
        );
        assert_eq!(
            harness.text_content("#dp-rem-invalid-reason"),
            "Some(Unavailable)"
        );

        // 5. Native <form> reset restores default_value (2026-04-15) and clears invalid state (DEF-5C-08)
        click_and_pump(&mut harness, "#dp-rem-reset");
        assert_eq!(
            harness.attr("input[name='rem_date']", "value").as_deref(),
            Some("2026-04-15")
        );
        assert_eq!(harness.attr("#dp-rem-input", "data-invalid"), None);
        assert!(harness.text_content("#dp-rem-complete-count").parse::<usize>().unwrap_or(0) >= 1);

        clear_active_overlay_stack();
    }

    #[component]
    fn NativeDatePickerStayOpenScenario() -> Element {
        rsx! {
            div {
                style: "padding: 16px;",
                DatePickerRoot {
                    id: "dp-stay-open",
                    locale: CalendarLocale::EnUs,
                    default_placeholder: d(2026, 4, 15),
                    close_on_date_select: false,
                    div {
                        style: "display: flex; align-items: center; gap: 8px;",
                        DatePickerInput {
                            id: "dp-so-input",
                            name: Some("stay_open_date".to_string()),
                            style: "display: flex; gap: 4px;",
                            render_children: move |segments: Vec<DateFieldSegmentItem>| rsx! {
                                for seg in segments {
                                    DatePickerSegment {
                                        key: "{seg.index}",
                                        id: format!("dp-so-seg-{}", seg.index),
                                        part: seg.kind(),
                                        index: Some(seg.index),
                                        style: "display: inline-block; min-width: 24px; height: 24px;",
                                    }
                                }
                            }
                        }
                        DatePickerTrigger {
                            id: "dp-so-trigger",
                            style: "width: 80px; height: 28px;",
                            "Calendar"
                        }
                    }
                    DatePickerPortal {
                        DatePickerContent {
                            id: "dp-so-content",
                            style: "width: 280px; padding: 8px; background: white;",
                            DatePickerCalendar {
                                render_children: move |snap: CalendarSnapshot| rsx! {
                                    for month in snap.months {
                                        DatePickerMonthGrid { month, weekdays: snap.weekdays.clone() }
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
    fn native_date_picker_close_on_date_select_false_keeps_popover_open() {
        let _lock = NATIVE_TEST_LOCK.lock().unwrap_or_else(|e| e.into_inner());
        clear_active_overlay_stack();
        let mut harness = mount_native_scenario(NativeDatePickerStayOpenScenario);

        click_and_pump(&mut harness, "#dp-so-trigger");
        assert_eq!(
            harness.attr("#dp-so-content", "data-state").as_deref(),
            Some("open")
        );

        // Selecting a calendar day updates value/segments AND keeps popover open
        click_and_pump(&mut harness, "[data-calendar-day][data-value='2026-04-20']");
        assert_eq!(
            harness
                .attr("input[name='stay_open_date']", "value")
                .as_deref(),
            Some("2026-04-20")
        );
        assert_eq!(harness.text_content("#dp-so-seg-0"), "04");
        assert_eq!(harness.text_content("#dp-so-seg-2"), "20");
        assert_eq!(harness.text_content("#dp-so-seg-4"), "2026");
        assert_eq!(
            harness.attr("#dp-so-content", "data-state").as_deref(),
            Some("open")
        );
        assert_eq!(
            harness.attr("#dp-so-trigger", "aria-expanded").as_deref(),
            Some("true")
        );

        clear_active_overlay_stack();
    }
}

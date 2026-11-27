use std::{cell::RefCell, rc::Rc};

use dioxus::prelude::*;
use monoxus::{
    calendar::{CalendarSnapshot, use_calendar_context},
    date_field::DateFieldSegmentItem,
    date_picker::{
        DATE_PICKER_OPEN_FOCUS_SELECTOR, DATE_PICKER_PARTS, DatePicker, DatePickerArrow,
        DatePickerCalendar, DatePickerCell, DatePickerClose, DatePickerContent,
        DatePickerContentStatic, DatePickerDay, DatePickerGrid, DatePickerGridBody,
        DatePickerGridHead, DatePickerGridRow, DatePickerHeadCell, DatePickerHeader,
        DatePickerHeading, DatePickerInput, DatePickerLabel, DatePickerMonthSelect,
        DatePickerNextButton, DatePickerPart, DatePickerPortal, DatePickerPrevButton,
        DatePickerRelationships, DatePickerRoot, DatePickerSegment, DatePickerTrigger,
        DatePickerYearSelect, use_date_picker_context,
    },
    foundation::{
        compose::FloatingContentChildProps,
        date_time::{
            CalendarDate, CalendarLocale, CalendarMonth, DateValue, FieldInvalidEvent,
            FieldInvalidReason, MonthFormat,
        },
        overlay::{active_overlay_layers_snapshot, unregister_active_overlay_layer},
        shared::ScopeHandle,
    },
    popover::PopoverContext,
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
                                            child: move |_props: Vec<Attribute>| rsx! {
                                                DatePickerCustomMonthSelect {}
                                            },
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
    fn DatePickerCustomMonthSelect() -> Element {
        let mut ctx = use_calendar_context();
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
                id: "dp-month-select",
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
                                style: "width: 280px; height: 340px; padding: 8px; background: white;",
                                child: move |c: FloatingContentChildProps| {
                                    rsx! {
                                        div {
                                            ..c.wrapper_props,
                                            section {
                                                "data-as-child-picker": "true",
                                                ..c.props,
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
                                },
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

    #[component]
    fn PickerWrapperStyleProbe() -> Element {
        let popover = use_context::<PopoverContext>();
        let style = popover.runtime.read().wrapper_style();
        rsx! {
            span { id: "dp-fm-wrapper-style", "{style}" }
        }
    }

    #[component]
    fn DatePickerStage3Child(attrs: Vec<Attribute>, stage: &'static str) -> Element {
        let snap = use_calendar_context().snapshot();
        rsx! {
            div {
                "data-cb-stage": stage,
                ..attrs,
                for month in snap.months {
                    DatePickerMonthGrid { month, weekdays: snap.weekdays.clone() }
                }
            }
        }
    }

    #[component]
    fn NativeDatePickerStage3FmScenario() -> Element {
        use monoxus::foundation::date_time::{
            CalendarDateTime, CalendarTime, Granularity, HourCycle,
        };

        let mut show_content = use_signal(|| true);
        let mut cb_stage = use_signal(|| 0_u8);
        let mut swap_trigger = use_signal(|| false);

        let dt = DateValue::DateTime(CalendarDateTime::new(
            CalendarDate::new(2026, 4, 15).expect("valid date"),
            CalendarTime::from_hms(14, 35, 0).expect("valid time"),
        ));

        let child = match *cb_stage.read() {
            0 => None,
            1 => Some(Callback::new(|props: Vec<Attribute>| rsx! {
                DatePickerStage3Child { attrs: props, stage: "cb1" }
            })),
            _ => Some(Callback::new(|props: Vec<Attribute>| rsx! {
                DatePickerStage3Child { attrs: props, stage: "cb2" }
            })),
        };
        let on_key_down = match *cb_stage.read() {
            0 => None,
            1 => Some(EventHandler::new(|_evt: KeyboardEvent| {})),
            _ => Some(EventHandler::new(|_evt: KeyboardEvent| {})),
        };

        let trig_key = if *swap_trigger.read() { "t-b" } else { "t-a" };
        let trig_id = if *swap_trigger.read() {
            "dp-fm-trig-b"
        } else {
            "dp-fm-trig-a"
        };

        rsx! {
            div {
                style: "padding: 16px;",
                button {
                    id: "dp-fm-swap-trigger",
                    r#type: "button",
                    style: "width: 80px; height: 28px;",
                    onclick: move |_| swap_trigger.toggle(),
                    "Swap Trigger"
                }

                // Floating picker testing FM-DT-1 (B4 z-index drop), FM-DT-2 (time preservation), FM-DT-5 (trigger swap), FM-API-4
                DatePickerRoot {
                    id: "dp-fm-floating",
                    locale: CalendarLocale::Iso,
                    granularity: Granularity::Minute,
                    hour_cycle: HourCycle::H24,
                    hide_time_zone: true,
                    default_value: dt,
                    close_on_date_select: true,
                    PickerWrapperStyleProbe {}
                    div {
                        style: "display: flex; align-items: center; gap: 8px;",
                        DatePickerInput {
                            id: "dp-fm-input",
                            name: Some("fm_dt".to_string()),
                            style: "display: flex; gap: 4px;",
                            render_children: move |segments: Vec<DateFieldSegmentItem>| rsx! {
                                for seg in segments {
                                    DatePickerSegment {
                                        key: "{seg.index}",
                                        id: format!("dp-fm-seg-{}", seg.index),
                                        part: seg.kind(),
                                        index: Some(seg.index),
                                        style: "display: inline-block; min-width: 24px; height: 24px;",
                                    }
                                }
                            }
                        }
                        for (tkey, tid) in [(trig_key, trig_id)] {
                            DatePickerTrigger {
                                key: "{tkey}",
                                id: tid.to_string(),
                                style: "width: 80px; height: 28px;",
                                "Open"
                            }
                        }
                    }
                    DatePickerPortal {
                        if *show_content.read() {
                            DatePickerContent {
                                id: "dp-fm-content",
                                style: "width: 280px; z-index: 77; background: white;",
                                button {
                                    id: "dp-fm-step-cb",
                                    r#type: "button",
                                    style: "width: 80px; height: 24px;",
                                    onclick: move |_| {
                                        let next = (*cb_stage.read() + 1) % 3;
                                        cb_stage.set(next);
                                    },
                                    "Step CB"
                                }
                                button {
                                    id: "dp-fm-toggle-content",
                                    r#type: "button",
                                    style: "width: 80px; height: 24px;",
                                    onclick: move |_| show_content.toggle(),
                                    "Unmount Content"
                                }
                                DatePickerCalendar {
                                    id: "dp-fm-cal",
                                    child,
                                    on_key_down,
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

                // Static picker testing FM-DT-1 (no floating wrapper, data-date-picker-content, Escape)
                DatePickerRoot {
                    id: "dp-fm-static",
                    locale: CalendarLocale::EnUs,
                    default_placeholder: d(2026, 4, 15),
                    DatePickerTrigger {
                        id: "dp-fm-static-trigger",
                        style: "width: 80px; height: 28px;",
                        "Static"
                    }
                    DatePickerContentStatic {
                        id: "dp-fm-static-content",
                        style: "width: 280px; background: white;",
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

    #[test]
    fn native_date_picker_fm_dt1_dt2_dt5_and_api4() {
        let _lock = NATIVE_TEST_LOCK.lock().unwrap_or_else(|e| e.into_inner());
        clear_active_overlay_stack();
        let mut harness = mount_native_scenario(NativeDatePickerStage3FmScenario);

        // FM-DT-5: keyed trigger swap does not clobber trailing_trigger_id
        click_and_pump(&mut harness, "#dp-fm-swap-trigger");
        focus_node(&mut harness, "#dp-fm-seg-8");
        send_key_and_pump(&mut harness, Key::Named(NamedKey::ArrowRight), Code::ArrowRight);
        assert_native_focus_id(&harness, "dp-fm-trig-b");

        // Open floating picker -> verify FM-API-4 (`DatePickerCalendar` callback toggling None -> Some(cb1) -> Some(cb2))
        click_and_pump(&mut harness, "#dp-fm-trig-b");
        assert_eq!(
            harness.attr("#dp-fm-content", "data-state").as_deref(),
            Some("open")
        );
        assert_eq!(
            harness
                .attr("#dp-fm-content", "data-date-picker-content")
                .as_deref(),
            Some("")
        );
        click_and_pump(&mut harness, "#dp-fm-step-cb");
        assert_eq!(
            harness.attr("#dp-fm-cal", "data-cb-stage").as_deref(),
            Some("cb1")
        );
        click_and_pump(&mut harness, "#dp-fm-step-cb");
        assert_eq!(
            harness.attr("#dp-fm-cal", "data-cb-stage").as_deref(),
            Some("cb2")
        );

        // FM-DT-2: selecting calendar day preserves wall-clock CalendarTime (14:35:00) and closes
        click_and_pump(&mut harness, "[data-calendar-day][data-value='2026-04-20']");
        assert_eq!(
            harness.attr("input[name='fm_dt']", "value").as_deref(),
            Some("2026-04-20T14:35:00")
        );
        assert_eq!(
            harness.attr("#dp-fm-trig-b", "aria-expanded").as_deref(),
            Some("false")
        );

        // FM-DT-1 (B4): unmounting DatePickerContent clears hoisted z-index on PopoverRuntime
        click_and_pump(&mut harness, "#dp-fm-trig-b");
        assert_eq!(
            harness.attr("#dp-fm-content", "data-state").as_deref(),
            Some("open")
        );
        click_and_pump(&mut harness, "#dp-fm-toggle-content");
        assert!(
            !harness
                .text_content("#dp-fm-wrapper-style")
                .contains("z-index: 77"),
            "DatePickerContent unmount must clear hoisted z-index"
        );

        // FM-DT-1: DatePickerContentStatic renders without floating wrapper, emits data-date-picker-content, honors Escape
        click_and_pump(&mut harness, "#dp-fm-static-trigger");
        assert_eq!(
            harness
                .attr("#dp-fm-static-content", "data-date-picker-content")
                .as_deref(),
            Some("")
        );
        assert!(harness.query("#dp-fm-static-content-wrapper").is_none());
        send_key_and_pump(&mut harness, Key::Named(NamedKey::Escape), Code::Escape);
        assert_eq!(
            harness
                .attr("#dp-fm-static-trigger", "aria-expanded")
                .as_deref(),
            Some("false")
        );

        clear_active_overlay_stack();
    }

    #[component]
    fn NativeDatePickerReactivePredicateScenario() -> Element {
        let mut pred_stage = use_signal(|| 0_u8);
        let mut last_invalid_reason = use_signal(|| Option::<FieldInvalidReason>::None);
        let stage = *pred_stage.read();
        let is_date_unavailable = if stage == 0 {
            Some(Callback::new(|date: CalendarDate| date.day == 10))
        } else {
            Some(Callback::new(|date: CalendarDate| date.day == 20))
        };

        rsx! {
            div {
                style: "padding: 16px;",
                button {
                    id: "dp-pred-swap",
                    r#type: "button",
                    style: "width: 80px; height: 28px;",
                    onclick: move |_| {
                        let next = 1 - *pred_stage.read();
                        pred_stage.set(next);
                    },
                    "Swap"
                }
                DatePickerRoot {
                    id: "dp-pred-root",
                    locale: CalendarLocale::EnUs,
                    default_placeholder: d(2026, 4, 15),
                    is_date_unavailable,
                    on_invalid: move |evt: FieldInvalidEvent| {
                        last_invalid_reason.set(Some(evt.reason));
                    },
                    div {
                        style: "display: flex; align-items: center; gap: 8px;",
                        DatePickerInput {
                            id: "dp-pred-input",
                            name: Some("pred_date".to_string()),
                            style: "display: flex; gap: 4px;",
                            render_children: move |segments: Vec<DateFieldSegmentItem>| rsx! {
                                for seg in segments {
                                    DatePickerSegment {
                                        key: "{seg.index}",
                                        id: format!("dp-pred-seg-{}", seg.index),
                                        part: seg.kind(),
                                        index: Some(seg.index),
                                        style: "display: inline-block; min-width: 24px; height: 24px;",
                                    }
                                }
                            }
                        }
                        DatePickerTrigger {
                            id: "dp-pred-trigger",
                            style: "width: 80px; height: 28px;",
                            "Open"
                        }
                    }
                    DatePickerPortal {
                        DatePickerContent {
                            id: "dp-pred-content",
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
                span {
                    id: "dp-pred-invalid-reason",
                    "{last_invalid_reason.read():?}"
                }
            }
        }
    }

    #[test]
    fn native_date_picker_reactive_predicate_swap_updates_field_invalid_from_none() {
        let _lock = NATIVE_TEST_LOCK.lock().unwrap_or_else(|e| e.into_inner());
        clear_active_overlay_stack();
        let mut harness = mount_native_scenario(NativeDatePickerReactivePredicateScenario);

        // Mounted with value: None -> initially valid
        assert_eq!(
            harness.attr("input[name='pred_date']", "value").as_deref(),
            Some("")
        );
        assert_eq!(harness.attr("#dp-pred-input", "data-invalid"), None);

        // Enter 2026-04-20 into segments (valid under cb1 which blocks day 10)
        focus_node(&mut harness, "#dp-pred-seg-0");
        send_key_and_pump(&mut harness, Key::Character("4".to_string()), Code::Digit4);
        send_key_and_pump(&mut harness, Key::Character("2".to_string()), Code::Digit2);
        send_key_and_pump(&mut harness, Key::Character("0".to_string()), Code::Digit0);
        send_key_and_pump(&mut harness, Key::Character("2".to_string()), Code::Digit2);
        send_key_and_pump(&mut harness, Key::Character("0".to_string()), Code::Digit0);
        send_key_and_pump(&mut harness, Key::Character("2".to_string()), Code::Digit2);
        send_key_and_pump(&mut harness, Key::Character("6".to_string()), Code::Digit6);
        assert_eq!(
            harness.attr("input[name='pred_date']", "value").as_deref(),
            Some("2026-04-20")
        );
        assert_eq!(harness.attr("#dp-pred-input", "data-invalid"), None);

        // Swap is_date_unavailable Some(cb1) -> Some(cb2) (blocks day 20) without changing value/placeholder
        click_and_pump(&mut harness, "#dp-pred-swap");
        assert_eq!(
            harness.attr("#dp-pred-input", "data-invalid").as_deref(),
            Some("")
        );
        assert_eq!(
            harness.text_content("#dp-pred-invalid-reason"),
            "Some(Unavailable)"
        );

        // Calendar popup also reflects updated predicate
        click_and_pump(&mut harness, "#dp-pred-trigger");
        assert_eq!(
            harness
                .attr("[data-calendar-day][data-value='2026-04-20']", "data-unavailable")
                .as_deref(),
            Some("")
        );
        assert_eq!(
            harness.attr("[data-calendar-day][data-value='2026-04-10']", "data-unavailable"),
            None
        );
        send_key_and_pump(&mut harness, Key::Named(NamedKey::Escape), Code::Escape);

        // Swap back Some(cb2) -> Some(cb1) -> field data-invalid clears immediately
        click_and_pump(&mut harness, "#dp-pred-swap");
        assert_eq!(harness.attr("#dp-pred-input", "data-invalid"), None);

        clear_active_overlay_stack();
    }
}

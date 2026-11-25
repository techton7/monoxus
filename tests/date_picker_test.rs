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
        date_time::{CalendarDate, CalendarLocale, CalendarMonth, DateValue},
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
}

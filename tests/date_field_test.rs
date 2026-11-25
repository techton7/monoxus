use dioxus::prelude::*;
use monoxus::{
    date_field::{
        DATE_FIELD_PARTS, DateField, DateFieldInput, DateFieldLabel, DateFieldPart,
        DateFieldRelationships, DateFieldRoot, DateFieldSegment, DateFieldSegmentItem,
        use_date_field_context,
    },
    foundation::{
        date_time::{
            CalendarDate, CalendarDateTime, CalendarLocale, CalendarTime, DateValue, Granularity,
            HourCycle,
        },
        shared::ScopeHandle,
    },
};

fn d(y: i32, m: u8, day: u8) -> DateValue {
    DateValue::Date(CalendarDate::new(y, m, day).expect("valid date"))
}

#[test]
fn date_field_public_contracts_and_vdom_smoke() {
    assert_eq!(
        DATE_FIELD_PARTS
            .iter()
            .map(DateFieldPart::as_str)
            .collect::<Vec<_>>(),
        ["input", "label", "segment"]
    );
    let scope = ScopeHandle::root("date-field").child("contract");
    let rels = DateFieldRelationships::new(scope.clone());
    assert_eq!(rels.input_id(), scope.qualify("input"));
    assert_eq!(rels.label_id(), scope.qualify("label"));

    #[component]
    fn DateFieldVdomProbe() -> Element {
        let ctx = use_date_field_context();
        let segs = ctx.segments();
        assert_eq!(segs.len(), 5);
        rsx! {
            DateFieldLabel { "Birthday" }
            DateFieldInput {
                name: Some("birthday".to_string()),
                for seg in segs {
                    DateFieldSegment {
                        key: "{seg.index}",
                        part: seg.kind(),
                        index: Some(seg.index),
                    }
                }
            }
        }
    }

    #[component]
    fn DateFieldVdomApp() -> Element {
        rsx! {
            DateField {
                id: "vdom-df",
                default_value: d(2026, 4, 15),
                DateFieldVdomProbe {}
            }
        }
    }

    let mut dom = VirtualDom::new(DateFieldVdomApp);
    dom.rebuild_in_place();

    #[component]
    fn DateTimeRenderChildrenApp() -> Element {
        let dt = DateValue::DateTime(CalendarDateTime::new(
            CalendarDate::new(2026, 4, 15).unwrap(),
            CalendarTime::from_hms(14, 30, 0).unwrap(),
        ));
        rsx! {
            DateFieldRoot {
                id: "vdom-dtf",
                default_value: dt,
                granularity: Granularity::Minute,
                hour_cycle: HourCycle::H24,
                locale: CalendarLocale::Iso,
                DateFieldLabel { "Appointment" }
                DateFieldInput {
                    name: Some("appt".to_string()),
                    render_children: move |segments: Vec<DateFieldSegmentItem>| rsx! {
                        for seg in segments {
                            DateFieldSegment {
                                key: "{seg.index}",
                                part: seg.kind(),
                                index: Some(seg.index),
                            }
                        }
                    }
                }
            }
        }
    }

    let mut dt_dom = VirtualDom::new(DateTimeRenderChildrenApp);
    dt_dom.rebuild_in_place();
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
    fn NativeDateFieldScenario() -> Element {
        rsx! {
            div {
                style: "padding: 16px;",
                DateFieldRoot {
                    id: "native-df",
                    locale: CalendarLocale::EnUs,
                    min_value: d(2026, 1, 10),
                    max_value: d(2026, 12, 20),
                    default_placeholder: d(2026, 3, 15),
                    DateFieldLabel {
                        id: "df-label",
                        style: "display: block; width: 120px; height: 24px;",
                        "Event Date"
                    }
                    DateFieldInput {
                        id: "df-input",
                        name: Some("event_date".to_string()),
                        style: "display: flex; gap: 4px;",
                        render_children: move |segments: Vec<DateFieldSegmentItem>| rsx! {
                            for seg in segments {
                                DateFieldSegment {
                                    key: "{seg.index}",
                                    id: format!("df-seg-{}", seg.index),
                                    part: seg.kind(),
                                    index: Some(seg.index),
                                    style: "display: inline-block; min-width: 24px; height: 24px;",
                                }
                            }
                        }
                    }
                }
            }
        }
    }

    #[test]
    fn native_date_field_digit_entry_auto_advance_validation_and_hidden_input() {
        let _lock = NATIVE_TEST_LOCK.lock().unwrap_or_else(|e| e.into_inner());
        let mut harness = mount_native_scenario(NativeDateFieldScenario);

        // Label click focuses first editable segment (month = index 0)
        click_and_pump(&mut harness, "#df-label");
        assert_native_focus_id(&harness, "df-seg-0");
        assert_eq!(
            harness.attr("#df-seg-0", "role").as_deref(),
            Some("spinbutton")
        );
        assert_eq!(
            harness.attr("#df-seg-0", "data-segment").as_deref(),
            Some("month")
        );
        assert_eq!(
            harness.attr("#df-seg-0", "data-placeholder").as_deref(),
            Some("")
        );

        // Type "4" in month (> 1) -> "04" and auto-advances to day (index 2)
        send_key_and_pump(&mut harness, Key::Character("4".into()), Code::Digit4);
        assert_eq!(harness.text_content("#df-seg-0"), "04");
        assert_eq!(harness.attr("#df-seg-0", "data-placeholder"), None);
        assert_native_focus_id(&harness, "df-seg-2");

        // Type "1" then "8" in day -> "18" and auto-advances to year (index 4)
        send_key_and_pump(&mut harness, Key::Character("1".into()), Code::Digit1);
        assert_native_focus_id(&harness, "df-seg-2");
        send_key_and_pump(&mut harness, Key::Character("8".into()), Code::Digit8);
        assert_eq!(harness.text_content("#df-seg-2"), "18");
        assert_native_focus_id(&harness, "df-seg-4");

        // Type "2026" in year -> commits 2026-04-18 into hidden input
        send_key_and_pump(&mut harness, Key::Character("2".into()), Code::Digit2);
        send_key_and_pump(&mut harness, Key::Character("0".into()), Code::Digit0);
        send_key_and_pump(&mut harness, Key::Character("2".into()), Code::Digit2);
        send_key_and_pump(&mut harness, Key::Character("6".into()), Code::Digit6);
        assert_eq!(harness.text_content("#df-seg-4"), "2026");
        assert_eq!(
            harness.attr("input[name='event_date']", "value").as_deref(),
            Some("2026-04-18")
        );
        assert_eq!(harness.attr("#df-input", "data-invalid"), None);

        // ArrowDown on year -> 2025 (< min_value 2026-01-10) -> data-invalid="" + aria-invalid="true"
        focus_node(&mut harness, "#df-seg-4");
        send_key_and_pump(&mut harness, Key::Named(NamedKey::ArrowDown), Code::ArrowDown);
        assert_eq!(harness.text_content("#df-seg-4"), "2025");
        assert_eq!(
            harness.attr("#df-input", "data-invalid").as_deref(),
            Some("")
        );
        assert_eq!(
            harness.attr("#df-seg-4", "aria-invalid").as_deref(),
            Some("true")
        );

        // Declarative VisuallyHidden live region (5C D-6)
        assert_eq!(
            harness.attr("[role='status']", "aria-live").as_deref(),
            Some("polite")
        );
    }
}

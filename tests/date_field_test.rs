use dioxus::prelude::*;
use monoxus::{
    date_field::{
        DATE_FIELD_PARTS, DateField, DateFieldInput, DateFieldLabel, DateFieldPart,
        DateFieldRelationships, DateFieldRoot, DateFieldSegment, DateFieldSegmentItem,
        use_date_field_context,
    },
    foundation::{
        date_time::{
            CalendarDate, CalendarDateTime, CalendarLocale, CalendarTime, DateValue,
            EditableSegmentKind, Granularity, HourCycle,
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

    #[component]
    fn NativeDateFieldRemediationScenario() -> Element {
        rsx! {
            div {
                style: "padding: 16px;",
                form {
                    id: "df-rem-form",
                    DateFieldRoot {
                        id: "df-rem-default",
                        locale: CalendarLocale::EnUs,
                        default_value: d(2026, 6, 15),
                        required: true,
                        DateFieldLabel {
                            id: "df-custom-lbl",
                            style: "display: block; width: 120px; height: 24px;",
                            "Default Date"
                        }
                        DateFieldInput {
                            id: "df-rem-input",
                            name: Some("rem_default".to_string()),
                            style: "display: flex; gap: 4px;",
                            render_children: move |segments: Vec<DateFieldSegmentItem>| rsx! {
                                for seg in segments {
                                    DateFieldSegment {
                                        key: "{seg.index}",
                                        id: format!("df-rem-seg-{}", seg.index),
                                        part: seg.kind(),
                                        index: Some(seg.index),
                                        style: "display: inline-block; min-width: 24px; height: 24px;",
                                    }
                                }
                            }
                        }
                    }
                    DateFieldRoot {
                        id: "df-rem-empty",
                        locale: CalendarLocale::EnUs,
                        DateFieldLabel { id: "df-empty-lbl", "Empty Date" }
                        DateFieldInput {
                            id: "df-empty-input",
                            name: Some("rem_empty".to_string()),
                            style: "display: flex; gap: 4px;",
                            render_children: move |segments: Vec<DateFieldSegmentItem>| rsx! {
                                for seg in segments {
                                    DateFieldSegment {
                                        key: "{seg.index}",
                                        id: format!("df-empty-seg-{}", seg.index),
                                        part: seg.kind(),
                                        index: Some(seg.index),
                                        style: "display: inline-block; min-width: 24px; height: 24px;",
                                    }
                                }
                            }
                        }
                    }
                    button {
                        id: "df-rem-reset",
                        r#type: "reset",
                        style: "width: 80px; height: 28px;",
                        "Reset"
                    }
                }
                DateFieldRoot {
                    id: "df-rem-disabled",
                    locale: CalendarLocale::EnUs,
                    default_value: d(2026, 6, 15),
                    disabled: true,
                    DateFieldLabel { id: "df-dis-lbl", "Disabled Date" }
                    DateFieldInput {
                        id: "df-dis-input",
                        style: "display: flex; gap: 4px;",
                        render_children: move |segments: Vec<DateFieldSegmentItem>| rsx! {
                            for seg in segments {
                                DateFieldSegment {
                                    key: "{seg.index}",
                                    id: format!("df-dis-seg-{}", seg.index),
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
    fn native_date_field_year_cap_custom_label_required_disabled_and_form_reset() {
        let _lock = NATIVE_TEST_LOCK.lock().unwrap_or_else(|e| e.into_inner());
        let mut harness = mount_native_scenario(NativeDateFieldRemediationScenario);

        // DEF-5C-07: custom label id synced to aria-labelledby
        assert_eq!(
            harness.attr("#df-rem-input", "aria-labelledby").as_deref(),
            Some("df-custom-lbl")
        );
        // DEF-5C-15: required emits aria-required="true" and data-required=""
        assert_eq!(
            harness.attr("#df-rem-input", "aria-required").as_deref(),
            Some("true")
        );
        assert_eq!(
            harness.attr("#df-rem-input", "data-required").as_deref(),
            Some("")
        );

        // DEF-5C-02: typing 6 '2's into year stays capped at "2222"
        focus_node(&mut harness, "#df-rem-seg-4");
        for _ in 0..6 {
            send_key_and_pump(&mut harness, Key::Character("2".into()), Code::Digit2);
        }
        assert_eq!(harness.text_content("#df-rem-seg-4"), "2222");
        assert_eq!(
            harness.attr("input[name='rem_default']", "value").as_deref(),
            Some("2222-06-15")
        );

        // Type partial digit into empty field (uncommitted buffer)
        focus_node(&mut harness, "#df-empty-seg-0");
        send_key_and_pump(&mut harness, Key::Character("3".into()), Code::Digit3);
        assert_eq!(harness.text_content("#df-empty-seg-0"), "03");

        // DEF-5C-03 & DEF-5C-08: form reset restores default_value AND clears partial buffer
        click_and_pump(&mut harness, "#df-rem-reset");
        assert_eq!(harness.text_content("#df-rem-seg-4"), "2026");
        assert_eq!(
            harness.attr("input[name='rem_default']", "value").as_deref(),
            Some("2026-06-15")
        );
        assert_eq!(harness.text_content("#df-empty-seg-0"), "mm");
        assert_eq!(
            harness.attr("#df-empty-seg-0", "data-placeholder").as_deref(),
            Some("")
        );

        // DEF-5C-12: disabled segment has no tabindex and ignores keydown mutations
        assert_eq!(harness.attr("#df-dis-seg-0", "tabindex"), None);
        assert_eq!(
            harness.attr("#df-dis-seg-0", "data-disabled").as_deref(),
            Some("")
        );
    }

    #[component]
    fn NativeDateFieldReadOnlyScenario() -> Element {
        rsx! {
            div {
                style: "padding: 16px;",
                DateFieldRoot {
                    id: "df-ro-all",
                    locale: CalendarLocale::EnUs,
                    default_value: d(2026, 6, 15),
                    read_only: true,
                    DateFieldLabel { id: "df-ro-lbl", "ReadOnly Date" }
                    DateFieldInput {
                        id: "df-ro-input",
                        name: Some("ro_date".to_string()),
                        style: "display: flex; gap: 4px;",
                        render_children: move |segments: Vec<DateFieldSegmentItem>| rsx! {
                            for seg in segments {
                                DateFieldSegment {
                                    key: "{seg.index}",
                                    id: format!("df-ro-seg-{}", seg.index),
                                    part: seg.kind(),
                                    index: Some(seg.index),
                                    style: "display: inline-block; min-width: 24px; height: 24px;",
                                }
                            }
                        }
                    }
                }
                DateFieldRoot {
                    id: "df-ro-partial",
                    locale: CalendarLocale::EnUs,
                    default_value: d(2026, 6, 15),
                    readonly_segments: vec![EditableSegmentKind::Year],
                    DateFieldLabel { id: "df-roseg-lbl", "Partial ReadOnly Date" }
                    DateFieldInput {
                        id: "df-roseg-input",
                        name: Some("roseg_date".to_string()),
                        style: "display: flex; gap: 4px;",
                        render_children: move |segments: Vec<DateFieldSegmentItem>| rsx! {
                            for seg in segments {
                                DateFieldSegment {
                                    key: "{seg.index}",
                                    id: format!("df-roseg-seg-{}", seg.index),
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
    fn native_date_field_read_only_and_readonly_segments() {
        let _lock = NATIVE_TEST_LOCK.lock().unwrap_or_else(|e| e.into_inner());
        let mut harness = mount_native_scenario(NativeDateFieldReadOnlyScenario);

        // Whole-field read_only: true marks editable segments readonly and blocks mutations
        assert_eq!(
            harness.attr("#df-ro-seg-0", "aria-readonly").as_deref(),
            Some("true")
        );
        assert_eq!(
            harness.attr("#df-ro-seg-0", "data-readonly").as_deref(),
            Some("")
        );
        focus_node(&mut harness, "#df-ro-seg-0");
        send_key_and_pump(&mut harness, Key::Named(NamedKey::ArrowUp), Code::ArrowUp);
        send_key_and_pump(&mut harness, Key::Character("9".into()), Code::Digit9);
        assert_eq!(harness.text_content("#df-ro-seg-0"), "06");
        assert_eq!(
            harness.attr("input[name='ro_date']", "value").as_deref(),
            Some("2026-06-15")
        );

        // readonly_segments: [Year] locks Year only while keeping Month/Day editable
        assert_eq!(
            harness.attr("#df-roseg-seg-4", "aria-readonly").as_deref(),
            Some("true")
        );
        assert_eq!(
            harness.attr("#df-roseg-seg-4", "data-readonly").as_deref(),
            Some("")
        );
        assert_eq!(
            harness.attr("#df-roseg-seg-2", "aria-readonly").as_deref(),
            Some("false")
        );
        assert_eq!(harness.attr("#df-roseg-seg-2", "data-readonly"), None);

        // Year ignores edits
        focus_node(&mut harness, "#df-roseg-seg-4");
        send_key_and_pump(&mut harness, Key::Named(NamedKey::ArrowUp), Code::ArrowUp);
        send_key_and_pump(&mut harness, Key::Character("1".into()), Code::Digit1);
        assert_eq!(harness.text_content("#df-roseg-seg-4"), "2026");
        assert_eq!(
            harness.attr("input[name='roseg_date']", "value").as_deref(),
            Some("2026-06-15")
        );

        // Day remains editable
        focus_node(&mut harness, "#df-roseg-seg-2");
        send_key_and_pump(&mut harness, Key::Named(NamedKey::ArrowUp), Code::ArrowUp);
        assert_eq!(harness.text_content("#df-roseg-seg-2"), "16");
        assert_eq!(
            harness.attr("input[name='roseg_date']", "value").as_deref(),
            Some("2026-06-16")
        );
    }
}

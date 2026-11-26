use dioxus::prelude::*;
use monoxus::{
    foundation::{
        date_time::{CalendarLocale, CalendarTime, HourCycle, TimeGranularity, TimeValue},
        shared::ScopeHandle,
    },
    time_field::{
        TIME_FIELD_PARTS, TimeField, TimeFieldInput, TimeFieldLabel, TimeFieldPart,
        TimeFieldRelationships, TimeFieldRoot, TimeFieldSegment, TimeFieldSegmentItem,
        use_time_field_context,
    },
};

fn t(h: u8, m: u8, s: u8) -> TimeValue {
    TimeValue::Time(CalendarTime::from_hms(h, m, s).expect("valid time"))
}

#[test]
fn time_field_public_contracts_and_vdom_smoke() {
    assert_eq!(
        TIME_FIELD_PARTS
            .iter()
            .map(TimeFieldPart::as_str)
            .collect::<Vec<_>>(),
        ["input", "label", "segment"]
    );
    let scope = ScopeHandle::root("time-field").child("contract");
    let rels = TimeFieldRelationships::new(scope.clone());
    assert_eq!(rels.input_id(), scope.qualify("input"));
    assert_eq!(rels.label_id(), scope.qualify("label"));

    #[component]
    fn TimeFieldVdomProbe() -> Element {
        let ctx = use_time_field_context();
        let segs = ctx.segments();
        assert!(!segs.is_empty());
        rsx! {
            TimeFieldLabel { "Alarm" }
            TimeFieldInput {
                name: Some("alarm".to_string()),
                for seg in segs {
                    TimeFieldSegment {
                        key: "{seg.index}",
                        part: seg.kind(),
                        index: Some(seg.index),
                    }
                }
            }
        }
    }

    #[component]
    fn TimeFieldVdomApp() -> Element {
        rsx! {
            TimeField {
                id: "vdom-tf",
                default_value: t(9, 30, 0),
                granularity: TimeGranularity::Minute,
                hour_cycle: HourCycle::H12,
                locale: CalendarLocale::EnUs,
                TimeFieldVdomProbe {}
            }
        }
    }

    let mut dom = VirtualDom::new(TimeFieldVdomApp);
    dom.rebuild_in_place();
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
    fn NativeTimeFieldScenario() -> Element {
        rsx! {
            div {
                style: "padding: 16px;",
                TimeFieldRoot {
                    id: "native-tf",
                    locale: CalendarLocale::EnUs,
                    granularity: TimeGranularity::Minute,
                    hour_cycle: HourCycle::H12,
                    default_value: t(9, 15, 0),
                    TimeFieldLabel { id: "tf-label", "Meeting Time" }
                    TimeFieldInput {
                        id: "tf-input",
                        name: Some("meeting_time".to_string()),
                        style: "display: flex; gap: 4px;",
                        render_children: move |segments: Vec<TimeFieldSegmentItem>| rsx! {
                            for seg in segments {
                                TimeFieldSegment {
                                    key: "{seg.index}",
                                    id: format!("tf-seg-{}", seg.index),
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
    fn native_time_field_day_period_toggle_data_attr_and_clear_to_none() {
        let _lock = NATIVE_TEST_LOCK.lock().unwrap_or_else(|e| e.into_inner());
        let mut harness = mount_native_scenario(NativeTimeFieldScenario);

        // data-time-field-segment="" emitted (5C D-4)
        assert_eq!(
            harness.attr("#tf-seg-0", "data-time-field-segment").as_deref(),
            Some("")
        );
        assert_eq!(
            harness.attr("input[name='meeting_time']", "value").as_deref(),
            Some("09:15:00")
        );

        // Focus dayPeriod segment (index 4: hour=0, ":"=1, min=2, " "=3, dayPeriod=4)
        focus_node(&mut harness, "#tf-seg-4");
        assert_native_focus_id(&harness, "tf-seg-4");
        send_key_and_pump(&mut harness, Key::Character("p".into()), Code::KeyP);
        assert_eq!(harness.text_content("#tf-seg-4"), "PM");
        assert_eq!(
            harness.attr("input[name='meeting_time']", "value").as_deref(),
            Some("21:15:00")
        );

        // Delete on minute (index 2) clears value to None -> hidden input becomes "" (5C D-4)
        focus_node(&mut harness, "#tf-seg-2");
        send_key_and_pump(&mut harness, Key::Named(NamedKey::Delete), Code::Delete);
        assert_eq!(
            harness.attr("input[name='meeting_time']", "value").as_deref(),
            Some("")
        );
    }

    #[component]
    fn NativeTimeFieldRemediationScenario() -> Element {
        rsx! {
            div {
                style: "padding: 16px;",
                form {
                    id: "tf-rem-form",
                    TimeFieldRoot {
                        id: "tf-empty-12h",
                        locale: CalendarLocale::EnUs,
                        granularity: TimeGranularity::Minute,
                        hour_cycle: HourCycle::H12,
                        required: true,
                        TimeFieldLabel { id: "tf-custom-lbl", "Empty 12h" }
                        TimeFieldInput {
                            id: "tf-empty-input",
                            name: Some("empty_12h".to_string()),
                            style: "display: flex; gap: 4px;",
                            render_children: move |segments: Vec<TimeFieldSegmentItem>| rsx! {
                                for seg in segments {
                                    TimeFieldSegment {
                                        key: "{seg.index}",
                                        id: format!("tf-empty-seg-{}", seg.index),
                                        part: seg.kind(),
                                        index: Some(seg.index),
                                        style: "display: inline-block; min-width: 24px; height: 24px;",
                                    }
                                }
                            }
                        }
                    }
                    TimeFieldRoot {
                        id: "tf-toggle-12h",
                        locale: CalendarLocale::EnUs,
                        granularity: TimeGranularity::Minute,
                        hour_cycle: HourCycle::H12,
                        TimeFieldLabel { id: "tf-toggle-lbl", "Toggle 12h" }
                        TimeFieldInput {
                            id: "tf-toggle-input",
                            name: Some("toggle_12h".to_string()),
                            style: "display: flex; gap: 4px;",
                            render_children: move |segments: Vec<TimeFieldSegmentItem>| rsx! {
                                for seg in segments {
                                    TimeFieldSegment {
                                        key: "{seg.index}",
                                        id: format!("tf-toggle-seg-{}", seg.index),
                                        part: seg.kind(),
                                        index: Some(seg.index),
                                        style: "display: inline-block; min-width: 24px; height: 24px;",
                                    }
                                }
                            }
                        }
                    }
                    TimeFieldRoot {
                        id: "tf-bs-12h",
                        locale: CalendarLocale::EnUs,
                        granularity: TimeGranularity::Minute,
                        hour_cycle: HourCycle::H12,
                        default_value: t(22, 30, 0),
                        TimeFieldLabel { id: "tf-bs-lbl", "Backspace 12h" }
                        TimeFieldInput {
                            id: "tf-bs-input",
                            name: Some("bs_12h".to_string()),
                            style: "display: flex; gap: 4px;",
                            render_children: move |segments: Vec<TimeFieldSegmentItem>| rsx! {
                                for seg in segments {
                                    TimeFieldSegment {
                                        key: "{seg.index}",
                                        id: format!("tf-bs-seg-{}", seg.index),
                                        part: seg.kind(),
                                        index: Some(seg.index),
                                        style: "display: inline-block; min-width: 24px; height: 24px;",
                                    }
                                }
                            }
                        }
                    }
                    button {
                        id: "tf-rem-reset",
                        r#type: "reset",
                        style: "width: 80px; height: 28px;",
                        "Reset"
                    }
                }
            }
        }
    }

    #[test]
    fn native_time_field_empty_12h_commit_day_period_toggle_backspace_and_reset() {
        let _lock = NATIVE_TEST_LOCK.lock().unwrap_or_else(|e| e.into_inner());
        let mut harness = mount_native_scenario(NativeTimeFieldRemediationScenario);

        // DEF-5C-07 & DEF-5C-15: custom label id & required attrs
        assert_eq!(
            harness.attr("#tf-empty-input", "aria-labelledby").as_deref(),
            Some("tf-custom-lbl")
        );
        assert_eq!(
            harness.attr("#tf-empty-input", "aria-required").as_deref(),
            Some("true")
        );
        assert_eq!(
            harness.attr("#tf-empty-input", "data-required").as_deref(),
            Some("")
        );

        // DEF-5C-01: Empty 12h starts with placeholder "AM", typing 09 30 commits immediately
        assert_eq!(harness.text_content("#tf-empty-seg-4"), "AM");
        assert_eq!(
            harness.attr("#tf-empty-seg-4", "data-placeholder").as_deref(),
            Some("")
        );
        focus_node(&mut harness, "#tf-empty-seg-0");
        send_key_and_pump(&mut harness, Key::Character("0".into()), Code::Digit0);
        send_key_and_pump(&mut harness, Key::Character("9".into()), Code::Digit9);
        assert_native_focus_id(&harness, "tf-empty-seg-2");
        send_key_and_pump(&mut harness, Key::Character("3".into()), Code::Digit3);
        send_key_and_pump(&mut harness, Key::Character("0".into()), Code::Digit0);
        assert_eq!(
            harness.attr("input[name='empty_12h']", "value").as_deref(),
            Some("09:30:00")
        );
        assert_eq!(harness.attr("#tf-empty-seg-4", "data-placeholder"), None);

        // DEF-5C-01: ArrowUp on empty 12h dayPeriod toggles AM -> PM without filling hour/minute
        focus_node(&mut harness, "#tf-toggle-seg-4");
        send_key_and_pump(&mut harness, Key::Named(NamedKey::ArrowUp), Code::ArrowUp);
        assert_eq!(harness.text_content("#tf-toggle-seg-4"), "PM");
        assert_eq!(harness.text_content("#tf-toggle-seg-0"), "––");
        assert_eq!(harness.text_content("#tf-toggle-seg-2"), "––");
        assert_eq!(
            harness.attr("input[name='toggle_12h']", "value").as_deref(),
            Some("")
        );

        // DEF-5C-16: Backspace on 12h "10" (22:30 PM) immediately displays "01" and commits 13:30:00
        focus_node(&mut harness, "#tf-bs-seg-0");
        send_key_and_pump(&mut harness, Key::Named(NamedKey::Backspace), Code::Backspace);
        assert_eq!(harness.text_content("#tf-bs-seg-0"), "01");
        assert_eq!(
            harness.attr("input[name='bs_12h']", "value").as_deref(),
            Some("13:30:00")
        );

        // DEF-5C-03 & DEF-5C-08: form reset restores default_value (22:30:00) and clears empty field
        click_and_pump(&mut harness, "#tf-rem-reset");
        assert_eq!(
            harness.attr("input[name='bs_12h']", "value").as_deref(),
            Some("22:30:00")
        );
        assert_eq!(harness.text_content("#tf-bs-seg-0"), "10");
        assert_eq!(
            harness.attr("input[name='empty_12h']", "value").as_deref(),
            Some("")
        );
        assert_eq!(harness.text_content("#tf-empty-seg-0"), "––");
    }

    #[component]
    fn NativeTimeFieldFmDt3ValidateAndLabelSwapScenario() -> Element {
        use monoxus::foundation::date_time::{FieldInvalidEvent, FieldInvalidReason};

        let mut enforce_custom = use_signal(|| false);
        let mut swap_label = use_signal(|| false);
        let mut invalid_reason = use_signal(|| Option::<FieldInvalidReason>::None);

        let validate = if *enforce_custom.read() {
            Some(Callback::new(|_v: TimeValue| {
                Some(vec!["Outside business hours".to_string()])
            }))
        } else {
            None
        };
        let label_key = if *swap_label.read() { "lbl-b" } else { "lbl-a" };
        let label_id = if *swap_label.read() {
            "tf-b5-lbl-b"
        } else {
            "tf-b5-lbl-a"
        };

        rsx! {
            div {
                style: "padding: 16px;",
                button {
                    id: "tf-b5-toggle-validate",
                    r#type: "button",
                    style: "width: 100px; height: 28px;",
                    onclick: move |_| enforce_custom.toggle(),
                    "Toggle Validate"
                }
                button {
                    id: "tf-b5-swap-label",
                    r#type: "button",
                    style: "width: 100px; height: 28px;",
                    onclick: move |_| swap_label.toggle(),
                    "Swap Label"
                }
                TimeFieldRoot {
                    id: "tf-b5-root",
                    locale: CalendarLocale::Iso,
                    granularity: TimeGranularity::Minute,
                    hour_cycle: HourCycle::H24,
                    default_value: t(10, 0, 0),
                    validate,
                    on_invalid: move |evt: FieldInvalidEvent| {
                        invalid_reason.set(Some(evt.reason));
                    },
                    for (lkey, lid) in [(label_key, label_id)] {
                        TimeFieldLabel {
                            key: "{lkey}",
                            id: lid.to_string(),
                            "Meeting Time"
                        }
                    }
                    TimeFieldInput {
                        id: "tf-b5-input",
                        render_children: move |segments: Vec<TimeFieldSegmentItem>| rsx! {
                            for seg in segments {
                                TimeFieldSegment {
                                    key: "{seg.index}",
                                    id: format!("tf-b5-seg-{}", seg.index),
                                    part: seg.kind(),
                                    index: Some(seg.index),
                                }
                            }
                        }
                    }
                }
                span { id: "tf-b5-reason", "{invalid_reason.read():?}" }
            }
        }
    }

    #[test]
    fn native_time_field_fm_dt3_dynamic_validate_reactivity_and_keyed_label_swap() {
        let _lock = NATIVE_TEST_LOCK.lock().unwrap_or_else(|e| e.into_inner());
        let mut harness = mount_native_scenario(NativeTimeFieldFmDt3ValidateAndLabelSwapScenario);

        // Initially valid
        assert_eq!(harness.attr("#tf-b5-input", "data-invalid"), None);
        assert_eq!(harness.attr("#tf-b5-seg-0", "aria-invalid"), None);

        // B5 (FM-DT-3): swapping `validate` without changing `value` immediately updates `data-invalid` and fires `on_invalid`
        click_and_pump(&mut harness, "#tf-b5-toggle-validate");
        assert_eq!(
            harness.attr("#tf-b5-input", "data-invalid").as_deref(),
            Some("")
        );
        assert_eq!(
            harness.attr("#tf-b5-seg-0", "aria-invalid").as_deref(),
            Some("true")
        );
        assert_eq!(
            harness.text_content("#tf-b5-reason"),
            "Some(Custom([\"Outside business hours\"]))"
        );

        // Clearing `validate` immediately clears invalid state
        click_and_pump(&mut harness, "#tf-b5-toggle-validate");
        assert_eq!(harness.attr("#tf-b5-input", "data-invalid"), None);
        assert_eq!(harness.attr("#tf-b5-seg-0", "aria-invalid"), None);

        // FM-DT-5: keyed label swap preserves replacement `custom_label_id`
        assert_eq!(
            harness.attr("#tf-b5-input", "aria-labelledby").as_deref(),
            Some("tf-b5-lbl-a")
        );
        click_and_pump(&mut harness, "#tf-b5-swap-label");
        assert_eq!(
            harness.attr("#tf-b5-input", "aria-labelledby").as_deref(),
            Some("tf-b5-lbl-b")
        );
    }
}

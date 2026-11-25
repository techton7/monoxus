use dioxus::prelude::*;
use monoxus::{
    foundation::{
        date_time::{CalendarLocale, CalendarTime, HourCycle, TimeGranularity, TimeRange, TimeValue},
        shared::ScopeHandle,
    },
    time_field::TimeFieldSegmentItem,
    time_range_field::{
        RangeFieldInputKind, TIME_RANGE_FIELD_PARTS, TimeRangeField, TimeRangeFieldInput,
        TimeRangeFieldLabel, TimeRangeFieldPart, TimeRangeFieldRelationships, TimeRangeFieldRoot,
        TimeRangeFieldSegment,
    },
};

fn t(h: u8, m: u8, s: u8) -> TimeValue {
    TimeValue::Time(CalendarTime::from_hms(h, m, s).expect("valid time"))
}

#[test]
fn time_range_field_public_contracts_and_vdom_smoke() {
    assert_eq!(
        TIME_RANGE_FIELD_PARTS
            .iter()
            .map(TimeRangeFieldPart::as_str)
            .collect::<Vec<_>>(),
        ["root", "input", "label", "segment"]
    );
    let scope = ScopeHandle::root("time-range-field").child("contract");
    let rels = TimeRangeFieldRelationships::new(scope.clone());
    assert_eq!(rels.root_id(), scope.token());
    assert_eq!(rels.label_id(), scope.qualify("label"));

    #[component]
    fn TimeRangeFieldVdomApp() -> Element {
        rsx! {
            TimeRangeField {
                id: "vdom-trf",
                default_value: TimeRange::new(Some(t(9, 0, 0)), Some(t(17, 0, 0))),
                granularity: TimeGranularity::Minute,
                hour_cycle: HourCycle::H24,
                locale: CalendarLocale::Iso,
                TimeRangeFieldLabel { "Business Hours" }
                TimeRangeFieldInput {
                    kind: RangeFieldInputKind::Start,
                    name: Some("open_time".to_string()),
                    render_children: move |segments: Vec<TimeFieldSegmentItem>| rsx! {
                        for seg in segments {
                            TimeRangeFieldSegment {
                                key: "{seg.index}",
                                part: seg.kind(),
                                index: Some(seg.index),
                            }
                        }
                    }
                }
                TimeRangeFieldInput {
                    kind: RangeFieldInputKind::End,
                    name: Some("close_time".to_string()),
                    render_children: move |segments: Vec<TimeFieldSegmentItem>| rsx! {
                        for seg in segments {
                            TimeRangeFieldSegment {
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

    let mut dom = VirtualDom::new(TimeRangeFieldVdomApp);
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
    fn NativeTimeRangeFieldScenario() -> Element {
        rsx! {
            div {
                style: "padding: 16px;",
                TimeRangeFieldRoot {
                    id: "trf-root",
                    locale: CalendarLocale::Iso,
                    granularity: TimeGranularity::Minute,
                    hour_cycle: HourCycle::H24,
                    default_value: TimeRange::new(Some(t(20, 0, 0)), Some(t(5, 0, 0))),
                    TimeRangeFieldLabel { id: "trf-label", "Overnight Shift" }
                    TimeRangeFieldInput {
                        id: "trf-start",
                        kind: RangeFieldInputKind::Start,
                        name: Some("shift_start".to_string()),
                        style: "display: flex; gap: 4px;",
                        render_children: move |segments: Vec<TimeFieldSegmentItem>| rsx! {
                            for seg in segments {
                                TimeRangeFieldSegment {
                                    key: "{seg.index}",
                                    id: format!("trf-start-seg-{}", seg.index),
                                    part: seg.kind(),
                                    index: Some(seg.index),
                                    style: "display: inline-block; min-width: 24px; height: 24px;",
                                }
                            }
                        }
                    }
                    TimeRangeFieldInput {
                        id: "trf-end",
                        kind: RangeFieldInputKind::End,
                        name: Some("shift_end".to_string()),
                        style: "display: flex; gap: 4px;",
                        render_children: move |segments: Vec<TimeFieldSegmentItem>| rsx! {
                            for seg in segments {
                                TimeRangeFieldSegment {
                                    key: "{seg.index}",
                                    id: format!("trf-end-seg-{}", seg.index),
                                    part: seg.kind(),
                                    index: Some(seg.index),
                                    style: "display: inline-block; min-width: 24px; height: 24px;",
                                }
                            }
                        }
                    }
                }
                TimeRangeFieldRoot {
                    id: "trf-same-day-root",
                    locale: CalendarLocale::Iso,
                    granularity: TimeGranularity::Minute,
                    hour_cycle: HourCycle::H24,
                    default_value: TimeRange::new(Some(t(10, 0, 0)), Some(t(10, 0, 0))),
                    validate: Callback::new(|range: TimeRange| {
                        range
                            .is_inverted()
                            .then(|| vec!["End time must be after start time".to_string()])
                    }),
                    TimeRangeFieldLabel { id: "trf-same-day-label", "Same-Day Shift" }
                    TimeRangeFieldInput {
                        id: "trf-same-day-start",
                        kind: RangeFieldInputKind::Start,
                        name: Some("same_day_start".to_string()),
                        style: "display: flex; gap: 4px;",
                        render_children: move |segments: Vec<TimeFieldSegmentItem>| rsx! {
                            for seg in segments {
                                TimeRangeFieldSegment {
                                    key: "{seg.index}",
                                    id: format!("trf-same-day-start-seg-{}", seg.index),
                                    part: seg.kind(),
                                    index: Some(seg.index),
                                    style: "display: inline-block; min-width: 24px; height: 24px;",
                                }
                            }
                        }
                    }
                    TimeRangeFieldInput {
                        id: "trf-same-day-end",
                        kind: RangeFieldInputKind::End,
                        name: Some("same_day_end".to_string()),
                        style: "display: flex; gap: 4px;",
                        render_children: move |segments: Vec<TimeFieldSegmentItem>| rsx! {
                            for seg in segments {
                                TimeRangeFieldSegment {
                                    key: "{seg.index}",
                                    id: format!("trf-same-day-end-seg-{}", seg.index),
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
    fn native_time_range_field_cross_traversal_and_overnight_validation() {
        let _lock = NATIVE_TEST_LOCK.lock().unwrap_or_else(|e| e.into_inner());
        let mut harness = mount_native_scenario(NativeTimeRangeFieldScenario);

        // Overnight range (20:00..05:00) is valid by default (5C D-2 / bits-ui parity)
        assert_eq!(harness.attr("#trf-root", "data-invalid"), None);
        assert_eq!(harness.attr("#trf-start", "data-invalid"), None);
        assert_eq!(harness.attr("#trf-end", "data-invalid"), None);
        assert_eq!(harness.attr("#trf-start-seg-0", "aria-invalid"), None);

        // 24h minute segments: hour=0, ":"=1, minute=2
        focus_node(&mut harness, "#trf-start-seg-2");
        assert_native_focus_id(&harness, "trf-start-seg-2");

        // ArrowRight crosses from Start minute -> End hour
        send_key_and_pump(&mut harness, Key::Named(NamedKey::ArrowRight), Code::ArrowRight);
        assert_native_focus_id(&harness, "trf-end-seg-0");

        // Decrement End hour -> 04:00 (still overnight 20:00..04:00) -> remains valid
        send_key_and_pump(&mut harness, Key::Named(NamedKey::ArrowDown), Code::ArrowDown);
        assert_eq!(harness.attr("#trf-root", "data-invalid"), None);
        assert_eq!(harness.attr("#trf-start", "data-invalid"), None);
        assert_eq!(harness.attr("#trf-end", "data-invalid"), None);
        assert_eq!(harness.attr("#trf-start-seg-0", "aria-invalid"), None);
        assert_eq!(
            harness.attr("input[name='shift_end']", "value").as_deref(),
            Some("04:00:00")
        );

        // Opt-in same-day ordering via `validate` callback invalidates root, inputs, segments
        focus_node(&mut harness, "#trf-same-day-end-seg-0");
        assert_native_focus_id(&harness, "trf-same-day-end-seg-0");
        send_key_and_pump(&mut harness, Key::Named(NamedKey::ArrowDown), Code::ArrowDown);
        assert_eq!(
            harness.attr("#trf-same-day-root", "data-invalid").as_deref(),
            Some("")
        );
        assert_eq!(
            harness.attr("#trf-same-day-start", "data-invalid").as_deref(),
            Some("")
        );
        assert_eq!(
            harness.attr("#trf-same-day-end", "data-invalid").as_deref(),
            Some("")
        );
        assert_eq!(
            harness
                .attr("#trf-same-day-start-seg-0", "aria-invalid")
                .as_deref(),
            Some("true")
        );
        assert_eq!(
            harness
                .attr("input[name='same_day_end']", "value")
                .as_deref(),
            Some("09:00:00")
        );

        // Increment End hour back to 10:00 -> valid
        send_key_and_pump(&mut harness, Key::Named(NamedKey::ArrowUp), Code::ArrowUp);
        assert_eq!(harness.attr("#trf-same-day-root", "data-invalid"), None);
        assert_eq!(
            harness.attr("#trf-same-day-start-seg-0", "aria-invalid"),
            None
        );
    }

    #[component]
    fn NativeTimeRangeRemediationScenario() -> Element {
        rsx! {
            div {
                style: "padding: 16px;",
                form {
                    id: "trf-rem-form",
                    TimeRangeFieldRoot {
                        id: "trf-rem-root",
                        locale: CalendarLocale::Iso,
                        granularity: TimeGranularity::Minute,
                        hour_cycle: HourCycle::H24,
                        default_value: TimeRange::new(Some(t(9, 0, 0)), Some(t(17, 0, 0))),
                        TimeRangeFieldLabel { id: "trf-custom-lbl", "Hours" }
                        TimeRangeFieldInput {
                            id: "trf-rem-start",
                            kind: RangeFieldInputKind::Start,
                            name: Some("rem_trf_start".to_string()),
                            style: "display: flex; gap: 4px;",
                            render_children: move |segments: Vec<TimeFieldSegmentItem>| rsx! {
                                for seg in segments {
                                    TimeRangeFieldSegment {
                                        key: "{seg.index}",
                                        id: format!("trf-rem-start-seg-{}", seg.index),
                                        part: seg.kind(),
                                        index: Some(seg.index),
                                        style: "display: inline-block; min-width: 24px; height: 24px;",
                                    }
                                }
                            }
                        }
                        TimeRangeFieldInput {
                            id: "trf-rem-end",
                            kind: RangeFieldInputKind::End,
                            name: Some("rem_trf_end".to_string()),
                            style: "display: flex; gap: 4px;",
                            render_children: move |segments: Vec<TimeFieldSegmentItem>| rsx! {
                                for seg in segments {
                                    TimeRangeFieldSegment {
                                        key: "{seg.index}",
                                        id: format!("trf-rem-end-seg-{}", seg.index),
                                        part: seg.kind(),
                                        index: Some(seg.index),
                                        style: "display: inline-block; min-width: 24px; height: 24px;",
                                    }
                                }
                            }
                        }
                    }
                    button {
                        id: "trf-rem-reset",
                        r#type: "reset",
                        style: "width: 80px; height: 28px;",
                        "Reset"
                    }
                }
                TimeRangeFieldRoot {
                    id: "trf-partial-root",
                    locale: CalendarLocale::Iso,
                    granularity: TimeGranularity::Minute,
                    hour_cycle: HourCycle::H24,
                    min_value: Some(t(9, 0, 0)),
                    default_value: TimeRange::new(Some(t(7, 0, 0)), None),
                    TimeRangeFieldLabel { id: "trf-partial-lbl", "Partial Time Min" }
                    TimeRangeFieldInput {
                        id: "trf-partial-start",
                        kind: RangeFieldInputKind::Start,
                        render_children: move |segments: Vec<TimeFieldSegmentItem>| rsx! {
                            for seg in segments {
                                TimeRangeFieldSegment {
                                    key: "{seg.index}",
                                    id: format!("trf-partial-start-seg-{}", seg.index),
                                    part: seg.kind(),
                                    index: Some(seg.index),
                                }
                            }
                        }
                    }
                    TimeRangeFieldInput {
                        id: "trf-partial-end",
                        kind: RangeFieldInputKind::End,
                        render_children: move |segments: Vec<TimeFieldSegmentItem>| rsx! {
                            for seg in segments {
                                TimeRangeFieldSegment {
                                    key: "{seg.index}",
                                    id: format!("trf-partial-end-seg-{}", seg.index),
                                    part: seg.kind(),
                                    index: Some(seg.index),
                                }
                            }
                        }
                    }
                }
            }
        }
    }

    #[test]
    fn native_time_range_field_custom_label_partial_min_max_and_form_reset() {
        let _lock = NATIVE_TEST_LOCK.lock().unwrap_or_else(|e| e.into_inner());
        let mut harness = mount_native_scenario(NativeTimeRangeRemediationScenario);

        // DEF-5C-07: custom label id synced to root & inputs
        assert_eq!(
            harness.attr("#trf-rem-root", "aria-labelledby").as_deref(),
            Some("trf-custom-lbl")
        );
        assert_eq!(
            harness.attr("#trf-rem-start", "aria-labelledby").as_deref(),
            Some("trf-custom-lbl")
        );

        // DEF-5C-13 / 5C D-8: partial min marks root, label, and Start invalid, NOT empty End
        assert_eq!(
            harness.attr("#trf-partial-root", "data-invalid").as_deref(),
            Some("")
        );
        assert_eq!(
            harness.attr("#trf-partial-start", "data-invalid").as_deref(),
            Some("")
        );
        assert_eq!(harness.attr("#trf-partial-end", "data-invalid"), None);

        // DEF-5C-08: edit Start then reset restores default_value.start & default_value.end
        focus_node(&mut harness, "#trf-rem-start-seg-0");
        send_key_and_pump(&mut harness, Key::Named(NamedKey::ArrowUp), Code::ArrowUp);
        assert_eq!(
            harness.attr("input[name='rem_trf_start']", "value").as_deref(),
            Some("10:00:00")
        );
        click_and_pump(&mut harness, "#trf-rem-reset");
        assert_eq!(
            harness.attr("input[name='rem_trf_start']", "value").as_deref(),
            Some("09:00:00")
        );
        assert_eq!(
            harness.attr("input[name='rem_trf_end']", "value").as_deref(),
            Some("17:00:00")
        );
    }
}

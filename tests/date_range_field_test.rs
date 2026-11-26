use dioxus::prelude::*;
use monoxus::{
    date_field::DateFieldSegmentItem,
    date_range_field::{
        DATE_RANGE_FIELD_PARTS, DateRangeField, DateRangeFieldInput, DateRangeFieldLabel,
        DateRangeFieldPart, DateRangeFieldRelationships, DateRangeFieldRoot, DateRangeFieldSegment,
        RangeFieldInputKind,
    },
    foundation::{
        date_time::{CalendarDate, CalendarLocale, DateValue, DateValueRange},
        shared::{Direction, ScopeHandle},
    },
};

fn d(y: i32, m: u8, day: u8) -> DateValue {
    DateValue::Date(CalendarDate::new(y, m, day).expect("valid date"))
}

#[test]
fn date_range_field_public_contracts_and_vdom_smoke() {
    assert_eq!(
        DATE_RANGE_FIELD_PARTS
            .iter()
            .map(DateRangeFieldPart::as_str)
            .collect::<Vec<_>>(),
        ["root", "input", "label", "segment"]
    );
    let scope = ScopeHandle::root("date-range-field").child("contract");
    let rels = DateRangeFieldRelationships::new(scope.clone());
    assert_eq!(rels.root_id(), scope.token());
    assert_eq!(rels.label_id(), scope.qualify("label"));

    #[component]
    fn DateRangeFieldVdomApp() -> Element {
        rsx! {
            DateRangeField {
                id: "vdom-drf",
                default_value: DateValueRange::new(Some(d(2026, 5, 10)), Some(d(2026, 5, 20))),
                DateRangeFieldLabel { "Trip Dates" }
                DateRangeFieldInput {
                    kind: RangeFieldInputKind::Start,
                    name: Some("start_date".to_string()),
                    render_children: move |segments: Vec<DateFieldSegmentItem>| rsx! {
                        for seg in segments {
                            DateRangeFieldSegment {
                                key: "{seg.index}",
                                part: seg.kind(),
                                index: Some(seg.index),
                            }
                        }
                    }
                }
                DateRangeFieldInput {
                    kind: RangeFieldInputKind::End,
                    name: Some("end_date".to_string()),
                    render_children: move |segments: Vec<DateFieldSegmentItem>| rsx! {
                        for seg in segments {
                            DateRangeFieldSegment {
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

    let mut dom = VirtualDom::new(DateRangeFieldVdomApp);
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
    fn NativeDateRangeFieldScenario() -> Element {
        rsx! {
            div {
                style: "padding: 16px;",
                DateRangeFieldRoot {
                    id: "drf-root",
                    locale: CalendarLocale::EnUs,
                    default_value: DateValueRange::new(
                        Some(d(2026, 5, 15)),
                        Some(d(2026, 5, 15)),
                    ),
                    DateRangeFieldLabel { id: "drf-label", "Booking Range" }
                    DateRangeFieldInput {
                        id: "drf-start",
                        kind: RangeFieldInputKind::Start,
                        name: Some("range_start".to_string()),
                        style: "display: flex; gap: 4px;",
                        render_children: move |segments: Vec<DateFieldSegmentItem>| rsx! {
                            for seg in segments {
                                DateRangeFieldSegment {
                                    key: "{seg.index}",
                                    id: format!("drf-start-seg-{}", seg.index),
                                    part: seg.kind(),
                                    index: Some(seg.index),
                                    style: "display: inline-block; min-width: 24px; height: 24px;",
                                }
                            }
                        }
                    }
                    DateRangeFieldInput {
                        id: "drf-end",
                        kind: RangeFieldInputKind::End,
                        name: Some("range_end".to_string()),
                        style: "display: flex; gap: 4px;",
                        render_children: move |segments: Vec<DateFieldSegmentItem>| rsx! {
                            for seg in segments {
                                DateRangeFieldSegment {
                                    key: "{seg.index}",
                                    id: format!("drf-end-seg-{}", seg.index),
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
    fn native_date_range_field_cross_traversal_and_range_order_invalidation() {
        let _lock = NATIVE_TEST_LOCK.lock().unwrap_or_else(|e| e.into_inner());
        let mut harness = mount_native_scenario(NativeDateRangeFieldScenario);

        // Role & dual hidden inputs
        assert_eq!(harness.attr("#drf-root", "role").as_deref(), Some("group"));
        assert_eq!(
            harness.attr("#drf-start", "role").as_deref(),
            Some("presentation")
        );
        assert_eq!(
            harness.attr("#drf-end", "role").as_deref(),
            Some("presentation")
        );
        assert_eq!(
            harness.attr("input[name='range_start']", "value").as_deref(),
            Some("2026-05-15")
        );
        assert_eq!(
            harness.attr("input[name='range_end']", "value").as_deref(),
            Some("2026-05-15")
        );

        // Cross-field ArrowRight from last Start segment (year=4) -> first End segment (month=0)
        focus_node(&mut harness, "#drf-start-seg-4");
        assert_native_focus_id(&harness, "drf-start-seg-4");
        send_key_and_pump(&mut harness, Key::Named(NamedKey::ArrowRight), Code::ArrowRight);
        assert_native_focus_id(&harness, "drf-end-seg-0");

        // Cross-field ArrowLeft reverses back to last Start segment
        send_key_and_pump(&mut harness, Key::Named(NamedKey::ArrowLeft), Code::ArrowLeft);
        assert_native_focus_id(&harness, "drf-start-seg-4");

        // Increment Start year -> 2027-05-15 > End 2026-05-15 -> RangeOrder invalidates root, inputs, label, segments (5C D-2 / 5C D-4)
        send_key_and_pump(&mut harness, Key::Named(NamedKey::ArrowUp), Code::ArrowUp);
        assert_eq!(
            harness.attr("#drf-root", "data-invalid").as_deref(),
            Some("")
        );
        assert_eq!(
            harness.attr("#drf-label", "data-invalid").as_deref(),
            Some("")
        );
        assert_eq!(
            harness.attr("#drf-start", "data-invalid").as_deref(),
            Some("")
        );
        assert_eq!(
            harness.attr("#drf-end", "data-invalid").as_deref(),
            Some("")
        );
        assert_eq!(
            harness.attr("#drf-end-seg-0", "aria-invalid").as_deref(),
            Some("true")
        );

        // Decrement Start year back to 2026 -> valid again
        send_key_and_pump(&mut harness, Key::Named(NamedKey::ArrowDown), Code::ArrowDown);
        assert_eq!(harness.attr("#drf-root", "data-invalid"), None);
        assert_eq!(harness.attr("#drf-end", "data-invalid"), None);
        assert_eq!(harness.attr("#drf-end-seg-0", "aria-invalid"), None);
    }

    #[component]
    fn NativeDateRangeRemediationScenario() -> Element {
        rsx! {
            div {
                style: "padding: 16px;",
                form {
                    id: "drf-rem-form",
                    DateRangeFieldRoot {
                        id: "drf-rem-root",
                        locale: CalendarLocale::EnUs,
                        min_value: Some(d(2026, 5, 10)),
                        default_value: DateValueRange::new(
                            Some(d(2026, 5, 12)),
                            Some(d(2026, 5, 20)),
                        ),
                        DateRangeFieldLabel { id: "drf-custom-lbl", "Range With Reset" }
                        DateRangeFieldInput {
                            id: "drf-rem-start",
                            kind: RangeFieldInputKind::Start,
                            name: Some("rem_start".to_string()),
                            style: "display: flex; gap: 4px;",
                            render_children: move |segments: Vec<DateFieldSegmentItem>| rsx! {
                                for seg in segments {
                                    DateRangeFieldSegment {
                                        key: "{seg.index}",
                                        id: format!("drf-rem-start-seg-{}", seg.index),
                                        part: seg.kind(),
                                        index: Some(seg.index),
                                        style: "display: inline-block; min-width: 24px; height: 24px;",
                                    }
                                }
                            }
                        }
                        DateRangeFieldInput {
                            id: "drf-rem-end",
                            kind: RangeFieldInputKind::End,
                            name: Some("rem_end".to_string()),
                            style: "display: flex; gap: 4px;",
                            render_children: move |segments: Vec<DateFieldSegmentItem>| rsx! {
                                for seg in segments {
                                    DateRangeFieldSegment {
                                        key: "{seg.index}",
                                        id: format!("drf-rem-end-seg-{}", seg.index),
                                        part: seg.kind(),
                                        index: Some(seg.index),
                                        style: "display: inline-block; min-width: 24px; height: 24px;",
                                    }
                                }
                            }
                        }
                    }
                    button {
                        id: "drf-rem-reset",
                        r#type: "reset",
                        style: "width: 80px; height: 28px;",
                        "Reset"
                    }
                }
                DateRangeFieldRoot {
                    id: "drf-partial-root",
                    locale: CalendarLocale::EnUs,
                    min_value: Some(d(2026, 5, 10)),
                    default_value: DateValueRange::new(Some(d(2026, 5, 5)), None),
                    DateRangeFieldLabel { id: "drf-partial-lbl", "Partial Min" }
                    DateRangeFieldInput {
                        id: "drf-partial-start",
                        kind: RangeFieldInputKind::Start,
                        render_children: move |segments: Vec<DateFieldSegmentItem>| rsx! {
                            for seg in segments {
                                DateRangeFieldSegment {
                                    key: "{seg.index}",
                                    id: format!("drf-partial-start-seg-{}", seg.index),
                                    part: seg.kind(),
                                    index: Some(seg.index),
                                }
                            }
                        }
                    }
                    DateRangeFieldInput {
                        id: "drf-partial-end",
                        kind: RangeFieldInputKind::End,
                        render_children: move |segments: Vec<DateFieldSegmentItem>| rsx! {
                            for seg in segments {
                                DateRangeFieldSegment {
                                    key: "{seg.index}",
                                    id: format!("drf-partial-end-seg-{}", seg.index),
                                    part: seg.kind(),
                                    index: Some(seg.index),
                                }
                            }
                        }
                    }
                }
                DateRangeFieldRoot {
                    id: "drf-rtl-root",
                    locale: CalendarLocale::EnUs,
                    dir: Some(Direction::Rtl),
                    default_value: DateValueRange::new(
                        Some(d(2026, 5, 10)),
                        Some(d(2026, 5, 15)),
                    ),
                    DateRangeFieldLabel { id: "drf-rtl-lbl", "RTL Range" }
                    DateRangeFieldInput {
                        id: "drf-rtl-start",
                        kind: RangeFieldInputKind::Start,
                        render_children: move |segments: Vec<DateFieldSegmentItem>| rsx! {
                            for seg in segments {
                                DateRangeFieldSegment {
                                    key: "{seg.index}",
                                    id: format!("drf-rtl-start-seg-{}", seg.index),
                                    part: seg.kind(),
                                    index: Some(seg.index),
                                }
                            }
                        }
                    }
                    DateRangeFieldInput {
                        id: "drf-rtl-end",
                        kind: RangeFieldInputKind::End,
                        render_children: move |segments: Vec<DateFieldSegmentItem>| rsx! {
                            for seg in segments {
                                DateRangeFieldSegment {
                                    key: "{seg.index}",
                                    id: format!("drf-rtl-end-seg-{}", seg.index),
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
    fn native_date_range_field_rtl_custom_label_partial_min_max_and_form_reset() {
        let _lock = NATIVE_TEST_LOCK.lock().unwrap_or_else(|e| e.into_inner());
        let mut harness = mount_native_scenario(NativeDateRangeRemediationScenario);

        // DEF-5C-07: custom label id synced to root and child inputs
        assert_eq!(
            harness.attr("#drf-rem-root", "aria-labelledby").as_deref(),
            Some("drf-custom-lbl")
        );
        assert_eq!(
            harness.attr("#drf-rem-start", "aria-labelledby").as_deref(),
            Some("drf-custom-lbl")
        );

        // DEF-5C-13 / 5C D-8: partial min violation marks root, label, and Start invalid, NOT empty End
        assert_eq!(
            harness.attr("#drf-partial-root", "data-invalid").as_deref(),
            Some("")
        );
        assert_eq!(
            harness.attr("#drf-partial-lbl", "data-invalid").as_deref(),
            Some("")
        );
        assert_eq!(
            harness.attr("#drf-partial-start", "data-invalid").as_deref(),
            Some("")
        );
        assert_eq!(harness.attr("#drf-partial-end", "data-invalid"), None);

        // DEF-5C-08: edit Start and End, then form reset restores BOTH default endpoints
        focus_node(&mut harness, "#drf-rem-start-seg-4");
        send_key_and_pump(&mut harness, Key::Named(NamedKey::ArrowUp), Code::ArrowUp);
        assert_eq!(
            harness.attr("input[name='rem_start']", "value").as_deref(),
            Some("2027-05-12")
        );
        click_and_pump(&mut harness, "#drf-rem-reset");
        assert_eq!(
            harness.attr("input[name='rem_start']", "value").as_deref(),
            Some("2026-05-12")
        );
        assert_eq!(
            harness.attr("input[name='rem_end']", "value").as_deref(),
            Some("2026-05-20")
        );

        // RTL cross-field traversal: ArrowLeft moves Next across Start -> End; ArrowRight reverses
        focus_node(&mut harness, "#drf-rtl-start-seg-4");
        send_key_and_pump(&mut harness, Key::Named(NamedKey::ArrowLeft), Code::ArrowLeft);
        assert_native_focus_id(&harness, "drf-rtl-end-seg-0");
        send_key_and_pump(&mut harness, Key::Named(NamedKey::ArrowRight), Code::ArrowRight);
        assert_native_focus_id(&harness, "drf-rtl-start-seg-4");
    }

    #[component]
    fn NativeDateRangeFieldFmDt4Scenario() -> Element {
        use monoxus::foundation::date_time::{FieldInvalidEvent, FieldInvalidReason};

        let mut inverted_reason = use_signal(|| Option::<FieldInvalidReason>::None);

        rsx! {
            div {
                style: "padding: 16px;",
                // Inverted DateRangeField -> RangeOrder invalidates root, Start, AND End
                DateRangeFieldRoot {
                    id: "drf-fm4-inv-root",
                    locale: CalendarLocale::EnUs,
                    default_value: DateValueRange::new(Some(d(2026, 5, 20)), Some(d(2026, 5, 10))),
                    on_invalid: move |evt: FieldInvalidEvent| {
                        inverted_reason.set(Some(evt.reason));
                    },
                    DateRangeFieldInput {
                        id: "drf-fm4-inv-start",
                        kind: RangeFieldInputKind::Start,
                        render_children: move |segments: Vec<DateFieldSegmentItem>| rsx! {
                            for seg in segments {
                                DateRangeFieldSegment {
                                    key: "{seg.index}",
                                    part: seg.kind(),
                                    index: Some(seg.index),
                                }
                            }
                        }
                    }
                    DateRangeFieldInput {
                        id: "drf-fm4-inv-end",
                        kind: RangeFieldInputKind::End,
                        render_children: move |segments: Vec<DateFieldSegmentItem>| rsx! {
                            for seg in segments {
                                DateRangeFieldSegment {
                                    key: "{seg.index}",
                                    part: seg.kind(),
                                    index: Some(seg.index),
                                }
                            }
                        }
                    }
                }
                span { id: "drf-fm4-inv-reason", "{inverted_reason.read():?}" }

                // Start unavailable + End valid -> per-endpoint isolation (5C D-8)
                DateRangeFieldRoot {
                    id: "drf-fm4-iso-root",
                    locale: CalendarLocale::EnUs,
                    default_value: DateValueRange::new(Some(d(2026, 5, 12)), Some(d(2026, 5, 20))),
                    is_date_unavailable: move |date: CalendarDate| date.day == 12,
                    DateRangeFieldInput {
                        id: "drf-fm4-iso-start",
                        kind: RangeFieldInputKind::Start,
                        render_children: move |segments: Vec<DateFieldSegmentItem>| rsx! {
                            for seg in segments {
                                DateRangeFieldSegment {
                                    key: "{seg.index}",
                                    part: seg.kind(),
                                    index: Some(seg.index),
                                }
                            }
                        }
                    }
                    DateRangeFieldInput {
                        id: "drf-fm4-iso-end",
                        kind: RangeFieldInputKind::End,
                        render_children: move |segments: Vec<DateFieldSegmentItem>| rsx! {
                            for seg in segments {
                                DateRangeFieldSegment {
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
    }

    #[test]
    fn native_date_range_field_fm_dt4_inverted_range_order_and_endpoint_isolation() {
        let _lock = NATIVE_TEST_LOCK.lock().unwrap_or_else(|e| e.into_inner());
        let harness = mount_native_scenario(NativeDateRangeFieldFmDt4Scenario);

        // Inverted range invalidates root, Start, and End with RangeOrder
        assert_eq!(
            harness.attr("#drf-fm4-inv-root", "data-invalid").as_deref(),
            Some("")
        );
        assert_eq!(
            harness.attr("#drf-fm4-inv-start", "data-invalid").as_deref(),
            Some("")
        );
        assert_eq!(
            harness.attr("#drf-fm4-inv-end", "data-invalid").as_deref(),
            Some("")
        );

        // Start unavailable + End valid invalidates root & Start ONLY
        assert_eq!(
            harness.attr("#drf-fm4-iso-root", "data-invalid").as_deref(),
            Some("")
        );
        assert_eq!(
            harness.attr("#drf-fm4-iso-start", "data-invalid").as_deref(),
            Some("")
        );
        assert_eq!(harness.attr("#drf-fm4-iso-end", "data-invalid"), None);
    }

    #[component]
    fn NativeDateRangeFieldDropSafetyScenario() -> Element {
        let mut show_start = use_signal(|| true);
        let mut show_end = use_signal(|| true);
        let mut start_key_idx = use_signal(|| 0_u8);

        let start_key = if *start_key_idx.read() == 0 {
            "start-0"
        } else {
            "start-1"
        };

        rsx! {
            div {
                style: "padding: 16px;",
                div {
                    style: "display: flex; gap: 8px; margin-bottom: 8px;",
                    button {
                        id: "drf-drop-toggle-start",
                        r#type: "button",
                        style: "width: 90px; height: 28px;",
                        onclick: move |_| show_start.toggle(),
                        "Toggle Start"
                    }
                    button {
                        id: "drf-drop-toggle-end",
                        r#type: "button",
                        style: "width: 90px; height: 28px;",
                        onclick: move |_| show_end.toggle(),
                        "Toggle End"
                    }
                    button {
                        id: "drf-drop-swap-start-key",
                        r#type: "button",
                        style: "width: 90px; height: 28px;",
                        onclick: move |_| {
                            let next = 1 - *start_key_idx.read();
                            start_key_idx.set(next);
                        },
                        "Swap Key"
                    }
                }
                DateRangeFieldRoot {
                    id: "drf-drop-root",
                    locale: CalendarLocale::EnUs,
                    default_value: DateValueRange::new(
                        Some(d(2026, 5, 10)),
                        Some(d(2026, 5, 20)),
                    ),
                    DateRangeFieldLabel {
                        id: "drf-drop-label",
                        style: "display: block; width: 120px; height: 24px;",
                        "Drop Safe Range"
                    }
                    if *show_start.read() {
                        for sk in [start_key] {
                            DateRangeFieldInput {
                                key: "{sk}",
                                id: format!("drf-drop-{sk}"),
                                kind: RangeFieldInputKind::Start,
                                style: "display: flex; gap: 4px;",
                                render_children: move |segments: Vec<DateFieldSegmentItem>| rsx! {
                                    for seg in segments {
                                        DateRangeFieldSegment {
                                            key: "{seg.index}",
                                            id: format!("drf-drop-{sk}-seg-{}", seg.index),
                                            part: seg.kind(),
                                            index: Some(seg.index),
                                            style: "display: inline-block; min-width: 24px; height: 24px;",
                                        }
                                    }
                                }
                            }
                        }
                    }
                    if *show_end.read() {
                        DateRangeFieldInput {
                            id: "drf-drop-end",
                            kind: RangeFieldInputKind::End,
                            style: "display: flex; gap: 4px;",
                            render_children: move |segments: Vec<DateFieldSegmentItem>| rsx! {
                                for seg in segments {
                                    DateRangeFieldSegment {
                                        key: "{seg.index}",
                                        id: format!("drf-drop-end-seg-{}", seg.index),
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
    }

    #[test]
    fn native_date_range_field_unmount_endpoint_drop_safety_and_keyed_swap() {
        let _lock = NATIVE_TEST_LOCK.lock().unwrap_or_else(|e| e.into_inner());
        let mut harness = mount_native_scenario(NativeDateRangeFieldDropSafetyScenario);

        // 1. Keyed replacement of Start must not clobber start_runtime on old drop
        click_and_pump(&mut harness, "#drf-drop-swap-start-key");
        assert!(harness.query("#drf-drop-start-1").is_some());
        click_and_pump(&mut harness, "#drf-drop-label");
        assert_native_focus_id(&harness, "drf-drop-start-1-seg-0");

        // 2. Unmount End -> ArrowRight from Start's last segment & label click cause zero ValueDroppedError
        click_and_pump(&mut harness, "#drf-drop-toggle-end");
        assert!(harness.query("#drf-drop-end").is_none());
        focus_node(&mut harness, "#drf-drop-start-1-seg-4");
        send_key_and_pump(&mut harness, Key::Named(NamedKey::ArrowRight), Code::ArrowRight);
        click_and_pump(&mut harness, "#drf-drop-label");
        assert_native_focus_id(&harness, "drf-drop-start-1-seg-0");

        // 3. Remount End, unmount Start -> label click falls back to End & ArrowLeft from End causes zero panic
        click_and_pump(&mut harness, "#drf-drop-toggle-end");
        click_and_pump(&mut harness, "#drf-drop-toggle-start");
        assert!(harness.query("#drf-drop-start-1").is_none());
        assert!(harness.query("#drf-drop-end").is_some());
        click_and_pump(&mut harness, "#drf-drop-label");
        assert_native_focus_id(&harness, "drf-drop-end-seg-0");
        send_key_and_pump(&mut harness, Key::Named(NamedKey::ArrowLeft), Code::ArrowLeft);
        assert_native_focus_id(&harness, "drf-drop-end-seg-0");

        // 4. Unmount both endpoints -> label click causes zero panic
        click_and_pump(&mut harness, "#drf-drop-toggle-end");
        assert!(harness.query("#drf-drop-end").is_none());
        click_and_pump(&mut harness, "#drf-drop-label");
    }
}

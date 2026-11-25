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
        shared::ScopeHandle,
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
        NATIVE_TEST_LOCK, assert_native_focus_id, focus_node, mount_native_scenario,
        send_key_and_pump,
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
}

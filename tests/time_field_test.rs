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
        NATIVE_TEST_LOCK, assert_native_focus_id, focus_node, mount_native_scenario,
        send_key_and_pump,
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
}

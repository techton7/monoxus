use crate::foundation::{
    date_time::{CalendarDate, CalendarDateTime, CalendarTime, DateRange, DateValue, DateValueRange},
    shared::ScopeHandle,
};

use super::{
    attrs::{DateRangePickerContentAttrs, DateRangePickerTriggerAttrs},
    relationships::{DATE_RANGE_PICKER_OPEN_FOCUS_SELECTOR, DateRangePickerRelationships},
    state::{DateRangePickerDefinition, DateRangePickerState, merge_calendar_range_into_value},
    types::{DATE_RANGE_PICKER_PARTS, DateRangePickerPart},
};

fn d(y: i32, m: u8, day: u8) -> CalendarDate {
    CalendarDate::new(y, m, day).expect("valid date")
}

#[test]
fn date_range_picker_parts_and_relationships_are_deterministic() {
    assert_eq!(DATE_RANGE_PICKER_PARTS.len(), 11);
    assert_eq!(DateRangePickerPart::Portal.as_str(), "portal");
    assert_eq!(DateRangePickerPart::ContentStatic.as_str(), "content-static");

    let scope = ScopeHandle::root("date-range-picker").child("stay");
    let rels = DateRangePickerRelationships::new(scope.clone());
    assert_eq!(rels.root_id(), scope.token());
    assert_eq!(rels.trigger_id(), scope.qualify("trigger"));
    assert_eq!(rels.content_id(), scope.qualify("content"));
    assert_eq!(
        rels.open_focus_selector(),
        DATE_RANGE_PICKER_OPEN_FOCUS_SELECTOR
    );
}

#[test]
fn date_range_picker_preserves_time_components_when_picking_range() {
    let t_start = CalendarTime::from_hms(9, 0, 0).unwrap();
    let t_end = CalendarTime::from_hms(17, 30, 0).unwrap();
    let current = DateValueRange::new(
        Some(DateValue::DateTime(CalendarDateTime::new(d(2026, 4, 1), t_start))),
        Some(DateValue::DateTime(CalendarDateTime::new(d(2026, 4, 5), t_end))),
    );
    let placeholder = DateValue::DateTime(CalendarDateTime::new(d(2026, 4, 1), t_start));

    let merged = merge_calendar_range_into_value(
        current,
        placeholder,
        DateRange::new(Some(d(2026, 4, 10)), Some(d(2026, 4, 15))),
    );
    assert_eq!(
        merged.start,
        Some(DateValue::DateTime(CalendarDateTime::new(d(2026, 4, 10), t_start)))
    );
    assert_eq!(
        merged.end,
        Some(DateValue::DateTime(CalendarDateTime::new(d(2026, 4, 15), t_end)))
    );
}

#[test]
fn date_range_picker_closes_only_on_complete_range_commit() {
    let ph = DateValue::Date(d(2026, 4, 15));
    let mut state = DateRangePickerState::new(
        DateRangePickerDefinition::new(),
        DateValueRange::default(),
        ph,
        true,
    );

    // Partial selection (start only) must NOT close the popover.
    let partial = state.commit_calendar_range_select(DateRange::new(Some(d(2026, 4, 10)), None));
    assert!(!partial.should_close);
    assert!(state.is_open());

    // Complete range closes the popover when close_on_range_select = true.
    let complete = state.commit_calendar_range_select(DateRange::new(
        Some(d(2026, 4, 10)),
        Some(d(2026, 4, 14)),
    ));
    assert!(complete.should_close);
    assert!(!state.is_open());
    assert_eq!(
        complete.next_value,
        DateValueRange::new(
            Some(DateValue::Date(d(2026, 4, 10))),
            Some(DateValue::Date(d(2026, 4, 14)))
        )
    );

    // When close_on_range_select = false, popover remains open.
    let mut stay_open = DateRangePickerState::new(
        DateRangePickerDefinition::new().with_close_on_range_select(false),
        DateValueRange::default(),
        ph,
        true,
    );
    let outcome = stay_open.commit_calendar_range_select(DateRange::new(
        Some(d(2026, 4, 10)),
        Some(d(2026, 4, 14)),
    ));
    assert!(!outcome.should_close);
    assert!(stay_open.is_open());
}

#[test]
fn date_range_picker_attrs_follow_boolean_omission_invariant() {
    let trigger = DateRangePickerTriggerAttrs::new(
        "drp-trigger".into(),
        "drp-content".into(),
        true,
        false,
    );
    assert_eq!(trigger.aria_haspopup, "dialog");
    assert_eq!(trigger.aria_expanded, "true");
    assert_eq!(trigger.tabindex, Some("0"));
    assert_eq!(trigger.data_segment, "trigger");
    assert_eq!(trigger.data_state, "open");
    assert_eq!(trigger.data_disabled, None);

    let content = DateRangePickerContentAttrs::new(
        "drp-content".into(),
        true,
        Some("bottom"),
        Some("start"),
    );
    assert_eq!(content.role, "dialog");
    assert_eq!(content.tabindex, "-1");
    assert_eq!(content.data_state, "open");
    assert_eq!(content.data_date_range_picker_content, "");
}

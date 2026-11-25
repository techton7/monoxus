use crate::foundation::{
    date_time::{CalendarDate, CalendarDateTime, CalendarTime, DateValue},
    shared::ScopeHandle,
};

use super::{
    attrs::{DatePickerContentAttrs, DatePickerTriggerAttrs},
    relationships::{DATE_PICKER_OPEN_FOCUS_SELECTOR, DatePickerRelationships},
    state::{
        DatePickerDefinition, DatePickerState, merge_calendar_date_into_value,
        merge_calendar_placeholder,
    },
    types::{DATE_PICKER_PARTS, DatePickerPart},
};

fn d(y: i32, m: u8, day: u8) -> CalendarDate {
    CalendarDate::new(y, m, day).expect("valid date")
}

#[test]
fn date_picker_parts_and_relationships_are_deterministic() {
    assert_eq!(DATE_PICKER_PARTS.len(), 11);
    assert_eq!(DatePickerPart::Trigger.as_str(), "trigger");
    assert_eq!(DatePickerPart::ContentStatic.as_str(), "content-static");

    let scope = ScopeHandle::root("date-picker").child("birth");
    let rels = DatePickerRelationships::new(scope.clone());
    assert_eq!(rels.root_id(), scope.token());
    assert_eq!(rels.trigger_id(), scope.qualify("trigger"));
    assert_eq!(rels.content_id(), scope.qualify("content"));
    assert_eq!(rels.open_focus_selector(), DATE_PICKER_OPEN_FOCUS_SELECTOR);
    assert_ne!(rels.popover_scope().token(), rels.field_scope().token());
    assert_ne!(rels.field_scope().token(), rels.calendar_scope().token());
}

#[test]
fn date_picker_calendar_select_preserves_time_component() {
    let time = CalendarTime::from_hms(14, 30, 45).unwrap();
    let initial = DateValue::DateTime(CalendarDateTime::new(d(2026, 4, 10), time));
    let picked = d(2026, 5, 22);

    let merged = merge_calendar_date_into_value(Some(initial), initial, picked);
    assert_eq!(merged, DateValue::DateTime(CalendarDateTime::new(picked, time)));

    let ph = merge_calendar_placeholder(initial, d(2026, 6, 1));
    assert_eq!(ph, DateValue::DateTime(CalendarDateTime::new(d(2026, 6, 1), time)));
}

#[test]
fn date_picker_close_on_date_select_respects_policy_and_disabled() {
    let ph = DateValue::Date(d(2026, 4, 15));
    let mut state = DatePickerState::new(DatePickerDefinition::new(), None, ph, true);
    assert!(state.is_open());

    let outcome = state.commit_calendar_date_select(d(2026, 4, 20));
    assert_eq!(outcome.next_value, Some(DateValue::Date(d(2026, 4, 20))));
    assert_eq!(outcome.next_placeholder, DateValue::Date(d(2026, 4, 20)));
    assert!(outcome.should_close);
    assert!(!state.is_open());

    let mut stay_open = DatePickerState::new(
        DatePickerDefinition::new().with_close_on_date_select(false),
        None,
        ph,
        true,
    );
    let outcome = stay_open.commit_calendar_date_select(d(2026, 4, 21));
    assert!(!outcome.should_close);
    assert!(stay_open.is_open());

    let mut disabled = DatePickerState::new(
        DatePickerDefinition::new().with_disabled(true),
        None,
        ph,
        true,
    );
    assert!(!disabled.is_open());
    assert!(!disabled.set_open(true));
}

#[test]
fn date_picker_attrs_follow_boolean_omission_invariant() {
    let trigger_closed = DatePickerTriggerAttrs::new(
        "dp-trigger".into(),
        "dp-content".into(),
        false,
        false,
    );
    assert_eq!(trigger_closed.aria_haspopup, "dialog");
    assert_eq!(trigger_closed.aria_expanded, "false");
    assert_eq!(trigger_closed.tabindex, Some("0"));
    assert_eq!(trigger_closed.data_segment, "trigger");
    assert_eq!(trigger_closed.data_state, "closed");
    assert_eq!(trigger_closed.data_disabled, None);

    let trigger_disabled = DatePickerTriggerAttrs::new(
        "dp-trigger".into(),
        "dp-content".into(),
        false,
        true,
    );
    assert_eq!(trigger_disabled.tabindex, None);
    assert_eq!(trigger_disabled.data_disabled, Some(""));

    let content = DatePickerContentAttrs::new(
        "dp-content".into(),
        true,
        Some("bottom"),
        Some("start"),
    );
    assert_eq!(content.role, "dialog");
    assert_eq!(content.tabindex, "-1");
    assert_eq!(content.data_state, "open");
    assert_eq!(content.data_date_picker_content, "");
}

use crate::foundation::{
    compose::Slottable,
    date_time::{
        CalendarLocale, CalendarTime, DayPeriod, FieldInvalidReason, FieldSegmentIntent, HourCycle,
        SegmentKind, TimeGranularity, TimeValue,
    },
    shared::{Direction, ScopeHandle},
};

use super::{
    attrs::{
        TimeFieldInputAttrs, TimeFieldLabelAttrs, TimeFieldSegmentAttrs,
        project_time_field_input_as_child, project_time_field_label_as_child,
        project_time_field_segment_as_child,
    },
    relationships::TimeFieldRelationships,
    state::{TimeFieldDefinition, TimeFieldState, first_interactive_time_segment},
    types::{TIME_FIELD_PARTS, TimeFieldPart, TimeFieldRangeRole},
};

fn t(h: u8, m: u8, s: u8) -> TimeValue {
    TimeValue::Time(CalendarTime::from_hms(h, m, s).unwrap())
}

#[test]
fn time_field_parts_and_segment_clear_to_none() {
    assert_eq!(
        TIME_FIELD_PARTS
            .iter()
            .map(TimeFieldPart::as_str)
            .collect::<Vec<_>>(),
        ["input", "label", "segment"]
    );

    let def = TimeFieldDefinition::new()
        .with_granularity(TimeGranularity::Minute)
        .with_hour_cycle(Some(HourCycle::H24));
    let mut state = TimeFieldState::new(def, Some(t(14, 30, 0)), t(0, 0, 0));
    assert_eq!(state.iso_value(), "14:30:00");

    // Clearing minute sets committed value to None (5C D-4 fix)
    let res = state
        .apply_intent(SegmentKind::Minute, FieldSegmentIntent::Delete)
        .unwrap();
    assert!(res.cleared_value);
    assert_eq!(state.value(), None);
    assert_eq!(state.iso_value(), "");
}

#[test]
fn time_field_validation_and_day_period_toggle() {
    let def = TimeFieldDefinition::new()
        .with_locale(CalendarLocale::EnUs)
        .with_hour_cycle(Some(HourCycle::H12))
        .with_min_value(Some(t(9, 0, 0)))
        .with_max_value(Some(t(17, 0, 0)));
    let mut state = TimeFieldState::new(def, Some(t(8, 30, 0)), t(9, 0, 0));

    assert_eq!(state.evaluate_validation(None), Some(FieldInvalidReason::Min));
    assert!(state.is_invalid());

    // Toggle AM -> PM moves 08:30 AM to 20:30 (Max violation)
    state.apply_intent(
        SegmentKind::DayPeriod,
        FieldSegmentIntent::DayPeriodKey(DayPeriod::Pm),
    );
    assert_eq!(state.value(), Some(t(20, 30, 0)));
    assert_eq!(state.evaluate_validation(None), Some(FieldInvalidReason::Max));

    // Valid value + external_invalid override
    state.set_value(Some(t(12, 0, 0)));
    assert_eq!(state.evaluate_validation(None), None);
    state.set_external_invalid(true);
    assert!(state.is_invalid());
}

#[test]
fn time_field_attrs_emit_data_time_field_segment_and_as_child() {
    let scope = ScopeHandle::root("time-field").child("attrs");
    let rels = TimeFieldRelationships::new(scope);
    let state = TimeFieldState::new(
        TimeFieldDefinition::new().with_hour_cycle(Some(HourCycle::H12)),
        Some(t(14, 15, 0)),
        t(0, 0, 0),
    );
    let snap = state.snapshot(&rels);
    let first = first_interactive_time_segment(&snap.segments).unwrap();

    let template = CalendarLocale::EnUs.field_template();
    let seg_attrs = TimeFieldSegmentAttrs::new(
        first,
        None,
        Some(rels.label_id()),
        Some(rels.description_id()),
        None,
        true,
        &template,
        false,
        false,
        false,
    );
    assert_eq!(seg_attrs.data_time_field_segment, "");
    assert_eq!(seg_attrs.data_segment, "hour");
    assert_eq!(seg_attrs.role, Some("spinbutton"));
    assert_eq!(seg_attrs.aria_valuetext.as_deref(), Some("2 PM"));
    assert_eq!(seg_attrs.data_invalid, None);

    let input_attrs = TimeFieldInputAttrs::new(
        rels.input_id().to_string(),
        TimeFieldRangeRole::Standalone,
        Some(rels.label_id().to_string()),
        Some(rels.description_id().to_string()),
        Direction::Ltr,
        false,
        false,
    );
    assert_eq!(input_attrs.data_time_field_input, "");
    let label_attrs = TimeFieldLabelAttrs::new(rels.label_id().to_string(), false, false);
    assert_eq!(label_attrs.data_time_field_label, "");

    let (_, p_in, _) = project_time_field_input_as_child(&input_attrs, "div", Slottable::new(1));
    let (_, p_lbl, _) = project_time_field_label_as_child(&label_attrs, "div", Slottable::new(2));
    let (_, p_seg, _) = project_time_field_segment_as_child(&seg_attrs, "span", Slottable::new(3));
    assert_eq!(p_in, input_attrs);
    assert_eq!(p_lbl, label_attrs);
    assert_eq!(p_seg, seg_attrs);
}

#[test]
fn time_field_reset_value_clears_partial_buffers_and_required_attrs() {
    let scope = ScopeHandle::root("time-field").child("reset-attrs");
    let rels = TimeFieldRelationships::new(scope);
    let mut state = TimeFieldState::new(
        TimeFieldDefinition::new()
            .with_hour_cycle(Some(HourCycle::H12))
            .with_required(true),
        None,
        t(9, 0, 0),
    );

    state.apply_intent(SegmentKind::Hour, FieldSegmentIntent::Digit(5));
    assert_eq!(state.value(), None);
    assert_eq!(state.engine().values().hour.as_deref(), Some("05"));

    // DEF-5C-03: reset_value(None) clears partial buffers even when value == None
    state.reset_value(None);
    assert_eq!(state.engine().values().hour, None);

    // DEF-5C-15: required emits aria-required="true" & data-required=""
    let req_attrs = TimeFieldInputAttrs::new(
        rels.input_id().to_string(),
        TimeFieldRangeRole::Standalone,
        Some(rels.label_id().to_string()),
        None,
        Direction::Ltr,
        false,
        false,
    )
    .with_required(true);
    assert_eq!(req_attrs.aria_required, Some("true"));
    assert_eq!(req_attrs.data_required, Some(""));
}

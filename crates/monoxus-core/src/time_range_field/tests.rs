use crate::foundation::{
    compose::{Slottable, project_attrs_as_child},
    date_time::{CalendarTime, FieldInvalidReason, TimeRange, TimeValue},
    shared::{Direction, ScopeHandle},
};

use super::{
    attrs::{TimeRangeFieldLabelAttrs, TimeRangeFieldRootAttrs},
    relationships::TimeRangeFieldRelationships,
    state::{TimeRangeFieldDefinition, TimeRangeFieldState},
    types::{RangeFieldInputKind, TIME_RANGE_FIELD_PARTS, TimeRangeFieldPart},
};

fn t(h: u8, m: u8, s: u8) -> TimeValue {
    TimeValue::Time(CalendarTime::from_hms(h, m, s).unwrap())
}

#[test]
fn time_range_field_parts_and_relationships() {
    assert_eq!(
        TIME_RANGE_FIELD_PARTS
            .iter()
            .map(TimeRangeFieldPart::as_str)
            .collect::<Vec<_>>(),
        ["root", "input", "label", "segment"]
    );
    let scope = ScopeHandle::root("time-range-field").child("test");
    let rels = TimeRangeFieldRelationships::new(scope.clone());
    assert_eq!(rels.root_id(), scope.token());
    assert_eq!(rels.label_id(), scope.qualify("label"));
    assert_ne!(
        rels.start_relationships().input_id(),
        rels.end_relationships().input_id()
    );
}

#[test]
fn time_range_field_validation_precedence_and_overnight_ranges() {
    // Overnight range (20:00..05:00) is valid by default without min/max or custom validate
    let mut overnight = TimeRangeFieldState::new(
        TimeRangeFieldDefinition::new(),
        TimeRange::new(Some(t(20, 0, 0)), Some(t(5, 0, 0))),
        t(20, 0, 0),
    );
    assert_eq!(overnight.evaluate_validation(None), None);
    assert!(!overnight.is_invalid());
    assert!(!overnight.is_part_invalid(RangeFieldInputKind::Start));
    assert!(!overnight.is_part_invalid(RangeFieldInputKind::End));

    // Same-day ordering can be enforced via consumer `validate` callback -> FieldInvalidReason::Custom
    let custom_err = overnight.evaluate_validation(
        overnight
            .value()
            .is_inverted()
            .then(|| vec!["End time must be after start time".to_string()]),
    );
    assert_eq!(
        custom_err,
        Some(FieldInvalidReason::Custom(vec![
            "End time must be after start time".to_string()
        ]))
    );
    assert!(overnight.is_invalid());
    assert!(overnight.is_part_invalid(RangeFieldInputKind::Start));
    assert!(overnight.is_part_invalid(RangeFieldInputKind::End));

    // Bounded TimeRangeField: in-bounds end < start is still valid unless custom validate rejects it
    let def = TimeRangeFieldDefinition::new()
        .with_min_value(Some(t(8, 0, 0)))
        .with_max_value(Some(t(18, 0, 0)));
    let mut state = TimeRangeFieldState::new(
        def,
        TimeRange::new(Some(t(14, 0, 0)), Some(t(10, 0, 0))),
        t(9, 0, 0),
    );
    assert_eq!(state.evaluate_validation(None), None);
    assert!(!state.is_invalid());

    state.set_part_value(RangeFieldInputKind::Start, Some(t(7, 30, 0)));
    assert_eq!(
        state.evaluate_validation(None),
        Some(FieldInvalidReason::Min)
    );
    assert!(state.is_invalid());
    assert!(state.is_part_invalid(RangeFieldInputKind::Start));
    assert!(!state.is_part_invalid(RangeFieldInputKind::End));

    state.set_part_value(RangeFieldInputKind::Start, Some(t(9, 0, 0)));
    state.set_part_value(RangeFieldInputKind::End, Some(t(19, 0, 0)));
    assert_eq!(
        state.evaluate_validation(None),
        Some(FieldInvalidReason::Max)
    );
    assert!(!state.is_part_invalid(RangeFieldInputKind::Start));
    assert!(state.is_part_invalid(RangeFieldInputKind::End));
}

#[test]
fn time_range_field_attrs_and_as_child() {
    let scope = ScopeHandle::root("time-range-field").child("attrs");
    let rels = TimeRangeFieldRelationships::new(scope);

    let root_attrs = TimeRangeFieldRootAttrs::new(
        rels.root_id().to_string(),
        rels.label_id().to_string(),
        Direction::Ltr,
        false,
        true,
    );
    assert_eq!(root_attrs.role, "group");
    assert_eq!(root_attrs.data_marker, "");
    assert_eq!(root_attrs.data_invalid, None);
    assert_eq!(root_attrs.data_disabled, Some(""));

    let label_attrs = TimeRangeFieldLabelAttrs::new(rels.label_id().to_string(), false, true);
    assert_eq!(label_attrs.data_marker, "");
    assert_eq!(label_attrs.data_disabled, Some(""));

    let (_, proj_root, _) = project_attrs_as_child(&root_attrs, "div", Slottable::new("r"));
    let (_, proj_label, _) = project_attrs_as_child(&label_attrs, "span", Slottable::new("l"));
    assert_eq!(proj_root, root_attrs);
    assert_eq!(proj_label, label_attrs);
}

#[test]
fn time_range_field_partial_min_max_scoping() {
    let def = TimeRangeFieldDefinition::new()
        .with_min_value(Some(t(8, 0, 0)))
        .with_max_value(Some(t(18, 0, 0)));

    // Only Start violates Min while End is empty -> only Start is invalid (5C D-8 / DEF-5C-13)
    let mut state = TimeRangeFieldState::new(
        def,
        TimeRange::new(Some(t(7, 0, 0)), None),
        t(8, 0, 0),
    );
    assert!(state.is_invalid());
    assert!(state.is_part_invalid(RangeFieldInputKind::Start));
    assert!(!state.is_part_invalid(RangeFieldInputKind::End));

    // Start in-bounds, End violates Max -> only End is invalid
    state.set_value(TimeRange::new(Some(t(9, 0, 0)), Some(t(19, 0, 0))));
    state.evaluate_validation(None);
    assert!(state.is_invalid());
    assert!(!state.is_part_invalid(RangeFieldInputKind::Start));
    assert!(state.is_part_invalid(RangeFieldInputKind::End));

    // In-bounds overnight/inverted range is valid by default
    state.set_value(TimeRange::new(Some(t(15, 0, 0)), Some(t(10, 0, 0))));
    state.evaluate_validation(None);
    assert_eq!(state.invalid_reason(), None);
    assert!(!state.is_part_invalid(RangeFieldInputKind::Start));
    assert!(!state.is_part_invalid(RangeFieldInputKind::End));

    // Custom validation (e.g. opt-in same-day order check) -> both Start and End are invalid
    state.evaluate_validation(Some(vec!["End must be after start".to_string()]));
    assert_eq!(
        state.invalid_reason(),
        Some(&FieldInvalidReason::Custom(vec![
            "End must be after start".to_string()
        ]))
    );
    assert!(state.is_part_invalid(RangeFieldInputKind::Start));
    assert!(state.is_part_invalid(RangeFieldInputKind::End));
}

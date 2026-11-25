use crate::foundation::{
    compose::Slottable,
    date_time::{CalendarTime, FieldInvalidReason, TimeRange, TimeValue},
    shared::{Direction, ScopeHandle},
};

use super::{
    attrs::{
        TimeRangeFieldLabelAttrs, TimeRangeFieldRootAttrs, project_time_range_field_label_as_child,
        project_time_range_field_root_as_child,
    },
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
fn time_range_field_validation_precedence_with_range_order() {
    let def = TimeRangeFieldDefinition::new()
        .with_min_value(Some(t(8, 0, 0)))
        .with_max_value(Some(t(18, 0, 0)));
    let mut state = TimeRangeFieldState::new(
        def,
        TimeRange::new(Some(t(14, 0, 0)), Some(t(10, 0, 0))),
        t(9, 0, 0),
    );

    assert_eq!(
        state.evaluate_validation(None),
        Some(FieldInvalidReason::RangeOrder)
    );
    assert!(state.is_invalid());

    state.set_part_value(RangeFieldInputKind::End, Some(t(16, 0, 0)));
    assert_eq!(state.evaluate_validation(None), None);
    assert!(!state.is_invalid());

    state.set_part_value(RangeFieldInputKind::Start, Some(t(7, 30, 0)));
    assert_eq!(
        state.evaluate_validation(None),
        Some(FieldInvalidReason::Min)
    );
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
    assert_eq!(root_attrs.data_time_range_field_root, "");
    assert_eq!(root_attrs.data_invalid, None);
    assert_eq!(root_attrs.data_disabled, Some(""));

    let label_attrs = TimeRangeFieldLabelAttrs::new(rels.label_id().to_string(), false, true);
    assert_eq!(label_attrs.data_time_range_field_label, "");
    assert_eq!(label_attrs.data_disabled, Some(""));

    let (_, proj_root, _) =
        project_time_range_field_root_as_child(&root_attrs, "div", Slottable::new("r"));
    let (_, proj_label, _) =
        project_time_range_field_label_as_child(&label_attrs, "span", Slottable::new("l"));
    assert_eq!(proj_root, root_attrs);
    assert_eq!(proj_label, label_attrs);
}

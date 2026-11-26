use crate::foundation::{
    compose::{Slottable, project_attrs_as_child},
    date_time::{CalendarDate, DateValue, DateValueRange, FieldInvalidReason},
    shared::{Direction, ScopeHandle},
};

use super::{
    attrs::{DateRangeFieldLabelAttrs, DateRangeFieldRootAttrs},
    relationships::DateRangeFieldRelationships,
    state::{DateRangeFieldDefinition, DateRangeFieldState},
    types::{DATE_RANGE_FIELD_PARTS, DateRangeFieldPart, RangeFieldInputKind},
};

fn d(y: i32, m: u8, day: u8) -> DateValue {
    DateValue::Date(CalendarDate::new(y, m, day).unwrap())
}

#[test]
fn date_range_field_parts_and_relationships() {
    assert_eq!(
        DATE_RANGE_FIELD_PARTS
            .iter()
            .map(DateRangeFieldPart::as_str)
            .collect::<Vec<_>>(),
        ["root", "input", "label", "segment"]
    );
    assert_eq!(RangeFieldInputKind::Start.as_str(), "start");
    assert_eq!(RangeFieldInputKind::End.as_str(), "end");

    let scope = ScopeHandle::root("date-range-field").child("test");
    let rels = DateRangeFieldRelationships::new(scope.clone());
    assert_eq!(rels.root_id(), scope.token());
    assert_eq!(rels.label_id(), scope.qualify("label"));
    assert_ne!(
        rels.start_relationships().input_id(),
        rels.end_relationships().input_id()
    );
}

#[test]
fn date_range_field_validation_precedence_with_range_order() {
    let def = DateRangeFieldDefinition::new()
        .with_min_value(Some(d(2025, 1, 5)))
        .with_max_value(Some(d(2025, 1, 20)));
    let mut state = DateRangeFieldState::new(
        def,
        DateValueRange::new(Some(d(2025, 1, 15)), Some(d(2025, 1, 10))),
        d(2025, 1, 1),
    );

    // Built-in end < start -> RangeOrder (5C D-2)
    assert_eq!(
        state.evaluate_validation(None),
        Some(FieldInvalidReason::RangeOrder)
    );
    assert!(state.is_invalid());

    // Custom beats RangeOrder
    assert_eq!(
        state.evaluate_validation(Some(vec!["Blocked".into()])),
        Some(FieldInvalidReason::Custom(vec!["Blocked".into()]))
    );

    // Fix order -> valid
    state.set_part_value(RangeFieldInputKind::End, Some(d(2025, 1, 18)));
    assert_eq!(state.evaluate_validation(None), None);
    assert!(!state.is_invalid());

    // Max violation
    state.set_part_value(RangeFieldInputKind::End, Some(d(2025, 1, 25)));
    assert_eq!(
        state.evaluate_validation(None),
        Some(FieldInvalidReason::Max)
    );
    assert!(state.is_invalid());
}

#[test]
fn date_range_field_attrs_and_as_child() {
    let scope = ScopeHandle::root("date-range-field").child("attrs");
    let rels = DateRangeFieldRelationships::new(scope);

    let root_attrs = DateRangeFieldRootAttrs::new(
        rels.root_id().to_string(),
        rels.label_id().to_string(),
        Direction::Ltr,
        true,
        false,
    );
    assert_eq!(root_attrs.role, "group");
    assert_eq!(root_attrs.data_marker, "");
    assert_eq!(root_attrs.data_invalid, Some(""));
    assert_eq!(root_attrs.data_disabled, None);

    let label_attrs = DateRangeFieldLabelAttrs::new(rels.label_id().to_string(), true, false);
    assert_eq!(label_attrs.data_marker, "");
    assert_eq!(label_attrs.data_invalid, Some(""));

    let (_, proj_root, _) = project_attrs_as_child(&root_attrs, "div", Slottable::new("r"));
    let (_, proj_label, _) = project_attrs_as_child(&label_attrs, "span", Slottable::new("l"));
    assert_eq!(proj_root, root_attrs);
    assert_eq!(proj_label, label_attrs);
}

#[test]
fn date_range_field_partial_min_max_scoping() {
    let def = DateRangeFieldDefinition::new()
        .with_min_value(Some(d(2025, 1, 10)))
        .with_max_value(Some(d(2025, 1, 20)));

    // Only Start violates Min while End is empty -> only Start is invalid (5C D-8 / DEF-5C-13)
    let mut state = DateRangeFieldState::new(
        def,
        DateValueRange::new(Some(d(2025, 1, 5)), None),
        d(2025, 1, 10),
    );
    assert!(state.is_invalid());
    assert!(state.is_part_invalid(RangeFieldInputKind::Start));
    assert!(!state.is_part_invalid(RangeFieldInputKind::End));

    // Start in-bounds, End violates Max -> only End is invalid
    state.set_value(DateValueRange::new(Some(d(2025, 1, 12)), Some(d(2025, 1, 25))));
    state.evaluate_validation(None);
    assert!(state.is_invalid());
    assert!(!state.is_part_invalid(RangeFieldInputKind::Start));
    assert!(state.is_part_invalid(RangeFieldInputKind::End));

    // RangeOrder violation -> both Start and End are invalid
    state.set_value(DateValueRange::new(Some(d(2025, 1, 16)), Some(d(2025, 1, 12))));
    state.evaluate_validation(None);
    assert_eq!(state.invalid_reason(), Some(&FieldInvalidReason::RangeOrder));
    assert!(state.is_part_invalid(RangeFieldInputKind::Start));
    assert!(state.is_part_invalid(RangeFieldInputKind::End));
}

#[test]
fn date_range_field_partial_unavailable_scoping() {
    let def = DateRangeFieldDefinition::new();
    let mut state = DateRangeFieldState::new(
        def,
        DateValueRange::new(Some(d(2025, 1, 15)), None),
        d(2025, 1, 1),
    );

    // Start unavailable while End is empty -> root invalid with Unavailable, only Start part invalid
    assert_eq!(
        state.evaluate_validation_with_unavailable(true, false, None),
        Some(FieldInvalidReason::Unavailable)
    );
    assert!(state.is_invalid());
    assert!(state.is_part_invalid(RangeFieldInputKind::Start));
    assert!(!state.is_part_invalid(RangeFieldInputKind::End));

    // Start valid, End unavailable -> only End part invalid
    state.set_value(DateValueRange::new(Some(d(2025, 1, 10)), Some(d(2025, 1, 22))));
    assert_eq!(
        state.evaluate_validation_with_unavailable(false, true, None),
        Some(FieldInvalidReason::Unavailable)
    );
    assert!(state.is_invalid());
    assert!(!state.is_part_invalid(RangeFieldInputKind::Start));
    assert!(state.is_part_invalid(RangeFieldInputKind::End));
}

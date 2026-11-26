use crate::foundation::{
    compose::{Slottable, project_attrs_as_child},
    date_time::{
        CalendarDate, CalendarLocale, DateValue, EditableSegmentKind, FieldInvalidReason,
        FieldSegmentIntent, Granularity, SegmentFocusMove, SegmentKind,
    },
    shared::{Direction, ScopeHandle},
};

use super::{
    attrs::{DateFieldInputAttrs, DateFieldLabelAttrs, DateFieldSegmentAttrs},
    relationships::DateFieldRelationships,
    state::{
        DateFieldDefinition, DateFieldState, first_interactive_segment, last_interactive_segment,
        resolve_adjacent_interactive_index,
    },
    types::{DATE_FIELD_PARTS, DateFieldPart, DateFieldRangeRole},
};

fn d(y: i32, m: u8, day: u8) -> DateValue {
    DateValue::Date(CalendarDate::new(y, m, day).unwrap())
}

#[test]
fn date_field_parts_and_relationships() {
    assert_eq!(
        DATE_FIELD_PARTS
            .iter()
            .map(DateFieldPart::as_str)
            .collect::<Vec<_>>(),
        ["input", "label", "segment"]
    );
    let scope = ScopeHandle::root("date-field").child("test");
    let rels = DateFieldRelationships::new(scope.clone());
    assert_eq!(rels.input_id(), scope.qualify("input"));
    assert_eq!(rels.label_id(), scope.qualify("label"));
    assert_eq!(rels.description_id(), scope.qualify("description"));
    assert_eq!(rels.live_region_id(), scope.qualify("live-region"));
    assert_eq!(
        rels.segment_id(0, SegmentKind::Month),
        scope.qualify("segment-0-month")
    );
}

#[test]
fn date_field_validation_precedence_and_external_invalid_override() {
    let def = DateFieldDefinition::new()
        .with_min_value(Some(d(2025, 1, 10)))
        .with_max_value(Some(d(2025, 1, 20)));
    let mut state = DateFieldState::new(def, Some(d(2025, 1, 5)), d(2025, 1, 15));

    // Min violation
    assert_eq!(state.evaluate_validation(None), Some(FieldInvalidReason::Min));
    assert!(state.is_invalid());

    // Custom beats Min
    assert_eq!(
        state.evaluate_validation(Some(vec!["Custom error".into()])),
        Some(FieldInvalidReason::Custom(vec!["Custom error".into()]))
    );

    // Valid value clears local invalidity, but external_invalid still marks field invalid (5C D-4)
    state.set_value(Some(d(2025, 1, 15)));
    assert_eq!(state.evaluate_validation(None), None);
    assert!(!state.is_invalid());

    state.set_external_invalid(true);
    assert!(state.is_invalid());
}

#[test]
fn date_field_readonly_segment_and_disabled_guards() {
    let def = DateFieldDefinition::new().with_readonly_segments(vec![EditableSegmentKind::Year]);
    let mut state = DateFieldState::new(def, Some(d(2025, 3, 15)), d(2025, 3, 15));

    // Mutating readonly Year is blocked, while navigation is allowed
    assert!(
        state
            .apply_intent(SegmentKind::Year, FieldSegmentIntent::Increment)
            .is_none()
    );
    assert!(
        state
            .apply_intent(SegmentKind::Year, FieldSegmentIntent::NextSegment)
            .is_some()
    );

    // Mutating Month works
    let res = state
        .apply_intent(SegmentKind::Month, FieldSegmentIntent::Increment)
        .unwrap();
    assert_eq!(res.committed_date_value, Some(d(2025, 4, 15)));
}

#[test]
fn date_field_attrs_boolean_omission_and_as_child_projection() {
    let scope = ScopeHandle::root("date-field").child("attrs");
    let rels = DateFieldRelationships::new(scope);
    let state = DateFieldState::new(
        DateFieldDefinition::new()
            .with_locale(CalendarLocale::EnUs)
            .with_granularity(Some(Granularity::Day)),
        None,
        d(2025, 5, 1),
    );
    let snap = state.snapshot(&rels);
    assert_eq!(snap.segments.len(), 5);

    let first = first_interactive_segment(&snap.segments).unwrap();
    let last = last_interactive_segment(&snap.segments).unwrap();
    assert_eq!(first.kind(), SegmentKind::Month);
    assert_eq!(last.kind(), SegmentKind::Year);
    assert_eq!(
        resolve_adjacent_interactive_index(&snap.segments, first.index, SegmentFocusMove::Prev),
        Err(SegmentFocusMove::Prev)
    );
    assert_eq!(
        resolve_adjacent_interactive_index(&snap.segments, first.index, SegmentFocusMove::Next),
        Ok(2)
    );

    let input_attrs = DateFieldInputAttrs::new(
        rels.input_id().to_string(),
        DateFieldRangeRole::Standalone,
        Some(rels.label_id().to_string()),
        None,
        Direction::Ltr,
        false,
        false,
    );
    assert_eq!(input_attrs.role, "group");
    assert_eq!(input_attrs.data_invalid, None);
    assert_eq!(input_attrs.data_disabled, None);
    assert_eq!(input_attrs.data_range_field_part, None);

    let label_attrs = DateFieldLabelAttrs::new(rels.label_id().to_string(), true, false);
    assert_eq!(label_attrs.data_invalid, Some(""));
    assert_eq!(label_attrs.data_disabled, None);

    let template = CalendarLocale::EnUs.field_template();
    let seg_attrs = DateFieldSegmentAttrs::new(
        first,
        None,
        Some(rels.label_id()),
        Some(rels.description_id()),
        Some("err-1"),
        true,
        &template,
        true,
        false,
        false,
    );
    assert_eq!(seg_attrs.role, Some("spinbutton"));
    assert_eq!(seg_attrs.tabindex, Some("0"));
    assert_eq!(seg_attrs.aria_invalid, Some("true"));
    assert_eq!(seg_attrs.data_placeholder, Some(""));
    assert_eq!(seg_attrs.data_invalid, Some(""));
    assert_eq!(seg_attrs.data_disabled, None);
    assert_eq!(seg_attrs.data_readonly, None);
    assert!(
        seg_attrs
            .aria_describedby
            .as_deref()
            .unwrap()
            .contains("err-1")
    );

    assert_eq!(input_attrs.data_marker, "");
    assert_eq!(label_attrs.data_marker, "");
    assert_eq!(seg_attrs.data_marker, "");

    let (_, proj_in, _) = project_attrs_as_child(&input_attrs, "div", Slottable::new(1));
    assert_eq!(proj_in, input_attrs);
    let (_, proj_lbl, _) = project_attrs_as_child(&label_attrs, "div", Slottable::new(2));
    assert_eq!(proj_lbl, label_attrs);
    let (_, proj_seg, _) = project_attrs_as_child(&seg_attrs, "span", Slottable::new(3));
    assert_eq!(proj_seg, seg_attrs);
}

#[test]
fn date_field_reset_value_clears_partial_buffers_and_required_timezone_attrs() {
    let scope = ScopeHandle::root("date-field").child("reset-attrs");
    let rels = DateFieldRelationships::new(scope);
    let mut state = DateFieldState::new(
        DateFieldDefinition::new()
            .with_locale(CalendarLocale::EnUs)
            .with_granularity(Some(Granularity::Minute))
            .with_hide_time_zone(false)
            .with_required(true),
        None,
        d(2026, 5, 1),
    );

    // Type partial month ("03") while committed value remains None
    state.apply_intent(SegmentKind::Month, FieldSegmentIntent::Digit(3));
    assert_eq!(state.value(), None);
    assert_eq!(state.engine().values().month.as_deref(), Some("03"));

    // DEF-5C-03: reset_value(None) unconditionally clears partial buffers
    state.reset_value(None);
    assert_eq!(state.engine().values().month, None);

    // DEF-5C-15: required emits aria-required="true" & data-required=""
    let req_attrs = DateFieldInputAttrs::new(
        rels.input_id().to_string(),
        DateFieldRangeRole::Standalone,
        Some(rels.label_id().to_string()),
        None,
        Direction::Ltr,
        false,
        false,
    )
    .with_required(true);
    assert_eq!(req_attrs.aria_required, Some("true"));
    assert_eq!(req_attrs.data_required, Some(""));

    // DEF-5C-14: TimeZoneName emits aria-invalid and aria-disabled consistently
    let snap = state.snapshot(&rels);
    let tz_item = snap
        .segments
        .iter()
        .find(|s| s.kind() == SegmentKind::TimeZoneName)
        .expect("timeZoneName segment");
    let template = CalendarLocale::EnUs.field_template();
    let tz_attrs = DateFieldSegmentAttrs::new(
        tz_item,
        None,
        Some(rels.label_id()),
        None,
        None,
        false,
        &template,
        true,
        true,
        true,
    );
    assert_eq!(tz_attrs.aria_invalid, Some("true"));
    assert_eq!(tz_attrs.aria_disabled, Some("true"));
    assert_eq!(tz_attrs.data_invalid, Some(""));
    assert_eq!(tz_attrs.data_disabled, Some(""));
}

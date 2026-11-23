use crate::foundation::{
    compose::Slottable,
    date_time::{CalendarDate, CalendarLocale, DateRange},
    shared::ScopeHandle,
};

use super::*;

fn d(y: i32, m: u8, day: u8) -> CalendarDate {
    CalendarDate::new(y, m, day).expect("valid date")
}

#[test]
fn range_calendar_parts_and_relationships_are_deterministic() {
    assert_eq!(RANGE_CALENDAR_PARTS.len(), 14);
    assert_eq!(RangeCalendarPart::Root.as_str(), "root");
    assert_eq!(RangeCalendarPart::Day.as_str(), "day");

    let scope = ScopeHandle::root("range-calendar").child("test");
    let rel = RangeCalendarRelationships::new(scope.clone());
    assert_eq!(rel.root_id(), scope.token());
    assert_eq!(
        rel.day_id(0, d(2024, 1, 5)),
        scope.qualify("day-0-2024-01-05")
    );
}

#[test]
fn forward_backward_selection_constraints_and_escape_work() {
    let locale = CalendarLocale::EnUs;
    let d5 = d(2024, 1, 5);
    let d7 = d(2024, 1, 7);
    let d10 = d(2024, 1, 10);

    // 1. Start selection
    let u1 = apply_range_click(
        &RangeCalendarState::default(),
        d10,
        false,
        false,
        false,
        false,
        None,
        None,
        false,
        &locale,
        |_| false,
        |_| false,
    )
    .unwrap();
    assert_eq!(u1.start_value, Some(d10));
    assert_eq!(u1.end_value, None);
    assert_eq!(u1.range_completed, None);
    assert_eq!(
        u1.announcement.as_deref(),
        Some("Selected Date: January 10, 2024")
    );

    // 2. Escape clears partial start range (5B D-1)
    let partial_state = RangeCalendarState {
        value: u1.value,
        start_value: u1.start_value,
        end_value: u1.end_value,
        last_pressed: u1.last_pressed,
        focused_value: Some(d7),
    };
    let esc = apply_range_escape(&partial_state, false, false, &locale).unwrap();
    assert_eq!(esc.start_value, None);
    assert_eq!(esc.end_value, None);
    assert_eq!(
        esc.announcement.as_deref(),
        Some("Selected date is now empty.")
    );

    // Escape on complete range is a no-op
    let complete_state = RangeCalendarState::from_value(DateRange::new(Some(d5), Some(d10)));
    assert_eq!(apply_range_escape(&complete_state, false, false, &locale), None);

    // 3. Backward selection reorders endpoints
    let u2 = apply_range_click(
        &partial_state,
        d5,
        false,
        false,
        false,
        false,
        None,
        None,
        false,
        &locale,
        |_| false,
        |_| false,
    )
    .unwrap();
    assert_eq!(u2.start_value, Some(d5));
    assert_eq!(u2.end_value, Some(d10));
    assert_eq!(u2.range_completed, Some(DateRange::new(Some(d5), Some(d10))));
    assert_eq!(
        u2.announcement.as_deref(),
        Some("Selected Dates: January 5, 2024 to January 10, 2024")
    );

    // 4. Read-only guard blocks click activation (5B D-3)
    assert_eq!(
        apply_range_click(
            &partial_state,
            d5,
            true,
            false,
            false,
            false,
            None,
            None,
            false,
            &locale,
            |_| false,
            |_| false,
        ),
        None
    );

    // 5. exclude_disabled resets to clicked start when disabled day is inside
    let u3 = apply_range_click(
        &partial_state,
        d5,
        false,
        false,
        false,
        false,
        None,
        None,
        true,
        &locale,
        |day| day == d7,
        |_| false,
    )
    .unwrap();
    assert_eq!(u3.start_value, Some(d5));
    assert_eq!(u3.end_value, None);
    assert_eq!(u3.range_completed, None);

    // 6. min_days / max_days resets when violated
    let u4 = apply_range_click(
        &RangeCalendarState::from_value(DateRange::new(Some(d5), None)),
        d7,
        false,
        false,
        false,
        false,
        Some(5),
        None,
        false,
        &locale,
        |_| false,
        |_| false,
    )
    .unwrap();
    assert_eq!(u4.start_value, Some(d7));
    assert_eq!(u4.end_value, None);
}

#[test]
fn range_cell_and_day_attributes_publish_exact_data_attrs() {
    let cell = RangeCalendarCellAttrs::new(
        String::from("rc-cell"),
        d(2024, 1, 7),
        true,
        false,
        false,
        false,
        false,
        false,
        false,
        false,
        false,
        false,
        false,
        true,
        true,
        true,
    );
    assert_eq!(cell.aria_selected, "true");
    assert_eq!(cell.data_range_middle, Some(""));
    assert_eq!(cell.data_highlighted, Some(""));
    assert_eq!(cell.data_selection_start, None);
    assert_eq!(cell.data_selection_end, None);

    let day = RangeCalendarDayAttrs::from_cell(
        String::from("rc-day"),
        d(2024, 1, 7),
        &CalendarLocale::EnUs,
        &cell,
        false,
    );
    assert_eq!(day.tabindex, Some(-1));
    assert_eq!(day.data_range_calendar_day, "");
    assert_eq!(day.data_bits_day, "");

    let (_, proj, _) = project_range_calendar_day_as_child(&day, "slot", Slottable::new("7"));
    assert_eq!(proj, day);
}

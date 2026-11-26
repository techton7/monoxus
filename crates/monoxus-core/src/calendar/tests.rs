use crate::foundation::{
    compose::{Slottable, project_attrs_as_child},
    date_time::{CalendarDate, CalendarLocale, MonthFormat, YearFormat},
    shared::{Direction, ScopeHandle},
};

use super::*;

fn d(y: i32, m: u8, day: u8) -> CalendarDate {
    CalendarDate::new(y, m, day).expect("valid date")
}

#[test]
fn calendar_parts_and_relationships_are_deterministic() {
    assert_eq!(CALENDAR_PARTS.len(), 14);
    assert_eq!(CalendarPart::Root.as_str(), "root");
    assert_eq!(CalendarPart::MonthSelect.as_str(), "month-select");

    let scope = ScopeHandle::root("calendar").child("test");
    let rel = CalendarRelationships::new(scope.clone());
    assert_eq!(rel.root_id(), scope.token());
    assert_eq!(rel.grid_id(0), scope.qualify("grid-0"));
    assert_eq!(
        rel.day_id(1, d(2024, 2, 5)),
        scope.qualify("day-1-2024-02-05")
    );
}

#[test]
fn single_selection_and_prevent_deselect_transitions() {
    let locale = CalendarLocale::EnUs;
    let jan10 = d(2024, 1, 10);

    // Select
    let u1 = apply_single_click(None, jan10, false, false, false, false, &locale).unwrap();
    assert_eq!(u1.next_value, Some(jan10));
    assert_eq!(u1.date_selected, Some(jan10));
    assert_eq!(u1.announcement, "Selected Date: January 10, 2024");

    // Deselect
    let u2 = apply_single_click(Some(jan10), jan10, false, false, false, false, &locale).unwrap();
    assert_eq!(u2.next_value, None);
    assert_eq!(u2.date_selected, None);
    assert_eq!(u2.next_placeholder, Some(jan10));
    assert_eq!(u2.announcement, "Selected date is now empty.");

    // Prevent deselect keeps value
    let u3 = apply_single_click(Some(jan10), jan10, true, false, false, false, &locale).unwrap();
    assert_eq!(u3.next_value, Some(jan10));
    assert_eq!(u3.date_selected, Some(jan10));

    // Read-only / disabled / unavailable block activation
    assert_eq!(
        apply_single_click(None, jan10, false, true, false, false, &locale),
        None
    );
    assert_eq!(
        apply_single_click(None, jan10, false, false, true, false, &locale),
        None
    );
    assert_eq!(
        apply_single_click(None, jan10, false, false, false, true, &locale),
        None
    );
}

#[test]
fn multiple_selection_respects_max_days_reset_and_prevent_deselect() {
    let d1 = d(2024, 1, 1);
    let d2 = d(2024, 1, 2);
    let d3 = d(2024, 1, 3);

    let s1 = apply_multiple_click(&[], d1, false, Some(2), false, false, false).unwrap();
    assert_eq!(s1.next_values, vec![d1]);

    let s2 =
        apply_multiple_click(&s1.next_values, d2, false, Some(2), false, false, false).unwrap();
    assert_eq!(s2.next_values, vec![d1, d2]);

    // Exceeding max_days resets to [clicked]
    let s3 =
        apply_multiple_click(&s2.next_values, d3, false, Some(2), false, false, false).unwrap();
    assert_eq!(s3.next_values, vec![d3]);

    // prevent_deselect prevents removing selected date
    let s4 =
        apply_multiple_click(&s2.next_values, d1, true, Some(2), false, false, false).unwrap();
    assert_eq!(s4.next_values, vec![d1, d2]);

    // Normal deselect removes and moves placeholder on empty
    let s5 =
        apply_multiple_click(&s1.next_values, d1, false, Some(2), false, false, false).unwrap();
    assert!(s5.next_values.is_empty());
    assert_eq!(s5.next_placeholder, Some(d1));
}

#[test]
fn boolean_data_attrs_are_omitted_when_false_and_as_child_projects() {
    let root = CalendarRootAttrs::new(
        String::from("cal-1"),
        CalendarKind::Calendar,
        String::from("Event January 2024"),
        Direction::Rtl,
        false,
        false,
        true,
    );
    assert_eq!(root.role, "application");
    assert_eq!(root.dir, "rtl");
    assert_eq!(root.data_calendar_root, Some(""));
    assert_eq!(root.data_range_calendar_root, None);
    assert_eq!(root.data_invalid, None);
    assert_eq!(root.data_disabled, None);
    assert_eq!(root.data_readonly, Some(""));

    let cell = CalendarCellAttrs::new(
        String::from("cell-1"),
        d(2024, 1, 15),
        true,
        false,
        false,
        true,
        false,
        false,
        true,
        true,
    );
    assert_eq!(cell.aria_selected, "true");
    assert_eq!(cell.aria_disabled, "false");
    assert_eq!(cell.data_selected, Some(""));
    assert_eq!(cell.data_today, Some(""));
    assert_eq!(cell.data_focused, Some(""));
    assert_eq!(cell.data_disabled, None);
    assert_eq!(cell.data_outside_month, None);

    let day = CalendarDayAttrs::from_cell(
        String::from("day-1"),
        d(2024, 1, 15),
        &CalendarLocale::EnUs,
        &cell,
        true,
    );
    assert_eq!(day.tabindex, Some(0));
    assert_eq!(day.aria_label, "Monday, January 15, 2024");
    assert_eq!(day.data_bits_day, "");

    let disabled_outside_cell = CalendarCellAttrs::new(
        String::from("cell-out"),
        d(2023, 12, 31),
        false,
        false,
        false,
        false,
        true,
        true,
        false,
        true,
    );
    assert_eq!(disabled_outside_cell.data_disabled, Some(""));
    let disabled_day = CalendarDayAttrs::from_cell(
        String::from("day-out"),
        d(2023, 12, 31),
        &CalendarLocale::EnUs,
        &disabled_outside_cell,
        false,
    );
    assert_eq!(disabled_day.tabindex, None);

    let (_, proj, _) = project_attrs_as_child(&day, "slot", Slottable::new("15"));
    assert_eq!(proj, day);
}

#[test]
fn calendar_select_option_builders_and_resolution() {
    let en = CalendarLocale::EnUs;
    let ko = CalendarLocale::KoKr;

    let default_months = build_calendar_month_options(None, &en, MonthFormat::Long);
    assert_eq!(default_months.len(), 12);
    assert_eq!(default_months[0].value, 1);
    assert_eq!(default_months[0].value_string(), "1");
    assert_eq!(default_months[0].label, "January");
    assert_eq!(default_months[11].value, 12);
    assert_eq!(default_months[11].value_string(), "12");
    assert_eq!(default_months[11].label, "December");

    let subset_months = build_calendar_month_options(Some(&[2, 7]), &ko, MonthFormat::Short);
    assert_eq!(
        subset_months,
        vec![
            CalendarSelectOption {
                value: 2,
                label: String::from("2월"),
            },
            CalendarSelectOption {
                value: 7,
                label: String::from("7월"),
            },
        ]
    );

    let resolved = resolve_selected_calendar_option(&subset_months, 7);
    assert_eq!(
        resolved,
        Some(CalendarSelectOption {
            value: 7,
            label: String::from("7월"),
        })
    );
    assert_eq!(resolve_selected_calendar_option(&subset_months, 3), None);

    let years_numeric = build_calendar_year_options(&[2024, 2025], &en, YearFormat::Numeric);
    assert_eq!(
        years_numeric,
        vec![
            CalendarSelectOption {
                value: 2024,
                label: String::from("2024"),
            },
            CalendarSelectOption {
                value: 2025,
                label: String::from("2025"),
            },
        ]
    );
    assert_eq!(years_numeric[0].value_string(), "2024");

    let years_two_digit_ko = build_calendar_year_options(&[2024], &ko, YearFormat::TwoDigit);
    assert_eq!(years_two_digit_ko.len(), 1);
    assert_eq!(years_two_digit_ko[0].value, 2024);
    assert_eq!(
        years_two_digit_ko[0].label,
        ko.year_label(2024, YearFormat::TwoDigit)
    );
}

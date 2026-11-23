use std::borrow::Cow;

use crate::foundation::shared::Direction;

use super::*;

fn date(y: i32, m: u8, d: u8) -> CalendarDate {
    CalendarDate::new(y, m, d).expect("valid test date")
}

#[test]
fn proleptic_gregorian_leap_years_and_epoch_roundtrip() {
    assert!(!is_leap_year(1900));
    assert!(is_leap_year(2000));
    assert!(is_leap_year(2024));
    assert!(!is_leap_year(2023));

    assert_eq!(days_in_month(1900, 2), 28);
    assert_eq!(days_in_month(2000, 2), 29);
    assert_eq!(days_in_month(2024, 2), 29);
    assert_eq!(days_in_month(2023, 2), 28);
    assert_eq!(days_in_month(2024, 4), 30);
    assert_eq!(days_in_month(2024, 13), 0);

    assert_eq!(CalendarDate::new(2023, 2, 29), None);
    assert_eq!(CalendarDate::new(2024, 2, 29), Some(date(2024, 2, 29)));

    let epoch = date(1970, 1, 1);
    assert_eq!(epoch.days_since_epoch(), 0);
    assert_eq!(epoch.weekday(), Weekday::Thursday);
    assert_eq!(CalendarDate::from_days_since_epoch(0), epoch);

    for sample in [
        date(1899, 12, 31),
        date(1900, 3, 1),
        date(1969, 12, 31),
        date(1980, 1, 20),
        date(2000, 2, 29),
        date(2024, 2, 29),
        date(2100, 3, 1),
    ] {
        let days = sample.days_since_epoch();
        assert_eq!(CalendarDate::from_days_since_epoch(days), sample);
    }
}

#[test]
fn date_arithmetic_clamping_and_iso_parsing() {
    assert_eq!(date(2024, 1, 31).add_months_clamped(1), date(2024, 2, 29));
    assert_eq!(date(2023, 1, 31).add_months_clamped(1), date(2023, 2, 28));
    assert_eq!(date(2024, 2, 29).add_years_clamped(1), date(2025, 2, 28));
    assert_eq!(date(2024, 1, 1).add_months_clamped(-1), date(2023, 12, 1));
    assert_eq!(date(2023, 12, 15).add_months_clamped(1), date(2024, 1, 15));

    assert_eq!(
        CalendarDate::from_iso_str("2024-02-29"),
        Some(date(2024, 2, 29))
    );
    assert_eq!(CalendarDate::from_iso_str("2023-02-29"), None);
    assert_eq!(date(2024, 2, 5).to_iso_string(), "2024-02-05");

    assert_eq!(
        default_placeholder_date(
            None,
            Some(date(2025, 3, 1)),
            Some(date(2025, 6, 1)),
            date(2024, 1, 15),
        ),
        date(2025, 3, 1)
    );
    assert_eq!(
        default_placeholder_date(
            Some(date(2025, 4, 10)),
            Some(date(2025, 3, 1)),
            Some(date(2025, 6, 1)),
            date(2024, 1, 15),
        ),
        date(2025, 4, 10)
    );
}

#[test]
fn calendar_time_and_datetime_preserve_time_across_date_changes() {
    let time = CalendarTime::new(14, 30, 5, 250).expect("valid time");
    assert_eq!(CalendarTime::new(24, 0, 0, 0), None);
    let dt = CalendarDateTime::new(date(2024, 1, 10), time);
    assert_eq!(dt.to_iso_string(), "2024-01-10T14:30:05.250");
    let shifted = dt.with_date(date(2024, 2, 20));
    assert_eq!(shifted.date, date(2024, 2, 20));
    assert_eq!(shifted.time, time);
}

#[test]
fn locale_formatting_supports_en_us_ko_kr_and_custom_labels() {
    let d = date(1980, 1, 20);
    let en = CalendarLocale::EnUs;
    assert_eq!(en.cell_aria_label(d), "Sunday, January 20, 1980");
    assert_eq!(
        en.heading_label(&[date(1980, 1, 1)], MonthFormat::Long, YearFormat::Numeric),
        "January 1980"
    );
    assert_eq!(
        en.heading_label(
            &[date(1980, 1, 1), date(1980, 2, 1)],
            MonthFormat::Long,
            YearFormat::Numeric
        ),
        "January - February 1980"
    );
    assert_eq!(
        en.heading_label(
            &[date(1979, 12, 1), date(1980, 1, 1)],
            MonthFormat::Long,
            YearFormat::Numeric
        ),
        "December 1979 - January 1980"
    );
    assert_eq!(en.selected_announcement(d), "Selected Date: January 20, 1980");

    let ko = CalendarLocale::KoKr;
    assert_eq!(ko.cell_aria_label(d), "1980년 1월 20일 일요일");
    assert_eq!(
        ko.heading_label(&[date(1980, 1, 1)], MonthFormat::Long, YearFormat::Numeric),
        "1980년 1월"
    );
    assert_eq!(
        ko.heading_label(
            &[date(1980, 1, 1), date(1980, 2, 1)],
            MonthFormat::Long,
            YearFormat::Numeric
        ),
        "1980년 1월 - 2월"
    );

    let mut custom_labels = CalendarLabels::en_us();
    custom_labels.week_starts_on = Weekday::Monday;
    custom_labels.empty_announcement = Cow::Borrowed("Cleared");
    let custom = CalendarLocale::Custom(custom_labels);
    assert_eq!(custom.default_week_starts_on(), Weekday::Monday);
    assert_eq!(custom.empty_announcement(), "Cleared");
}

#[test]
fn month_grid_respects_week_starts_on_and_fixed_weeks() {
    for start in [
        Weekday::Sunday,
        Weekday::Monday,
        Weekday::Tuesday,
        Weekday::Wednesday,
        Weekday::Thursday,
        Weekday::Friday,
        Weekday::Saturday,
    ] {
        let month = build_month_grid(date(2024, 1, 1), start, false);
        assert_eq!(month.weeks.first().unwrap()[0].weekday(), start);
        assert!(month.weeks.iter().all(|w| w.len() == 7));
        assert!(month.dates.contains(&date(2024, 1, 1)));
        assert!(month.dates.contains(&date(2024, 1, 31)));

        let fixed = build_month_grid(date(2024, 1, 1), start, true);
        assert_eq!(fixed.dates.len(), 42);
        assert_eq!(fixed.weeks.len(), 6);
    }

    // Feb 2026 starts on Sunday Feb 1 and ends on Saturday Feb 28 (exact 28 days = 4 weeks).
    let feb_2026 = build_month_grid(date(2026, 2, 1), Weekday::Sunday, false);
    assert_eq!(feb_2026.dates.len(), 28);
    let feb_2026_fixed = build_month_grid(date(2026, 2, 1), Weekday::Sunday, true);
    assert_eq!(feb_2026_fixed.dates.len(), 42);
    assert_eq!(feb_2026_fixed.dates.last(), Some(&date(2026, 3, 14)));
}

#[test]
fn default_year_range_and_page_disablement_match_contract() {
    let default_years = default_year_range(2024, 2024, None, None);
    assert_eq!(default_years.len(), 111);
    assert_eq!(default_years.first(), Some(&1924));
    assert_eq!(default_years.last(), Some(&2034));

    let early_years = default_year_range(1900, 2024, None, None);
    assert_eq!(early_years.first(), Some(&1890));
    assert_eq!(early_years.last(), Some(&2034));

    let bounded = default_year_range(2024, 2024, Some(date(2020, 3, 1)), Some(date(2026, 9, 1)));
    assert_eq!(bounded, vec![2020, 2021, 2022, 2023, 2024, 2025, 2026]);

    let single_year = default_year_range(2024, 2024, Some(date(2028, 1, 1)), Some(date(2025, 1, 1)));
    assert_eq!(single_year, vec![2025]);

    let jan = build_visible_months(date(2024, 1, 15), 1, Weekday::Sunday, false);
    assert!(is_prev_page_disabled(&jan, Some(date(2024, 1, 5)), false));
    assert!(!is_prev_page_disabled(&jan, Some(date(2023, 12, 31)), false));
    assert!(is_next_page_disabled(&jan, Some(date(2024, 1, 20)), false));
    assert!(!is_next_page_disabled(&jan, Some(date(2024, 2, 1)), false));
}

#[test]
fn grid_navigation_covers_ltr_rtl_vertical_paging_home_end_and_bounds() {
    let jan = build_visible_months(date(2024, 1, 15), 1, Weekday::Sunday, false);

    // LTR Left/Right
    let right = resolve_grid_navigation(
        date(2024, 1, 15),
        &jan,
        1,
        Weekday::Sunday,
        Direction::Ltr,
        CalendarNavKey::Right,
        None,
        None,
        false,
        true,
        |d| d == date(2024, 1, 16),
    )
    .expect("skips disabled Jan 16");
    assert_eq!(right.target_date, date(2024, 1, 17));
    assert!(!right.paged);

    // RTL flips Left/Right
    let rtl_left = resolve_grid_navigation(
        date(2024, 1, 15),
        &jan,
        1,
        Weekday::Sunday,
        Direction::Rtl,
        CalendarNavKey::Left,
        None,
        None,
        false,
        true,
        |_| false,
    )
    .expect("rtl left advances");
    assert_eq!(rtl_left.target_date, date(2024, 1, 16));

    // Down across disabled same weekday & month boundary
    let down = resolve_grid_navigation(
        date(2024, 1, 24),
        &jan,
        1,
        Weekday::Sunday,
        Direction::Ltr,
        CalendarNavKey::Down,
        None,
        None,
        false,
        true,
        |d| d == date(2024, 1, 31),
    )
    .expect("skips disabled Jan 31 and pages to Feb 7");
    assert_eq!(down.target_date, date(2024, 2, 7));
    assert!(down.paged);
    assert_eq!(down.next_view_start, Some(date(2024, 2, 1)));

    // PageUp / PageDown / Shift+PageDown
    let page_down = resolve_grid_navigation(
        date(2024, 1, 31),
        &jan,
        1,
        Weekday::Sunday,
        Direction::Ltr,
        CalendarNavKey::PageDown,
        None,
        None,
        false,
        true,
        |_| false,
    )
    .expect("clamps Jan 31 -> Feb 29");
    assert_eq!(page_down.target_date, date(2024, 2, 29));
    assert!(page_down.paged);

    let shift_page_down = resolve_grid_navigation(
        date(2024, 2, 29),
        &build_visible_months(date(2024, 2, 29), 1, Weekday::Sunday, false),
        1,
        Weekday::Sunday,
        Direction::Ltr,
        CalendarNavKey::ShiftPageDown,
        None,
        None,
        false,
        true,
        |_| false,
    )
    .expect("clamps 2024-02-29 -> 2025-02-28");
    assert_eq!(shift_page_down.target_date, date(2025, 2, 28));

    // Home / End inside first week of Jan 2024 (with disable_days_outside_month = true)
    let home = resolve_grid_navigation(
        date(2024, 1, 3),
        &jan,
        1,
        Weekday::Sunday,
        Direction::Ltr,
        CalendarNavKey::Home,
        None,
        None,
        false,
        true,
        |_| false,
    )
    .expect("Home skips Dec 31 outside month -> Jan 1");
    assert_eq!(home.target_date, date(2024, 1, 1));
    let end = resolve_grid_navigation(
        date(2024, 1, 3),
        &jan,
        1,
        Weekday::Sunday,
        Direction::Ltr,
        CalendarNavKey::End,
        None,
        None,
        false,
        true,
        |_| false,
    )
    .expect("End lands on Saturday Jan 6");
    assert_eq!(end.target_date, date(2024, 1, 6));
}

#[test]
fn range_validation_and_highlighted_preview() {
    assert!(are_all_days_between_valid(
        date(2024, 1, 5),
        date(2024, 1, 6),
        |_| true,
        |_| true,
    ));
    assert!(!are_all_days_between_valid(
        date(2024, 1, 5),
        date(2024, 1, 8),
        |_| false,
        |d| d == date(2024, 1, 7),
    ));

    assert!(is_range_valid(
        date(2024, 1, 10),
        date(2024, 1, 6),
        Some(5),
        Some(5),
        true,
        |d| d == date(2024, 1, 12),
    ));
    assert!(!is_range_valid(
        date(2024, 1, 5),
        date(2024, 1, 8),
        Some(3),
        Some(5),
        true,
        |d| d == date(2024, 1, 7),
    ));

    assert_eq!(
        compute_highlighted_range(
            Some(date(2024, 1, 10)),
            None,
            Some(date(2024, 1, 6)),
            |_| false,
            |_| false,
        ),
        Some((date(2024, 1, 6), date(2024, 1, 10)))
    );
    assert_eq!(
        compute_highlighted_range(
            Some(date(2024, 1, 5)),
            None,
            Some(date(2024, 1, 8)),
            |_| false,
            |d| d == date(2024, 1, 7),
        ),
        None
    );
}

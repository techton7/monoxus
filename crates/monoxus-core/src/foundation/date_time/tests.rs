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

fn time(h: u8, m: u8, s: u8) -> CalendarTime {
    CalendarTime::from_hms(h, m, s).expect("valid test time")
}

#[test]
fn calendar_time_and_datetime_parsing_arithmetic_cycling_and_clamping() {
    assert_eq!(CalendarTime::from_iso_str("09:30"), Some(time(9, 30, 0)));
    assert_eq!(
        CalendarTime::from_iso_str("14:05:09"),
        Some(time(14, 5, 9))
    );
    assert_eq!(
        CalendarTime::from_iso_str("14:05:09.25"),
        CalendarTime::new(14, 5, 9, 250)
    );
    assert_eq!(CalendarTime::from_iso_str("24:00:00"), None);
    assert_eq!("23:59:59".parse::<CalendarTime>(), Ok(time(23, 59, 59)));

    assert_eq!(hour_12_display(0), 12);
    assert_eq!(hour_12_display(1), 1);
    assert_eq!(hour_12_display(12), 12);
    assert_eq!(hour_12_display(13), 1);
    assert_eq!(hour_12_display(23), 11);

    assert_eq!(hour_12_to_24(12, DayPeriod::Am), Some(0));
    assert_eq!(hour_12_to_24(1, DayPeriod::Am), Some(1));
    assert_eq!(hour_12_to_24(12, DayPeriod::Pm), Some(12));
    assert_eq!(hour_12_to_24(1, DayPeriod::Pm), Some(13));
    assert_eq!(hour_12_to_24(0, DayPeriod::Am), None);
    assert_eq!(hour_12_to_24(13, DayPeriod::Pm), None);

    // Arithmetic wrap vs clamp
    assert_eq!(time(23, 30, 0).add_hours(2), time(1, 30, 0));
    assert_eq!(time(0, 15, 0).add_minutes(-30), time(23, 45, 0));
    assert_eq!(time(23, 59, 50).add_seconds(20), time(0, 0, 10));
    assert_eq!(
        time(23, 30, 0).add_hours_clamped(2),
        CalendarTime::new(23, 59, 59, 999).unwrap()
    );
    assert_eq!(time(0, 15, 0).add_minutes_clamped(-30), CalendarTime::midnight());

    // Cycling without carry
    assert_eq!(time(11, 59, 59).cycle_hour(1, HourCycle::H12), time(0, 59, 59));
    assert_eq!(time(0, 0, 0).cycle_hour(-1, HourCycle::H12), time(11, 0, 0));
    assert_eq!(time(23, 0, 0).cycle_hour(1, HourCycle::H12), time(12, 0, 0));
    assert_eq!(time(23, 0, 0).cycle_hour(1, HourCycle::H24), time(0, 0, 0));
    assert_eq!(time(10, 59, 0).cycle_minute(1), time(10, 0, 0));
    assert_eq!(time(10, 0, 59).cycle_second(1), time(10, 0, 0));
    assert_eq!(time(9, 15, 0).toggle_day_period(), time(21, 15, 0));

    // Clamp
    assert_eq!(
        time(7, 0, 0).clamp(Some(time(9, 0, 0)), Some(time(17, 0, 0))),
        time(9, 0, 0)
    );
    assert_eq!(
        time(19, 0, 0).clamp(Some(time(9, 0, 0)), Some(time(17, 0, 0))),
        time(17, 0, 0)
    );

    // CalendarDateTime ISO + carry
    let dt = CalendarDateTime::from_iso_str("2024-02-28T23:30:00").expect("valid dt");
    assert_eq!(
        dt.add_hours(2),
        CalendarDateTime::new(date(2024, 2, 29), time(1, 30, 0))
    );
    assert_eq!(
        dt.cycle_hour(2, HourCycle::H24),
        CalendarDateTime::new(date(2024, 2, 28), time(1, 30, 0))
    );
}

#[test]
fn date_value_time_value_and_range_order_validation() {
    let d1 = DateValue::Date(date(2024, 5, 10));
    let dt1 = DateValue::DateTime(CalendarDateTime::new(date(2024, 5, 10), time(9, 0, 0)));
    let d2 = DateValue::Date(date(2024, 5, 11));
    assert!(d1 < dt1);
    assert!(dt1 < d2);
    assert_eq!(DateValue::from_iso_str("2024-05-10"), Some(d1));
    assert_eq!(DateValue::from_iso_str("2024-05-10T09:00:00"), Some(dt1));
    assert_eq!(
        Granularity::infer_from_date_value(None, d1),
        Granularity::Day
    );
    assert_eq!(
        Granularity::infer_from_date_value(None, dt1),
        Granularity::Minute
    );

    // DateRange <-> DateValueRange conversions
    let dr = DateRange::new(Some(date(2024, 5, 10)), Some(date(2024, 5, 15)));
    let dvr: DateValueRange = dr.into();
    assert_eq!(dvr.start, Some(DateValue::Date(date(2024, 5, 10))));
    assert_eq!(DateRange::from(dvr), dr);

    // Built-in end < start RangeOrder validation (5C D-2)
    let inverted_dates = DateValueRange::new(Some(d2), Some(d1));
    assert!(inverted_dates.is_inverted());
    assert_eq!(
        inverted_dates.validate_range_order(),
        Err(FieldInvalidReason::RangeOrder)
    );
    assert_eq!(
        validate_date_value_range(inverted_dates, Some(d1), Some(d2), None),
        Err(FieldInvalidReason::RangeOrder)
    );
    assert_eq!(
        validate_date_value_range(
            inverted_dates,
            Some(d1),
            Some(d2),
            Some(vec!["custom error".to_string()])
        ),
        Err(FieldInvalidReason::Custom(vec!["custom error".to_string()]))
    );

    let t_start = TimeValue::Time(time(14, 0, 0));
    let t_end = TimeValue::Time(time(9, 0, 0));
    let inverted_times = TimeRange::new(Some(t_start), Some(t_end));
    assert!(inverted_times.is_inverted());
    assert_eq!(
        validate_time_range(inverted_times, None, None, None),
        Err(FieldInvalidReason::RangeOrder)
    );
    let valid_times = TimeRange::new(Some(t_end), Some(t_start));
    assert!(!valid_times.is_inverted());
    assert_eq!(
        validate_time_range(
            valid_times,
            Some(TimeValue::Time(time(10, 0, 0))),
            None,
            None
        ),
        Err(FieldInvalidReason::Min)
    );
}

#[test]
fn field_locale_formats_and_granularity_hour_omits_second() {
    let en_us = FieldLocaleFormat::EnUs.template();
    assert_eq!(en_us.default_hour_cycle, HourCycle::H12);
    assert_eq!(en_us.placeholders.year, "yyyy");
    assert_eq!(en_us.aria_labels.month, "month");

    // 5C D-4 fix: Granularity::Hour omits both minute and second
    let hour_tokens = en_us.date_field_tokens(Granularity::Hour, Some(HourCycle::H12), false);
    assert!(!hour_tokens.contains(&FieldTemplateToken::Segment(SegmentKind::Minute)));
    assert!(!hour_tokens.contains(&FieldTemplateToken::Segment(SegmentKind::Second)));
    assert!(hour_tokens.contains(&FieldTemplateToken::Segment(SegmentKind::Hour)));
    assert!(hour_tokens.contains(&FieldTemplateToken::Segment(SegmentKind::DayPeriod)));

    let en_gb = FieldLocaleFormat::EnGb.template();
    assert_eq!(en_gb.default_hour_cycle, HourCycle::H24);
    assert_eq!(
        en_gb.date_tokens[0],
        FieldTemplateToken::Segment(SegmentKind::Day)
    );

    let ko_kr = FieldLocaleFormat::KoKr.template();
    assert_eq!(ko_kr.default_hour_cycle, HourCycle::H12);
    assert_eq!(ko_kr.placeholders.year, "연도");
    assert_eq!(ko_kr.placeholders.month, "월");
    assert_eq!(ko_kr.placeholders.day, "일");
    assert_eq!(ko_kr.am_label, "오전");
    assert_eq!(ko_kr.pm_label, "오후");
    let ko_hour = ko_kr.time_tokens(HourCycle::H12, TimeGranularity::Hour);
    assert_eq!(
        ko_hour,
        vec![
            FieldTemplateToken::Segment(SegmentKind::DayPeriod),
            FieldTemplateToken::Literal(Cow::Borrowed(" ")),
            FieldTemplateToken::Segment(SegmentKind::Hour),
        ]
    );

    let iso = FieldLocaleFormat::Iso.template();
    assert_eq!(iso.default_hour_cycle, HourCycle::H24);
    assert_eq!(
        iso.date_tokens[0],
        FieldTemplateToken::Segment(SegmentKind::Year)
    );
    assert_eq!(
        CalendarLocale::from_tag("en-GB"),
        CalendarLocale::EnGb
    );
    assert_eq!(
        CalendarLocale::from_tag("iso"),
        CalendarLocale::Iso
    );
}

#[test]
fn segment_engine_digit_entry_auto_advance_overflow_and_zero_rules() {
    let mut engine = SegmentEngine::new_date(
        FieldLocaleFormat::EnUs,
        Granularity::Day,
        None,
        true,
        DateValue::Date(date(2026, 6, 15)),
    );

    // Leading 0 waits; 0 then 0 stays "0" on 1-based month; 0 then 5 -> "05" + advance
    let r = engine.apply_intent(SegmentKind::Month, FieldSegmentIntent::Digit(0));
    assert_eq!(engine.values().month.as_deref(), Some("0"));
    assert_eq!(r.move_focus, None);
    let r = engine.apply_intent(SegmentKind::Month, FieldSegmentIntent::Digit(0));
    assert_eq!(engine.values().month.as_deref(), Some("0"));
    assert_eq!(r.move_focus, None);
    let r = engine.apply_intent(SegmentKind::Month, FieldSegmentIntent::Digit(5));
    assert_eq!(engine.values().month.as_deref(), Some("05"));
    assert_eq!(r.move_focus, Some(SegmentFocusMove::Next));

    // Immediate auto-advance when d > max / 10 (day: 4 > 3 -> "04" + advance)
    let r = engine.apply_intent(SegmentKind::Day, FieldSegmentIntent::Digit(4));
    assert_eq!(engine.values().day.as_deref(), Some("04"));
    assert_eq!(r.move_focus, Some(SegmentFocusMove::Next));

    // Overflow overwrite (month: blur -> 1 -> 5 -> "05" + advance)
    engine.apply_intent(SegmentKind::Month, FieldSegmentIntent::Blur);
    let r = engine.apply_intent(SegmentKind::Month, FieldSegmentIntent::Digit(1));
    assert_eq!(engine.values().month.as_deref(), Some("1"));
    assert_eq!(r.move_focus, None);
    let r = engine.apply_intent(SegmentKind::Month, FieldSegmentIntent::Digit(5));
    assert_eq!(engine.values().month.as_deref(), Some("05"));
    assert_eq!(r.move_focus, Some(SegmentFocusMove::Next));

    // 4-digit year entry + commit
    for (digit, expected_move) in [
        (2, None),
        (0, None),
        (2, None),
        (6, Some(SegmentFocusMove::Next)),
    ] {
        let r = engine.apply_intent(SegmentKind::Year, FieldSegmentIntent::Digit(digit));
        assert_eq!(r.move_focus, expected_move);
    }
    assert_eq!(
        engine.to_date_value(),
        Some(DateValue::Date(date(2026, 5, 4)))
    );

    // 00 allowed on 0-based segments (24h hour & minute)
    let mut time_engine = SegmentEngine::new_time(
        FieldLocaleFormat::Iso,
        TimeGranularity::Minute,
        Some(HourCycle::H24),
        TimeValue::Time(CalendarTime::midnight()),
    );
    let r = time_engine.apply_intent(SegmentKind::Hour, FieldSegmentIntent::Digit(0));
    assert_eq!(r.move_focus, None);
    let r = time_engine.apply_intent(SegmentKind::Hour, FieldSegmentIntent::Digit(0));
    assert_eq!(time_engine.values().hour.as_deref(), Some("00"));
    assert_eq!(r.move_focus, Some(SegmentFocusMove::Next));
    let r = time_engine.apply_intent(SegmentKind::Minute, FieldSegmentIntent::Digit(0));
    assert_eq!(r.move_focus, None);
    let r = time_engine.apply_intent(SegmentKind::Minute, FieldSegmentIntent::Digit(0));
    assert_eq!(time_engine.values().minute.as_deref(), Some("00"));
    assert_eq!(r.move_focus, Some(SegmentFocusMove::Next));
    assert_eq!(
        r.committed_time_value,
        Some(TimeValue::Time(CalendarTime::midnight()))
    );
}

#[test]
fn segment_engine_leap_year_clamping_backspace_blur_and_day_period() {
    let mut engine = SegmentEngine::new_date(
        FieldLocaleFormat::Iso,
        Granularity::Day,
        None,
        true,
        DateValue::Date(date(2024, 2, 29)),
    );
    engine.sync_from_date_value(Some(DateValue::Date(date(2024, 2, 29))));

    // Year Backspace -> post-backspace edit mode -> 3 -> 2023 -> day clamps 29 -> 28
    let r = engine.apply_intent(SegmentKind::Year, FieldSegmentIntent::Backspace);
    assert_eq!(engine.values().year.as_deref(), Some("202"));
    assert_eq!(r.move_focus, None);
    let r = engine.apply_intent(SegmentKind::Year, FieldSegmentIntent::Digit(3));
    assert_eq!(engine.values().year.as_deref(), Some("2023"));
    assert_eq!(engine.values().day.as_deref(), Some("28"));
    assert_eq!(r.move_focus, Some(SegmentFocusMove::Next));
    assert_eq!(
        r.committed_date_value,
        Some(DateValue::Date(date(2023, 2, 28)))
    );

    // Delete clears segment and marks cleared_value = true
    let r = engine.apply_intent(SegmentKind::Day, FieldSegmentIntent::Delete);
    assert_eq!(engine.values().day, None);
    assert!(r.cleared_value);
    assert_eq!(r.committed_date_value, None);

    // Backspace on empty segment moves focus Prev
    let r = engine.apply_intent(SegmentKind::Day, FieldSegmentIntent::Backspace);
    assert_eq!(r.move_focus, Some(SegmentFocusMove::Prev));

    // Blur zero-pads single-digit day
    engine.apply_intent(SegmentKind::Day, FieldSegmentIntent::Digit(2));
    assert_eq!(engine.values().day.as_deref(), Some("2"));
    engine.apply_intent(SegmentKind::Day, FieldSegmentIntent::Blur);
    assert_eq!(engine.values().day.as_deref(), Some("02"));

    // 12h TimeField dayPeriod a/p toggle (5C D-4)
    assert_eq!(parse_day_period_key("a"), Some(DayPeriod::Am));
    assert_eq!(parse_day_period_key("A"), Some(DayPeriod::Am));
    assert_eq!(parse_day_period_key("p"), Some(DayPeriod::Pm));
    assert_eq!(parse_day_period_key("P"), Some(DayPeriod::Pm));
    assert_eq!(parse_day_period_key("x"), None);

    let mut t_engine = SegmentEngine::new_time(
        FieldLocaleFormat::EnUs,
        TimeGranularity::Minute,
        Some(HourCycle::H12),
        TimeValue::Time(time(9, 30, 0)),
    );
    t_engine.sync_from_time_value(Some(TimeValue::Time(time(9, 30, 0))));
    let r = t_engine.apply_intent(
        SegmentKind::DayPeriod,
        FieldSegmentIntent::DayPeriodKey(DayPeriod::Pm),
    );
    assert_eq!(
        r.committed_time_value,
        Some(TimeValue::Time(time(21, 30, 0)))
    );
    let r = t_engine.apply_intent(SegmentKind::Hour, FieldSegmentIntent::Increment);
    assert_eq!(
        r.committed_time_value,
        Some(TimeValue::Time(time(22, 30, 0)))
    );
}

#[test]
fn segment_engine_iso_and_localized_paste_parsing() {
    assert_eq!(
        parse_pasted_date_value("2026-08-15", Granularity::Day),
        Some(DateValue::Date(date(2026, 8, 15)))
    );
    assert_eq!(
        parse_pasted_date_value("2026-08-15T14:35:00", Granularity::Minute),
        Some(DateValue::DateTime(CalendarDateTime::new(
            date(2026, 8, 15),
            time(14, 35, 0)
        )))
    );
    assert_eq!(
        parse_pasted_date_value("08/15/2026", Granularity::Day),
        Some(DateValue::Date(date(2026, 8, 15)))
    );
    assert_eq!(
        parse_pasted_date_value("2026. 08. 15.", Granularity::Day),
        Some(DateValue::Date(date(2026, 8, 15)))
    );

    assert_eq!(
        parse_pasted_time_value("14:35:00"),
        Some(TimeValue::Time(time(14, 35, 0)))
    );
    assert_eq!(
        parse_pasted_time_value("2:35 PM"),
        Some(TimeValue::Time(time(14, 35, 0)))
    );
    assert_eq!(
        parse_pasted_time_value("12:00 AM"),
        Some(TimeValue::Time(time(0, 0, 0)))
    );
    assert_eq!(
        parse_pasted_time_value("오후 02:15:30"),
        Some(TimeValue::Time(time(14, 15, 30)))
    );

    let mut engine = SegmentEngine::new_date(
        FieldLocaleFormat::EnUs,
        Granularity::Day,
        None,
        true,
        DateValue::Date(date(2026, 1, 1)),
    );
    let r = engine.apply_intent(
        SegmentKind::Month,
        FieldSegmentIntent::Paste("2026-08-15".to_string()),
    );
    assert_eq!(
        r.committed_date_value,
        Some(DateValue::Date(date(2026, 8, 15)))
    );
}

use dioxus::prelude::*;
use monoxus::{
    foundation::{
        compose::Slottable,
        date_time::{CalendarDate, CalendarLocale, DateRange, MonthFormat, YearFormat},
        shared::ScopeHandle,
    },
    range_calendar::{
        RANGE_CALENDAR_PARTS, RangeCalendar, RangeCalendarCell, RangeCalendarDay,
        RangeCalendarGrid, RangeCalendarGridBody, RangeCalendarGridHead, RangeCalendarGridRow,
        RangeCalendarHeadCell, RangeCalendarHeader, RangeCalendarHeading, RangeCalendarMonthSelect,
        RangeCalendarNextButton, RangeCalendarPart, RangeCalendarPrevButton,
        RangeCalendarRelationships, RangeCalendarRoot, RangeCalendarSnapshot,
        RangeCalendarYearSelect, use_range_calendar_context,
    },
    select::{
        SelectContent, SelectItem, SelectItemData, SelectItemIndicator, SelectItemText,
        SelectPortal, SelectRoot, SelectTrigger, SelectValue, SelectViewport,
    },
};

#[cfg(feature = "native")]
use crate::common;

fn d(y: i32, m: u8, day: u8) -> CalendarDate {
    CalendarDate::new(y, m, day).expect("valid test date")
}

#[test]
fn range_calendar_public_contracts_and_as_child() {
    assert_eq!(
        RANGE_CALENDAR_PARTS
            .iter()
            .map(RangeCalendarPart::as_str)
            .collect::<Vec<_>>(),
        [
            "root",
            "header",
            "heading",
            "prev-button",
            "next-button",
            "grid",
            "grid-head",
            "grid-body",
            "grid-row",
            "head-cell",
            "cell",
            "day",
            "month-select",
            "year-select",
        ]
    );

    let scope = ScopeHandle::root("range-calendar").child("contract");
    let rels = RangeCalendarRelationships::new(scope.clone());
    assert_eq!(rels.root_id(), scope.token());
    assert_eq!(
        rels.day_id(0, d(2024, 1, 10)),
        scope.qualify("day-0-2024-01-10")
    );

    let cell = monoxus::range_calendar::RangeCalendarCellAttrs::new(
        rels.cell_id(0, d(2024, 1, 10)),
        d(2024, 1, 10),
        true,
        false,
        false,
        false,
        false,
        false,
        true,
        true,
        false,
        true,
        true,
        false,
        true,
        true,
    );
    let day = monoxus::range_calendar::RangeCalendarDayAttrs::from_cell(
        rels.day_id(0, d(2024, 1, 10)),
        d(2024, 1, 10),
        &CalendarLocale::EnUs,
        &cell,
        true,
    );
    let (_, proj, _) = monoxus::foundation::compose::project_attrs_as_child(
        &day,
        "div",
        Slottable::new("10"),
    );
    assert_eq!(proj, day);
}

#[component]
fn RangeCalendarContextProbe() -> Element {
    let ctx = use_range_calendar_context();
    let snap = ctx.snapshot();
    rsx! {
        RangeCalendarHeader {
            RangeCalendarPrevButton {}
            RangeCalendarHeading {}
            RangeCalendarNextButton {}
            RangeCalendarMonthSelect {}
            RangeCalendarYearSelect {}
        }
        for (m_idx, month) in snap.months.iter().enumerate() {
            RangeCalendarGrid {
                key: "{m_idx}",
                month_idx: Some(m_idx),
                RangeCalendarGridHead {
                    RangeCalendarGridRow {
                        for (w_idx, wd) in snap.weekdays.iter().enumerate() {
                            RangeCalendarHeadCell {
                                key: "{w_idx}",
                                "{wd}"
                            }
                        }
                    }
                }
                RangeCalendarGridBody {
                    for (r_idx, week) in month.weeks.iter().enumerate() {
                        RangeCalendarGridRow {
                            key: "{r_idx}",
                            for date in week.iter().copied() {
                                RangeCalendarCell {
                                    key: "{date}",
                                    date,
                                    month: month.value,
                                    RangeCalendarDay {}
                                }
                            }
                        }
                    }
                }
            }
        }
    }
}

#[component]
fn RangeCalendarVdomSmokeApp() -> Element {
    rsx! {
        RangeCalendar {
            id: "vdom-rc",
            default_placeholder: d(2024, 1, 1),
            default_value: DateRange::new(Some(d(2024, 1, 5)), Some(d(2024, 1, 10))),
            min_days: Some(2),
            max_days: Some(14),
            exclude_disabled: true,
            RangeCalendarContextProbe {}
        }
    }
}

#[test]
fn range_calendar_vdom_mounts_cleanly() {
    let mut dom = VirtualDom::new(RangeCalendarVdomSmokeApp);
    dom.rebuild_in_place();
}

#[test]
fn range_calendar_select_as_child_vdom_and_context_helpers() {
    #[component]
    fn RangeCalendarCustomSelectVdomProbe() -> Element {
        let ctx = use_range_calendar_context();
        assert_eq!(ctx.current_month(), 5);
        assert_eq!(ctx.current_year(), 2024);
        assert!(!ctx.default_years().is_empty());
        assert_eq!(ctx.month_options(MonthFormat::Long).len(), 12);
        assert_eq!(
            ctx.year_options(YearFormat::Numeric).len(),
            ctx.default_years().len()
        );

        let month_items = ctx.month_options(MonthFormat::Long);
        let month_val = ctx.current_month().to_string();
        let month_select_items: Vec<SelectItemData> = month_items
            .iter()
            .map(|o| SelectItemData::new(o.value_string(), o.label.clone(), false))
            .collect();
        let month_ctx = ctx.clone();

        rsx! {
            RangeCalendarHeader {
                RangeCalendarMonthSelect {
                    id: "vdom-rc-custom-month",
                    child: move |props: Vec<Attribute>| {
                        let is_disabled = props.iter().any(|a| a.name == "disabled");
                        assert!(!is_disabled);
                        let items = month_items.clone();
                        let select_items = month_select_items.clone();
                        let cur_val = month_val.clone();
                        let mut c = month_ctx.clone();
                        rsx! {
                            SelectRoot {
                                id: "vdom-rc-custom-month".to_string(),
                                items: Some(select_items),
                                value: Some(cur_val),
                                disabled: is_disabled,
                                on_value_change: move |next: Option<String>| {
                                    if let Some(s) = next
                                        && let Ok(m) = s.parse::<u8>()
                                    {
                                        c.set_month(m);
                                    }
                                },
                                SelectTrigger { SelectValue {} }
                                SelectPortal {
                                    SelectContent {
                                        SelectViewport {
                                            for opt in items {
                                                SelectItem {
                                                    key: "{opt.value}",
                                                    value: opt.value_string(),
                                                    text: opt.label.clone(),
                                                    SelectItemText { "{opt.label}" }
                                                }
                                            }
                                        }
                                    }
                                }
                            }
                        }
                    },
                }
                RangeCalendarYearSelect {
                    id: "vdom-rc-custom-year",
                    disabled: true,
                    child: move |props: Vec<Attribute>| {
                        let is_disabled = props.iter().any(|a| a.name == "disabled");
                        assert!(is_disabled);
                        rsx! {
                            button {
                                r#type: "button",
                                ..props,
                                "2024"
                            }
                        }
                    },
                }
            }
        }
    }

    #[component]
    fn RangeCalendarCustomSelectVdomApp() -> Element {
        rsx! {
            RangeCalendar {
                id: "vdom-rc-custom",
                default_placeholder: d(2024, 5, 1),
                RangeCalendarCustomSelectVdomProbe {}
            }
        }
    }

    let mut dom = VirtualDom::new(RangeCalendarCustomSelectVdomApp);
    dom.rebuild_in_place();
}

#[cfg(feature = "native")]
mod native_tests {
    use super::*;
    use common::native_fixture::*;
    use keyboard_types::{Code, Key, NamedKey};

    #[component]
    fn NativeRangeCalendarScenario() -> Element {
        let range_commits = use_signal(|| 0usize);
        let mut commits_sig = range_commits;

        rsx! {
            div {
                style: "padding: 16px;",
                RangeCalendarRoot {
                    id: "rc-main",
                    default_placeholder: d(2024, 1, 1),
                    on_range_select: move |_| commits_sig += 1,
                    render_children: move |snap: RangeCalendarSnapshot| rsx! {
                        div {
                            id: "rc-probe",
                            "data-start": snap.start_value.map(|x| x.to_iso_string()).unwrap_or_default(),
                            "data-end": snap.end_value.map(|x| x.to_iso_string()).unwrap_or_default(),
                            "data-commits": "{range_commits}",
                        }
                        RangeCalendarHeader {
                            id: "rc-header",
                            RangeCalendarPrevButton { id: "rc-prev", "Prev" }
                            RangeCalendarHeading { id: "rc-heading" }
                            RangeCalendarNextButton { id: "rc-next", "Next" }
                            RangeCalendarMonthSelect { id: "rc-month-sel" }
                        }
                        RangeCalendarGrid {
                            id: "rc-grid",
                            RangeCalendarGridHead {
                                RangeCalendarGridRow {
                                    for (w_idx, wd) in snap.weekdays.iter().enumerate() {
                                        RangeCalendarHeadCell { key: "{w_idx}", "{wd}" }
                                    }
                                }
                            }
                            RangeCalendarGridBody {
                                for (r_idx, week) in snap.months[0].weeks.iter().enumerate() {
                                    RangeCalendarGridRow {
                                        key: "{r_idx}",
                                        for date in week.iter().copied() {
                                            RangeCalendarCell {
                                                key: "{date}",
                                                id: "rc-cell-{date}",
                                                date,
                                                month: snap.months[0].value,
                                                RangeCalendarDay {
                                                    id: "rc-day-{date}",
                                                }
                                            }
                                        }
                                    }
                                }
                            }
                        }
                    }
                }

                // Constrained + readonly scenario
                RangeCalendarRoot {
                    id: "rc-constrained",
                    default_placeholder: d(2024, 1, 1),
                    min_days: Some(3),
                    max_days: Some(5),
                    exclude_disabled: true,
                    is_date_disabled: Callback::new(|date: CalendarDate| date == d(2024, 1, 15)),
                    render_children: move |snap: RangeCalendarSnapshot| rsx! {
                        div {
                            id: "rc-const-probe",
                            "data-start": snap.start_value.map(|x| x.to_iso_string()).unwrap_or_default(),
                            "data-end": snap.end_value.map(|x| x.to_iso_string()).unwrap_or_default(),
                        }
                        RangeCalendarGrid {
                            RangeCalendarGridBody {
                                for (r_idx, week) in snap.months[0].weeks.iter().enumerate() {
                                    RangeCalendarGridRow {
                                        key: "{r_idx}",
                                        for date in week.iter().copied() {
                                            RangeCalendarCell {
                                                key: "{date}",
                                                date,
                                                month: snap.months[0].value,
                                                RangeCalendarDay {
                                                    id: "rc2-day-{date}",
                                                }
                                            }
                                        }
                                    }
                                }
                            }
                        }
                    }
                }

                RangeCalendarRoot {
                    id: "rc-readonly",
                    default_placeholder: d(2024, 1, 1),
                    default_value: DateRange::new(Some(d(2024, 1, 5)), Some(d(2024, 1, 8))),
                    read_only: true,
                    render_children: move |snap: RangeCalendarSnapshot| rsx! {
                        div {
                            id: "rc-ro-probe",
                            "data-start": snap.start_value.map(|x| x.to_iso_string()).unwrap_or_default(),
                            "data-end": snap.end_value.map(|x| x.to_iso_string()).unwrap_or_default(),
                        }
                        RangeCalendarGrid {
                            RangeCalendarGridBody {
                                for (r_idx, week) in snap.months[0].weeks.iter().enumerate() {
                                    RangeCalendarGridRow {
                                        key: "{r_idx}",
                                        for date in week.iter().copied() {
                                            RangeCalendarCell {
                                                key: "{date}",
                                                date,
                                                month: snap.months[0].value,
                                                RangeCalendarDay {
                                                    id: "rc-ro-day-{date}",
                                                }
                                            }
                                        }
                                    }
                                }
                            }
                        }
                    }
                }
            }
        }
    }

    #[test]
    fn native_range_calendar_selection_highlight_constraints_escape_and_readonly() {
        let _lock = NATIVE_TEST_LOCK.lock().unwrap_or_else(|e| e.into_inner());
        let mut harness = mount_native_scenario(NativeRangeCalendarScenario);

        // Shared parts emit data-range-calendar-* attributes
        assert_eq!(
            harness.attr("#rc-header", "data-range-calendar-header").as_deref(),
            Some("")
        );
        assert_eq!(harness.attr("#rc-header", "data-calendar-header"), None);
        assert_eq!(
            harness.attr("#rc-month-sel", "data-range-calendar-month-select").as_deref(),
            Some("")
        );

        // Click Jan 10 -> start set, focus Jan 6 -> backward highlighted preview (Jan 6..Jan 10)
        click_and_pump(&mut harness, "#rc-day-2024-01-10");
        assert_eq!(
            harness.attr("#rc-probe", "data-start").as_deref(),
            Some("2024-01-10")
        );
        assert_eq!(
            harness.text_content("#rc-main [role='status']"),
            "Selected Date: January 10, 2024"
        );

        focus_node(&mut harness, "#rc-day-2024-01-06");
        for day in 6..=10 {
            let sel = format!("#rc-day-2024-01-{day:02}");
            assert_eq!(harness.attr(&sel, "data-highlighted").as_deref(), Some(""));
        }
        assert_eq!(harness.attr("#rc-day-2024-01-05", "data-highlighted"), None);

        // Escape clears partial start selection (5B D-1)
        send_key_and_pump(&mut harness, Key::Named(NamedKey::Escape), Code::Escape);
        assert_eq!(harness.attr("#rc-probe", "data-start").as_deref(), Some(""));
        assert_eq!(
            harness.text_content("#rc-main [role='status']"),
            "Selected date is now empty."
        );

        // Backward selection commits ordered range (Jan 10 -> Jan 6 => 2024-01-06..2024-01-10)
        click_and_pump(&mut harness, "#rc-day-2024-01-10");
        click_and_pump(&mut harness, "#rc-day-2024-01-06");
        assert_eq!(
            harness.attr("#rc-probe", "data-start").as_deref(),
            Some("2024-01-06")
        );
        assert_eq!(
            harness.attr("#rc-probe", "data-end").as_deref(),
            Some("2024-01-10")
        );
        assert_eq!(harness.attr("#rc-probe", "data-commits").as_deref(), Some("1"));
        assert_eq!(
            harness.attr("#rc-day-2024-01-06", "data-selection-start").as_deref(),
            Some("")
        );
        assert_eq!(
            harness.attr("#rc-day-2024-01-08", "data-range-middle").as_deref(),
            Some("")
        );
        assert_eq!(
            harness.attr("#rc-day-2024-01-10", "data-selection-end").as_deref(),
            Some("")
        );
        assert_eq!(
            harness.text_content("#rc-main [role='status']"),
            "Selected Dates: January 6, 2024 to January 10, 2024"
        );

        // Constrained calendar: min_days=3, max_days=5, exclude_disabled (Jan 15 disabled)
        click_and_pump(&mut harness, "#rc2-day-2024-01-05");
        // Jan 6 is only 2 days -> violates min_days=3, resets start to Jan 6
        click_and_pump(&mut harness, "#rc2-day-2024-01-06");
        assert_eq!(
            harness.attr("#rc-const-probe", "data-start").as_deref(),
            Some("2024-01-06")
        );
        assert_eq!(
            harness.attr("#rc-const-probe", "data-end").as_deref(),
            Some("")
        );
        // Jan 12 is 7 days -> violates max_days=5, resets start to Jan 12
        click_and_pump(&mut harness, "#rc2-day-2024-01-12");
        assert_eq!(
            harness.attr("#rc-const-probe", "data-start").as_deref(),
            Some("2024-01-12")
        );
        // Jan 16 spans disabled Jan 15 with exclude_disabled=true -> resets start to Jan 16
        click_and_pump(&mut harness, "#rc2-day-2024-01-16");
        assert_eq!(
            harness.attr("#rc-const-probe", "data-start").as_deref(),
            Some("2024-01-16")
        );
        assert_eq!(
            harness.attr("#rc-const-probe", "data-end").as_deref(),
            Some("")
        );
        // Jan 18 (3 days: 16,17,18) is valid!
        click_and_pump(&mut harness, "#rc2-day-2024-01-18");
        assert_eq!(
            harness.attr("#rc-const-probe", "data-start").as_deref(),
            Some("2024-01-16")
        );
        assert_eq!(
            harness.attr("#rc-const-probe", "data-end").as_deref(),
            Some("2024-01-18")
        );

        // Read-only guard (5B D-3): click & keyboard Enter do not mutate range, while ArrowRight moves focus
        click_and_pump(&mut harness, "#rc-ro-day-2024-01-12");
        assert_eq!(
            harness.attr("#rc-ro-probe", "data-start").as_deref(),
            Some("2024-01-05")
        );
        assert_eq!(
            harness.attr("#rc-ro-probe", "data-end").as_deref(),
            Some("2024-01-08")
        );
        focus_node(&mut harness, "#rc-ro-day-2024-01-05");
        send_key_and_pump(&mut harness, Key::Named(NamedKey::ArrowRight), Code::ArrowRight);
        assert_native_focus_id(&harness, "rc-ro-day-2024-01-06");
        send_key_and_pump(&mut harness, Key::Named(NamedKey::Enter), Code::Enter);
        assert_eq!(
            harness.attr("#rc-ro-probe", "data-start").as_deref(),
            Some("2024-01-05")
        );
        assert_eq!(
            harness.attr("#rc-ro-probe", "data-end").as_deref(),
            Some("2024-01-08")
        );
    }

    #[component]
    fn NativeRangeCustomSelect(
        attrs: Vec<Attribute>,
        trigger_tag: &'static str,
    ) -> Element {
        let ctx = use_range_calendar_context();
        let is_month = trigger_tag.contains("month");
        let is_disabled = attrs.iter().any(|a| a.name == "disabled");
        let current_val = if is_month {
            ctx.current_month().to_string()
        } else {
            ctx.current_year().to_string()
        };
        let items = if is_month {
            ctx.month_options(MonthFormat::Long)
        } else {
            ctx.year_options(YearFormat::Numeric)
        };
        let select_items: Vec<SelectItemData> = items
            .iter()
            .map(|o| SelectItemData::new(o.value_string(), o.label.clone(), false))
            .collect();

        let mut ctx_change = ctx.clone();
        let mut ctx_probe_sel = ctx.clone();
        let mut ctx_probe_val = ctx.clone();

        rsx! {
            div {
                "data-range-select-slot": "{trigger_tag}",
                SelectRoot {
                    id: trigger_tag.to_string(),
                    items: Some(select_items),
                    value: Some(current_val.clone()),
                    disabled: is_disabled,
                    on_value_change: move |next: Option<String>| {
                        if is_disabled {
                            return;
                        }
                        if let Some(s) = next
                            && let Ok(v) = s.parse::<i32>()
                        {
                            if is_month {
                                if let Ok(m) = u8::try_from(v) {
                                    ctx_change.set_month(m);
                                }
                            } else {
                                ctx_change.set_year(v);
                            }
                        }
                    },
                    SelectTrigger {
                        style: "min-width: 110px; height: 30px;",
                        SelectValue {}
                    }
                    SelectPortal {
                        SelectContent {
                            style: "background: white;",
                            SelectViewport {
                                style: "max-height: 200px;",
                                for opt in items {
                                    {
                                        let opt_val = opt.value_string();
                                        let is_selected = opt_val == current_val;
                                        rsx! {
                                            SelectItem {
                                                key: "{opt.value}",
                                                value: opt_val,
                                                text: opt.label.clone(),
                                                SelectItemText { "{opt.label}" }
                                                SelectItemIndicator {
                                                    if is_selected { "✓" }
                                                }
                                            }
                                        }
                                    }
                                }
                            }
                        }
                    }
                }
                button {
                    id: "{trigger_tag}-probe-on-select",
                    r#type: "button",
                    onclick: move |_| {
                        if is_disabled {
                            return;
                        }
                        if is_month {
                            ctx_probe_sel.set_month(4);
                        } else {
                            ctx_probe_sel.set_year(2027);
                        }
                    },
                    "on_select"
                }
                button {
                    id: "{trigger_tag}-probe-on-value-change",
                    r#type: "button",
                    onclick: move |_| {
                        if is_disabled {
                            return;
                        }
                        if is_month {
                            ctx_probe_val.set_month(7);
                        } else {
                            ctx_probe_val.set_year(2028);
                        }
                    },
                    "on_value_change"
                }
            }
        }
    }

    #[component]
    fn NativeRangeContextProbe() -> Element {
        let mut ctx = use_range_calendar_context();
        rsx! {
            div {
                id: "rc-ctx-probe",
                "data-month": "{ctx.current_month()}",
                "data-year": "{ctx.current_year()}",
                "data-month-opts": "{ctx.month_options(MonthFormat::Long).len()}",
                "data-year-opts": "{ctx.year_options(YearFormat::Numeric).len()}",
                "data-default-years": "{ctx.default_years().len()}",
            }
            button {
                id: "rc-ctx-set-oct",
                r#type: "button",
                onclick: move |_| {
                    ctx.set_month(10);
                    ctx.set_year(2025);
                },
                "ctx_set"
            }
        }
    }

    #[component]
    fn NativeRangeCustomSelectScenario() -> Element {
        rsx! {
            div {
                style: "padding: 16px; display: flex; flex-direction: column; gap: 16px;",
                RangeCalendarRoot {
                    id: "rc-custom",
                    default_placeholder: d(2024, 5, 1),
                    render_children: move |snap: RangeCalendarSnapshot| rsx! {
                        NativeRangeContextProbe {}
                        RangeCalendarHeader {
                            RangeCalendarPrevButton { id: "rc-custom-prev", "Prev" }
                            RangeCalendarHeading { id: "rc-custom-heading" }
                            RangeCalendarNextButton { id: "rc-custom-next", "Next" }
                            RangeCalendarMonthSelect {
                                id: "rc-custom-month",
                                child: move |props: Vec<Attribute>| rsx! {
                                    NativeRangeCustomSelect { attrs: props, trigger_tag: "rc-custom-month" }
                                },
                            }
                            RangeCalendarYearSelect {
                                id: "rc-custom-year",
                                child: move |props: Vec<Attribute>| rsx! {
                                    NativeRangeCustomSelect { attrs: props, trigger_tag: "rc-custom-year" }
                                },
                            }
                        }
                        RangeCalendarGrid {
                            RangeCalendarGridBody {
                                for (r_idx, week) in snap.months[0].weeks.iter().enumerate() {
                                    RangeCalendarGridRow {
                                        key: "{r_idx}",
                                        for date in week.iter().copied() {
                                            RangeCalendarCell {
                                                key: "{date}",
                                                date,
                                                month: snap.months[0].value,
                                                RangeCalendarDay {
                                                    id: "rc-custom-day-{date}",
                                                }
                                            }
                                        }
                                    }
                                }
                            }
                        }
                    }
                }

                RangeCalendarRoot {
                    id: "rc-custom-dis",
                    default_placeholder: d(2024, 5, 1),
                    disabled: true,
                    render_children: move |_snap: RangeCalendarSnapshot| rsx! {
                        RangeCalendarHeader {
                            RangeCalendarHeading { id: "rc-custom-dis-heading" }
                            RangeCalendarMonthSelect {
                                id: "rc-custom-dis-month",
                                child: move |props: Vec<Attribute>| rsx! {
                                    NativeRangeCustomSelect { attrs: props, trigger_tag: "rc-custom-dis-month" }
                                },
                            }
                            RangeCalendarYearSelect {
                                id: "rc-custom-dis-year",
                                child: move |props: Vec<Attribute>| rsx! {
                                    NativeRangeCustomSelect { attrs: props, trigger_tag: "rc-custom-dis-year" }
                                },
                            }
                        }
                    }
                }
            }
        }
    }

    #[test]
    fn native_range_calendar_custom_month_year_select_as_child_and_disabled() {
        let _lock = NATIVE_TEST_LOCK.lock().unwrap_or_else(|e| e.into_inner());
        let mut harness = mount_native_scenario(NativeRangeCustomSelectScenario);

        let month_trigger = "[data-range-select-slot='rc-custom-month'] [role='combobox']";
        let month_value = "[data-range-select-slot='rc-custom-month'] [role='combobox'] [data-placeholder]";
        let year_trigger = "[data-range-select-slot='rc-custom-year'] [role='combobox']";
        let year_value = "[data-range-select-slot='rc-custom-year'] [role='combobox'] [data-placeholder]";
        let dis_month_trigger = "[data-range-select-slot='rc-custom-dis-month'] [role='combobox']";
        let dis_year_trigger = "[data-range-select-slot='rc-custom-dis-year'] [role='combobox']";

        assert_eq!(harness.text_content("#rc-custom-heading"), "May 2024");
        assert_eq!(harness.text_content(month_value), "May");
        assert_eq!(harness.text_content(year_value), "2024");
        assert_eq!(harness.attr("#rc-ctx-probe", "data-month").as_deref(), Some("5"));
        assert_eq!(harness.attr("#rc-ctx-probe", "data-year").as_deref(), Some("2024"));
        assert_eq!(
            harness.attr("#rc-ctx-probe", "data-month-opts").as_deref(),
            Some("12")
        );
        assert_eq!(
            harness.attr("#rc-ctx-probe", "data-year-opts").as_deref(),
            harness.attr("#rc-ctx-probe", "data-default-years").as_deref()
        );

        // Select August (8)
        click_and_pump(&mut harness, month_trigger);
        advance_frame_and_pump(&mut harness, 0.02);
        click_and_pump(&mut harness, "[role='listbox'] [role='option'][data-value='8']");
        advance_frame_and_pump(&mut harness, 0.02);
        advance_frame_and_pump(&mut harness, 0.25);
        assert_eq!(harness.text_content("#rc-custom-heading"), "August 2024");
        assert_eq!(harness.text_content(month_value), "August");
        assert!(harness.query("#rc-custom-day-2024-08-10").is_some());

        // Select 2023 (visible above 2024 in scrollable 111-year viewport)
        click_and_pump(&mut harness, year_trigger);
        advance_frame_and_pump(&mut harness, 0.02);
        click_and_pump(&mut harness, "[role='listbox'] [role='option'][data-value='2023']");
        advance_frame_and_pump(&mut harness, 0.02);
        advance_frame_and_pump(&mut harness, 0.25);
        assert_eq!(harness.text_content("#rc-custom-heading"), "August 2023");
        assert_eq!(harness.text_content(year_value), "2023");

        // Select December (12), click Next -> January 2024 & reactive sync
        click_and_pump(&mut harness, month_trigger);
        advance_frame_and_pump(&mut harness, 0.02);
        click_and_pump(&mut harness, "[role='listbox'] [role='option'][data-value='12']");
        advance_frame_and_pump(&mut harness, 0.02);
        advance_frame_and_pump(&mut harness, 0.25);
        click_and_pump(&mut harness, "#rc-custom-next");
        advance_frame_and_pump(&mut harness, 0.02);
        assert_eq!(harness.text_content("#rc-custom-heading"), "January 2024");
        assert_eq!(harness.text_content(month_value), "January");
        assert_eq!(harness.text_content(year_value), "2024");

        click_and_pump(&mut harness, "#rc-custom-prev");
        advance_frame_and_pump(&mut harness, 0.02);
        assert_eq!(harness.text_content("#rc-custom-heading"), "December 2023");
        assert_eq!(harness.text_content(month_value), "December");
        assert_eq!(harness.text_content(year_value), "2023");

        // Context convenience setter & on_select
        click_and_pump(&mut harness, "#rc-ctx-set-oct");
        advance_frame_and_pump(&mut harness, 0.02);
        assert_eq!(harness.text_content("#rc-custom-heading"), "October 2025");
        click_and_pump(&mut harness, "#rc-custom-month-probe-on-select");
        advance_frame_and_pump(&mut harness, 0.02);
        assert_eq!(harness.text_content("#rc-custom-heading"), "April 2025");

        // Disabled root propagates to SelectTrigger & blocks callbacks
        assert_eq!(
            harness.attr(dis_month_trigger, "aria-disabled").as_deref(),
            Some("true")
        );
        assert_eq!(
            harness.attr(dis_year_trigger, "aria-disabled").as_deref(),
            Some("true")
        );
        click_and_pump(&mut harness, dis_month_trigger);
        assert!(harness.query("[role='listbox']").is_none());
        click_and_pump(&mut harness, "#rc-custom-dis-month-probe-on-value-change");
        click_and_pump(&mut harness, "#rc-custom-dis-year-probe-on-select");
        assert_eq!(harness.text_content("#rc-custom-dis-heading"), "May 2024");
    }

    #[component]
    fn NativeRangeCalendarPredicateAndAsChildEscapeScenario() -> Element {
        let mut pred_stage = use_signal(|| 0_u8);
        let stage = *pred_stage.read();
        let is_date_disabled = if stage == 0 {
            Some(Callback::new(|date: CalendarDate| date == d(2024, 1, 25)))
        } else {
            Some(Callback::new(|date: CalendarDate| date == d(2024, 1, 12)))
        };
        let is_date_unavailable = if stage == 0 {
            Some(Callback::new(|date: CalendarDate| date == d(2024, 1, 26)))
        } else {
            Some(Callback::new(|date: CalendarDate| date == d(2024, 1, 18)))
        };

        rsx! {
            div {
                style: "padding: 16px; display: flex; flex-direction: column; gap: 16px;",
                button {
                    id: "rc-pred-swap",
                    r#type: "button",
                    style: "width: 80px; height: 28px;",
                    onclick: move |_| {
                        let next = 1 - *pred_stage.read();
                        pred_stage.set(next);
                    },
                    "Swap"
                }

                // (a) Reactive predicate swap + exclude_disabled reconciliation
                RangeCalendarRoot {
                    id: "rc-pred",
                    default_placeholder: d(2024, 1, 1),
                    exclude_disabled: true,
                    is_date_disabled,
                    is_date_unavailable,
                    render_children: move |snap: RangeCalendarSnapshot| rsx! {
                        div {
                            id: "rc-pred-probe",
                            "data-start": snap.start_value.map(|x| x.to_iso_string()).unwrap_or_default(),
                            "data-end": snap.end_value.map(|x| x.to_iso_string()).unwrap_or_default(),
                        }
                        RangeCalendarGrid {
                            RangeCalendarGridBody {
                                for (r_idx, week) in snap.months[0].weeks.iter().enumerate() {
                                    RangeCalendarGridRow {
                                        key: "{r_idx}",
                                        for date in week.iter().copied() {
                                            RangeCalendarCell {
                                                key: "{date}",
                                                id: "rc-pred-cell-{date}",
                                                date,
                                                month: snap.months[0].value,
                                                RangeCalendarDay {
                                                    id: "rc-pred-day-{date}",
                                                    style: "display: inline-block; width: 28px; height: 28px;",
                                                }
                                            }
                                        }
                                    }
                                }
                            }
                        }
                    }
                }

                // (b) Standalone RangeCalendarRoot(child) Escape cancellation outside day cell
                RangeCalendarRoot {
                    id: "rc-ac",
                    default_placeholder: d(2024, 1, 1),
                    child: move |props: Vec<Attribute>| rsx! {
                        NativeRangeAsChildEscapeBody { attrs: props }
                    },
                }
            }
        }
    }

    #[component]
    fn NativeRangeAsChildEscapeBody(attrs: Vec<Attribute>) -> Element {
        let snap = use_range_calendar_context().snapshot();
        rsx! {
            section {
                tabindex: "-1",
                ..attrs,
                div {
                    id: "rc-ac-probe",
                    "data-start": snap.start_value.map(|x| x.to_iso_string()).unwrap_or_default(),
                    "data-end": snap.end_value.map(|x| x.to_iso_string()).unwrap_or_default(),
                }
                button {
                    id: "rc-ac-outside-day",
                    r#type: "button",
                    style: "width: 80px; height: 24px;",
                    "Header Focus Target"
                }
                RangeCalendarGrid {
                    RangeCalendarGridBody {
                        for (r_idx, week) in snap.months[0].weeks.iter().enumerate() {
                            RangeCalendarGridRow {
                                key: "{r_idx}",
                                for date in week.iter().copied() {
                                    RangeCalendarCell {
                                        key: "{date}",
                                        date,
                                        month: snap.months[0].value,
                                        RangeCalendarDay {
                                            id: "rc-ac-day-{date}",
                                            style: "display: inline-block; width: 28px; height: 28px;",
                                        }
                                    }
                                }
                            }
                        }
                    }
                }
            }
        }
    }

    #[test]
    fn native_range_calendar_reactive_predicate_reconciles_and_as_child_escape() {
        let _lock = NATIVE_TEST_LOCK.lock().unwrap_or_else(|e| e.into_inner());
        let mut harness =
            mount_native_scenario(NativeRangeCalendarPredicateAndAsChildEscapeScenario);

        // (a) Select valid range Jan 10..Jan 15 while Jan 25 is disabled
        click_and_pump(&mut harness, "#rc-pred-day-2024-01-10");
        click_and_pump(&mut harness, "#rc-pred-day-2024-01-15");
        assert_eq!(
            harness.attr("#rc-pred-probe", "data-start").as_deref(),
            Some("2024-01-10")
        );
        assert_eq!(
            harness.attr("#rc-pred-probe", "data-end").as_deref(),
            Some("2024-01-15")
        );
        assert_eq!(harness.attr("#rc-pred-cell-2024-01-12", "data-disabled"), None);
        assert_eq!(
            harness.attr("#rc-pred-cell-2024-01-25", "data-disabled").as_deref(),
            Some("")
        );
        assert_eq!(
            harness.attr("#rc-pred-cell-2024-01-26", "data-unavailable").as_deref(),
            Some("")
        );

        // Swap is_date_disabled -> Jan 12 (inside 10..15) and is_date_unavailable -> Jan 18
        click_and_pump(&mut harness, "#rc-pred-swap");

        // Cell attributes update immediately AND exclude_disabled reconciles (clears) the active range
        assert_eq!(
            harness.attr("#rc-pred-cell-2024-01-12", "data-disabled").as_deref(),
            Some("")
        );
        assert_eq!(
            harness.attr("#rc-pred-day-2024-01-12", "aria-disabled").as_deref(),
            Some("true")
        );
        assert_eq!(harness.attr("#rc-pred-cell-2024-01-25", "data-disabled"), None);
        assert_eq!(
            harness.attr("#rc-pred-cell-2024-01-18", "data-unavailable").as_deref(),
            Some("")
        );
        assert_eq!(harness.attr("#rc-pred-cell-2024-01-26", "data-unavailable"), None);
        assert_eq!(
            harness.attr("#rc-pred-probe", "data-start").as_deref(),
            Some("")
        );
        assert_eq!(
            harness.attr("#rc-pred-probe", "data-end").as_deref(),
            Some("")
        );

        // (b) Standalone RangeCalendarRoot(as_child: true): start partial range, move focus outside day cell, press Escape
        click_and_pump(&mut harness, "#rc-ac-day-2024-01-10");
        assert_eq!(
            harness.attr("#rc-ac-probe", "data-start").as_deref(),
            Some("2024-01-10")
        );
        assert_eq!(
            harness.attr("#rc-ac-probe", "data-end").as_deref(),
            Some("")
        );
        focus_node(&mut harness, "#rc-ac-outside-day");
        send_key_and_pump(&mut harness, Key::Named(NamedKey::Escape), Code::Escape);
        assert_eq!(
            harness.attr("#rc-ac-probe", "data-start").as_deref(),
            Some("")
        );
        assert_eq!(
            harness.attr("#rc-ac-probe", "data-end").as_deref(),
            Some("")
        );
    }
}

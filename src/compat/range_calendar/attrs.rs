// [KEEP-MONOXUS-BRAND]
use serde::Serialize;

use ui_sdk::foundation::date_time::{CalendarDate, CalendarLocale};

pub use ui_sdk::range_calendar::attrs::RangeCalendarCellAttrs;

// [KEEP-MONOXUS-BRAND] Preserve `data_bits_day` compatibility attribute alongside `data_range_calendar_day`.
#[derive(Clone, Debug, PartialEq, Eq)]
#[derive(Serialize)]
pub struct RangeCalendarDayAttrs {
    pub id: String,
    pub role: &'static str,
    pub aria_label: String,
    pub aria_disabled: &'static str,
    pub tabindex: Option<i32>,
    pub data_unavailable: Option<&'static str>,
    pub data_today: Option<&'static str>,
    pub data_outside_month: Option<&'static str>,
    pub data_outside_visible_months: Option<&'static str>,
    pub data_focused: Option<&'static str>,
    pub data_selection_start: Option<&'static str>,
    pub data_selection_end: Option<&'static str>,
    pub data_range_start: Option<&'static str>,
    pub data_range_end: Option<&'static str>,
    pub data_range_middle: Option<&'static str>,
    pub data_highlighted: Option<&'static str>,
    pub data_selected: Option<&'static str>,
    pub data_value: String,
    pub data_type: &'static str,
    pub data_disabled: Option<&'static str>,
    pub data_range_calendar_day: &'static str,
    pub data_bits_day: &'static str,
}

impl RangeCalendarDayAttrs {
    pub fn from_cell(
        id: String,
        date: CalendarDate,
        locale: &CalendarLocale,
        cell: &RangeCalendarCellAttrs,
        is_tabbable: bool,
    ) -> Self {
        Self::from(ui_sdk::range_calendar::attrs::RangeCalendarDayAttrs::from_cell(
            id,
            date,
            locale,
            cell,
            is_tabbable,
        ))
    }
}

impl From<ui_sdk::range_calendar::attrs::RangeCalendarDayAttrs> for RangeCalendarDayAttrs {
    fn from(attrs: ui_sdk::range_calendar::attrs::RangeCalendarDayAttrs) -> Self {
        Self {
            id: attrs.id,
            role: attrs.role,
            aria_label: attrs.aria_label,
            aria_disabled: attrs.aria_disabled,
            tabindex: attrs.tabindex,
            data_unavailable: attrs.data_unavailable,
            data_today: attrs.data_today,
            data_outside_month: attrs.data_outside_month,
            data_outside_visible_months: attrs.data_outside_visible_months,
            data_focused: attrs.data_focused,
            data_selection_start: attrs.data_selection_start,
            data_selection_end: attrs.data_selection_end,
            data_range_start: attrs.data_range_start,
            data_range_end: attrs.data_range_end,
            data_range_middle: attrs.data_range_middle,
            data_highlighted: attrs.data_highlighted,
            data_selected: attrs.data_selected,
            data_value: attrs.data_value,
            data_type: attrs.data_type,
            data_disabled: attrs.data_disabled,
            data_range_calendar_day: attrs.data_range_calendar_day,
            data_bits_day: "",
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use ui_sdk::foundation::compose::{Slottable, project_attrs_as_child};

    #[test]
    fn range_calendar_day_attrs_preserves_data_bits_day_compat() {
        let date = CalendarDate::new(2024, 1, 15).expect("valid date");
        let cell = RangeCalendarCellAttrs::new(
            String::from("cell-1"),
            date,
            true,
            false,
            false,
            true,
            false,
            false,
            true,
            true,
            false,
            true,
            false,
            false,
            true,
            true,
        );
        let day = RangeCalendarDayAttrs::from_cell(
            String::from("day-1"),
            date,
            &CalendarLocale::EnUs,
            &cell,
            true,
        );
        assert_eq!(day.tabindex, Some(0));
        assert_eq!(day.aria_label, "Monday, January 15, 2024");
        assert_eq!(day.data_selection_start, Some(""));
        assert_eq!(day.data_range_start, Some(""));
        assert_eq!(day.data_highlighted, Some(""));
        assert_eq!(day.data_range_calendar_day, "");
        assert_eq!(day.data_bits_day, "");

        let (_, projected, _) = project_attrs_as_child(&day, "slot", Slottable::new("15"));
        assert_eq!(projected, day);

        let disabled_cell = RangeCalendarCellAttrs::new(
            String::from("cell-dis"),
            date,
            false,
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
            false,
            true,
        );
        let disabled_day = RangeCalendarDayAttrs::from_cell(
            String::from("day-dis"),
            date,
            &CalendarLocale::EnUs,
            &disabled_cell,
            false,
        );
        assert_eq!(disabled_day.tabindex, None);
        assert_eq!(disabled_day.data_bits_day, "");
    }
}

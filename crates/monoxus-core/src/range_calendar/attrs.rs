use crate::foundation::{
    compose::{Slottable, project_as_child},
    date_time::{CalendarDate, CalendarLocale},
    shared::data_disabled_attr,
};

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct RangeCalendarCellAttrs {
    pub id: String,
    pub role: &'static str,
    pub aria_selected: &'static str,
    pub aria_disabled: &'static str,
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
    pub data_range_calendar_cell: &'static str,
}

impl RangeCalendarCellAttrs {
    #[allow(clippy::too_many_arguments)]
    pub fn new(
        id: String,
        date: CalendarDate,
        is_selected: bool,
        is_disabled: bool,
        is_unavailable: bool,
        is_today: bool,
        is_outside_month: bool,
        is_outside_visible_months: bool,
        is_focused: bool,
        is_selection_start: bool,
        is_selection_end: bool,
        is_range_start: bool,
        is_range_end: bool,
        is_range_middle: bool,
        is_highlighted: bool,
        disable_days_outside_month: bool,
    ) -> Self {
        let effective_disabled = is_disabled || (is_outside_month && disable_days_outside_month);
        let aria_disabled = effective_disabled || is_unavailable;
        Self {
            id,
            role: "gridcell",
            aria_selected: if is_selected { "true" } else { "false" },
            aria_disabled: if aria_disabled { "true" } else { "false" },
            data_unavailable: data_disabled_attr(is_unavailable),
            data_today: data_disabled_attr(is_today),
            data_outside_month: data_disabled_attr(is_outside_month),
            data_outside_visible_months: data_disabled_attr(is_outside_visible_months),
            data_focused: data_disabled_attr(is_focused),
            data_selection_start: data_disabled_attr(is_selection_start),
            data_selection_end: data_disabled_attr(is_selection_end),
            data_range_start: data_disabled_attr(is_range_start),
            data_range_end: data_disabled_attr(is_range_end),
            data_range_middle: data_disabled_attr(is_range_middle),
            data_highlighted: data_disabled_attr(is_highlighted),
            data_selected: data_disabled_attr(is_selected),
            data_value: date.to_iso_string(),
            data_type: "date",
            data_disabled: data_disabled_attr(effective_disabled),
            data_range_calendar_cell: "",
        }
    }
}

#[derive(Clone, Debug, PartialEq, Eq)]
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
        let is_disabled = cell.data_disabled.is_some();
        let tabindex = if is_disabled {
            None
        } else if is_tabbable {
            Some(0)
        } else {
            Some(-1)
        };
        Self {
            id,
            role: "button",
            aria_label: locale.cell_aria_label(date),
            aria_disabled: cell.aria_disabled,
            tabindex,
            data_unavailable: cell.data_unavailable,
            data_today: cell.data_today,
            data_outside_month: cell.data_outside_month,
            data_outside_visible_months: cell.data_outside_visible_months,
            data_focused: cell.data_focused,
            data_selection_start: cell.data_selection_start,
            data_selection_end: cell.data_selection_end,
            data_range_start: cell.data_range_start,
            data_range_end: cell.data_range_end,
            data_range_middle: cell.data_range_middle,
            data_highlighted: cell.data_highlighted,
            data_selected: cell.data_selected,
            data_value: cell.data_value.clone(),
            data_type: cell.data_type,
            data_disabled: cell.data_disabled,
            data_range_calendar_day: "",
            data_bits_day: "",
        }
    }
}

pub fn project_range_calendar_day_as_child<T, C>(
    attrs: &RangeCalendarDayAttrs,
    target: T,
    content: Slottable<C>,
) -> (T, RangeCalendarDayAttrs, C) {
    let (target, content) = project_as_child(target, content);
    (target, attrs.clone(), content)
}

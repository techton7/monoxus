use crate::foundation::{
    compose::{Slottable, project_as_child},
    date_time::{CalendarDate, CalendarLocale},
    shared::{Direction, data_disabled_attr},
};

use super::types::{CalendarKind, CalendarPart};

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct CalendarRootAttrs {
    pub id: String,
    pub role: &'static str,
    pub aria_label: String,
    pub dir: &'static str,
    pub data_calendar_root: Option<&'static str>,
    pub data_range_calendar_root: Option<&'static str>,
    pub data_invalid: Option<&'static str>,
    pub data_disabled: Option<&'static str>,
    pub data_readonly: Option<&'static str>,
}

impl CalendarRootAttrs {
    pub fn new(
        id: String,
        kind: CalendarKind,
        full_calendar_label: String,
        dir: Direction,
        is_invalid: bool,
        disabled: bool,
        read_only: bool,
    ) -> Self {
        Self {
            id,
            role: "application",
            aria_label: full_calendar_label,
            dir: dir.as_str(),
            data_calendar_root: data_disabled_attr(kind == CalendarKind::Calendar),
            data_range_calendar_root: data_disabled_attr(kind == CalendarKind::RangeCalendar),
            data_invalid: data_disabled_attr(is_invalid),
            data_disabled: data_disabled_attr(disabled),
            data_readonly: data_disabled_attr(read_only),
        }
    }
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct CalendarHeaderAttrs {
    pub id: String,
    pub data_disabled: Option<&'static str>,
    pub data_readonly: Option<&'static str>,
    pub data_calendar_header: Option<&'static str>,
    pub data_range_calendar_header: Option<&'static str>,
}

impl CalendarHeaderAttrs {
    pub fn new(id: String, kind: CalendarKind, disabled: bool, read_only: bool) -> Self {
        Self {
            id,
            data_disabled: data_disabled_attr(disabled),
            data_readonly: data_disabled_attr(read_only),
            data_calendar_header: data_disabled_attr(kind == CalendarKind::Calendar),
            data_range_calendar_header: data_disabled_attr(kind == CalendarKind::RangeCalendar),
        }
    }
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct CalendarHeadingAttrs {
    pub id: String,
    pub aria_hidden: &'static str,
    pub data_disabled: Option<&'static str>,
    pub data_readonly: Option<&'static str>,
    pub data_calendar_heading: Option<&'static str>,
    pub data_range_calendar_heading: Option<&'static str>,
}

impl CalendarHeadingAttrs {
    pub fn new(id: String, kind: CalendarKind, disabled: bool, read_only: bool) -> Self {
        Self {
            id,
            aria_hidden: "true",
            data_disabled: data_disabled_attr(disabled),
            data_readonly: data_disabled_attr(read_only),
            data_calendar_heading: data_disabled_attr(kind == CalendarKind::Calendar),
            data_range_calendar_heading: data_disabled_attr(kind == CalendarKind::RangeCalendar),
        }
    }
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct CalendarNavButtonAttrs {
    pub id: String,
    pub role: &'static str,
    pub r#type: &'static str,
    pub aria_label: &'static str,
    pub aria_disabled: &'static str,
    pub disabled: bool,
    pub data_disabled: Option<&'static str>,
    pub data_calendar_prev_button: Option<&'static str>,
    pub data_calendar_next_button: Option<&'static str>,
    pub data_range_calendar_prev_button: Option<&'static str>,
    pub data_range_calendar_next_button: Option<&'static str>,
}

impl CalendarNavButtonAttrs {
    pub fn prev(id: String, kind: CalendarKind, disabled: bool) -> Self {
        Self {
            id,
            role: "button",
            r#type: "button",
            aria_label: "Previous",
            aria_disabled: if disabled { "true" } else { "false" },
            disabled,
            data_disabled: data_disabled_attr(disabled),
            data_calendar_prev_button: data_disabled_attr(kind == CalendarKind::Calendar),
            data_calendar_next_button: None,
            data_range_calendar_prev_button: data_disabled_attr(kind == CalendarKind::RangeCalendar),
            data_range_calendar_next_button: None,
        }
    }

    pub fn next(id: String, kind: CalendarKind, disabled: bool) -> Self {
        Self {
            id,
            role: "button",
            r#type: "button",
            aria_label: "Next",
            aria_disabled: if disabled { "true" } else { "false" },
            disabled,
            data_disabled: data_disabled_attr(disabled),
            data_calendar_prev_button: None,
            data_calendar_next_button: data_disabled_attr(kind == CalendarKind::Calendar),
            data_range_calendar_prev_button: None,
            data_range_calendar_next_button: data_disabled_attr(kind == CalendarKind::RangeCalendar),
        }
    }
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct CalendarGridAttrs {
    pub id: String,
    pub tabindex: i32,
    pub role: &'static str,
    pub aria_readonly: &'static str,
    pub aria_disabled: &'static str,
    pub data_readonly: Option<&'static str>,
    pub data_disabled: Option<&'static str>,
    pub data_calendar_grid: Option<&'static str>,
    pub data_range_calendar_grid: Option<&'static str>,
}

impl CalendarGridAttrs {
    pub fn new(id: String, kind: CalendarKind, disabled: bool, read_only: bool) -> Self {
        Self {
            id,
            tabindex: -1,
            role: "grid",
            aria_readonly: if read_only { "true" } else { "false" },
            aria_disabled: if disabled { "true" } else { "false" },
            data_readonly: data_disabled_attr(read_only),
            data_disabled: data_disabled_attr(disabled),
            data_calendar_grid: data_disabled_attr(kind == CalendarKind::Calendar),
            data_range_calendar_grid: data_disabled_attr(kind == CalendarKind::RangeCalendar),
        }
    }
}

/// Shared structural attribute bundle for `GridHead`, `GridBody`, `GridRow`, and `HeadCell`.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct CalendarSubpartAttrs {
    pub id: String,
    pub part: CalendarPart,
    pub data_disabled: Option<&'static str>,
    pub data_readonly: Option<&'static str>,
    pub data_calendar_part: Option<&'static str>,
    pub data_range_calendar_part: Option<&'static str>,
}

impl CalendarSubpartAttrs {
    pub fn new(
        id: String,
        kind: CalendarKind,
        part: CalendarPart,
        disabled: bool,
        read_only: bool,
    ) -> Self {
        Self {
            id,
            part,
            data_disabled: data_disabled_attr(disabled),
            data_readonly: data_disabled_attr(read_only),
            data_calendar_part: data_disabled_attr(kind == CalendarKind::Calendar),
            data_range_calendar_part: data_disabled_attr(kind == CalendarKind::RangeCalendar),
        }
    }
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct CalendarCellAttrs {
    pub id: String,
    pub role: &'static str,
    pub aria_selected: &'static str,
    pub aria_disabled: &'static str,
    pub data_unavailable: Option<&'static str>,
    pub data_today: Option<&'static str>,
    pub data_outside_month: Option<&'static str>,
    pub data_outside_visible_months: Option<&'static str>,
    pub data_focused: Option<&'static str>,
    pub data_selected: Option<&'static str>,
    pub data_value: String,
    pub data_type: &'static str,
    pub data_disabled: Option<&'static str>,
    pub data_calendar_cell: &'static str,
}

impl CalendarCellAttrs {
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
            data_selected: data_disabled_attr(is_selected),
            data_value: date.to_iso_string(),
            data_type: "date",
            data_disabled: data_disabled_attr(effective_disabled),
            data_calendar_cell: "",
        }
    }
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct CalendarDayAttrs {
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
    pub data_selected: Option<&'static str>,
    pub data_value: String,
    pub data_type: &'static str,
    pub data_disabled: Option<&'static str>,
    pub data_calendar_day: &'static str,
    pub data_bits_day: &'static str,
}

impl CalendarDayAttrs {
    pub fn from_cell(
        id: String,
        date: CalendarDate,
        locale: &CalendarLocale,
        cell: &CalendarCellAttrs,
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
            data_selected: cell.data_selected,
            data_value: cell.data_value.clone(),
            data_type: cell.data_type,
            data_disabled: cell.data_disabled,
            data_calendar_day: "",
            data_bits_day: "",
        }
    }
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct CalendarSelectAttrs {
    pub id: String,
    pub aria_label: String,
    pub disabled: bool,
    pub data_disabled: Option<&'static str>,
    pub data_calendar_month_select: Option<&'static str>,
    pub data_calendar_year_select: Option<&'static str>,
    pub data_range_calendar_month_select: Option<&'static str>,
    pub data_range_calendar_year_select: Option<&'static str>,
}

impl CalendarSelectAttrs {
    pub fn month(
        id: String,
        kind: CalendarKind,
        aria_label: Option<String>,
        disabled: bool,
    ) -> Self {
        Self {
            id,
            aria_label: aria_label.unwrap_or_else(|| String::from("Select a month")),
            disabled,
            data_disabled: data_disabled_attr(disabled),
            data_calendar_month_select: data_disabled_attr(kind == CalendarKind::Calendar),
            data_calendar_year_select: None,
            data_range_calendar_month_select: data_disabled_attr(
                kind == CalendarKind::RangeCalendar,
            ),
            data_range_calendar_year_select: None,
        }
    }

    pub fn year(
        id: String,
        kind: CalendarKind,
        aria_label: Option<String>,
        disabled: bool,
    ) -> Self {
        Self {
            id,
            aria_label: aria_label.unwrap_or_else(|| String::from("Select a year")),
            disabled,
            data_disabled: data_disabled_attr(disabled),
            data_calendar_month_select: None,
            data_calendar_year_select: data_disabled_attr(kind == CalendarKind::Calendar),
            data_range_calendar_month_select: None,
            data_range_calendar_year_select: data_disabled_attr(
                kind == CalendarKind::RangeCalendar,
            ),
        }
    }
}

pub fn project_calendar_root_as_child<T, C>(
    attrs: &CalendarRootAttrs,
    target: T,
    content: Slottable<C>,
) -> (T, CalendarRootAttrs, C) {
    let (target, content) = project_as_child(target, content);
    (target, attrs.clone(), content)
}

pub fn project_calendar_day_as_child<T, C>(
    attrs: &CalendarDayAttrs,
    target: T,
    content: Slottable<C>,
) -> (T, CalendarDayAttrs, C) {
    let (target, content) = project_as_child(target, content);
    (target, attrs.clone(), content)
}

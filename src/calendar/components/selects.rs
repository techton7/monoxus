use dioxus::prelude::*;

use crate::foundation::date_time::{MonthFormat, YearFormat};

use super::super::{
    attrs::CalendarSelectAttrs,
    runtime::use_calendar_shared_context,
    types::CalendarSelectOption,
};

#[derive(Clone, PartialEq, Props)]
pub struct CalendarMonthSelectProps {
    #[props(default)]
    pub id: Option<String>,
    #[props(default)]
    pub class: Option<String>,
    #[props(default)]
    pub style: Option<String>,
    #[props(default)]
    pub months: Option<Vec<u8>>,
    #[props(default)]
    pub month_format: MonthFormat,
    #[props(default)]
    pub month_formatter: Option<Callback<u8, String>>,
    #[props(default = false)]
    pub disabled: bool,
    #[props(default)]
    pub aria_label: Option<String>,
    #[props(default)]
    pub on_change: Option<EventHandler<FormEvent>>,
    #[props(default)]
    pub on_mounted: Option<EventHandler<MountedEvent>>,
    #[props(extends = GlobalAttributes)]
    pub attributes: Vec<Attribute>,
    #[props(default)]
    pub children: Option<Element>,
}

#[component]
pub fn CalendarMonthSelect(props: CalendarMonthSelectProps) -> Element {
    let ctx = use_calendar_shared_context();
    let attrs: CalendarSelectAttrs =
        ctx.runtime
            .month_select_attrs(props.id.clone(), props.aria_label.clone(), props.disabled);
    let current_month = ctx.runtime.current_month();
    let locale = ctx.runtime.locale();

    let month_numbers: Vec<u8> = props
        .months
        .clone()
        .unwrap_or_else(|| (1..=12).collect());
    let month_items: Vec<CalendarSelectOption> = month_numbers
        .into_iter()
        .map(|month| {
            let label = if let Some(ref fmt_cb) = props.month_formatter {
                fmt_cb.call(month)
            } else {
                locale.month_label(month, props.month_format)
            };
            CalendarSelectOption {
                value: month as i32,
                label,
            }
        })
        .collect();

    let mut runtime = ctx.runtime.clone();
    let is_disabled = attrs.disabled;
    let consumer_on_change = props.on_change;

    rsx! {
        select {
            id: "{attrs.id}",
            class: props.class,
            style: props.style,
            aria_label: "{attrs.aria_label}",
            disabled: attrs.disabled,
            value: "{current_month}",
            "data-disabled": attrs.data_disabled,
            "data-calendar-month-select": attrs.data_calendar_month_select,
            "data-range-calendar-month-select": attrs.data_range_calendar_month_select,
            onchange: move |evt: FormEvent| {
                if is_disabled {
                    return;
                }
                if let Some(ref handler) = consumer_on_change {
                    handler.call(evt.clone());
                }
                if let Ok(month) = evt.value().trim().parse::<u8>() {
                    runtime.set_month(month);
                }
            },
            onmounted: move |evt| {
                if let Some(ref handler) = props.on_mounted {
                    handler.call(evt);
                }
            },
            ..props.attributes,
            if let Some(children) = props.children {
                {children}
            } else {
                for item in month_items {
                    option {
                        key: "{item.value}",
                        value: "{item.value}",
                        selected: item.value == current_month as i32,
                        "{item.label}"
                    }
                }
            }
        }
    }
}

#[derive(Clone, PartialEq, Props)]
pub struct CalendarYearSelectProps {
    #[props(default)]
    pub id: Option<String>,
    #[props(default)]
    pub class: Option<String>,
    #[props(default)]
    pub style: Option<String>,
    #[props(default)]
    pub years: Option<Vec<i32>>,
    #[props(default)]
    pub year_format: YearFormat,
    #[props(default)]
    pub year_formatter: Option<Callback<i32, String>>,
    #[props(default = false)]
    pub disabled: bool,
    #[props(default)]
    pub aria_label: Option<String>,
    #[props(default)]
    pub on_change: Option<EventHandler<FormEvent>>,
    #[props(default)]
    pub on_mounted: Option<EventHandler<MountedEvent>>,
    #[props(extends = GlobalAttributes)]
    pub attributes: Vec<Attribute>,
    #[props(default)]
    pub children: Option<Element>,
}

#[component]
pub fn CalendarYearSelect(props: CalendarYearSelectProps) -> Element {
    let ctx = use_calendar_shared_context();
    let attrs =
        ctx.runtime
            .year_select_attrs(props.id.clone(), props.aria_label.clone(), props.disabled);
    let current_year = ctx.runtime.current_year();
    let locale = ctx.runtime.locale();

    let year_numbers: Vec<i32> = props
        .years
        .clone()
        .filter(|v| !v.is_empty())
        .unwrap_or_else(|| ctx.runtime.default_years());
    let year_items: Vec<CalendarSelectOption> = year_numbers
        .into_iter()
        .map(|year| {
            let label = if let Some(ref fmt_cb) = props.year_formatter {
                fmt_cb.call(year)
            } else {
                locale.year_label(year, props.year_format)
            };
            CalendarSelectOption { value: year, label }
        })
        .collect();

    let mut runtime = ctx.runtime.clone();
    let is_disabled = attrs.disabled;
    let consumer_on_change = props.on_change;

    rsx! {
        select {
            id: "{attrs.id}",
            class: props.class,
            style: props.style,
            aria_label: "{attrs.aria_label}",
            disabled: attrs.disabled,
            value: "{current_year}",
            "data-disabled": attrs.data_disabled,
            "data-calendar-year-select": attrs.data_calendar_year_select,
            "data-range-calendar-year-select": attrs.data_range_calendar_year_select,
            onchange: move |evt: FormEvent| {
                if is_disabled {
                    return;
                }
                if let Some(ref handler) = consumer_on_change {
                    handler.call(evt.clone());
                }
                if let Ok(year) = evt.value().trim().parse::<i32>() {
                    runtime.set_year(year);
                }
            },
            onmounted: move |evt| {
                if let Some(ref handler) = props.on_mounted {
                    handler.call(evt);
                }
            },
            ..props.attributes,
            if let Some(children) = props.children {
                {children}
            } else {
                for item in year_items {
                    option {
                        key: "{item.value}",
                        value: "{item.value}",
                        selected: item.value == current_year,
                        "{item.label}"
                    }
                }
            }
        }
    }
}

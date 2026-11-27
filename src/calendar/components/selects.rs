use dioxus::prelude::*;

use crate::foundation::{
    compose::{collect_user_attrs, merge_props, push_attr_opt, push_event_handler},
    date_time::{MonthFormat, YearFormat},
};

use super::super::{
    attrs::CalendarSelectAttrs,
    runtime::use_calendar_shared_context,
    types::{CalendarSelectOption, build_calendar_month_options, build_calendar_year_options},
};

#[derive(Clone, PartialEq, Props)]
pub struct CalendarMonthSelectProps {
    #[props(default, into)]
    pub id: Option<String>,
    #[props(default, into)]
    pub class: Option<String>,
    #[props(default, into)]
    pub style: Option<String>,
    #[props(default)]
    pub months: Option<Vec<u8>>,
    #[props(default)]
    pub month_format: MonthFormat,
    #[props(default)]
    pub month_formatter: Option<Callback<u8, String>>,
    #[props(default = false)]
    pub disabled: bool,
    #[props(default, into)]
    pub aria_label: Option<String>,
    #[props(default)]
    pub child: Option<Callback<Vec<Attribute>, Element>>,
    #[props(default)]
    pub on_change: Option<EventHandler<FormEvent>>,
    #[props(default)]
    pub on_mounted: Option<EventHandler<MountedEvent>>,
    #[props(extends = GlobalAttributes)]
    pub attributes: Vec<Attribute>,
    #[props(default)]
    pub children: Option<Element>,
}

#[derive(Clone, PartialEq, Props)]
pub struct CalendarYearSelectProps {
    #[props(default, into)]
    pub id: Option<String>,
    #[props(default, into)]
    pub class: Option<String>,
    #[props(default, into)]
    pub style: Option<String>,
    #[props(default)]
    pub years: Option<Vec<i32>>,
    #[props(default)]
    pub year_format: YearFormat,
    #[props(default)]
    pub year_formatter: Option<Callback<i32, String>>,
    #[props(default = false)]
    pub disabled: bool,
    #[props(default, into)]
    pub aria_label: Option<String>,
    #[props(default)]
    pub child: Option<Callback<Vec<Attribute>, Element>>,
    #[props(default)]
    pub on_change: Option<EventHandler<FormEvent>>,
    #[props(default)]
    pub on_mounted: Option<EventHandler<MountedEvent>>,
    #[props(extends = GlobalAttributes)]
    pub attributes: Vec<Attribute>,
    #[props(default)]
    pub children: Option<Element>,
}

#[allow(clippy::too_many_arguments)]
fn render_calendar_select(
    attrs: CalendarSelectAttrs,
    value_i32: i32,
    items: Vec<CalendarSelectOption>,
    is_month: bool,
    class: Option<String>,
    style: Option<String>,
    child: Option<Callback<Vec<Attribute>, Element>>,
    consumer_on_change: Option<EventHandler<FormEvent>>,
    on_mounted: Option<EventHandler<MountedEvent>>,
    attributes: Vec<Attribute>,
    children: Option<Element>,
    mut runtime: super::super::runtime::CalendarBaseRuntime,
) -> Element {
    let value_string = value_i32.to_string();
    let is_disabled = attrs.disabled;

    let base_on_change = EventHandler::new(move |evt: FormEvent| {
        if is_disabled {
            return;
        }
        if let Ok(parsed) = evt.value().trim().parse::<i32>() {
            if is_month {
                if let Ok(m) = u8::try_from(parsed)
                    && (1..=12).contains(&m)
                {
                    runtime.set_month(m);
                }
            } else {
                runtime.set_year(parsed);
            }
        }
    });

    let mut base = Vec::with_capacity(11);
    base.push(Attribute::new("id", attrs.id, None, false));
    base.push(Attribute::new("aria-label", attrs.aria_label, None, false));
    if is_disabled {
        base.push(Attribute::new("disabled", true, None, false));
    }
    base.push(Attribute::new("value", value_string, None, false));
    push_attr_opt(&mut base, "data-disabled", attrs.data_disabled);
    push_attr_opt(
        &mut base,
        "data-calendar-month-select",
        attrs.data_calendar_month_select,
    );
    push_attr_opt(
        &mut base,
        "data-calendar-year-select",
        attrs.data_calendar_year_select,
    );
    push_attr_opt(
        &mut base,
        "data-range-calendar-month-select",
        attrs.data_range_calendar_month_select,
    );
    push_attr_opt(
        &mut base,
        "data-range-calendar-year-select",
        attrs.data_range_calendar_year_select,
    );
    push_event_handler(&mut base, "onchange", Some(base_on_change));

    let mut user = collect_user_attrs(class, style, attributes);
    if !is_disabled {
        push_event_handler(&mut user, "onchange", consumer_on_change);
    }
    push_event_handler(&mut user, "onmounted", on_mounted);

    let merged = merge_props(base, user);

    if let Some(child) = child {
        return child.call(merged);
    }

    rsx! {
        select {
            ..merged,
            if let Some(children) = children {
                {children}
            } else {
                for item in items {
                    option {
                        key: "{item.value}",
                        value: "{item.value}",
                        selected: item.value == value_i32,
                        "{item.label}"
                    }
                }
            }
        }
    }
}

#[component]
pub fn CalendarMonthSelect(props: CalendarMonthSelectProps) -> Element {
    let ctx = use_calendar_shared_context();
    let attrs = ctx
        .runtime
        .month_select_attrs(props.id, props.aria_label, props.disabled);
    let current_month = ctx.runtime.current_month() as i32;
    let locale = ctx.runtime.locale();

    let items = if let Some(ref fmt_cb) = props.month_formatter {
        let month_numbers: Vec<u8> = props.months.clone().unwrap_or_else(|| (1..=12).collect());
        month_numbers
            .into_iter()
            .map(|month| CalendarSelectOption {
                value: month as i32,
                label: fmt_cb.call(month),
            })
            .collect()
    } else {
        build_calendar_month_options(props.months.as_deref(), &locale, props.month_format)
    };

    render_calendar_select(
        attrs,
        current_month,
        items,
        true,
        props.class,
        props.style,
        props.child,
        props.on_change,
        props.on_mounted,
        props.attributes,
        props.children,
        ctx.runtime,
    )
}

#[component]
pub fn CalendarYearSelect(props: CalendarYearSelectProps) -> Element {
    let ctx = use_calendar_shared_context();
    let attrs = ctx
        .runtime
        .year_select_attrs(props.id, props.aria_label, props.disabled);
    let current_year = ctx.runtime.current_year();
    let locale = ctx.runtime.locale();

    let year_numbers: Vec<i32> = props
        .years
        .clone()
        .filter(|v| !v.is_empty())
        .unwrap_or_else(|| ctx.runtime.default_years());
    let items = if let Some(ref fmt_cb) = props.year_formatter {
        year_numbers
            .into_iter()
            .map(|year| CalendarSelectOption {
                value: year,
                label: fmt_cb.call(year),
            })
            .collect()
    } else {
        build_calendar_year_options(&year_numbers, &locale, props.year_format)
    };

    render_calendar_select(
        attrs,
        current_year,
        items,
        false,
        props.class,
        props.style,
        props.child,
        props.on_change,
        props.on_mounted,
        props.attributes,
        props.children,
        ctx.runtime,
    )
}

use dioxus::prelude::*;

use crate::foundation::{
    compose::merge_global_attributes,
    date_time::{MonthFormat, YearFormat},
};

use super::super::{
    attrs::CalendarSelectAttrs,
    runtime::use_calendar_shared_context,
    types::{
        CalendarSelectOption, build_calendar_month_options, build_calendar_year_options,
        resolve_selected_calendar_option,
    },
};

/// Slot props passed to `render_child` when `CalendarMonthSelect` or `CalendarYearSelect`
/// is rendered with `as_child: true`.
#[derive(Clone, PartialEq, Props)]
pub struct CalendarSelectRenderProps {
    pub attrs: CalendarSelectAttrs,
    pub value: i32,
    pub value_string: String,
    pub items: Vec<CalendarSelectOption>,
    pub selected_item: Option<CalendarSelectOption>,
    pub disabled: bool,
    #[props(default)]
    pub class: Option<String>,
    #[props(default)]
    pub style: Option<String>,
    pub on_change: EventHandler<FormEvent>,
    pub on_value_change: EventHandler<Option<String>>,
    pub on_select: EventHandler<i32>,
    #[props(default)]
    pub on_mounted: Option<EventHandler<MountedEvent>>,
    #[props(extends = GlobalAttributes)]
    pub attributes: Vec<Attribute>,
    #[props(default)]
    pub children: Option<Element>,
}

impl CalendarSelectRenderProps {
    /// Converts `items` into `SelectItemData` so `SelectRoot` pre-populates
    /// `selected_labels` for `SelectValue` before first open.
    pub fn select_items(&self) -> Vec<crate::select::SelectItemData> {
        self.items
            .iter()
            .map(|opt| {
                crate::select::SelectItemData::new(opt.value_string(), opt.label.clone(), false)
            })
            .collect()
    }

    /// Returns the localized label for the currently selected option, falling back to `value_string`.
    pub fn selected_label(&self) -> String {
        self.selected_item
            .as_ref()
            .map(|item| item.label.clone())
            .unwrap_or_else(|| self.value_string.clone())
    }
}

pub type CalendarMonthSelectRenderProps = CalendarSelectRenderProps;
pub type CalendarYearSelectRenderProps = CalendarSelectRenderProps;

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
    #[props(default = false)]
    pub as_child: bool,
    #[props(default)]
    pub render_child: Option<Callback<CalendarSelectRenderProps, Element>>,
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
    #[props(default = false)]
    pub as_child: bool,
    #[props(default)]
    pub render_child: Option<Callback<CalendarSelectRenderProps, Element>>,
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
    as_child: bool,
    render_child: Option<Callback<CalendarSelectRenderProps, Element>>,
    consumer_on_change: Option<EventHandler<FormEvent>>,
    on_mounted: Option<EventHandler<MountedEvent>>,
    attributes: Vec<Attribute>,
    children: Option<Element>,
    runtime: super::super::runtime::CalendarBaseRuntime,
) -> Element {
    let value_string = value_i32.to_string();
    let selected_item = resolve_selected_calendar_option(&items, value_i32);
    let is_disabled = attrs.disabled;

    let apply_val = move |rt: &mut super::super::runtime::CalendarBaseRuntime, val: i32| {
        if is_month {
            if let Ok(m) = u8::try_from(val)
                && (1..=12).contains(&m)
            {
                rt.set_month(m);
            }
        } else {
            rt.set_year(val);
        }
    };

    let mut rt_change = runtime.clone();
    let on_change = Callback::new(move |evt: FormEvent| {
        if is_disabled {
            return;
        }
        if let Some(handler) = consumer_on_change {
            handler.call(evt.clone());
        }
        if let Ok(parsed) = evt.value().trim().parse::<i32>() {
            apply_val(&mut rt_change, parsed);
        }
    });

    let mut rt_value_change = runtime.clone();
    let on_value_change = Callback::new(move |next: Option<String>| {
        if is_disabled {
            return;
        }
        if let Some(raw) = next
            && let Ok(parsed) = raw.trim().parse::<i32>()
        {
            apply_val(&mut rt_value_change, parsed);
        }
    });

    let mut rt_select = runtime;
    let on_select = Callback::new(move |val: i32| {
        if is_disabled {
            return;
        }
        apply_val(&mut rt_select, val);
    });

    if as_child
        && let Some(render_child) = render_child.as_ref()
    {
        return render_child.call(CalendarSelectRenderProps {
            attrs,
            value: value_i32,
            value_string,
            items,
            selected_item,
            disabled: is_disabled,
            class,
            style,
            on_change,
            on_value_change,
            on_select,
            on_mounted,
            attributes,
            children,
        });
    }

    let merged = merge_global_attributes(class.as_deref(), style.as_deref(), attributes);

    rsx! {
        select {
            id: "{attrs.id}",
            class: merged.class.as_deref(),
            style: merged.style.as_deref(),
            aria_label: "{attrs.aria_label}",
            disabled: attrs.disabled,
            value: "{value_string}",
            "data-disabled": attrs.data_disabled,
            "data-calendar-month-select": attrs.data_calendar_month_select,
            "data-calendar-year-select": attrs.data_calendar_year_select,
            "data-range-calendar-month-select": attrs.data_range_calendar_month_select,
            "data-range-calendar-year-select": attrs.data_range_calendar_year_select,
            onchange: move |evt| on_change.call(evt),
            onmounted: move |evt| {
                if let Some(handler) = on_mounted {
                    handler.call(evt);
                }
            },
            ..merged.rest,
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
    let attrs =
        ctx.runtime
            .month_select_attrs(props.id.clone(), props.aria_label.clone(), props.disabled);
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
        props.as_child,
        props.render_child,
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
        props.as_child,
        props.render_child,
        props.on_change,
        props.on_mounted,
        props.attributes,
        props.children,
        ctx.runtime,
    )
}

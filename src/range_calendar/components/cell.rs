use dioxus::prelude::*;

use crate::foundation::{
    compose::merge_global_attributes, date_time::CalendarDate, state::sync_signal,
};

use super::super::{attrs::RangeCalendarCellAttrs, runtime::use_range_calendar_context};

#[derive(Clone, Copy, PartialEq)]
pub struct RangeCalendarCellContext {
    pub date: Signal<CalendarDate>,
    pub month: Signal<CalendarDate>,
}

pub fn use_range_calendar_cell_context() -> RangeCalendarCellContext {
    use_context::<RangeCalendarCellContext>()
}

#[derive(Clone, PartialEq, Props)]
pub struct RangeCalendarCellRenderProps {
    pub attrs: RangeCalendarCellAttrs,
    pub date: CalendarDate,
    pub month: CalendarDate,
    #[props(default)]
    pub class: Option<String>,
    #[props(default)]
    pub style: Option<String>,
    #[props(default)]
    pub on_mounted: Option<EventHandler<MountedEvent>>,
    #[props(extends = GlobalAttributes)]
    pub attributes: Vec<Attribute>,
    pub children: Element,
}

#[derive(Clone, PartialEq, Props)]
pub struct RangeCalendarCellProps {
    #[props(default)]
    pub id: Option<String>,
    pub date: CalendarDate,
    pub month: CalendarDate,
    #[props(default)]
    pub class: Option<String>,
    #[props(default)]
    pub style: Option<String>,
    #[props(default = false)]
    pub as_child: bool,
    #[props(default)]
    pub render_child: Option<Callback<RangeCalendarCellRenderProps, Element>>,
    #[props(default)]
    pub on_mounted: Option<EventHandler<MountedEvent>>,
    #[props(extends = GlobalAttributes)]
    pub attributes: Vec<Attribute>,
    pub children: Element,
}

#[component]
pub fn RangeCalendarCell(props: RangeCalendarCellProps) -> Element {
    let ctx = use_range_calendar_context();
    let _ = ctx.runtime.predicate_rev();
    let mut date_sig = use_signal(|| props.date);
    sync_signal(&mut date_sig, props.date);
    let mut month_sig = use_signal(|| props.month);
    sync_signal(&mut month_sig, props.month);

    let cell_attrs = ctx
        .runtime
        .cell_attrs(props.date, props.month, props.id.clone());

    use_context_provider(|| RangeCalendarCellContext {
        date: date_sig,
        month: month_sig,
    });

    if props.as_child
        && let Some(render_child) = props.render_child.as_ref()
    {
        return render_child.call(RangeCalendarCellRenderProps {
            attrs: cell_attrs,
            date: props.date,
            month: props.month,
            class: props.class,
            style: props.style,
            on_mounted: props.on_mounted,
            attributes: props.attributes,
            children: props.children,
        });
    }

    let merged = merge_global_attributes(
        props.class.as_deref(),
        props.style.as_deref(),
        props.attributes,
    );
    let on_mounted = props.on_mounted;

    rsx! {
        td {
            id: "{cell_attrs.id}",
            class: merged.class.as_deref(),
            style: merged.style.as_deref(),
            role: cell_attrs.role,
            aria_selected: cell_attrs.aria_selected,
            aria_disabled: cell_attrs.aria_disabled,
            "data-unavailable": cell_attrs.data_unavailable,
            "data-today": cell_attrs.data_today,
            "data-outside-month": cell_attrs.data_outside_month,
            "data-outside-visible-months": cell_attrs.data_outside_visible_months,
            "data-focused": cell_attrs.data_focused,
            "data-selection-start": cell_attrs.data_selection_start,
            "data-selection-end": cell_attrs.data_selection_end,
            "data-range-start": cell_attrs.data_range_start,
            "data-range-end": cell_attrs.data_range_end,
            "data-range-middle": cell_attrs.data_range_middle,
            "data-highlighted": cell_attrs.data_highlighted,
            "data-selected": cell_attrs.data_selected,
            "data-value": "{cell_attrs.data_value}",
            "data-type": cell_attrs.data_type,
            "data-disabled": cell_attrs.data_disabled,
            "data-range-calendar-cell": cell_attrs.data_range_calendar_cell,
            onmounted: move |evt| {
                if let Some(handler) = on_mounted {
                    handler.call(evt);
                }
            },
            ..merged.rest,
            {props.children}
        }
    }
}

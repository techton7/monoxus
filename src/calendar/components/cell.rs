use dioxus::prelude::*;

use crate::foundation::{
    compose::{collect_user_attrs, merge_props, push_attr_opt, push_event_handler},
    date_time::CalendarDate,
    state::sync_signal,
};

use super::super::runtime::use_calendar_context;

#[derive(Clone, Copy, PartialEq)]
pub struct CalendarCellContext {
    pub date: Signal<CalendarDate>,
    pub month: Signal<CalendarDate>,
}

pub fn use_calendar_cell_context() -> CalendarCellContext {
    use_context::<CalendarCellContext>()
}

#[derive(Clone, PartialEq, Props)]
pub struct CalendarCellProps {
    #[props(default, into)]
    pub id: Option<String>,
    pub date: CalendarDate,
    pub month: CalendarDate,
    #[props(default, into)]
    pub class: Option<String>,
    #[props(default, into)]
    pub style: Option<String>,
    #[props(default)]
    pub child: Option<Callback<Vec<Attribute>, Element>>,
    #[props(default)]
    pub on_mounted: Option<EventHandler<MountedEvent>>,
    #[props(extends = GlobalAttributes)]
    pub attributes: Vec<Attribute>,
    pub children: Element,
}

#[component]
pub fn CalendarCell(props: CalendarCellProps) -> Element {
    let CalendarCellProps {
        id,
        date,
        month,
        class,
        style,
        child,
        on_mounted,
        attributes,
        children,
    } = props;

    let ctx = use_calendar_context();
    let _ = ctx.runtime.predicate_rev();
    let mut date_sig = use_signal(|| date);
    sync_signal(&mut date_sig, date);
    let mut month_sig = use_signal(|| month);
    sync_signal(&mut month_sig, month);

    let cell_attrs = ctx.runtime.cell_attrs(date, month, id);

    use_context_provider(|| CalendarCellContext {
        date: date_sig,
        month: month_sig,
    });

    let mut base = Vec::with_capacity(14);
    base.push(Attribute::new("id", cell_attrs.id, None, false));
    base.push(Attribute::new("role", cell_attrs.role, None, false));
    base.push(Attribute::new("aria-selected", cell_attrs.aria_selected, None, false));
    base.push(Attribute::new("aria-disabled", cell_attrs.aria_disabled, None, false));
    push_attr_opt(&mut base, "data-unavailable", cell_attrs.data_unavailable);
    push_attr_opt(&mut base, "data-today", cell_attrs.data_today);
    push_attr_opt(
        &mut base,
        "data-outside-month",
        cell_attrs.data_outside_month,
    );
    push_attr_opt(
        &mut base,
        "data-outside-visible-months",
        cell_attrs.data_outside_visible_months,
    );
    push_attr_opt(&mut base, "data-focused", cell_attrs.data_focused);
    push_attr_opt(&mut base, "data-selected", cell_attrs.data_selected);
    base.push(Attribute::new("data-value", cell_attrs.data_value, None, false));
    base.push(Attribute::new("data-type", cell_attrs.data_type, None, false));
    push_attr_opt(&mut base, "data-disabled", cell_attrs.data_disabled);
    base.push(Attribute::new(
        "data-calendar-cell",
        cell_attrs.data_calendar_cell,
        None,
        false,
    ));

    let mut user = collect_user_attrs(class, style, attributes);
    push_event_handler(&mut user, "onmounted", on_mounted);

    let merged = merge_props(base, user);

    if let Some(child) = child {
        return child.call(merged);
    }

    rsx! {
        td {
            ..merged,
            {children}
        }
    }
}

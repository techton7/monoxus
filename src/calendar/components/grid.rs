use std::borrow::Cow;

use dioxus::prelude::*;

use crate::foundation::{
    compose::{collect_user_attrs, merge_props, push_attr_opt, push_event_handler},
    shared::use_stable_id,
};

use super::super::{runtime::use_calendar_shared_context, types::CalendarPart};

#[derive(Clone, PartialEq, Props)]
pub struct CalendarGridProps {
    #[props(default, into)]
    pub id: Option<String>,
    #[props(default)]
    pub month_idx: Option<usize>,
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
pub fn CalendarGrid(props: CalendarGridProps) -> Element {
    let CalendarGridProps {
        id,
        month_idx,
        class,
        style,
        child,
        on_mounted,
        attributes,
        children,
    } = props;

    let ctx = use_calendar_shared_context();
    let stable_id = use_stable_id(id.clone().map(Cow::Owned)).into_owned();
    let custom_or_stable = id.or_else(|| {
        if month_idx.is_some() {
            None
        } else {
            Some(stable_id)
        }
    });
    let attrs = ctx
        .runtime
        .grid_attrs(month_idx.unwrap_or(0), custom_or_stable);

    let mut base = Vec::with_capacity(9);
    base.push(Attribute::new("id", attrs.id, None, false));
    base.push(Attribute::new("tabindex", attrs.tabindex, None, false));
    base.push(Attribute::new("role", attrs.role, None, false));
    base.push(Attribute::new("aria-readonly", attrs.aria_readonly, None, false));
    base.push(Attribute::new("aria-disabled", attrs.aria_disabled, None, false));
    push_attr_opt(&mut base, "data-readonly", attrs.data_readonly);
    push_attr_opt(&mut base, "data-disabled", attrs.data_disabled);
    push_attr_opt(&mut base, "data-calendar-grid", attrs.data_calendar_grid);
    push_attr_opt(
        &mut base,
        "data-range-calendar-grid",
        attrs.data_range_calendar_grid,
    );

    let mut user = collect_user_attrs(class, style, attributes);
    push_event_handler(&mut user, "onmounted", on_mounted);

    let merged = merge_props(base, user);

    if let Some(child) = child {
        return child.call(merged);
    }

    rsx! {
        table {
            ..merged,
            {children}
        }
    }
}

#[derive(Clone, PartialEq, Props)]
pub struct CalendarSubpartProps {
    #[props(default, into)]
    pub id: Option<String>,
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

fn build_subpart_merged(
    part: CalendarPart,
    cal_attr_name: &'static str,
    range_attr_name: &'static str,
    id: Option<String>,
    class: Option<String>,
    style: Option<String>,
    on_mounted: Option<EventHandler<MountedEvent>>,
    attributes: Vec<Attribute>,
) -> Vec<Attribute> {
    let ctx = use_calendar_shared_context();
    let stable_id = use_stable_id(id.clone().map(Cow::Owned)).into_owned();
    let attrs = ctx.runtime.subpart_attrs(part, stable_id, id);

    let mut base = Vec::with_capacity(5);
    base.push(Attribute::new("id", attrs.id, None, false));
    push_attr_opt(&mut base, "data-disabled", attrs.data_disabled);
    push_attr_opt(&mut base, "data-readonly", attrs.data_readonly);
    push_attr_opt(&mut base, cal_attr_name, attrs.data_calendar_part);
    push_attr_opt(&mut base, range_attr_name, attrs.data_range_calendar_part);

    let mut user = collect_user_attrs(class, style, attributes);
    push_event_handler(&mut user, "onmounted", on_mounted);

    merge_props(base, user)
}

#[component]
pub fn CalendarGridHead(props: CalendarSubpartProps) -> Element {
    let merged = build_subpart_merged(
        CalendarPart::GridHead,
        "data-calendar-grid-head",
        "data-range-calendar-grid-head",
        props.id,
        props.class,
        props.style,
        props.on_mounted,
        props.attributes,
    );
    if let Some(child) = props.child {
        return child.call(merged);
    }
    rsx! {
        thead {
            ..merged,
            {props.children}
        }
    }
}

#[component]
pub fn CalendarGridBody(props: CalendarSubpartProps) -> Element {
    let merged = build_subpart_merged(
        CalendarPart::GridBody,
        "data-calendar-grid-body",
        "data-range-calendar-grid-body",
        props.id,
        props.class,
        props.style,
        props.on_mounted,
        props.attributes,
    );
    if let Some(child) = props.child {
        return child.call(merged);
    }
    rsx! {
        tbody {
            ..merged,
            {props.children}
        }
    }
}

#[component]
pub fn CalendarGridRow(props: CalendarSubpartProps) -> Element {
    let merged = build_subpart_merged(
        CalendarPart::GridRow,
        "data-calendar-grid-row",
        "data-range-calendar-grid-row",
        props.id,
        props.class,
        props.style,
        props.on_mounted,
        props.attributes,
    );
    if let Some(child) = props.child {
        return child.call(merged);
    }
    rsx! {
        tr {
            ..merged,
            {props.children}
        }
    }
}

#[component]
pub fn CalendarHeadCell(props: CalendarSubpartProps) -> Element {
    let merged = build_subpart_merged(
        CalendarPart::HeadCell,
        "data-calendar-head-cell",
        "data-range-calendar-head-cell",
        props.id,
        props.class,
        props.style,
        props.on_mounted,
        props.attributes,
    );
    if let Some(child) = props.child {
        return child.call(merged);
    }
    rsx! {
        th {
            ..merged,
            {props.children}
        }
    }
}

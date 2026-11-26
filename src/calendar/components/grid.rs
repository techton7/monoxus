use std::borrow::Cow;

use dioxus::prelude::*;

use crate::foundation::{compose::merge_global_attributes, shared::use_stable_id};

use super::super::{runtime::use_calendar_shared_context, types::CalendarPart};

#[derive(Clone, PartialEq, Props)]
pub struct CalendarGridProps {
    #[props(default)]
    pub id: Option<String>,
    #[props(default)]
    pub month_idx: Option<usize>,
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

#[component]
pub fn CalendarGrid(props: CalendarGridProps) -> Element {
    let ctx = use_calendar_shared_context();
    let stable_id = use_stable_id(props.id.clone().map(Cow::Owned)).into_owned();
    let custom_or_stable = props.id.clone().or_else(|| {
        if props.month_idx.is_some() {
            None
        } else {
            Some(stable_id)
        }
    });
    let attrs = ctx
        .runtime
        .grid_attrs(props.month_idx.unwrap_or(0), custom_or_stable);
    let merged = merge_global_attributes(
        props.class.as_deref(),
        props.style.as_deref(),
        props.attributes,
    );
    let on_mounted = props.on_mounted;

    rsx! {
        table {
            id: "{attrs.id}",
            class: merged.class.as_deref(),
            style: merged.style.as_deref(),
            tabindex: "{attrs.tabindex}",
            role: attrs.role,
            "aria-readonly": attrs.aria_readonly,
            aria_disabled: attrs.aria_disabled,
            "data-readonly": attrs.data_readonly,
            "data-disabled": attrs.data_disabled,
            "data-calendar-grid": attrs.data_calendar_grid,
            "data-range-calendar-grid": attrs.data_range_calendar_grid,
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

#[derive(Clone, PartialEq, Props)]
pub struct CalendarSubpartProps {
    #[props(default)]
    pub id: Option<String>,
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

#[component]
pub fn CalendarGridHead(props: CalendarSubpartProps) -> Element {
    let ctx = use_calendar_shared_context();
    let stable_id = use_stable_id(props.id.clone().map(Cow::Owned)).into_owned();
    let attrs = ctx
        .runtime
        .subpart_attrs(CalendarPart::GridHead, stable_id, props.id);
    let merged = merge_global_attributes(
        props.class.as_deref(),
        props.style.as_deref(),
        props.attributes,
    );
    let on_mounted = props.on_mounted;

    rsx! {
        thead {
            id: "{attrs.id}",
            class: merged.class.as_deref(),
            style: merged.style.as_deref(),
            "data-disabled": attrs.data_disabled,
            "data-readonly": attrs.data_readonly,
            "data-calendar-grid-head": attrs.data_calendar_part,
            "data-range-calendar-grid-head": attrs.data_range_calendar_part,
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

#[component]
pub fn CalendarGridBody(props: CalendarSubpartProps) -> Element {
    let ctx = use_calendar_shared_context();
    let stable_id = use_stable_id(props.id.clone().map(Cow::Owned)).into_owned();
    let attrs = ctx
        .runtime
        .subpart_attrs(CalendarPart::GridBody, stable_id, props.id);
    let merged = merge_global_attributes(
        props.class.as_deref(),
        props.style.as_deref(),
        props.attributes,
    );
    let on_mounted = props.on_mounted;

    rsx! {
        tbody {
            id: "{attrs.id}",
            class: merged.class.as_deref(),
            style: merged.style.as_deref(),
            "data-disabled": attrs.data_disabled,
            "data-readonly": attrs.data_readonly,
            "data-calendar-grid-body": attrs.data_calendar_part,
            "data-range-calendar-grid-body": attrs.data_range_calendar_part,
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

#[component]
pub fn CalendarGridRow(props: CalendarSubpartProps) -> Element {
    let ctx = use_calendar_shared_context();
    let stable_id = use_stable_id(props.id.clone().map(Cow::Owned)).into_owned();
    let attrs = ctx
        .runtime
        .subpart_attrs(CalendarPart::GridRow, stable_id, props.id);
    let merged = merge_global_attributes(
        props.class.as_deref(),
        props.style.as_deref(),
        props.attributes,
    );
    let on_mounted = props.on_mounted;

    rsx! {
        tr {
            id: "{attrs.id}",
            class: merged.class.as_deref(),
            style: merged.style.as_deref(),
            "data-disabled": attrs.data_disabled,
            "data-readonly": attrs.data_readonly,
            "data-calendar-grid-row": attrs.data_calendar_part,
            "data-range-calendar-grid-row": attrs.data_range_calendar_part,
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

#[component]
pub fn CalendarHeadCell(props: CalendarSubpartProps) -> Element {
    let ctx = use_calendar_shared_context();
    let stable_id = use_stable_id(props.id.clone().map(Cow::Owned)).into_owned();
    let attrs = ctx
        .runtime
        .subpart_attrs(CalendarPart::HeadCell, stable_id, props.id);
    let merged = merge_global_attributes(
        props.class.as_deref(),
        props.style.as_deref(),
        props.attributes,
    );
    let on_mounted = props.on_mounted;

    rsx! {
        th {
            id: "{attrs.id}",
            class: merged.class.as_deref(),
            style: merged.style.as_deref(),
            "data-disabled": attrs.data_disabled,
            "data-readonly": attrs.data_readonly,
            "data-calendar-head-cell": attrs.data_calendar_part,
            "data-range-calendar-head-cell": attrs.data_range_calendar_part,
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

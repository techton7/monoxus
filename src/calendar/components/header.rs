use dioxus::prelude::*;

use super::super::{attrs::CalendarHeaderAttrs, runtime::use_calendar_shared_context};

#[derive(Clone, PartialEq, Props)]
pub struct CalendarHeaderRenderProps {
    pub attrs: CalendarHeaderAttrs,
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
pub struct CalendarHeaderProps {
    #[props(default)]
    pub id: Option<String>,
    #[props(default)]
    pub class: Option<String>,
    #[props(default)]
    pub style: Option<String>,
    #[props(default = false)]
    pub as_child: bool,
    #[props(default)]
    pub render_child: Option<Callback<CalendarHeaderRenderProps, Element>>,
    #[props(default)]
    pub on_mounted: Option<EventHandler<MountedEvent>>,
    #[props(extends = GlobalAttributes)]
    pub attributes: Vec<Attribute>,
    pub children: Element,
}

#[component]
pub fn CalendarHeader(props: CalendarHeaderProps) -> Element {
    let ctx = use_calendar_shared_context();
    let attrs = ctx.runtime.header_attrs(props.id.clone());

    if props.as_child {
        if let Some(render_child) = props.render_child.as_ref() {
            return render_child.call(CalendarHeaderRenderProps {
                attrs,
                class: props.class,
                style: props.style,
                on_mounted: props.on_mounted,
                attributes: props.attributes,
                children: props.children,
            });
        }
    }

    rsx! {
        header {
            id: "{attrs.id}",
            class: props.class,
            style: props.style,
            "data-calendar-header": attrs.data_calendar_header,
            "data-range-calendar-header": attrs.data_range_calendar_header,
            "data-disabled": attrs.data_disabled,
            "data-readonly": attrs.data_readonly,
            onmounted: move |evt| {
                if let Some(ref handler) = props.on_mounted {
                    handler.call(evt);
                }
            },
            ..props.attributes,
            {props.children}
        }
    }
}

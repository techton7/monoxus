use dioxus::prelude::*;

use super::super::{attrs::CalendarHeadingAttrs, runtime::use_calendar_shared_context};

#[derive(Clone, PartialEq, Props)]
pub struct CalendarHeadingRenderProps {
    pub attrs: CalendarHeadingAttrs,
    pub heading_value: String,
    #[props(default)]
    pub class: Option<String>,
    #[props(default)]
    pub style: Option<String>,
    #[props(default)]
    pub on_mounted: Option<EventHandler<MountedEvent>>,
    #[props(extends = GlobalAttributes)]
    pub attributes: Vec<Attribute>,
    #[props(default)]
    pub children: Option<Element>,
}

#[derive(Clone, PartialEq, Props)]
pub struct CalendarHeadingProps {
    #[props(default)]
    pub id: Option<String>,
    #[props(default)]
    pub class: Option<String>,
    #[props(default)]
    pub style: Option<String>,
    #[props(default = false)]
    pub as_child: bool,
    #[props(default)]
    pub render_child: Option<Callback<CalendarHeadingRenderProps, Element>>,
    #[props(default)]
    pub on_mounted: Option<EventHandler<MountedEvent>>,
    #[props(extends = GlobalAttributes)]
    pub attributes: Vec<Attribute>,
    #[props(default)]
    pub children: Option<Element>,
}

#[component]
pub fn CalendarHeading(props: CalendarHeadingProps) -> Element {
    let ctx = use_calendar_shared_context();
    let attrs = ctx.runtime.heading_attrs(props.id.clone());
    let heading_value = ctx.runtime.heading_value();

    if props.as_child {
        if let Some(render_child) = props.render_child.as_ref() {
            return render_child.call(CalendarHeadingRenderProps {
                attrs,
                heading_value,
                class: props.class,
                style: props.style,
                on_mounted: props.on_mounted,
                attributes: props.attributes,
                children: props.children,
            });
        }
    }

    rsx! {
        div {
            id: "{attrs.id}",
            class: props.class,
            style: props.style,
            "aria-hidden": attrs.aria_hidden,
            "data-calendar-heading": attrs.data_calendar_heading,
            "data-range-calendar-heading": attrs.data_range_calendar_heading,
            "data-disabled": attrs.data_disabled,
            "data-readonly": attrs.data_readonly,
            onmounted: move |evt| {
                if let Some(ref handler) = props.on_mounted {
                    handler.call(evt);
                }
            },
            ..props.attributes,
            if let Some(children) = props.children {
                {children}
            } else {
                "{heading_value}"
            }
        }
    }
}

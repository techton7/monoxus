use dioxus::prelude::*;

use crate::foundation::compose::{
    collect_user_attrs, merge_props, push_attr_opt, push_event_handler,
};

use super::super::runtime::use_calendar_shared_context;

#[derive(Clone, PartialEq, Props)]
pub struct CalendarHeadingProps {
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
    #[props(default)]
    pub children: Option<Element>,
}

#[component]
pub fn CalendarHeading(props: CalendarHeadingProps) -> Element {
    let CalendarHeadingProps {
        id,
        class,
        style,
        child,
        on_mounted,
        attributes,
        children,
    } = props;

    let ctx = use_calendar_shared_context();
    let attrs = ctx.runtime.heading_attrs(id);
    let heading_value = ctx.runtime.heading_value();

    let mut base = Vec::with_capacity(6);
    base.push(Attribute::new("id", attrs.id, None, false));
    base.push(Attribute::new("aria-hidden", attrs.aria_hidden, None, false));
    push_attr_opt(&mut base, "data-calendar-heading", attrs.data_calendar_heading);
    push_attr_opt(
        &mut base,
        "data-range-calendar-heading",
        attrs.data_range_calendar_heading,
    );
    push_attr_opt(&mut base, "data-disabled", attrs.data_disabled);
    push_attr_opt(&mut base, "data-readonly", attrs.data_readonly);

    let mut user = collect_user_attrs(class, style, attributes);
    push_event_handler(&mut user, "onmounted", on_mounted);

    let merged = merge_props(base, user);

    if let Some(child) = child {
        return child.call(merged);
    }

    rsx! {
        div {
            ..merged,
            if let Some(children) = children {
                {children}
            } else {
                "{heading_value}"
            }
        }
    }
}

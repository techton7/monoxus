use dioxus::prelude::*;

use crate::foundation::compose::{
    collect_user_attrs, merge_props, push_attr_opt, push_event_handler,
};

use super::super::runtime::use_calendar_shared_context;

#[derive(Clone, PartialEq, Props)]
pub struct CalendarHeaderProps {
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

#[component]
pub fn CalendarHeader(props: CalendarHeaderProps) -> Element {
    let CalendarHeaderProps {
        id,
        class,
        style,
        child,
        on_mounted,
        attributes,
        children,
    } = props;

    let ctx = use_calendar_shared_context();
    let attrs = ctx.runtime.header_attrs(id);

    let mut base = Vec::with_capacity(5);
    base.push(Attribute::new("id", attrs.id, None, false));
    push_attr_opt(&mut base, "data-calendar-header", attrs.data_calendar_header);
    push_attr_opt(
        &mut base,
        "data-range-calendar-header",
        attrs.data_range_calendar_header,
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
        header {
            ..merged,
            {children}
        }
    }
}

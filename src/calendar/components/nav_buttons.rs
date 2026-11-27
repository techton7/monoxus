use dioxus::prelude::*;

use crate::foundation::compose::{
    collect_user_attrs, merge_props, push_attr_opt, push_event_handler,
};

use super::super::runtime::use_calendar_shared_context;

#[derive(Clone, PartialEq, Props)]
pub struct CalendarNavButtonProps {
    #[props(default, into)]
    pub id: Option<String>,
    #[props(default, into)]
    pub class: Option<String>,
    #[props(default, into)]
    pub style: Option<String>,
    #[props(default)]
    pub child: Option<Callback<Vec<Attribute>, Element>>,
    #[props(default)]
    pub on_click: Option<EventHandler<MouseEvent>>,
    #[props(default)]
    pub on_mounted: Option<EventHandler<MountedEvent>>,
    #[props(extends = GlobalAttributes)]
    pub attributes: Vec<Attribute>,
    #[props(default)]
    pub children: Option<Element>,
}

fn render_calendar_nav_button(props: CalendarNavButtonProps, sign: i32) -> Element {
    let CalendarNavButtonProps {
        id,
        class,
        style,
        child,
        on_click,
        on_mounted,
        attributes,
        children,
    } = props;

    let ctx = use_calendar_shared_context();
    let attrs = if sign < 0 {
        ctx.runtime.prev_button_attrs(id)
    } else {
        ctx.runtime.next_button_attrs(id)
    };
    let is_disabled = attrs.disabled;

    let mut runtime = ctx.runtime.clone();
    let default_label = if sign < 0 { "Previous" } else { "Next" };

    let mut base = Vec::with_capacity(12);
    base.push(Attribute::new("id", attrs.id, None, false));
    base.push(Attribute::new("type", attrs.r#type, None, false));
    base.push(Attribute::new("role", attrs.role, None, false));
    base.push(Attribute::new("aria-label", attrs.aria_label, None, false));
    base.push(Attribute::new("aria-disabled", attrs.aria_disabled, None, false));
    if is_disabled {
        base.push(Attribute::new("disabled", true, None, false));
    }
    push_attr_opt(&mut base, "data-disabled", attrs.data_disabled);
    push_attr_opt(
        &mut base,
        "data-calendar-prev-button",
        attrs.data_calendar_prev_button,
    );
    push_attr_opt(
        &mut base,
        "data-calendar-next-button",
        attrs.data_calendar_next_button,
    );
    push_attr_opt(
        &mut base,
        "data-range-calendar-prev-button",
        attrs.data_range_calendar_prev_button,
    );
    push_attr_opt(
        &mut base,
        "data-range-calendar-next-button",
        attrs.data_range_calendar_next_button,
    );
    push_event_handler(
        &mut base,
        "onclick",
        Some(EventHandler::new(move |evt: MouseEvent| {
            if is_disabled {
                evt.prevent_default();
                return;
            }
            if evt.default_action_enabled() {
                runtime.page_by(sign);
            }
        })),
    );

    let mut user = collect_user_attrs(class, style, attributes);
    if !is_disabled {
        push_event_handler(&mut user, "onclick", on_click);
    }
    push_event_handler(&mut user, "onmounted", on_mounted);

    let merged = merge_props(base, user);

    if let Some(child) = child {
        return child.call(merged);
    }

    rsx! {
        button {
            ..merged,
            if let Some(children) = children {
                {children}
            } else {
                "{default_label}"
            }
        }
    }
}

#[component]
pub fn CalendarPrevButton(props: CalendarNavButtonProps) -> Element {
    render_calendar_nav_button(props, -1)
}

#[component]
pub fn CalendarNextButton(props: CalendarNavButtonProps) -> Element {
    render_calendar_nav_button(props, 1)
}

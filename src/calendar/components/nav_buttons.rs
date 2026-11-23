use dioxus::prelude::*;

use super::super::{attrs::CalendarNavButtonAttrs, runtime::use_calendar_shared_context};

#[derive(Clone, PartialEq, Props)]
pub struct CalendarNavButtonRenderProps {
    pub attrs: CalendarNavButtonAttrs,
    #[props(default)]
    pub class: Option<String>,
    #[props(default)]
    pub style: Option<String>,
    pub on_click: EventHandler<MouseEvent>,
    #[props(default)]
    pub on_mounted: Option<EventHandler<MountedEvent>>,
    #[props(extends = GlobalAttributes)]
    pub attributes: Vec<Attribute>,
    #[props(default)]
    pub children: Option<Element>,
}

#[derive(Clone, PartialEq, Props)]
pub struct CalendarNavButtonProps {
    #[props(default)]
    pub id: Option<String>,
    #[props(default)]
    pub class: Option<String>,
    #[props(default)]
    pub style: Option<String>,
    #[props(default = false)]
    pub as_child: bool,
    #[props(default)]
    pub render_child: Option<Callback<CalendarNavButtonRenderProps, Element>>,
    #[props(default)]
    pub on_click: Option<EventHandler<MouseEvent>>,
    #[props(default)]
    pub on_mounted: Option<EventHandler<MountedEvent>>,
    #[props(extends = GlobalAttributes)]
    pub attributes: Vec<Attribute>,
    #[props(default)]
    pub children: Option<Element>,
}

#[component]
pub fn CalendarPrevButton(props: CalendarNavButtonProps) -> Element {
    let ctx = use_calendar_shared_context();
    let attrs = ctx.runtime.prev_button_attrs(props.id.clone());
    let is_disabled = attrs.disabled;

    let mut runtime = ctx.runtime.clone();
    let consumer_on_click = props.on_click;
    let on_click = Callback::new(move |evt: MouseEvent| {
        if is_disabled {
            evt.prevent_default();
            return;
        }
        if let Some(ref handler) = consumer_on_click {
            handler.call(evt.clone());
        }
        if evt.default_action_enabled() {
            runtime.prev_page();
        }
    });

    if props.as_child {
        if let Some(render_child) = props.render_child.as_ref() {
            return render_child.call(CalendarNavButtonRenderProps {
                attrs,
                class: props.class,
                style: props.style,
                on_click,
                on_mounted: props.on_mounted,
                attributes: props.attributes,
                children: props.children,
            });
        }
    }

    rsx! {
        button {
            id: "{attrs.id}",
            class: props.class,
            style: props.style,
            r#type: attrs.r#type,
            role: attrs.role,
            aria_label: attrs.aria_label,
            aria_disabled: attrs.aria_disabled,
            disabled: attrs.disabled,
            "data-disabled": attrs.data_disabled,
            "data-calendar-prev-button": attrs.data_calendar_prev_button,
            "data-range-calendar-prev-button": attrs.data_range_calendar_prev_button,
            onclick: move |evt| on_click.call(evt),
            onmounted: move |evt| {
                if let Some(ref handler) = props.on_mounted {
                    handler.call(evt);
                }
            },
            ..props.attributes,
            if let Some(children) = props.children {
                {children}
            } else {
                "Previous"
            }
        }
    }
}

#[component]
pub fn CalendarNextButton(props: CalendarNavButtonProps) -> Element {
    let ctx = use_calendar_shared_context();
    let attrs = ctx.runtime.next_button_attrs(props.id.clone());
    let is_disabled = attrs.disabled;

    let mut runtime = ctx.runtime.clone();
    let consumer_on_click = props.on_click;
    let on_click = Callback::new(move |evt: MouseEvent| {
        if is_disabled {
            evt.prevent_default();
            return;
        }
        if let Some(ref handler) = consumer_on_click {
            handler.call(evt.clone());
        }
        if evt.default_action_enabled() {
            runtime.next_page();
        }
    });

    if props.as_child {
        if let Some(render_child) = props.render_child.as_ref() {
            return render_child.call(CalendarNavButtonRenderProps {
                attrs,
                class: props.class,
                style: props.style,
                on_click,
                on_mounted: props.on_mounted,
                attributes: props.attributes,
                children: props.children,
            });
        }
    }

    rsx! {
        button {
            id: "{attrs.id}",
            class: props.class,
            style: props.style,
            r#type: attrs.r#type,
            role: attrs.role,
            aria_label: attrs.aria_label,
            aria_disabled: attrs.aria_disabled,
            disabled: attrs.disabled,
            "data-disabled": attrs.data_disabled,
            "data-calendar-next-button": attrs.data_calendar_next_button,
            "data-range-calendar-next-button": attrs.data_range_calendar_next_button,
            onclick: move |evt| on_click.call(evt),
            onmounted: move |evt| {
                if let Some(ref handler) = props.on_mounted {
                    handler.call(evt);
                }
            },
            ..props.attributes,
            if let Some(children) = props.children {
                {children}
            } else {
                "Next"
            }
        }
    }
}

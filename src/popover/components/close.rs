use dioxus::prelude::*;

use crate::foundation::{
    compose::{
        collect_user_attrs, merge_props, push_bool_data_attr, push_event_handler,
    },
    shared::use_stable_id,
};
use crate::popover::components::PopoverContext;

#[derive(Props, Clone, PartialEq)]
pub struct PopoverCloseProps {
    #[props(default, into)]
    pub id: Option<String>,
    #[props(default, into)]
    pub class: Option<String>,
    #[props(default, into)]
    pub style: Option<String>,
    #[props(default = false)]
    pub disabled: bool,
    #[props(default)]
    pub on_click: Option<EventHandler<MouseEvent>>,
    #[props(default)]
    pub on_key_down: Option<EventHandler<KeyboardEvent>>,
    #[props(default)]
    pub on_mounted: Option<EventHandler<MountedEvent>>,
    #[props(default)]
    pub child: Option<Callback<Vec<Attribute>, Element>>,
    #[props(extends = GlobalAttributes)]
    pub attributes: Vec<Attribute>,
    #[props(default)]
    pub children: Element,
}

#[component]
pub fn PopoverClose(props: PopoverCloseProps) -> Element {
    let PopoverCloseProps {
        id,
        class,
        style,
        disabled,
        on_click,
        on_key_down,
        on_mounted,
        child,
        attributes,
        children,
    } = props;
    let ctx = use_context::<PopoverContext>();
    let runtime = ctx.runtime.read().clone();
    let close = runtime.close();
    let stable_fallback = use_stable_id(None);
    let close_id = id.unwrap_or_else(|| format!("{stable_fallback}-close"));
    let data_state = close.data_state().as_str();

    let mut mount_cb = runtime.mount_close();
    let mut pd_cb = runtime.close_pointer_down();
    let mut click_cb = runtime.close_click();

    let mut base = Vec::with_capacity(8);
    base.push(Attribute::new("id", close_id, None, false));
    base.push(Attribute::new("type", "button", None, false));
    if disabled {
        base.push(Attribute::new("disabled", true, None, false));
    }
    base.push(Attribute::new("data-state", data_state, None, false));
    push_bool_data_attr(&mut base, "data-disabled", disabled);
    push_event_handler(
        &mut base,
        "onmounted",
        Some(EventHandler::new(move |evt: MountedEvent| mount_cb(evt))),
    );
    push_event_handler(
        &mut base,
        "onpointerdown",
        Some(EventHandler::new(move |evt: PointerEvent| {
            if !disabled {
                pd_cb(evt);
            }
        })),
    );
    push_event_handler(
        &mut base,
        "onclick",
        Some(EventHandler::new(move |evt: MouseEvent| {
            if disabled {
                evt.prevent_default();
                return;
            }
            if evt.default_action_enabled() {
                click_cb(evt);
            }
        })),
    );

    let mut user = collect_user_attrs(class, style, attributes);
    push_event_handler(&mut user, "onmounted", on_mounted);
    if !disabled {
        push_event_handler(&mut user, "onclick", on_click);
    }
    push_event_handler(&mut user, "onkeydown", on_key_down);

    let merged = merge_props(base, user);

    if let Some(child) = child {
        return child.call(merged);
    }

    rsx! {
        button {
            ..merged,
            {children}
        }
    }
}

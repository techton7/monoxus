use std::{
    cell::{Cell, RefCell},
    rc::Rc,
};

use dioxus::core::{AttributeValue, Event, ListenerCallback};
use dioxus::html::PlatformEventData;
use dioxus::prelude::*;

use crate::foundation::compose::{
    collect_user_attrs, merge_props, push_attr_opt, push_event_handler,
};
use crate::hover_card::components::HoverCardRootContext;

#[derive(Clone, PartialEq, Props)]
pub struct HoverCardTriggerProps {
    #[props(default, into)]
    pub id: Option<String>,
    #[props(default, into)]
    pub class: Option<String>,
    #[props(default, into)]
    pub style: Option<String>,
    #[props(default, into)]
    pub href: Option<String>,
    #[props(default, into)]
    pub target: Option<String>,
    #[props(default, into)]
    pub rel: Option<String>,
    #[props(default = false)]
    pub disabled: bool,
    #[props(default)]
    pub on_click: Option<EventHandler<MouseEvent>>,
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
pub fn HoverCardTrigger(props: HoverCardTriggerProps) -> Element {
    let ctx = use_context::<HoverCardRootContext>();
    let runtime = ctx.runtime.read();
    let trigger = runtime.trigger(
        props.href.clone(),
        props.target.clone(),
        props.rel.clone(),
        props.disabled,
    );
    let trigger_id = props.id.clone().unwrap_or_else(|| trigger.id().to_string());

    let registered_id_cell = use_hook(|| Rc::new(RefCell::new(None::<String>)));
    if let Some(ref custom_id) = props.id {
        if registered_id_cell.borrow().as_deref() != Some(custom_id.as_str()) {
            if let Some(prev) = registered_id_cell.borrow().as_deref() {
                runtime.unregister_trigger_id(prev);
            }
            runtime.register_trigger_id(custom_id.clone());
            *registered_id_cell.borrow_mut() = Some(custom_id.clone());
        }
    } else if let Some(prev) = registered_id_cell.borrow_mut().take() {
        runtime.unregister_trigger_id(&prev);
    }

    let cleanup_id_cell = Rc::clone(&registered_id_cell);
    let cleanup_runtime = runtime.clone();
    use_drop(move || {
        if let Some(id) = cleanup_id_cell.borrow_mut().take() {
            cleanup_runtime.unregister_trigger_id(&id);
        }
    });

    let mounted_once = use_hook(|| Rc::new(Cell::new(false)));
    let runtime_sig = ctx.runtime;
    use_effect(use_reactive((&trigger_id,), move |_| {
        if mounted_once.get() {
            runtime_sig.peek().resync_positioning_if_open();
        } else {
            mounted_once.set(true);
        }
    }));

    let mut on_mount_runtime = runtime.mount_trigger();
    let mut on_touch_start = runtime.trigger_touch_start();
    let mut on_pointer_down = runtime.trigger_pointer_down();
    let mut on_pointer_enter = runtime.trigger_pointer_enter(props.disabled);
    let mut on_pointer_leave = runtime.trigger_pointer_leave(props.disabled);
    let mut on_focus = runtime.trigger_focus(props.disabled);
    let mut on_blur = runtime.trigger_blur();
    let mut on_key_down = runtime.escape_keydown();
    let effective_disabled = trigger.is_disabled();

    let href = trigger.href().map(str::to_owned);
    let target = trigger.target().map(str::to_owned);
    let rel = trigger.rel().map(str::to_owned);
    let tabindex = trigger.tabindex().map(|t| t.to_string());
    let aria_disabled = trigger.aria_disabled();
    let data_state = trigger.data_state().as_str();
    let data_disabled = trigger.data_disabled();
    let part_attr = trigger.data_monoxus_hover_card_trigger();

    let mut base = Vec::with_capacity(18);
    base.push(Attribute::new("id", trigger_id, None, false));
    push_attr_opt(&mut base, "href", href);
    push_attr_opt(&mut base, "target", target);
    push_attr_opt(&mut base, "rel", rel);
    push_attr_opt(&mut base, "tabindex", tabindex);
    push_attr_opt(&mut base, "aria-disabled", aria_disabled);
    base.push(Attribute::new("data-state", data_state, None, false));
    push_attr_opt(&mut base, "data-disabled", data_disabled);
    base.push(Attribute::new(
        "data-monoxus-hover-card-trigger",
        part_attr,
        None,
        false,
    ));
    base.push(Attribute::new(
        "onmounted",
        AttributeValue::Listener(
            ListenerCallback::<PlatformEventData>::new(move |evt: Event<PlatformEventData>| {
                on_mount_runtime(evt.map(|d| MountedData::from(d)));
            })
            .erase(),
        ),
        None,
        false,
    ));
    base.push(Attribute::new(
        "ontouchstart",
        AttributeValue::Listener(
            ListenerCallback::<PlatformEventData>::new(move |evt: Event<PlatformEventData>| {
                on_touch_start(evt.map(|d| TouchData::from(d)));
            })
            .erase(),
        ),
        None,
        false,
    ));
    base.push(Attribute::new(
        "onpointerdown",
        AttributeValue::Listener(
            ListenerCallback::<PlatformEventData>::new(move |evt: Event<PlatformEventData>| {
                on_pointer_down(evt.map(|d| PointerData::from(d)));
            })
            .erase(),
        ),
        None,
        false,
    ));
    base.push(Attribute::new(
        "onpointerenter",
        AttributeValue::Listener(
            ListenerCallback::<PlatformEventData>::new(move |evt: Event<PlatformEventData>| {
                on_pointer_enter(evt.map(|d| PointerData::from(d)));
            })
            .erase(),
        ),
        None,
        false,
    ));
    base.push(Attribute::new(
        "onpointerleave",
        AttributeValue::Listener(
            ListenerCallback::<PlatformEventData>::new(move |evt: Event<PlatformEventData>| {
                on_pointer_leave(evt.map(|d| PointerData::from(d)));
            })
            .erase(),
        ),
        None,
        false,
    ));
    base.push(Attribute::new(
        "onfocus",
        AttributeValue::Listener(
            ListenerCallback::<PlatformEventData>::new(move |evt: Event<PlatformEventData>| {
                on_focus(evt.map(|d| FocusData::from(d)));
            })
            .erase(),
        ),
        None,
        false,
    ));
    base.push(Attribute::new(
        "onblur",
        AttributeValue::Listener(
            ListenerCallback::<PlatformEventData>::new(move |evt: Event<PlatformEventData>| {
                on_blur(evt.map(|d| FocusData::from(d)));
            })
            .erase(),
        ),
        None,
        false,
    ));
    base.push(Attribute::new(
        "onkeydown",
        AttributeValue::Listener(
            ListenerCallback::<PlatformEventData>::new(move |evt: Event<PlatformEventData>| {
                on_key_down(evt.map(|d| KeyboardData::from(d)));
            })
            .erase(),
        ),
        None,
        false,
    ));
    if effective_disabled {
        base.push(Attribute::new(
            "onclick",
            AttributeValue::Listener(
                ListenerCallback::<PlatformEventData>::new(
                    move |evt: Event<PlatformEventData>| {
                        evt.prevent_default();
                    },
                )
                .erase(),
            ),
            None,
            false,
        ));
    }

    let mut user = collect_user_attrs(props.class, props.style, props.attributes);
    push_event_handler(&mut user, "onmounted", props.on_mounted);
    if !effective_disabled {
        push_event_handler(&mut user, "onclick", props.on_click);
    }

    let merged = merge_props(base, user);

    if let Some(child) = props.child {
        return child.call(merged);
    }

    rsx! {
        a {
            ..merged,
            {props.children}
        }
    }
}

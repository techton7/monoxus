use std::{
    cell::{Cell, RefCell},
    rc::Rc,
};

use dioxus::core::{AttributeValue, Event, ListenerCallback};
use dioxus::html::PlatformEventData;
use dioxus::prelude::*;

use crate::foundation::compose::{
    collect_user_attrs, merge_props, push_attr_opt, push_bool_data_attr,
};
use crate::tooltip::{components::TooltipRootContext, runtime::FloatingTooltipRootContext};

#[derive(Props, Clone, PartialEq)]
pub struct TooltipTriggerProps {
    #[props(default, into)]
    pub id: Option<String>,
    #[props(default, into)]
    pub class: Option<String>,
    #[props(default, into)]
    pub style: Option<String>,
    #[props(default = false)]
    pub disabled: bool,
    #[props(default)]
    pub child: Option<Callback<Vec<Attribute>, Element>>,
    #[props(extends = GlobalAttributes)]
    pub attributes: Vec<Attribute>,
    #[props(default)]
    pub children: Element,
}

#[component]
pub fn TooltipTrigger(props: TooltipTriggerProps) -> Element {
    let ctx = use_context::<TooltipRootContext>();
    let floating_ctx = use_context::<FloatingTooltipRootContext>();
    let runtime = ctx.runtime.read();
    let trigger = runtime.trigger();
    let trigger_id = props.id.clone().unwrap_or_else(|| trigger.id().to_string());

    let registered_id_cell = use_hook(|| Rc::new(RefCell::new(None::<String>)));
    if let Some(ref custom_id) = props.id {
        if registered_id_cell.borrow().as_deref() != Some(custom_id.as_str()) {
            if let Some(prev) = registered_id_cell.borrow().as_deref() {
                floating_ctx.unregister_trigger_id(prev);
            }
            floating_ctx.register_trigger_id(custom_id.clone());
            *registered_id_cell.borrow_mut() = Some(custom_id.clone());
        }
    } else if let Some(prev) = registered_id_cell.borrow_mut().take() {
        floating_ctx.unregister_trigger_id(&prev);
    }

    let cleanup_id_cell = Rc::clone(&registered_id_cell);
    use_drop(move || {
        if let Some(id) = cleanup_id_cell.borrow_mut().take() {
            floating_ctx.unregister_trigger_id(&id);
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

    let mut on_mount = runtime.mount_trigger();
    let mut on_pointer_down = runtime.trigger_pointer_down();
    let mut on_pointer_cancel = runtime.trigger_pointer_cancel();
    let mut on_mouse_enter = runtime.trigger_pointer_enter();
    let mut on_mouse_leave = runtime.trigger_pointer_leave();
    let mut on_focus = runtime.trigger_focus();
    let mut on_blur = runtime.trigger_blur();
    let mut on_click = runtime.trigger_click();
    let mut on_key_down = runtime.escape_keydown();

    let aria_describedby = trigger.aria_describedby().map(str::to_owned);
    let data_state = trigger.data_state().as_str();

    let mut base = Vec::with_capacity(15);
    base.push(Attribute::new("id", trigger_id, None, false));
    base.push(Attribute::new("type", "button", None, false));
    if props.disabled {
        base.push(Attribute::new("disabled", true, None, false));
    }
    push_attr_opt(&mut base, "aria-describedby", aria_describedby);
    base.push(Attribute::new("data-state", data_state, None, false));
    push_bool_data_attr(&mut base, "data-disabled", props.disabled);
    base.push(Attribute::new(
        "onmounted",
        AttributeValue::Listener(
            ListenerCallback::<PlatformEventData>::new(move |evt: Event<PlatformEventData>| {
                on_mount(evt.map(|d| MountedData::from(d)));
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
        "onpointercancel",
        AttributeValue::Listener(
            ListenerCallback::<PlatformEventData>::new(move |evt: Event<PlatformEventData>| {
                on_pointer_cancel(evt.map(|d| PointerData::from(d)));
            })
            .erase(),
        ),
        None,
        false,
    ));
    base.push(Attribute::new(
        "onmouseenter",
        AttributeValue::Listener(
            ListenerCallback::<PlatformEventData>::new(move |evt: Event<PlatformEventData>| {
                on_mouse_enter(evt.map(|d| MouseData::from(d)));
            })
            .erase(),
        ),
        None,
        false,
    ));
    base.push(Attribute::new(
        "onmouseleave",
        AttributeValue::Listener(
            ListenerCallback::<PlatformEventData>::new(move |evt: Event<PlatformEventData>| {
                on_mouse_leave(evt.map(|d| MouseData::from(d)));
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
        "onclick",
        AttributeValue::Listener(
            ListenerCallback::<PlatformEventData>::new(move |evt: Event<PlatformEventData>| {
                on_click(evt.map(|d| MouseData::from(d)));
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

    let user = collect_user_attrs(props.class, props.style, props.attributes);
    let merged = merge_props(base, user);

    if let Some(child) = props.child {
        return child.call(merged);
    }

    rsx! {
        button {
            ..merged,
            {props.children}
        }
    }
}

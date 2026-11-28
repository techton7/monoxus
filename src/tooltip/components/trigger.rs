use std::{
    cell::{Cell, RefCell},
    rc::Rc,
};

use dioxus::core::{AttributeValue, Event, ListenerCallback};
use dioxus::html::PlatformEventData;
use dioxus::prelude::*;

use crate::foundation::{
    compose::{collect_user_attrs, merge_props, push_attr_opt, push_bool_data_attr},
    keyboard::is_escape_key,
    overlay::{
        AnyTooltipHandle, OverlayPayload, TooltipHandleProp, use_overlay_handle_version,
    },
    shared::use_stable_id,
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
    #[props(default, into)]
    pub handle: TooltipHandleProp,
    #[props(default)]
    pub payload: Option<OverlayPayload>,
    #[props(default)]
    pub child: Option<Callback<Vec<Attribute>, Element>>,
    #[props(extends = GlobalAttributes)]
    pub attributes: Vec<Attribute>,
    #[props(default)]
    pub children: Element,
}

#[component]
pub fn TooltipTrigger(props: TooltipTriggerProps) -> Element {
    let _handle_version = use_overlay_handle_version(props.handle.as_any());
    let stable_id = use_stable_id(None);
    let ctx = try_consume_context::<TooltipRootContext>();
    let floating_ctx = try_consume_context::<FloatingTooltipRootContext>();
    let runtime = ctx.as_ref().map(|c| c.runtime.read().clone());
    let inline_trigger = runtime.as_ref().map(|rt| rt.trigger());

    let trigger_id = if let Some(ref custom_id) = props.id {
        custom_id.clone()
    } else if props.handle.is_none()
        && let Some(ref trigger) = inline_trigger
    {
        trigger.id().to_string()
    } else {
        format!("tooltip-trigger-{stable_id}")
    };

    let mounted_slot: Rc<RefCell<Option<Rc<MountedData>>>> =
        use_hook(|| Rc::new(RefCell::new(None)));

    let registered_id_cell = use_hook(|| Rc::new(RefCell::new(None::<String>)));
    if props.handle.is_none() {
        let fctx = floating_ctx
            .expect("TooltipTrigger must be rendered inside TooltipRoot or given a `handle` prop");
        if let Some(ref custom_id) = props.id {
            if registered_id_cell.borrow().as_deref() != Some(custom_id.as_str()) {
                if let Some(prev) = registered_id_cell.borrow().as_deref() {
                    fctx.unregister_trigger_id(prev);
                }
                fctx.register_trigger_id(custom_id.clone());
                *registered_id_cell.borrow_mut() = Some(custom_id.clone());
            }
        } else if let Some(prev) = registered_id_cell.borrow_mut().take() {
            fctx.unregister_trigger_id(&prev);
        }
    } else if let Some(prev) = registered_id_cell.borrow_mut().take()
        && let Some(fctx) = floating_ctx
    {
        fctx.unregister_trigger_id(&prev);
    }

    let cleanup_id_cell = Rc::clone(&registered_id_cell);
    use_drop(move || {
        if let Some(id) = cleanup_id_cell.borrow_mut().take()
            && let Some(fctx) = floating_ctx
        {
            fctx.unregister_trigger_id(&id);
        }
    });

    // Register/update/unregister on detached handle when `handle` is provided.
    let registered_handle_entry: Rc<RefCell<Option<(AnyTooltipHandle, String)>>> =
        use_hook(|| Rc::new(RefCell::new(None)));
    {
        let mut slot = registered_handle_entry.borrow_mut();
        match (&*slot, props.handle.as_any()) {
            (Some((prev_h, prev_id)), Some(next_h))
                if prev_h == next_h && prev_id == &trigger_id =>
            {
                next_h.update_trigger_payload(&trigger_id, props.payload.clone());
            }
            (_, Some(next_h)) => {
                if let Some((prev_h, prev_id)) = slot.take() {
                    prev_h.unregister_trigger(&prev_id);
                }
                next_h.register_trigger(
                    trigger_id.clone(),
                    mounted_slot.borrow().clone(),
                    props.payload.clone(),
                );
                *slot = Some((next_h.clone(), trigger_id.clone()));
            }
            (Some(_), None) => {
                if let Some((prev_h, prev_id)) = slot.take() {
                    prev_h.unregister_trigger(&prev_id);
                }
            }
            (None, None) => {}
        }
    }
    let handle_entry_for_drop = Rc::clone(&registered_handle_entry);
    use_drop(move || {
        if let Some((prev_h, prev_id)) = handle_entry_for_drop.borrow_mut().take() {
            prev_h.unregister_trigger(&prev_id);
        }
    });

    let mounted_once = use_hook(|| Rc::new(Cell::new(false)));
    let runtime_sig_opt = ctx.as_ref().map(|c| c.runtime);
    use_effect(use_reactive((&trigger_id,), move |_| {
        if mounted_once.get() {
            if let Some(sig) = runtime_sig_opt {
                sig.peek().resync_positioning_if_open();
            }
        } else {
            mounted_once.set(true);
        }
    }));

    let (aria_describedby, data_state) = if let Some(any_handle) = props.handle.as_any() {
        let opened_by_this = any_handle.is_opened_by_trigger(&trigger_id);
        let describedby = if opened_by_this {
            inline_trigger
                .as_ref()
                .and_then(|t| t.aria_describedby().map(str::to_owned))
        } else {
            None
        };
        (describedby, if opened_by_this { "open" } else { "closed" })
    } else {
        let trigger = inline_trigger
            .as_ref()
            .expect("TooltipTrigger requires TooltipRootContext when `handle` is not provided");
        (
            trigger.aria_describedby().map(str::to_owned),
            trigger.data_state().as_str(),
        )
    };

    let disabled = props.disabled;
    let handle_for_mount = props.handle.as_any().cloned();
    let tid_for_mount = trigger_id.clone();
    let mounted_slot_cb = Rc::clone(&mounted_slot);
    let mut inline_mount = if props.handle.is_none() {
        runtime.as_ref().map(|rt| rt.mount_trigger())
    } else {
        None
    };

    let handle_for_pd = props.handle.as_any().cloned();
    let tid_for_pd = trigger_id.clone();
    let mut inline_pointer_down = if props.handle.is_none() {
        runtime.as_ref().map(|rt| rt.trigger_pointer_down())
    } else {
        None
    };

    let mut inline_pointer_cancel = if props.handle.is_none() {
        runtime.as_ref().map(|rt| rt.trigger_pointer_cancel())
    } else {
        None
    };

    let handle_for_enter = props.handle.as_any().cloned();
    let tid_for_enter = trigger_id.clone();
    let mut inline_mouse_enter = if props.handle.is_none() {
        runtime.as_ref().map(|rt| rt.trigger_pointer_enter())
    } else {
        None
    };

    let handle_for_leave = props.handle.as_any().cloned();
    let tid_for_leave = trigger_id.clone();
    let mut inline_mouse_leave = if props.handle.is_none() {
        runtime.as_ref().map(|rt| rt.trigger_pointer_leave())
    } else {
        None
    };

    let handle_for_focus = props.handle.as_any().cloned();
    let tid_for_focus = trigger_id.clone();
    let mut inline_focus = if props.handle.is_none() {
        runtime.as_ref().map(|rt| rt.trigger_focus())
    } else {
        None
    };

    let handle_for_blur = props.handle.as_any().cloned();
    let tid_for_blur = trigger_id.clone();
    let mut inline_blur = if props.handle.is_none() {
        runtime.as_ref().map(|rt| rt.trigger_blur())
    } else {
        None
    };

    let handle_for_click = props.handle.as_any().cloned();
    let tid_for_click = trigger_id.clone();
    let mut inline_click = if props.handle.is_none() {
        runtime.as_ref().map(|rt| rt.trigger_click())
    } else {
        None
    };

    let handle_for_keydown = props.handle.as_any().cloned();
    let tid_for_keydown = trigger_id.clone();
    let mut inline_key_down = if props.handle.is_none() {
        runtime.as_ref().map(|rt| rt.escape_keydown())
    } else {
        None
    };

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
                let mounted_evt = evt.map(|d| MountedData::from(d));
                let data = mounted_evt.data();
                *mounted_slot_cb.borrow_mut() = Some(data.clone());
                if let Some(ref h) = handle_for_mount {
                    h.update_trigger_mounted(&tid_for_mount, Some(data));
                } else if let Some(ref mut cb) = inline_mount {
                    cb(mounted_evt);
                }
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
                if let Some(ref h) = handle_for_pd {
                    if h.is_opened_by_trigger(&tid_for_pd) {
                        h.close();
                    }
                } else if let Some(ref mut cb) = inline_pointer_down {
                    cb(evt.map(|d| PointerData::from(d)));
                }
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
                if let Some(ref mut cb) = inline_pointer_cancel {
                    cb(evt.map(|d| PointerData::from(d)));
                }
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
                if disabled {
                    return;
                }
                if let Some(ref h) = handle_for_enter {
                    h.activate_trigger(&tid_for_enter, true);
                } else if let Some(ref mut cb) = inline_mouse_enter {
                    cb(evt.map(|d| MouseData::from(d)));
                }
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
                if let Some(ref h) = handle_for_leave {
                    if h.is_opened_by_trigger(&tid_for_leave) {
                        h.close();
                    }
                } else if let Some(ref mut cb) = inline_mouse_leave {
                    cb(evt.map(|d| MouseData::from(d)));
                }
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
                if disabled {
                    return;
                }
                if let Some(ref h) = handle_for_focus {
                    h.activate_trigger(&tid_for_focus, true);
                } else if let Some(ref mut cb) = inline_focus {
                    cb(evt.map(|d| FocusData::from(d)));
                }
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
                if let Some(ref h) = handle_for_blur {
                    if h.is_opened_by_trigger(&tid_for_blur) {
                        h.close();
                    }
                } else if let Some(ref mut cb) = inline_blur {
                    cb(evt.map(|d| FocusData::from(d)));
                }
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
                if let Some(ref h) = handle_for_click {
                    if h.is_opened_by_trigger(&tid_for_click) {
                        h.close();
                    }
                } else if let Some(ref mut cb) = inline_click {
                    cb(evt.map(|d| MouseData::from(d)));
                }
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
                let kb_evt = evt.map(|d| KeyboardData::from(d));
                if let Some(ref h) = handle_for_keydown {
                    if is_escape_key(&kb_evt.key()) && h.is_opened_by_trigger(&tid_for_keydown) {
                        h.close();
                    }
                } else if let Some(ref mut cb) = inline_key_down {
                    cb(kb_evt);
                }
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

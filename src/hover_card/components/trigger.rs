use std::{
    cell::{Cell, RefCell},
    rc::Rc,
};

use dioxus::core::{AttributeValue, Event, ListenerCallback};
use dioxus::html::PlatformEventData;
use dioxus::prelude::*;

use crate::foundation::{
    compose::{collect_user_attrs, merge_props, push_attr_opt, push_event_handler},
    keyboard::is_escape_key,
    overlay::{
        AnyHoverCardHandle, HoverCardHandleProp, OverlayPayload, use_overlay_handle_version,
    },
    shared::use_stable_id,
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
    #[props(default, into)]
    pub handle: HoverCardHandleProp,
    #[props(default)]
    pub payload: Option<OverlayPayload>,
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
    let _handle_version = use_overlay_handle_version(props.handle.as_any());
    let stable_id = use_stable_id(None);
    let ctx = try_consume_context::<HoverCardRootContext>();
    let runtime = ctx.as_ref().map(|c| c.runtime.read().clone());
    let inline_trigger = runtime.as_ref().map(|rt| {
        rt.trigger(
            props.href.clone(),
            props.target.clone(),
            props.rel.clone(),
            props.disabled,
        )
    });

    let trigger_id = if let Some(ref custom_id) = props.id {
        custom_id.clone()
    } else if props.handle.is_none()
        && let Some(ref trigger) = inline_trigger
    {
        trigger.id().to_string()
    } else {
        format!("hover-card-trigger-{stable_id}")
    };

    let mounted_slot: Rc<RefCell<Option<Rc<MountedData>>>> =
        use_hook(|| Rc::new(RefCell::new(None)));

    let registered_id_cell = use_hook(|| Rc::new(RefCell::new(None::<String>)));
    if props.handle.is_none() {
        let rt = runtime.as_ref().expect(
            "HoverCardTrigger must be rendered inside HoverCardRoot or given a `handle` prop",
        );
        if let Some(ref custom_id) = props.id {
            if registered_id_cell.borrow().as_deref() != Some(custom_id.as_str()) {
                if let Some(prev) = registered_id_cell.borrow().as_deref() {
                    rt.unregister_trigger_id(prev);
                }
                rt.register_trigger_id(custom_id.clone());
                *registered_id_cell.borrow_mut() = Some(custom_id.clone());
            }
        } else if let Some(prev) = registered_id_cell.borrow_mut().take() {
            rt.unregister_trigger_id(&prev);
        }
    } else if let Some(prev) = registered_id_cell.borrow_mut().take()
        && let Some(ref rt) = runtime
    {
        rt.unregister_trigger_id(&prev);
    }

    let cleanup_id_cell = Rc::clone(&registered_id_cell);
    let cleanup_runtime = runtime.clone();
    use_drop(move || {
        if let Some(id) = cleanup_id_cell.borrow_mut().take()
            && let Some(rt) = cleanup_runtime
        {
            rt.unregister_trigger_id(&id);
        }
    });

    // Register/update/unregister on detached handle when `handle` is provided.
    let registered_handle_entry: Rc<RefCell<Option<(AnyHoverCardHandle, String)>>> =
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

    let effective_disabled = inline_trigger
        .as_ref()
        .map(|t| t.is_disabled())
        .unwrap_or(props.disabled);
    let href = inline_trigger
        .as_ref()
        .map(|t| t.href().map(str::to_owned))
        .unwrap_or_else(|| props.href.clone());
    let target = inline_trigger
        .as_ref()
        .map(|t| t.target().map(str::to_owned))
        .unwrap_or_else(|| props.target.clone());
    let rel = inline_trigger
        .as_ref()
        .map(|t| t.rel().map(str::to_owned))
        .unwrap_or_else(|| props.rel.clone());
    let tabindex = inline_trigger
        .as_ref()
        .map(|t| t.tabindex().map(|v| v.to_string()))
        .unwrap_or_else(|| (!effective_disabled).then(|| "0".to_owned()));
    let aria_disabled = inline_trigger
        .as_ref()
        .map(|t| t.aria_disabled())
        .unwrap_or_else(|| effective_disabled.then_some("true"));
    let data_disabled = inline_trigger
        .as_ref()
        .map(|t| t.data_disabled())
        .unwrap_or_else(|| effective_disabled.then_some(""));
    let part_attr = inline_trigger
        .as_ref()
        .map(|t| t.data_monoxus_hover_card_trigger())
        .unwrap_or("");
    let data_state = if let Some(any_handle) = props.handle.as_any() {
        if any_handle.is_opened_by_trigger(&trigger_id) {
            "open"
        } else {
            "closed"
        }
    } else {
        inline_trigger
            .as_ref()
            .expect("HoverCardTrigger requires HoverCardRootContext when `handle` is not provided")
            .data_state()
            .as_str()
    };

    let handle_for_mount = props.handle.as_any().cloned();
    let tid_for_mount = trigger_id.clone();
    let mounted_slot_cb = Rc::clone(&mounted_slot);
    let mut inline_mount_runtime = if props.handle.is_none() {
        runtime.as_ref().map(|rt| rt.mount_trigger())
    } else {
        None
    };

    let mut inline_touch_start = if props.handle.is_none() {
        runtime.as_ref().map(|rt| rt.trigger_touch_start())
    } else {
        None
    };

    let mut inline_pointer_down = if props.handle.is_none() {
        runtime.as_ref().map(|rt| rt.trigger_pointer_down())
    } else {
        None
    };

    let handle_for_enter = props.handle.as_any().cloned();
    let tid_for_enter = trigger_id.clone();
    let mut inline_pointer_enter = if props.handle.is_none() {
        runtime.as_ref().map(|rt| rt.trigger_pointer_enter(props.disabled))
    } else {
        None
    };

    let handle_for_leave = props.handle.as_any().cloned();
    let tid_for_leave = trigger_id.clone();
    let mut inline_pointer_leave = if props.handle.is_none() {
        runtime.as_ref().map(|rt| rt.trigger_pointer_leave(props.disabled))
    } else {
        None
    };

    let handle_for_focus = props.handle.as_any().cloned();
    let tid_for_focus = trigger_id.clone();
    let mut inline_focus = if props.handle.is_none() {
        runtime.as_ref().map(|rt| rt.trigger_focus(props.disabled))
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

    let handle_for_keydown = props.handle.as_any().cloned();
    let tid_for_keydown = trigger_id.clone();
    let mut inline_key_down = if props.handle.is_none() {
        runtime.as_ref().map(|rt| rt.escape_keydown())
    } else {
        None
    };

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
                let mounted_evt = evt.map(|d| MountedData::from(d));
                let data = mounted_evt.data();
                *mounted_slot_cb.borrow_mut() = Some(data.clone());
                if let Some(ref h) = handle_for_mount {
                    h.update_trigger_mounted(&tid_for_mount, Some(data));
                } else if let Some(ref mut cb) = inline_mount_runtime {
                    cb(mounted_evt);
                }
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
                if let Some(ref mut cb) = inline_touch_start {
                    cb(evt.map(|d| TouchData::from(d)));
                } else {
                    evt.prevent_default();
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
                if let Some(ref mut cb) = inline_pointer_down {
                    cb(evt.map(|d| PointerData::from(d)));
                }
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
                let ptr_evt = evt.map(|d| PointerData::from(d));
                if let Some(ref h) = handle_for_enter {
                    if !effective_disabled && ptr_evt.pointer_type() != "touch" {
                        h.activate_trigger(&tid_for_enter, true);
                    }
                } else if let Some(ref mut cb) = inline_pointer_enter {
                    cb(ptr_evt);
                }
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
                let ptr_evt = evt.map(|d| PointerData::from(d));
                if let Some(ref h) = handle_for_leave {
                    if !effective_disabled
                        && ptr_evt.pointer_type() != "touch"
                        && h.is_opened_by_trigger(&tid_for_leave)
                    {
                        h.close();
                    }
                } else if let Some(ref mut cb) = inline_pointer_leave {
                    cb(ptr_evt);
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
                if let Some(ref h) = handle_for_focus {
                    if !effective_disabled {
                        h.activate_trigger(&tid_for_focus, true);
                    }
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

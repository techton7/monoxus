use std::{
    cell::{Cell, RefCell},
    rc::Rc,
};

use dioxus::prelude::*;

use crate::hover_card::components::HoverCardRootContext;

#[derive(Clone, PartialEq, Props)]
pub struct HoverCardTriggerProps {
    #[props(default)]
    pub id: Option<String>,
    #[props(default)]
    pub class: Option<String>,
    #[props(default)]
    pub style: Option<String>,
    #[props(default)]
    pub href: Option<String>,
    #[props(default)]
    pub target: Option<String>,
    #[props(default)]
    pub rel: Option<String>,
    #[props(default = false)]
    pub disabled: bool,
    #[props(default)]
    pub on_click: Option<EventHandler<MouseEvent>>,
    #[props(default)]
    pub on_mounted: Option<EventHandler<MountedEvent>>,
    #[props(extends = GlobalAttributes)]
    pub attributes: Vec<Attribute>,
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
    let consumer_on_mounted = props.on_mounted;
    let consumer_on_click = props.on_click;
    let effective_disabled = trigger.is_disabled();
    let merged = crate::foundation::compose::merge_global_attributes(
        props.class.as_deref(),
        props.style.as_deref(),
        props.attributes,
    );

    rsx! {
        a {
            id: "{trigger_id}",
            class: merged.class.as_deref(),
            style: merged.style.as_deref(),
            href: trigger.href(),
            target: trigger.target(),
            rel: trigger.rel(),
            tabindex: trigger.tabindex(),
            "aria-disabled": trigger.aria_disabled(),
            "data-state": trigger.data_state().as_str(),
            "data-disabled": trigger.data_disabled(),
            "data-monoxus-hover-card-trigger": trigger.data_monoxus_hover_card_trigger(),
            onmounted: move |evt| {
                on_mount_runtime(evt.clone());
                if let Some(ref handler) = consumer_on_mounted {
                    handler.call(evt);
                }
            },
            ontouchstart: runtime.trigger_touch_start(),
            onpointerdown: runtime.trigger_pointer_down(),
            onpointerenter: runtime.trigger_pointer_enter(props.disabled),
            onpointerleave: runtime.trigger_pointer_leave(props.disabled),
            onfocus: runtime.trigger_focus(props.disabled),
            onblur: runtime.trigger_blur(),
            onkeydown: runtime.escape_keydown(),
            onclick: move |evt: MouseEvent| {
                if effective_disabled {
                    evt.prevent_default();
                    return;
                }
                if let Some(ref handler) = consumer_on_click {
                    handler.call(evt);
                }
            },
            ..merged.rest,
            {props.children}
        }
    }
}

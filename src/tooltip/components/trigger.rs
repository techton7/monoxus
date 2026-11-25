use std::{
    cell::{Cell, RefCell},
    rc::Rc,
};

use dioxus::prelude::*;

use crate::foundation::shared::data_disabled_attr;
use crate::tooltip::components::TooltipRootContext;

#[derive(Props, Clone, PartialEq)]
pub struct TooltipTriggerProps {
    #[props(default)]
    pub id: Option<String>,
    #[props(default)]
    pub class: Option<String>,
    #[props(default)]
    pub style: Option<String>,
    #[props(default = false)]
    pub disabled: bool,
    pub children: Element,
}

#[component]
pub fn TooltipTrigger(props: TooltipTriggerProps) -> Element {
    let ctx = use_context::<TooltipRootContext>();
    let runtime = ctx.runtime.read();
    let trigger = runtime.trigger();
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

    rsx! {
        button {
            id: "{trigger_id}",
            r#type: "button",
            class: props.class.as_deref().unwrap_or_default(),
            style: props.style.as_deref().unwrap_or_default(),
            disabled: props.disabled,
            aria_describedby: trigger.aria_describedby(),
            "data-state": trigger.data_state().as_str(),
            "data-disabled": data_disabled_attr(props.disabled),
            onmounted: runtime.mount_trigger(),
            onpointerdown: runtime.trigger_pointer_down(),
            onpointercancel: runtime.trigger_pointer_cancel(),
            onmouseenter: runtime.trigger_pointer_enter(),
            onmouseleave: runtime.trigger_pointer_leave(),
            onfocus: runtime.trigger_focus(),
            onblur: runtime.trigger_blur(),
            onclick: runtime.trigger_click(),
            onkeydown: runtime.escape_keydown(),
            {props.children}
        }
    }
}

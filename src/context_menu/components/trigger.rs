use std::cell::{Cell, RefCell};
use std::rc::Rc;
use std::time::Duration;

use dioxus::prelude::*;
use futures_timer::Delay;

use crate::context_menu::{
    CONTEXT_MENU_LONG_PRESS_DELAY_MS, CONTEXT_MENU_TOUCH_CALLOUT_STYLE,
    ContextMenuTriggerAttributes,
};
use crate::foundation::{
    browser::spawn_root_task,
    compose::{collect_user_attrs, merge_props, push_attr_opt, push_event_handler},
    state::sync_signal,
};
use crate::menu::{MenuOpenFocusTarget, MenuPart, MenuRootContext, MenuVariant};

#[derive(Props, Clone, PartialEq)]
pub struct ContextMenuTriggerProps {
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
    pub children: Element,
}

/// Interactive region capturing right-click `contextmenu` and `700ms` non-mouse long-press (`CM-1`..`CM-7`).
#[component]
pub fn ContextMenuTrigger(props: ContextMenuTriggerProps) -> Element {
    let root = use_context::<MenuRootContext>();
    let id = props
        .id
        .unwrap_or_else(|| root.relationships.trigger_id().to_owned());

    {
        let mut tid = root.trigger_id;
        sync_signal(&mut tid, id.clone());
    }

    let lp_gen = use_hook(|| Rc::new(Cell::new(0u64)));
    let lp_task = use_hook(|| Rc::new(RefCell::new(None::<dioxus::core::Task>)));

    let cancel_lp = {
        let lp_gen = lp_gen.clone();
        let lp_task = lp_task.clone();
        Rc::new(move || {
            lp_gen.set(lp_gen.get().wrapping_add(1));
            if let Some(t) = lp_task.borrow_mut().take() {
                t.cancel();
            }
        })
    };

    let drop_cancel = cancel_lp.clone();
    dioxus::core::use_drop(move || {
        drop_cancel();
    });

    if props.disabled {
        cancel_lp();
    }

    let is_open = root.is_open();
    let attrs = ContextMenuTriggerAttributes::from_parts(id, is_open, props.disabled);

    let cm_root = root.clone();
    let cm_cancel = cancel_lp.clone();
    let on_context_menu = EventHandler::new(move |evt: MouseEvent| {
        if props.disabled {
            return;
        }
        cm_cancel();
        evt.prevent_default();
        evt.stop_propagation();
        let coords = evt.data().client_coordinates();
        {
            let mut va = cm_root.virtual_anchor;
            let _ = va.try_write().map(|mut w| *w = Some((coords.x, coords.y)));
            let mut tgt = cm_root.root_surface.open_focus_target;
            let _ = tgt
                .try_write()
                .map(|mut w| *w = MenuOpenFocusTarget::ContentContainer);
        }
        cm_root.request_open_change(true);
    });

    let pd_root = root;
    let pd_gen = lp_gen;
    let pd_task = lp_task;
    let pd_cancel = cancel_lp.clone();
    let on_pointer_down = EventHandler::new(move |evt: PointerEvent| {
        if props.disabled || evt.pointer_type() == "mouse" {
            return;
        }
        evt.stop_propagation();
        pd_cancel();
        let coords = evt.data().client_coordinates();
        let next_gen = pd_gen.get().wrapping_add(1);
        pd_gen.set(next_gen);
        let gen_ref = pd_gen.clone();
        let timer_root = pd_root.clone();
        let task = spawn_root_task(async move {
            Delay::new(Duration::from_millis(CONTEXT_MENU_LONG_PRESS_DELAY_MS)).await;
            if gen_ref.get() == next_gen {
                {
                    let mut va = timer_root.virtual_anchor;
                    let _ = va.try_write().map(|mut w| *w = Some((coords.x, coords.y)));
                    let mut tgt = timer_root.root_surface.open_focus_target;
                    let _ = tgt
                        .try_write()
                        .map(|mut w| *w = MenuOpenFocusTarget::ContentContainer);
                }
                timer_root.request_open_change(true);
            }
        });
        *pd_task.borrow_mut() = Some(task);
    });

    let pm_cancel = cancel_lp.clone();
    let on_pointer_move = EventHandler::new(move |evt: PointerEvent| {
        if evt.pointer_type() != "mouse" {
            pm_cancel();
        }
    });

    let pu_cancel = cancel_lp.clone();
    let on_pointer_up = EventHandler::new(move |_evt: PointerEvent| {
        pu_cancel();
    });

    let pc_cancel = cancel_lp;
    let on_pointer_cancel = EventHandler::new(move |_evt: PointerEvent| {
        pc_cancel();
    });

    let data_state_str = attrs.data_state.as_str();
    let part_attr = MenuPart::Trigger.static_data_attr_name(MenuVariant::ContextMenu);

    let mut base = Vec::with_capacity(11);
    base.push(Attribute::new("id", attrs.id, None, false));
    base.push(Attribute::new(
        "style",
        CONTEXT_MENU_TOUCH_CALLOUT_STYLE,
        None,
        false,
    ));
    base.push(Attribute::new("data-state", data_state_str, None, false));
    push_attr_opt(&mut base, "data-disabled", attrs.data_disabled);
    base.push(Attribute::new(part_attr, "", None, false));
    push_event_handler::<MouseData>(&mut base, "oncontextmenu", Some(on_context_menu));
    push_event_handler::<PointerData>(&mut base, "onpointerdown", Some(on_pointer_down));
    push_event_handler::<PointerData>(&mut base, "onpointermove", Some(on_pointer_move));
    push_event_handler::<PointerData>(&mut base, "onpointerup", Some(on_pointer_up));
    push_event_handler::<PointerData>(&mut base, "onpointercancel", Some(on_pointer_cancel));

    let mut user = collect_user_attrs(props.class, props.style, props.attributes);
    push_event_handler::<MouseData>(&mut user, "onclick", props.on_click);
    push_event_handler::<KeyboardData>(&mut user, "onkeydown", props.on_key_down);
    push_event_handler::<MountedData>(&mut user, "onmounted", props.on_mounted);
    let merged = merge_props(base, user);

    if let Some(child) = props.child {
        return child.call(merged);
    }

    rsx! {
        div {
            ..merged,
            {props.children}
        }
    }
}

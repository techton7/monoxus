use std::cell::{Cell, RefCell};
use std::rc::Rc;
use std::time::Duration;

use dioxus::prelude::*;
use futures_timer::Delay;

use crate::context_menu::{
    CONTEXT_MENU_LONG_PRESS_DELAY_MS, CONTEXT_MENU_TOUCH_CALLOUT_STYLE,
    ContextMenuTriggerAttributes,
};
use crate::foundation::browser::spawn_root_task;
use crate::menu::{
    MenuOpenFocusTarget, MenuPart, MenuRootContext, MenuVariant, merge_attributes_with_part,
};

/// Render props passed to `render_child` when `ContextMenuTrigger` uses `as_child: true`.
#[derive(Clone, PartialEq)]
pub struct ContextMenuTriggerRenderProps {
    pub attrs: ContextMenuTriggerAttributes,
    pub class: Option<String>,
    pub style: String,
    pub on_context_menu: EventHandler<MouseEvent>,
    pub on_pointer_down: EventHandler<PointerEvent>,
    pub on_pointer_move: EventHandler<PointerEvent>,
    pub on_pointer_up: EventHandler<PointerEvent>,
    pub on_pointer_cancel: EventHandler<PointerEvent>,
    pub attributes: Vec<Attribute>,
    pub children: Element,
}

#[derive(Props, Clone, PartialEq)]
pub struct ContextMenuTriggerProps {
    #[props(default)]
    pub id: Option<String>,
    #[props(default)]
    pub class: Option<String>,
    #[props(default)]
    pub style: Option<String>,
    #[props(default = false)]
    pub disabled: bool,
    #[props(default = false)]
    pub as_child: bool,
    #[props(default)]
    pub render_child: Option<Callback<ContextMenuTriggerRenderProps, Element>>,
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
        if tid.try_peek().is_ok_and(|g| *g != id) {
            let _ = tid.try_write().map(|mut w| *w = id.clone());
        }
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
    let merged_attrs = merge_attributes_with_part(
        props.attributes,
        MenuPart::Trigger.static_data_attr_name(MenuVariant::ContextMenu),
    );
    let merged_style = match props.style.as_deref() {
        Some(custom) if !custom.is_empty() => format!("{CONTEXT_MENU_TOUCH_CALLOUT_STYLE} {custom}"),
        _ => CONTEXT_MENU_TOUCH_CALLOUT_STYLE.to_owned(),
    };

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

    if props.as_child
        && let Some(render_child) = props.render_child.as_ref()
    {
        return render_child.call(ContextMenuTriggerRenderProps {
            attrs,
            class: props.class,
            style: merged_style,
            on_context_menu,
            on_pointer_down,
            on_pointer_move,
            on_pointer_up,
            on_pointer_cancel,
            attributes: merged_attrs,
            children: props.children,
        });
    }

    rsx! {
        div {
            id: "{attrs.id}",
            class: props.class.as_deref().unwrap_or_default(),
            style: "{merged_style}",
            "data-state": "{attrs.data_state.as_str()}",
            "data-disabled": attrs.data_disabled,
            oncontextmenu: move |evt| on_context_menu.call(evt),
            onpointerdown: move |evt| on_pointer_down.call(evt),
            onpointermove: move |evt| on_pointer_move.call(evt),
            onpointerup: move |evt| on_pointer_up.call(evt),
            onpointercancel: move |evt| on_pointer_cancel.call(evt),
            ..merged_attrs,
            {props.children}
        }
    }
}

use std::cell::RefCell;
use std::rc::Rc;

use dioxus::prelude::*;

use crate::rating_group::{
    attrs::RatingGroupItemAttributes,
    runtime::{RatingGroupItemContext, use_rating_group_context},
};

/// Render props passed to `render_child` when `RatingGroupItem` uses `as_child: true`.
#[derive(Clone, PartialEq, Props)]
pub struct RatingGroupItemRenderProps {
    pub id: String,
    #[props(default)]
    pub class: Option<String>,
    #[props(default)]
    pub style: Option<String>,
    pub attrs: RatingGroupItemAttributes,
    pub on_click: EventHandler<MouseEvent>,
    pub on_pointer_move: EventHandler<PointerEvent>,
    #[props(default)]
    pub on_mounted: Option<EventHandler<MountedEvent>>,
    #[props(extends = GlobalAttributes)]
    pub attributes: Vec<Attribute>,
    pub children: Element,
}

/// Properties for [`RatingGroupItem`].
#[derive(Clone, PartialEq, Props)]
pub struct RatingGroupItemProps {
    pub index: usize,
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
    pub render_child: Option<Callback<RatingGroupItemRenderProps, Element>>,
    #[props(default)]
    pub on_click: Option<EventHandler<MouseEvent>>,
    #[props(default)]
    pub on_pointer_move: Option<EventHandler<PointerEvent>>,
    #[props(default)]
    pub on_mounted: Option<EventHandler<MountedEvent>>,
    #[props(extends = GlobalAttributes)]
    pub attributes: Vec<Attribute>,
    #[props(default)]
    pub children: Element,
}

/// Presentational item inside a [`crate::rating_group::RatingGroupRoot`] handling pointer hit math
/// and hover preview while exposing its fill state (`active` / `partial` / `inactive`).
#[component]
pub fn RatingGroupItem(props: RatingGroupItemProps) -> Element {
    let ctx = use_rating_group_context();
    let runtime = ctx.0;

    let item_attrs = runtime.item_attrs(props.index, props.id.clone(), props.disabled);
    let item_dom_id = item_attrs.id.clone();

    let mut item_index_sig = use_signal(|| props.index);
    if *item_index_sig.peek() != props.index {
        item_index_sig.set(props.index);
    }
    let mut item_disabled_sig = use_signal(|| props.disabled);
    if *item_disabled_sig.peek() != props.disabled {
        item_disabled_sig.set(props.disabled);
    }

    use_context_provider(|| RatingGroupItemContext {
        runtime: runtime.clone(),
        index: item_index_sig,
        item_disabled: item_disabled_sig,
    });

    #[derive(Clone)]
    struct ItemHandlerProps {
        index: usize,
        disabled: bool,
        dom_id: String,
        on_click: Option<EventHandler<MouseEvent>>,
        on_pointer_move: Option<EventHandler<PointerEvent>>,
    }

    let handler_props_slot: Rc<RefCell<ItemHandlerProps>> = use_hook(|| {
        Rc::new(RefCell::new(ItemHandlerProps {
            index: props.index,
            disabled: props.disabled,
            dom_id: item_dom_id.clone(),
            on_click: props.on_click,
            on_pointer_move: props.on_pointer_move,
        }))
    });
    *handler_props_slot.borrow_mut() = ItemHandlerProps {
        index: props.index,
        disabled: props.disabled,
        dom_id: item_dom_id.clone(),
        on_click: props.on_click,
        on_pointer_move: props.on_pointer_move,
    };

    let on_click = use_hook(|| {
        let mut runtime = runtime.clone();
        let slot = Rc::clone(&handler_props_slot);
        Callback::new(move |evt: MouseEvent| {
            let current = slot.borrow().clone();
            if let Some(ref handler) = current.on_click {
                handler.call(evt.clone());
            }
            if !evt.default_action_enabled() {
                return;
            }
            let coords = evt.client_coordinates();
            runtime.handle_item_click(
                current.index,
                current.disabled,
                coords.x,
                coords.y,
                &current.dom_id,
            );
        })
    });

    let on_pointer_move = use_hook(|| {
        let mut runtime = runtime.clone();
        let slot = Rc::clone(&handler_props_slot);
        Callback::new(move |evt: PointerEvent| {
            let current = slot.borrow().clone();
            if let Some(ref handler) = current.on_pointer_move {
                handler.call(evt.clone());
            }
            let coords = evt.client_coordinates();
            let pointer_type = evt.pointer_type();
            runtime.handle_item_pointer_move(
                current.index,
                current.disabled,
                &pointer_type,
                coords.x,
                coords.y,
                &current.dom_id,
            );
        })
    });

    if props.as_child {
        if let Some(render_child) = props.render_child.as_ref() {
            return render_child.call(RatingGroupItemRenderProps {
                id: item_dom_id,
                class: props.class,
                style: props.style,
                attrs: item_attrs,
                on_click,
                on_pointer_move,
                on_mounted: props.on_mounted,
                attributes: props.attributes,
                children: props.children,
            });
        }
        #[cfg(debug_assertions)]
        eprintln!(
            "monoxus::RatingGroupItem: as_child requires render_child; rendering the default div"
        );
    }

    rsx! {
        div {
            id: "{item_dom_id}",
            class: props.class,
            style: props.style,
            role: item_attrs.role,
            "data-value": "{item_attrs.data_value}",
            "data-orientation": item_attrs.data_orientation,
            "data-disabled": item_attrs.data_disabled,
            "data-readonly": item_attrs.data_readonly,
            "data-state": item_attrs.data_state.as_str(),
            "data-rating-group-item": item_attrs.data_rating_group_item,
            onclick: move |evt| on_click.call(evt),
            onpointermove: move |evt| on_pointer_move.call(evt),
            onmounted: move |evt| {
                if let Some(handler) = props.on_mounted.as_ref() {
                    handler.call(evt);
                }
            },
            ..props.attributes,
            {props.children}
        }
    }
}

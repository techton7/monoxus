use std::cell::RefCell;
use std::rc::Rc;

use dioxus::prelude::*;

use crate::combobox::{
    attrs::ComboboxItemAttrs, runtime::use_combobox_context, types::ComboboxItemRenderState,
};

const DRAG_THRESHOLD_PX: f64 = 4.0;

#[derive(Clone, PartialEq)]
pub struct ComboboxItemRenderProps {
    pub attrs: ComboboxItemAttrs,
    pub state: ComboboxItemRenderState,
    pub class: Option<String>,
    pub style: Option<String>,
    pub on_mouse_down: EventHandler<MouseEvent>,
    pub on_pointer_down: EventHandler<PointerEvent>,
    pub on_pointer_up: EventHandler<PointerEvent>,
    pub on_click: EventHandler<MouseEvent>,
    pub on_pointer_move: EventHandler<PointerEvent>,
    pub attributes: Vec<Attribute>,
    pub children: Element,
}

#[derive(Props, Clone, PartialEq)]
pub struct ComboboxItemProps {
    #[props(default)]
    pub id: Option<String>,
    pub value: String,
    #[props(default)]
    pub label: Option<String>,
    #[props(default = false)]
    pub disabled: bool,
    #[props(default)]
    pub on_highlight: Option<EventHandler<()>>,
    #[props(default)]
    pub on_unhighlight: Option<EventHandler<()>>,
    #[props(default)]
    pub class: Option<String>,
    #[props(default)]
    pub style: Option<String>,
    #[props(default = false)]
    pub as_child: bool,
    #[props(default)]
    pub render_child: Option<Callback<ComboboxItemRenderProps, Element>>,
    #[props(default)]
    pub render_children: Option<Callback<ComboboxItemRenderState, Element>>,
    #[props(extends = GlobalAttributes)]
    pub attributes: Vec<Attribute>,
    #[props(default)]
    pub children: Element,
}

#[component]
pub fn ComboboxItem(props: ComboboxItemProps) -> Element {
    let ctx = use_combobox_context();
    let label_str = props.label.clone().unwrap_or_else(|| props.value.clone());
    let item_id = props
        .id
        .clone()
        .unwrap_or_else(|| ctx.runtime.relationships().item_id(&props.value));

    // DEF-CBX-12: Track latest dynamic `item_id` so prop ID changes unregister the stale ID
    // and `use_drop` always unregisters the current ID.
    let tracked_dom_id: Rc<RefCell<Option<String>>> = use_hook(|| Rc::new(RefCell::new(None)));

    {
        let rt = ctx.runtime.clone();
        let v = props.value.clone();
        let l = label_str.clone();
        let dom_id = item_id.clone();
        let dis = props.disabled;
        let tracked = Rc::clone(&tracked_dom_id);
        use_effect(use_reactive(
            (&v, &l, &dom_id, &dis),
            move |(val, lbl, id, disabled)| {
                let prev = tracked.borrow().clone();
                if let Some(prev_id) = prev {
                    if prev_id != id {
                        rt.unregister_item(&prev_id);
                    }
                }
                *tracked.borrow_mut() = Some(id.clone());
                rt.register_item(val, lbl, id, disabled);
            },
        ));
    }

    {
        let rt = ctx.runtime.clone();
        let tracked = Rc::clone(&tracked_dom_id);
        use_drop(move || {
            if let Some(dom_id) = tracked.borrow_mut().take() {
                rt.unregister_item(&dom_id);
            }
        });
    }

    let attrs = ctx
        .runtime
        .item_attributes(item_id, &props.value, &label_str, props.disabled);
    let is_hl = attrs.is_highlighted();
    let is_sel = attrs.is_selected();
    let is_dis = attrs.is_disabled();
    let render_state = ComboboxItemRenderState {
        selected: is_sel,
        highlighted: is_hl,
    };

    let mut prev_hl = use_signal(|| false);
    use_effect(use_reactive((&is_hl,), {
        let hl_cb = props.on_highlight;
        let unhl_cb = props.on_unhighlight;
        move |(current_hl,)| {
            let was_hl = *prev_hl.peek();
            if current_hl && !was_hl {
                prev_hl.set(true);
                if let Some(ref cb) = hl_cb {
                    cb.call(());
                }
            } else if !current_hl && was_hl {
                prev_hl.set(false);
                if let Some(ref cb) = unhl_cb {
                    cb.call(());
                }
            }
        }
    }));

    // Prevent mousedown/pointerdown focus transfer so `<input>` keeps DOM focus.
    let on_mouse_down = EventHandler::new(move |evt: MouseEvent| {
        evt.prevent_default();
    });

    let pd_rt = ctx.runtime.clone();
    let pd_val = props.value.clone();
    let on_pointer_down = EventHandler::new(move |evt: PointerEvent| {
        evt.prevent_default();
        pd_rt.set_suppress_next_click(false);
        if !is_dis {
            let coords = evt.client_coordinates();
            pd_rt.record_pressed_item(&pd_val, coords.x, coords.y);
        }
    });

    // DEF-CBX-07 / CBX-9: Enforce 4px drag threshold suppressing `onclick`, and prevent
    // double-selection on stationary scroll cross-item release.
    let pu_rt = ctx.runtime.clone();
    let pu_val = props.value.clone();
    let on_pointer_up = EventHandler::new(move |evt: PointerEvent| {
        if is_dis {
            return;
        }
        if let Some(press) = pu_rt.take_pressed_item() {
            let coords = evt.client_coordinates();
            let dx = coords.x - press.client_x;
            let dy = coords.y - press.client_y;
            let dist = dx.hypot(dy);
            if dist >= DRAG_THRESHOLD_PX {
                pu_rt.set_suppress_next_click(true);
            } else if press.value != pu_val {
                evt.prevent_default();
                pu_rt.set_suppress_next_click(true);
                pu_rt.select_item(&press.value);
            }
        }
    });

    let clk_rt = ctx.runtime.clone();
    let clk_val = props.value.clone();
    let on_click = EventHandler::new(move |evt: MouseEvent| {
        if is_dis {
            return;
        }
        evt.stop_propagation();
        if clk_rt.take_suppress_next_click() {
            return;
        }
        clk_rt.select_item(&clk_val);
    });

    let pm_rt = ctx.runtime.clone();
    let pm_val = props.value.clone();
    let on_pointer_move = EventHandler::new(move |evt: PointerEvent| {
        if is_dis || evt.pointer_type() == "touch" {
            return;
        }
        pm_rt.set_highlighted(Some(pm_val.clone()), false);
    });

    let body = if let Some(ref render_children) = props.render_children {
        render_children.call(render_state)
    } else {
        props.children
    };

    if props.as_child {
        if let Some(ref render_child) = props.render_child {
            return render_child.call(ComboboxItemRenderProps {
                attrs,
                state: render_state,
                class: props.class,
                style: props.style,
                on_mouse_down,
                on_pointer_down,
                on_pointer_up,
                on_click,
                on_pointer_move,
                attributes: props.attributes,
                children: body,
            });
        }
    }

    rsx! {
        div {
            id: "{attrs.id()}",
            role: "{attrs.role()}",
            "aria-selected": "{attrs.aria_selected()}",
            "aria-disabled": attrs.aria_disabled(),
            "data-combobox-item": "",
            "data-value": "{attrs.data_value()}",
            "data-label": "{attrs.data_label()}",
            "data-disabled": attrs.data_disabled(),
            "data-highlighted": attrs.data_highlighted(),
            "data-selected": attrs.data_selected(),
            class: props.class.as_deref().unwrap_or_default(),
            style: props.style.as_deref().unwrap_or_default(),
            onmousedown: move |evt| on_mouse_down.call(evt),
            onpointerdown: move |evt| on_pointer_down.call(evt),
            onpointerup: move |evt| on_pointer_up.call(evt),
            onclick: move |evt| on_click.call(evt),
            onpointermove: move |evt| on_pointer_move.call(evt),
            ..props.attributes,
            {body}
        }
    }
}

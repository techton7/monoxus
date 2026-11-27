use std::cell::RefCell;
use std::rc::Rc;

use dioxus::prelude::*;

use crate::combobox::{runtime::use_combobox_context, types::ComboboxItemRenderState};
use crate::foundation::compose::{
    collect_user_attrs, merge_props, push_attr_opt, push_event_handler,
};

const DRAG_THRESHOLD_PX: f64 = 4.0;

#[derive(Props, Clone, PartialEq)]
pub struct ComboboxItemProps {
    #[props(default, into)]
    pub id: Option<String>,
    #[props(into)]
    pub value: String,
    #[props(default, into)]
    pub label: Option<String>,
    #[props(default = false)]
    pub disabled: bool,
    #[props(default)]
    pub on_highlight: Option<EventHandler<()>>,
    #[props(default)]
    pub on_unhighlight: Option<EventHandler<()>>,
    #[props(default, into)]
    pub class: Option<String>,
    #[props(default, into)]
    pub style: Option<String>,
    #[props(default)]
    pub child: Option<Callback<Vec<Attribute>, Element>>,
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

    let id_str = attrs.id().to_owned();
    let role_str = attrs.role();
    let aria_selected = attrs.aria_selected();
    let aria_disabled = attrs.aria_disabled();
    let data_value = attrs.data_value().to_owned();
    let data_label = attrs.data_label().to_owned();
    let data_disabled = attrs.data_disabled();
    let data_highlighted = attrs.data_highlighted();
    let data_selected = attrs.data_selected();

    let mut base = Vec::with_capacity(15);
    base.push(Attribute::new("id", id_str, None, false));
    base.push(Attribute::new("role", role_str, None, false));
    base.push(Attribute::new("aria-selected", aria_selected, None, false));
    push_attr_opt(&mut base, "aria-disabled", aria_disabled);
    base.push(Attribute::new("data-combobox-item", "", None, false));
    base.push(Attribute::new("data-value", data_value, None, false));
    base.push(Attribute::new("data-label", data_label, None, false));
    push_attr_opt(&mut base, "data-disabled", data_disabled);
    push_attr_opt(&mut base, "data-highlighted", data_highlighted);
    push_attr_opt(&mut base, "data-selected", data_selected);
    push_event_handler(&mut base, "onmousedown", Some(on_mouse_down));
    push_event_handler(&mut base, "onpointerdown", Some(on_pointer_down));
    push_event_handler(&mut base, "onpointerup", Some(on_pointer_up));
    push_event_handler(&mut base, "onclick", Some(on_click));
    push_event_handler(&mut base, "onpointermove", Some(on_pointer_move));

    let user = collect_user_attrs(props.class, props.style, props.attributes);
    let merged = merge_props(base, user);

    if let Some(child) = props.child {
        return child.call(merged);
    }

    let body = if let Some(ref render_children) = props.render_children {
        render_children.call(render_state)
    } else {
        props.children
    };

    rsx! {
        div {
            ..merged,
            {body}
        }
    }
}

use std::{cell::RefCell, rc::Rc};

use dioxus::prelude::*;

use crate::foundation::{
    compose::{collect_user_attrs, merge_props},
    overlay::{
        FloatingArrowSpec, PlacementAlign, PlacementSide, compute_arrow_inline_style,
        render_floating_arrow,
    },
};
use crate::hover_card::components::HoverCardRootContext;

#[derive(Clone, PartialEq, Props)]
pub struct HoverCardArrowProps {
    #[props(default, into)]
    pub id: Option<String>,
    #[props(default, into)]
    pub class: Option<String>,
    #[props(default, into)]
    pub style: Option<String>,
    #[props(default = 10.0)]
    pub width: f32,
    #[props(default = 5.0)]
    pub height: f32,
    #[props(default)]
    pub child: Option<Callback<Vec<Attribute>, Element>>,
    #[props(extends = GlobalAttributes)]
    pub attributes: Vec<Attribute>,
    #[props(default)]
    pub children: Option<Element>,
}

#[component]
pub fn HoverCardArrow(props: HoverCardArrowProps) -> Element {
    let ctx = use_context::<HoverCardRootContext>();
    let runtime = ctx.runtime.read().clone();
    let arrow = runtime.arrow();
    let arrow_id = props.id.unwrap_or_else(|| arrow.id().to_string());

    let latest_aid = use_hook(|| Rc::new(RefCell::new(arrow_id.clone())));
    {
        let mut prev = latest_aid.borrow_mut();
        if *prev != arrow_id {
            runtime.unregister_arrow_id(&prev);
            *prev = arrow_id.clone();
        }
    }
    runtime.register_arrow_id(&arrow_id);
    let aid_for_drop = Rc::clone(&latest_aid);
    let rt_for_drop = runtime.clone();
    use_drop(move || {
        let current = aid_for_drop.borrow().clone();
        rt_for_drop.unregister_arrow_id(&current);
    });

    let placement = runtime.placement();
    let side = placement
        .as_ref()
        .map(|p| p.side())
        .unwrap_or_else(|| match arrow.data_side() {
            "top" => PlacementSide::Top,
            "right" => PlacementSide::Right,
            "left" => PlacementSide::Left,
            _ => PlacementSide::Bottom,
        });
    let align = placement
        .as_ref()
        .map(|p| p.align())
        .unwrap_or_else(|| match arrow.data_align() {
            "start" => PlacementAlign::Start,
            "end" => PlacementAlign::End,
            _ => PlacementAlign::Center,
        });
    let data_state = match arrow.data_state().as_str() {
        "open" => "open",
        _ => "closed",
    };
    let arrow_pos = placement.as_ref().map(|p| *p.arrow());
    let arrow_padding = runtime.effective_arrow_padding();

    if let Some(child) = props.child {
        if arrow_pos.is_some_and(|pos| pos.hidden()) {
            return rsx! {};
        }
        let pos_style = compute_arrow_inline_style(
            side,
            arrow_pos.map(|pos| (pos.x(), pos.y())),
            props.width,
            props.height,
            arrow_padding,
            true,
        );
        let base = vec![
            Attribute::new("id", arrow_id, None, false),
            Attribute::new("style", pos_style, None, false),
            Attribute::new("data-state", data_state, None, false),
            Attribute::new("data-side", side.as_str(), None, false),
            Attribute::new("data-align", align.as_str(), None, false),
            Attribute::new("aria-hidden", "true", None, false),
            Attribute::new("data-monoxus-hover-card-arrow", "", None, false),
        ];
        let user = collect_user_attrs(props.class, props.style, props.attributes);
        let merged = merge_props(base, user);
        return child.call(merged);
    }

    render_floating_arrow(FloatingArrowSpec {
        id: arrow_id,
        side,
        align,
        data_state,
        arrow: arrow_pos,
        width: props.width,
        height: props.height,
        arrow_padding,
        edge_offset: true,
        class: props.class,
        style: props.style,
        part_attr: Some("data-monoxus-hover-card-arrow"),
        attributes: props.attributes,
        children: props.children,
    })
}

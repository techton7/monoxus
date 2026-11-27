use std::{cell::RefCell, rc::Rc};

use dioxus::prelude::*;

use crate::foundation::{
    compose::{collect_user_attrs, merge_props, push_attr_opt},
    overlay::{
        FloatingArrowSpec, PlacementAlign, PlacementSide, compute_arrow_inline_style,
        render_floating_arrow,
    },
};
use crate::popover::components::PopoverContext;

#[derive(Props, Clone, PartialEq)]
pub struct PopoverArrowProps {
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
pub fn PopoverArrow(props: PopoverArrowProps) -> Element {
    let PopoverArrowProps {
        id,
        class,
        style,
        width,
        height,
        child,
        attributes,
        children,
    } = props;
    let ctx = use_context::<PopoverContext>();
    let runtime = ctx.runtime.read().clone();
    let arrow = runtime.arrow();
    let arrow_id = id.unwrap_or_else(|| arrow.id().to_string());

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

    if let Some(child) = child {
        if arrow_pos.is_some_and(|pos| pos.hidden()) {
            return rsx! {};
        }
        let pos_style = compute_arrow_inline_style(
            side,
            arrow_pos.map(|pos| (pos.x(), pos.y())),
            width,
            height,
            arrow_padding,
            false,
        );
        let mut base = Vec::with_capacity(6);
        base.push(Attribute::new("id", arrow_id, None, false));
        push_attr_opt(
            &mut base,
            "style",
            (!pos_style.is_empty()).then_some(pos_style),
        );
        base.push(Attribute::new("data-state", data_state, None, false));
        base.push(Attribute::new("data-side", side.as_str(), None, false));
        base.push(Attribute::new("data-align", align.as_str(), None, false));
        base.push(Attribute::new("aria-hidden", "true", None, false));

        let user = collect_user_attrs(class, style, attributes);
        let merged = merge_props(base, user);
        return child.call(merged);
    }

    render_floating_arrow(FloatingArrowSpec {
        id: arrow_id,
        side,
        align,
        data_state,
        arrow: arrow_pos,
        width,
        height,
        arrow_padding,
        edge_offset: false,
        class,
        style,
        part_attr: None,
        attributes,
        children,
    })
}

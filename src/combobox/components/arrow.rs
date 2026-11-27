use dioxus::prelude::*;

use crate::foundation::{
    compose::{collect_user_attrs, merge_props, push_attr_opt},
    overlay::{FloatingArrowSpec, compute_arrow_inline_style, render_floating_arrow},
    shared::use_stable_id,
};

use crate::combobox::runtime::use_combobox_context;

#[derive(Props, Clone, PartialEq)]
pub struct ComboboxArrowProps {
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
pub fn ComboboxArrow(props: ComboboxArrowProps) -> Element {
    let ctx = use_combobox_context();
    let arrow_id = use_stable_id(props.id.map(Into::into)).into_owned();

    ctx.runtime.register_arrow_id(&arrow_id);

    {
        let drop_rt = ctx.runtime.clone();
        let drop_id = arrow_id.clone();
        use_drop(move || {
            drop_rt.unregister_arrow_id(&drop_id);
        });
    }

    let placement = ctx.runtime.placement();
    let arrow_pos = placement.as_ref().map(|p| *p.arrow());
    let side = ctx.runtime.side();
    let align = ctx.runtime.align();
    let is_open = ctx.runtime.is_open();
    let data_state = if is_open { "open" } else { "closed" };
    let arrow_padding = ctx.runtime.arrow_padding();

    if let Some(child) = props.child {
        if arrow_pos.is_some_and(|p| p.hidden()) {
            return rsx! {};
        }
        let pos_style = compute_arrow_inline_style(
            side,
            arrow_pos.map(|p| (p.x(), p.y())),
            props.width,
            props.height,
            arrow_padding,
            false,
        );
        let mut base = Vec::with_capacity(7);
        base.push(Attribute::new("id", arrow_id, None, false));
        push_attr_opt(
            &mut base,
            "style",
            (!pos_style.is_empty()).then_some(pos_style),
        );
        base.push(Attribute::new("data-combobox-arrow", "", None, false));
        base.push(Attribute::new("data-state", data_state, None, false));
        base.push(Attribute::new("data-side", side.as_str(), None, false));
        base.push(Attribute::new("data-align", align.as_str(), None, false));
        base.push(Attribute::new("aria-hidden", "true", None, false));

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
        edge_offset: false,
        class: props.class,
        style: props.style,
        part_attr: Some("data-combobox-arrow"),
        attributes: props.attributes,
        children: props.children,
    })
}

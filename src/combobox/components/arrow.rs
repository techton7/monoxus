use dioxus::prelude::*;

use crate::foundation::{
    compose::{join_styles, merge_global_attributes},
    overlay::{
        FloatingArrowSpec, PlacementSide, compute_arrow_inline_style, render_floating_arrow,
    },
    shared::use_stable_id,
};

use crate::combobox::runtime::use_combobox_context;

#[derive(Clone, PartialEq)]
pub struct ComboboxArrowRenderProps {
    pub width: f32,
    pub height: f32,
    pub side: PlacementSide,
    pub class: Option<String>,
    pub style: String,
    pub attributes: Vec<Attribute>,
    pub children: Option<Element>,
}

#[derive(Props, Clone, PartialEq)]
pub struct ComboboxArrowProps {
    #[props(default)]
    pub id: Option<String>,
    #[props(default)]
    pub class: Option<String>,
    #[props(default)]
    pub style: Option<String>,
    #[props(default = 10.0)]
    pub width: f32,
    #[props(default = 5.0)]
    pub height: f32,
    #[props(default = false)]
    pub as_child: bool,
    #[props(default)]
    pub render_child: Option<Callback<ComboboxArrowRenderProps, Element>>,
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
    let arrow_padding = ctx.runtime.arrow_padding();

    if props.as_child
        && let Some(ref render_child) = props.render_child
    {
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
        let base_style = match props.style.as_deref() {
            Some(custom) => join_styles([pos_style.as_str(), custom]),
            None => Some(pos_style),
        };
        let merged = merge_global_attributes(
            props.class.as_deref(),
            base_style.as_deref(),
            props.attributes,
        );
        return render_child.call(ComboboxArrowRenderProps {
            width: props.width,
            height: props.height,
            side,
            class: merged.class,
            style: merged.style.unwrap_or_default(),
            attributes: merged.rest,
            children: props.children,
        });
    }

    render_floating_arrow(FloatingArrowSpec {
        id: arrow_id,
        side,
        align,
        data_state: if is_open { "open" } else { "closed" },
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

use dioxus::prelude::*;

use crate::foundation::{
    overlay::{FloatingArrowSpec, render_floating_arrow},
    shared::use_stable_id,
};

use super::types::SelectContext;

#[derive(Props, Clone, PartialEq)]
pub struct SelectArrowProps {
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
    #[props(extends = GlobalAttributes)]
    pub attributes: Vec<Attribute>,
    #[props(default)]
    pub children: Option<Element>,
}

#[component]
pub fn SelectArrow(props: SelectArrowProps) -> Element {
    let ctx = use_context::<SelectContext>();
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
    let side = ctx.runtime.side();
    let align = ctx.runtime.align();
    let is_open = ctx.runtime.is_open();

    render_floating_arrow(FloatingArrowSpec {
        id: arrow_id,
        side,
        align,
        data_state: if is_open { "open" } else { "closed" },
        arrow: placement.as_ref().map(|p| *p.arrow()),
        width: props.width,
        height: props.height,
        arrow_padding: ctx.runtime.arrow_padding(),
        edge_offset: false,
        class: props.class,
        style: props.style,
        part_attr: Some("data-monoxus-select-arrow"),
        attributes: props.attributes,
        children: props.children,
    })
}

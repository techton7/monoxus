use dioxus::prelude::*;

use crate::foundation::{
    overlay::{FloatingArrowSpec, render_floating_arrow},
    shared::use_stable_id,
};
use crate::menu::{MenuPart, MenuRootContext, MenuSurfaceContext};

#[derive(Props, Clone, PartialEq)]
pub struct MenuArrowProps {
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
    pub children: Option<Element>,
}

/// Optional floating arrow element anchored relative to the trigger / cursor.
#[component]
pub fn MenuArrow(props: MenuArrowProps) -> Element {
    let root = use_context::<MenuRootContext>();
    let surface_ctx = use_context::<MenuSurfaceContext>();
    let surface = surface_ctx.surface;
    let arrow_id = use_stable_id(props.id.map(Into::into)).into_owned();

    surface.register_arrow_id(&arrow_id);

    {
        let drop_surface = surface.clone();
        let drop_id = arrow_id.clone();
        use_drop(move || {
            drop_surface.unregister_arrow_id(&drop_id);
        });
    }

    let placement = surface.placement();
    let is_open = (surface.open)();
    let side = placement
        .as_ref()
        .map(|p| p.side())
        .unwrap_or_else(|| *surface.side.read());
    let align = placement
        .as_ref()
        .map(|p| p.align())
        .unwrap_or_else(|| *surface.align.read());

    render_floating_arrow(FloatingArrowSpec {
        id: arrow_id,
        side,
        align,
        data_state: if is_open { "open" } else { "closed" },
        arrow: placement.as_ref().map(|p| *p.arrow()),
        width: props.width,
        height: props.height,
        arrow_padding: *surface.arrow_padding.read(),
        edge_offset: false,
        class: props.class,
        style: props.style,
        part_attr: Some(MenuPart::Arrow.static_data_attr_name(root.variant)),
        attributes: props.attributes,
        children: props.children,
    })
}

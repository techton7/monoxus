use dioxus::prelude::*;

use crate::foundation::{
    compose::{collect_user_attrs, merge_props},
    overlay::{
        FloatingArrowSpec, compute_arrow_inline_style, render_floating_arrow,
    },
    shared::use_stable_id,
};
use crate::menu::{MenuPart, MenuRootContext, MenuSurfaceContext};

#[derive(Props, Clone, PartialEq)]
pub struct MenuArrowProps {
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
    let arrow = placement.as_ref().map(|p| *p.arrow());
    let data_state = if is_open { "open" } else { "closed" };
    let arrow_padding = *surface.arrow_padding.read();
    let part_attr = MenuPart::Arrow.static_data_attr_name(root.variant);

    if let Some(child) = props.child {
        if arrow.is_some_and(|pos| pos.hidden()) {
            return rsx! {};
        }
        let pos_style = compute_arrow_inline_style(
            side,
            arrow.map(|pos| (pos.x(), pos.y())),
            props.width,
            props.height,
            arrow_padding,
            false,
        );
        let base = vec![
            Attribute::new("id", arrow_id, None, false),
            Attribute::new("style", pos_style, None, false),
            Attribute::new("data-state", data_state, None, false),
            Attribute::new("data-side", side.as_str(), None, false),
            Attribute::new("data-align", align.as_str(), None, false),
            Attribute::new("aria-hidden", "true", None, false),
            Attribute::new(part_attr, "", None, false),
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
        arrow,
        width: props.width,
        height: props.height,
        arrow_padding,
        edge_offset: false,
        class: props.class,
        style: props.style,
        part_attr: Some(part_attr),
        attributes: props.attributes,
        children: props.children,
    })
}

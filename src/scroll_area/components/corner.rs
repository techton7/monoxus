use dioxus::prelude::*;

use crate::foundation::{
    compose::{collect_user_attrs, merge_props},
    shared::Direction,
};

use super::{
    super::{attrs::ScrollAreaCornerAttributes, state::has_corner},
    ScrollAreaContext,
};

#[derive(Clone, PartialEq, Props)]
pub struct ScrollAreaCornerProps {
    #[props(default, into)]
    pub id: Option<String>,
    #[props(default)]
    pub force_mount: bool,
    #[props(default)]
    pub children: Option<Element>,
    #[props(default, into)]
    pub class: Option<String>,
    #[props(default, into)]
    pub style: Option<String>,
    #[props(default)]
    pub child: Option<Callback<Vec<Attribute>, Element>>,
    #[props(extends = GlobalAttributes)]
    pub attributes: Vec<Attribute>,
}

#[component]
pub fn ScrollAreaCorner(props: ScrollAreaCornerProps) -> Element {
    let ctx = use_context::<ScrollAreaContext>();

    let corner_id = props
        .id
        .clone()
        .unwrap_or_else(|| ctx.relationships.read().corner_id().to_string());

    let scroll_type = ctx.scroll_type();
    let dir = ctx.dir();
    let has_both = ctx.runtime.scrollbar_x_enabled() && ctx.runtime.scrollbar_y_enabled();
    let has_corner_val = has_both
        && has_corner(
            scroll_type,
            ctx.runtime.scrollbar_x_visible(),
            ctx.runtime.scrollbar_y_visible(),
        );

    use_drop({
        let mut cw = ctx.runtime.corner_width;
        let mut ch = ctx.runtime.corner_height;
        move || {
            if let Ok(mut w) = cw.try_write() {
                *w = 0.0;
            }
            if let Ok(mut h) = ch.try_write() {
                *h = 0.0;
            }
        }
    });

    if !has_corner_val && !props.force_mount {
        return rsx! {};
    }

    let cw = ctx.runtime.corner_width();
    let ch = ctx.runtime.corner_height();

    let base_style = format!(
        "position: absolute; bottom: 0; width: {cw}px; height: {ch}px; right: {}; left: {};",
        if dir == Direction::Ltr { "0" } else { "auto" },
        if dir == Direction::Rtl { "0" } else { "auto" },
    );

    let corner_attrs = ScrollAreaCornerAttributes::new(&corner_id);

    let mut base = Vec::with_capacity(4);
    base.push(Attribute::new(
        "id",
        corner_attrs.id().to_string(),
        None,
        false,
    ));
    base.push(Attribute::new("style", base_style, None, false));
    base.push(Attribute::new(
        "data-state",
        corner_attrs.data_state(),
        None,
        false,
    ));
    base.push(Attribute::new(
        "data-monoxus-scroll-area-corner",
        "",
        None,
        false,
    ));

    let user = collect_user_attrs(props.class, props.style, props.attributes);
    let merged = merge_props(base, user);

    if let Some(child) = props.child {
        return child.call(merged);
    }

    rsx! {
        div {
            ..merged,
            {props.children}
        }
    }
}

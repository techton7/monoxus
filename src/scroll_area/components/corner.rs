use dioxus::prelude::*;

use crate::foundation::{
    compose::{join_styles, merge_global_attributes},
    shared::Direction,
};

use super::{
    super::{attrs::ScrollAreaCornerAttributes, state::has_corner},
    ScrollAreaContext,
};

#[derive(Clone, PartialEq, Props)]
pub struct ScrollAreaCornerProps {
    #[props(default)]
    pub id: Option<String>,
    #[props(default)]
    pub force_mount: bool,
    pub children: Option<Element>,
    pub class: Option<String>,
    pub style: Option<String>,
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
    let combined_style = join_styles([base_style.as_str(), props.style.as_deref().unwrap_or("")]);
    let merged = merge_global_attributes(
        props.class.as_deref(),
        combined_style.as_deref(),
        props.attributes,
    );

    let corner_attrs = ScrollAreaCornerAttributes::new(&corner_id);

    rsx! {
        div {
            id: "{corner_attrs.id()}",
            class: merged.class.as_deref(),
            style: merged.style.as_deref(),
            "data-state": "{corner_attrs.data_state()}",
            "data-monoxus-scroll-area-corner": "",
            ..merged.rest,
            {props.children}
        }
    }
}

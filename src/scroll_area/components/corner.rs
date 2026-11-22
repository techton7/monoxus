use dioxus::prelude::*;

use crate::foundation::shared::Direction;

use super::{
    super::{attrs::ScrollAreaCornerAttributes, state::has_corner},
    ScrollAreaContext,
};

#[derive(Clone, PartialEq, Props)]
pub struct ScrollAreaCornerProps {
    #[props(default)]
    pub id: Option<String>,
    pub as_child: Option<bool>,
    pub children: Option<Element>,
    pub class: Option<String>,
    pub style: Option<String>,
}

#[component]
pub fn ScrollAreaCorner(props: ScrollAreaCornerProps) -> Element {
    let ctx = use_context::<ScrollAreaContext>();

    let corner_id = props
        .id
        .clone()
        .unwrap_or_else(|| ctx.relationships.read().corner_id().to_string());

    let has_both = ctx.runtime.scrollbar_x_enabled() && ctx.runtime.scrollbar_y_enabled();
    let has_corner_val = has_both
        && has_corner(
            ctx.r#type,
            ctx.runtime.scrollbar_x_visible(),
            ctx.runtime.scrollbar_y_visible(),
        );

    use_drop({
        let mut cw = ctx.runtime.corner_width;
        let mut ch = ctx.runtime.corner_height;
        move || {
            cw.set(0.0);
            ch.set(0.0);
        }
    });

    if !has_corner_val && !props.as_child.unwrap_or(false) {
        return rsx! {};
    }

    let cw = ctx.runtime.corner_width();
    let ch = ctx.runtime.corner_height();

    let mut corner_style = format!(
        "position: absolute; bottom: 0; width: {cw}px; height: {ch}px; right: {}; left: {}; ",
        if ctx.dir == Direction::Ltr {
            "0"
        } else {
            "auto"
        },
        if ctx.dir == Direction::Rtl {
            "0"
        } else {
            "auto"
        },
    );
    if let Some(user_style) = &props.style {
        corner_style.push_str(user_style);
    }

    let corner_attrs = ScrollAreaCornerAttributes::new(&corner_id);

    rsx! {
        div {
            id: "{corner_attrs.id()}",
            class: props.class.as_deref(),
            style: "{corner_style}",
            "data-state": "{corner_attrs.data_state()}",
            "data-monoxus-scroll-area-corner": "",
            {props.children}
        }
    }
}

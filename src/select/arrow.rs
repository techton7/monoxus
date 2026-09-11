use dioxus::prelude::*;

use crate::foundation::overlay::PlacementSide;

use super::types::SelectContext;

#[component]
pub fn SelectArrow(
    #[props(default)] class: Option<String>,
    #[props(default)] style: Option<String>,
    #[props(default = 10.0)] width: f32,
    #[props(default = 5.0)] height: f32,
    children: Option<Element>,
) -> Element {
    let ctx = use_context::<SelectContext>();
    let placement = ctx.runtime.placement();
    let arrow_pad = ctx.runtime.arrow_padding();

    let arrow_pos = placement.as_ref().map(|p| p.arrow());
    let (arrow_x, arrow_y, is_hidden) = if let Some(pos) = arrow_pos {
        (pos.x(), pos.y(), pos.hidden())
    } else {
        (None, None, false)
    };

    if is_hidden {
        return rsx! {};
    }

    let side = ctx.runtime.side();
    let rotate_deg = match side {
        PlacementSide::Bottom => 0,
        PlacementSide::Top => 180,
        PlacementSide::Left => 90,
        PlacementSide::Right => -90,
    };

    let pos_style = match (arrow_x, arrow_y) {
        (Some(x), Some(y)) => format!("position: absolute; left: {}px; top: {}px; transform: rotate({}deg);", x, y, rotate_deg),
        (Some(x), None) => format!("position: absolute; left: {}px; transform: rotate({}deg);", x, rotate_deg),
        (None, Some(y)) => format!("position: absolute; top: {}px; transform: rotate({}deg);", y, rotate_deg),
        (None, None) => {
            if arrow_pad > 0.0 {
                format!("margin: 0 {}px; transform: rotate({}deg);", arrow_pad, rotate_deg)
            } else {
                format!("transform: rotate({}deg);", rotate_deg)
            }
        }
    };

    let merged_style = if let Some(custom) = style {
        format!("{pos_style} {custom}")
    } else {
        pos_style
    };

    if let Some(c) = children {
        return rsx! {
            div {
                class: class.as_deref().unwrap_or_default(),
                style: "{merged_style}",
                aria_hidden: "true",
                {c}
            }
        };
    }

    rsx! {
        svg {
            class: class.as_deref().unwrap_or_default(),
            style: "{merged_style}",
            width: "{width}",
            height: "{height}",
            view_box: "0 0 {width} {height}",
            "aria-hidden": "true",
            polygon {
                points: "0,{height} {width / 2.0},0 {width},{height}",
                fill: "currentColor",
            }
        }
    }
}

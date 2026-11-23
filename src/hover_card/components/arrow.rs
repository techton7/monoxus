use dioxus::prelude::*;

use crate::hover_card::components::HoverCardRootContext;

#[derive(Clone, PartialEq, Props)]
pub struct HoverCardArrowProps {
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
    pub children: Option<Element>,
}

#[component]
pub fn HoverCardArrow(props: HoverCardArrowProps) -> Element {
    let ctx = use_context::<HoverCardRootContext>();
    let runtime = ctx.runtime.read();
    let arrow = runtime.arrow();
    let arrow_id = props.id.unwrap_or_else(|| arrow.id().to_string());
    let placement = runtime.placement();

    let arrow_pos = placement.as_ref().map(|p| p.arrow());
    let (arrow_x, arrow_y, is_hidden) = if let Some(pos) = arrow_pos {
        (pos.x(), pos.y(), pos.hidden())
    } else {
        (None, None, false)
    };

    if is_hidden {
        return rsx! {};
    }

    let side_str = arrow.data_side();
    let rotate_deg = match side_str {
        "bottom" => 0,
        "top" => 180,
        "left" => 90,
        "right" => -90,
        _ => 0,
    };

    let edge_offset = match side_str {
        "bottom" => format!("top: -{}px;", props.height),
        "top" => format!("bottom: -{}px;", props.height),
        "left" => format!("right: -{}px;", props.height),
        "right" => format!("left: -{}px;", props.height),
        _ => String::new(),
    };

    let pos_style = match (arrow_x, arrow_y) {
        (Some(x), Some(y)) => format!(
            "position: absolute; left: {}px; top: {}px; transform: rotate({}deg);",
            x, y, rotate_deg
        ),
        (Some(x), None) => format!(
            "position: absolute; left: {}px; {} transform: rotate({}deg);",
            x, edge_offset, rotate_deg
        ),
        (None, Some(y)) => format!(
            "position: absolute; top: {}px; {} transform: rotate({}deg);",
            y, edge_offset, rotate_deg
        ),
        (None, None) => format!("transform: rotate({}deg);", rotate_deg),
    };

    let merged_style = if let Some(custom) = &props.style {
        format!("{pos_style} {custom}")
    } else {
        pos_style
    };

    if let Some(c) = props.children {
        return rsx! {
            div {
                id: "{arrow_id}",
                class: props.class.as_deref().unwrap_or_default(),
                style: "{merged_style}",
                "data-state": arrow.data_state().as_str(),
                "data-side": arrow.data_side(),
                "data-align": arrow.data_align(),
                "data-monoxus-hover-card-arrow": arrow.data_monoxus_hover_card_arrow(),
                "aria-hidden": "true",
                {c}
            }
        };
    }

    rsx! {
        svg {
            id: "{arrow_id}",
            class: props.class.as_deref().unwrap_or_default(),
            style: "{merged_style}",
            width: "{props.width}",
            height: "{props.height}",
            view_box: "0 0 {props.width} {props.height}",
            "data-state": arrow.data_state().as_str(),
            "data-side": arrow.data_side(),
            "data-align": arrow.data_align(),
            "data-monoxus-hover-card-arrow": arrow.data_monoxus_hover_card_arrow(),
            "aria-hidden": "true",
            polygon {
                points: "0,{props.height} {props.width / 2.0},0 {props.width},{props.height}",
                fill: "currentColor",
            }
        }
    }
}

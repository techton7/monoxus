use dioxus::prelude::*;

use crate::foundation::overlay::PlacementSide;

use crate::combobox::runtime::use_combobox_context;

#[derive(Props, Clone, PartialEq)]
pub struct ComboboxArrowProps {
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
pub fn ComboboxArrow(props: ComboboxArrowProps) -> Element {
    let ctx = use_combobox_context();
    let placement = ctx.runtime.placement();
    let arrow_pad = ctx.runtime.arrow_padding();

    let (arrow_x, arrow_y, is_hidden) = if let Some(ref p) = placement {
        let pos = p.arrow();
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
        (Some(x), Some(y)) => format!(
            "position: absolute; left: {x}px; top: {y}px; transform: rotate({rotate_deg}deg);"
        ),
        (Some(x), None) => {
            format!("position: absolute; left: {x}px; transform: rotate({rotate_deg}deg);")
        }
        (None, Some(y)) => {
            format!("position: absolute; top: {y}px; transform: rotate({rotate_deg}deg);")
        }
        (None, None) => {
            if arrow_pad > 0.0 {
                format!("margin: 0 {arrow_pad}px; transform: rotate({rotate_deg}deg);")
            } else {
                format!("transform: rotate({rotate_deg}deg);")
            }
        }
    };

    let merged_style = match props.style {
        Some(ref custom) if !custom.is_empty() => format!("{pos_style} {custom}"),
        _ => pos_style,
    };

    if let Some(c) = props.children {
        return rsx! {
            div {
                "aria-hidden": "true",
                "data-combobox-arrow": "",
                class: props.class.as_deref().unwrap_or_default(),
                style: "{merged_style}",
                ..props.attributes,
                {c}
            }
        };
    }

    rsx! {
        svg {
            "aria-hidden": "true",
            "data-combobox-arrow": "",
            class: props.class.as_deref().unwrap_or_default(),
            style: "{merged_style}",
            width: "{props.width}",
            height: "{props.height}",
            view_box: "0 0 {props.width} {props.height}",
            ..props.attributes,
            polygon {
                points: "0,{props.height} {props.width / 2.0},0 {props.width},{props.height}",
                fill: "currentColor",
            }
        }
    }
}

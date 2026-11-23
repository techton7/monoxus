use dioxus::prelude::*;

use crate::foundation::shared::use_stable_id;
use crate::menu::{
    MenuArrowAttributes, MenuPart, MenuRootContext, MenuSurfaceContext, merge_attributes_with_part,
};

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

    {
        let mut aid = surface.arrow_id;
        if aid.try_peek().is_ok_and(|g| g.as_deref() != Some(arrow_id.as_str())) {
            let _ = aid.try_write().map(|mut w| *w = Some(arrow_id.clone()));
        }
    }

    let placement = surface.placement.read().clone();
    let arrow_pos = placement.as_ref().map(|p| p.arrow());
    let (arrow_x, arrow_y, is_hidden) = if let Some(pos) = arrow_pos {
        (pos.x(), pos.y(), pos.hidden())
    } else {
        (None, None, false)
    };

    if is_hidden {
        return rsx! {};
    }

    let is_open = (surface.open)();
    let side_str = placement
        .as_ref()
        .map(|p| p.side().as_str())
        .unwrap_or_else(|| surface.side.read().as_str());
    let align_str = placement
        .as_ref()
        .map(|p| p.align().as_str())
        .unwrap_or_else(|| surface.align.read().as_str());

    let rotate_deg = match side_str {
        "bottom" => 0,
        "top" => 180,
        "left" => 90,
        "right" => -90,
        _ => 0,
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
        (None, None) => format!("transform: rotate({rotate_deg}deg);"),
    };

    let merged_style = match props.style.as_deref() {
        Some(custom) if !custom.is_empty() => format!("{pos_style} {custom}"),
        _ => pos_style,
    };

    let attrs = MenuArrowAttributes::new(arrow_id, is_open, side_str, align_str, root.variant);
    let merged_attrs = merge_attributes_with_part(
        props.attributes,
        MenuPart::Arrow.static_data_attr_name(root.variant),
    );

    if let Some(custom_children) = props.children {
        return rsx! {
            div {
                id: "{attrs.id}",
                class: props.class.as_deref().unwrap_or_default(),
                style: "{merged_style}",
                "data-state": "{attrs.data_state.as_str()}",
                "data-side": "{attrs.data_side}",
                "data-align": "{attrs.data_align}",
                "aria-hidden": "true",
                ..merged_attrs,
                {custom_children}
            }
        };
    }

    rsx! {
        svg {
            id: "{attrs.id}",
            class: props.class.as_deref().unwrap_or_default(),
            style: "{merged_style}",
            width: "{props.width}",
            height: "{props.height}",
            view_box: "0 0 {props.width} {props.height}",
            "data-state": "{attrs.data_state.as_str()}",
            "data-side": "{attrs.data_side}",
            "data-align": "{attrs.data_align}",
            "aria-hidden": "true",
            ..merged_attrs,
            polygon {
                points: "0,{props.height} {props.width / 2.0},0 {props.width},{props.height}",
                fill: "currentColor",
            }
        }
    }
}

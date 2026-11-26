use dioxus::prelude::*;

use crate::foundation::compose::{
    join_styles, merge_attributes_with_part, merge_global_attributes,
};

use super::floating::{
    FloatingArrowPosition, PlacementAlign, PlacementSide, compute_arrow_inline_style,
};

/// Plain Rust specification for [`render_floating_arrow`] (zero internal `#[component]` overhead,
/// compliant with `Dioxus #5894`).
pub(crate) struct FloatingArrowSpec {
    pub id: String,
    pub side: PlacementSide,
    pub align: PlacementAlign,
    pub data_state: &'static str,
    pub arrow: Option<FloatingArrowPosition>,
    pub width: f32,
    pub height: f32,
    pub arrow_padding: f32,
    pub edge_offset: bool,
    pub class: Option<String>,
    pub style: Option<String>,
    pub part_attr: Option<&'static str>,
    pub attributes: Vec<Attribute>,
    pub children: Option<Element>,
}

/// Plain Rust render helper for floating overlay arrows (`PopoverArrow`, `TooltipArrow`,
/// `HoverCardArrow`, `MenuArrow`, `SelectArrow`, `ComboboxArrow`) using
/// [`compute_arrow_inline_style`] and [`merge_global_attributes`].
pub(crate) fn render_floating_arrow(spec: FloatingArrowSpec) -> Element {
    if spec.arrow.is_some_and(|pos| pos.hidden()) {
        return rsx! {};
    }

    let pos_style = compute_arrow_inline_style(
        spec.side,
        spec.arrow.map(|pos| (pos.x(), pos.y())),
        spec.width,
        spec.height,
        spec.arrow_padding,
        spec.edge_offset,
    );

    let combined_base_style = match spec.style.as_deref() {
        Some(custom) => join_styles([pos_style.as_str(), custom]),
        None => Some(pos_style),
    };

    let attrs_with_part = match spec.part_attr {
        Some(part_attr) => merge_attributes_with_part(spec.attributes, part_attr),
        None => spec.attributes,
    };

    let merged = merge_global_attributes(
        spec.class.as_deref(),
        combined_base_style.as_deref(),
        attrs_with_part,
    );

    let side_str = spec.side.as_str();
    let align_str = spec.align.as_str();

    if let Some(children) = spec.children {
        return rsx! {
            div {
                id: "{spec.id}",
                class: merged.class.as_deref(),
                style: merged.style.as_deref(),
                "data-state": spec.data_state,
                "data-side": side_str,
                "data-align": align_str,
                "aria-hidden": "true",
                ..merged.rest,
                {children}
            }
        };
    }

    rsx! {
        svg {
            id: "{spec.id}",
            class: merged.class.as_deref(),
            style: merged.style.as_deref(),
            width: "{spec.width}",
            height: "{spec.height}",
            view_box: "0 0 {spec.width} {spec.height}",
            "data-state": spec.data_state,
            "data-side": side_str,
            "data-align": align_str,
            "aria-hidden": "true",
            ..merged.rest,
            polygon {
                points: "0,{spec.height} {spec.width / 2.0},0 {spec.width},{spec.height}",
                fill: "currentColor",
            }
        }
    }
}

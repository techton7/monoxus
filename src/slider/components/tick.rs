use dioxus::prelude::*;

use crate::foundation::compose::{join_styles, merge_global_attributes};

use crate::slider::runtime::use_slider_context;

/// Props for [`SliderTick`].
#[derive(Clone, PartialEq, Props)]
pub struct SliderTickProps {
    pub value: f64,
    #[props(default)]
    pub id: Option<String>,
    pub children: Option<Element>,
    pub class: Option<String>,
    pub style: Option<String>,
    #[props(extends = GlobalAttributes)]
    pub attributes: Vec<Attribute>,
}

/// Headless discrete tick indicator positioned along the slider track.
#[component]
pub fn SliderTick(props: SliderTickProps) -> Element {
    let ctx = use_slider_context();
    let attrs = ctx.0.tick_attrs(props.value);
    let tick_id = props.id.unwrap_or_else(|| attrs.id().to_string());

    let combined_style = join_styles([attrs.style(), props.style.as_deref().unwrap_or("")]);
    let merged = merge_global_attributes(
        props.class.as_deref(),
        combined_style.as_deref(),
        props.attributes,
    );

    rsx! {
        div {
            id: "{tick_id}",
            class: merged.class.as_deref().unwrap_or_default(),
            style: merged.style.as_deref().unwrap_or_default(),
            "data-orientation": "{attrs.data_orientation()}",
            "data-disabled": attrs.data_disabled(),
            "data-value": "{attrs.data_value()}",
            "data-bounded": attrs.data_bounded(),
            "data-selected": attrs.data_selected(),
            "data-monoxus-slider-tick": "",
            ..merged.rest,
            {props.children}
        }
    }
}

use dioxus::prelude::*;

use crate::foundation::compose::{join_styles, merge_global_attributes};

use crate::slider::{runtime::use_slider_context, types::SliderLabelPosition};

/// Props for [`SliderThumbLabel`].
#[derive(Clone, PartialEq, Props)]
pub struct SliderThumbLabelProps {
    #[props(default = 0)]
    pub index: usize,
    #[props(default)]
    pub position: Option<SliderLabelPosition>,
    #[props(default)]
    pub id: Option<String>,
    pub children: Option<Element>,
    pub class: Option<String>,
    pub style: Option<String>,
    #[props(extends = GlobalAttributes)]
    pub attributes: Vec<Attribute>,
}

/// Headless dynamic value bubble/tag tracking active thumb position.
#[component]
pub fn SliderThumbLabel(props: SliderThumbLabelProps) -> Element {
    let ctx = use_slider_context();
    let attrs = ctx.0.thumb_label_attrs(props.index, props.position);
    let label_id = props.id.unwrap_or_else(|| attrs.id().to_string());

    let combined_style = join_styles([attrs.style(), props.style.as_deref().unwrap_or("")]);
    let merged = merge_global_attributes(
        props.class.as_deref(),
        combined_style.as_deref(),
        props.attributes,
    );

    rsx! {
        div {
            id: "{label_id}",
            class: merged.class.as_deref().unwrap_or_default(),
            style: merged.style.as_deref().unwrap_or_default(),
            "data-orientation": "{attrs.data_orientation()}",
            "data-disabled": attrs.data_disabled(),
            "data-value": "{attrs.data_value()}",
            "data-active": attrs.data_active(),
            "data-position": "{attrs.data_position()}",
            "data-monoxus-slider-thumb-label": "",
            ..merged.rest,
            if let Some(ref children) = props.children {
                {children}
            } else {
                "{attrs.data_value()}"
            }
        }
    }
}

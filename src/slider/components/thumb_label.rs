use dioxus::prelude::*;

use crate::foundation::compose::{Slottable, project_as_child};

use crate::slider::{
    attrs::SliderThumbLabelAttributes, runtime::use_slider_context, types::SliderLabelPosition,
};

/// Projects child targets through [`AsChildSlot`] for `SliderThumbLabel`.
pub fn project_slider_thumb_label_as_child<T, C>(
    attrs: &SliderThumbLabelAttributes,
    target: T,
    content: Slottable<C>,
) -> (T, SliderThumbLabelAttributes, C) {
    let (target, content) = project_as_child(target, content);
    (target, attrs.clone(), content)
}

/// Props for [`SliderThumbLabel`].
#[derive(Clone, PartialEq, Props)]
pub struct SliderThumbLabelProps {
    #[props(default = 0)]
    pub index: usize,
    #[props(default)]
    pub position: Option<SliderLabelPosition>,
    #[props(default)]
    pub id: Option<String>,
    #[props(default)]
    pub as_child: Option<bool>,
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

    let combined_style = if let Some(ref s) = props.style {
        format!("{} {}", attrs.style(), s)
    } else {
        attrs.style().to_string()
    };

    rsx! {
        div {
            id: "{label_id}",
            class: props.class.as_deref().unwrap_or_default(),
            style: "{combined_style}",
            "data-orientation": "{attrs.data_orientation()}",
            "data-disabled": attrs.data_disabled(),
            "data-value": "{attrs.data_value()}",
            "data-active": attrs.data_active(),
            "data-position": "{attrs.data_position()}",
            "data-monoxus-slider-thumb-label": "",
            ..props.attributes,
            if let Some(ref children) = props.children {
                {children}
            } else {
                "{attrs.data_value()}"
            }
        }
    }
}

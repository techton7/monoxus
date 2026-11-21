use dioxus::prelude::*;

use crate::foundation::compose::{Slottable, project_as_child};

use crate::slider::{
    attrs::SliderTickLabelAttributes, runtime::use_slider_context, types::SliderLabelPosition,
};

/// Projects child targets through [`AsChildSlot`] for `SliderTickLabel`.
pub fn project_slider_tick_label_as_child<T, C>(
    attrs: &SliderTickLabelAttributes,
    target: T,
    content: Slottable<C>,
) -> (T, SliderTickLabelAttributes, C) {
    let (target, content) = project_as_child(target, content);
    (target, attrs.clone(), content)
}

/// Props for [`SliderTickLabel`].
#[derive(Clone, PartialEq, Props)]
pub struct SliderTickLabelProps {
    pub value: f64,
    #[props(default)]
    pub position: Option<SliderLabelPosition>,
    #[props(default)]
    pub id: Option<String>,
    #[props(default)]
    pub as_child: Option<bool>,
    pub children: Element,
    pub class: Option<String>,
    pub style: Option<String>,
    #[props(extends = GlobalAttributes)]
    pub attributes: Vec<Attribute>,
}

/// Headless label marking a discrete tick value along the slider track.
#[component]
pub fn SliderTickLabel(props: SliderTickLabelProps) -> Element {
    let ctx = use_slider_context();
    let attrs = ctx.0.tick_label_attrs(props.value, props.position);
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
            "data-bounded": attrs.data_bounded(),
            "data-selected": attrs.data_selected(),
            "data-position": "{attrs.data_position()}",
            "data-monoxus-slider-tick-label": "",
            ..props.attributes,
            {props.children}
        }
    }
}

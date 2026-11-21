use dioxus::prelude::*;

use crate::foundation::compose::{Slottable, project_as_child};

use crate::slider::{attrs::SliderTickAttributes, runtime::use_slider_context};

/// Projects child targets through [`AsChildSlot`] for `SliderTick`.
pub fn project_slider_tick_as_child<T, C>(
    attrs: &SliderTickAttributes,
    target: T,
    content: Slottable<C>,
) -> (T, SliderTickAttributes, C) {
    let (target, content) = project_as_child(target, content);
    (target, attrs.clone(), content)
}

/// Props for [`SliderTick`].
#[derive(Clone, PartialEq, Props)]
pub struct SliderTickProps {
    pub value: f64,
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

/// Headless discrete tick indicator positioned along the slider track.
#[component]
pub fn SliderTick(props: SliderTickProps) -> Element {
    let ctx = use_slider_context();
    let attrs = ctx.0.tick_attrs(props.value);
    let tick_id = props.id.unwrap_or_else(|| attrs.id().to_string());

    let combined_style = if let Some(ref s) = props.style {
        format!("{} {}", attrs.style(), s)
    } else {
        attrs.style().to_string()
    };

    rsx! {
        div {
            id: "{tick_id}",
            class: props.class.as_deref().unwrap_or_default(),
            style: "{combined_style}",
            "data-orientation": "{attrs.data_orientation()}",
            "data-disabled": attrs.data_disabled(),
            "data-value": "{attrs.data_value()}",
            "data-bounded": attrs.data_bounded(),
            "data-selected": attrs.data_selected(),
            "data-monoxus-slider-tick": "",
            ..props.attributes,
            {props.children}
        }
    }
}

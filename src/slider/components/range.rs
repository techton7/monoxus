use dioxus::prelude::*;

use crate::foundation::compose::{Slottable, join_styles, merge_global_attributes, project_as_child};

use crate::slider::{attrs::SliderRangeAttributes, runtime::use_slider_context};

/// Projects child targets through [`AsChildSlot`] for `SliderRange`.
pub fn project_slider_range_as_child<T, C>(
    attrs: &SliderRangeAttributes,
    target: T,
    content: Slottable<C>,
) -> (T, SliderRangeAttributes, C) {
    let (target, content) = project_as_child(target, content);
    (target, attrs.clone(), content)
}

/// Props for [`SliderRange`].
#[derive(Clone, PartialEq, Props)]
pub struct SliderRangeProps {
    #[props(default)]
    pub id: Option<String>,
    pub children: Option<Element>,
    pub class: Option<String>,
    pub style: Option<String>,
    #[props(extends = GlobalAttributes)]
    pub attributes: Vec<Attribute>,
}

/// Headless visual filled range indicator reflecting active span between slider bounds or thumbs.
#[component]
pub fn SliderRange(props: SliderRangeProps) -> Element {
    let ctx = use_slider_context();
    let attrs = ctx.0.range_attrs();
    let range_id = props.id.unwrap_or_else(|| attrs.id().to_string());

    let combined_style = join_styles([attrs.style(), props.style.as_deref().unwrap_or("")]);
    let merged = merge_global_attributes(
        props.class.as_deref(),
        combined_style.as_deref(),
        props.attributes,
    );

    rsx! {
        div {
            id: "{range_id}",
            class: merged.class.as_deref().unwrap_or_default(),
            style: merged.style.as_deref().unwrap_or_default(),
            "data-orientation": "{attrs.data_orientation()}",
            "data-disabled": attrs.data_disabled(),
            "data-monoxus-slider-range": "",
            ..merged.rest,
            {props.children}
        }
    }
}

use dioxus::prelude::*;

use crate::foundation::{
    browser::{ResizeObserverBox, use_element_size_with_mount},
    compose::{Slottable, project_as_child},
};

use crate::slider::{
    attrs::SliderTrackAttributes, runtime::use_slider_context, types::Orientation,
};

/// Projects child targets through [`AsChildSlot`] for `SliderTrack`.
pub fn project_slider_track_as_child<T, C>(
    attrs: &SliderTrackAttributes,
    target: T,
    content: Slottable<C>,
) -> (T, SliderTrackAttributes, C) {
    let (target, content) = project_as_child(target, content);
    (target, attrs.clone(), content)
}

/// Props for [`SliderTrack`].
#[derive(Clone, PartialEq, Props)]
pub struct SliderTrackProps {
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

/// Headless visual track container across which slider thumbs travel.
#[component]
pub fn SliderTrack(props: SliderTrackProps) -> Element {
    let ctx = use_slider_context();
    let attrs = ctx.0.track_attrs();
    let track_id = props.id.unwrap_or_else(|| attrs.id().to_string());

    use_effect(use_reactive((&track_id,), {
        let mut runtime = ctx.0.clone();
        move |(tid,)| {
            runtime.register_track_id(tid);
        }
    }));

    let mut mounted_rev = use_signal(|| 0u64);
    let track_size_sig =
        use_element_size_with_mount(&track_id, ResizeObserverBox::BorderBox, mounted_rev);
    use_effect({
        let mut runtime = ctx.0.clone();
        move || {
            if let Some(size) = *track_size_sig.read() {
                let s = match runtime.def.orientation {
                    Orientation::Vertical => size.height,
                    _ => size.width,
                };
                if runtime.track_size.peek().as_ref() != Some(&s) {
                    runtime.track_size.set(Some(s));
                }
            }
        }
    });

    let combined_style = if let Some(ref s) = props.style {
        format!("{}; {}", attrs.style(), s)
    } else {
        attrs.style().to_string()
    };

    rsx! {
        div {
            id: "{track_id}",
            class: props.class.as_deref().unwrap_or_default(),
            style: "{combined_style}",
            "data-orientation": "{attrs.data_orientation()}",
            "data-disabled": attrs.data_disabled(),
            "data-monoxus-slider-track": "",
            onmounted: move |_| mounted_rev += 1,
            ..props.attributes,
            {props.children}
        }
    }
}

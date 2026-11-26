use dioxus::prelude::*;

use crate::foundation::{
    browser::{ResizeObserverBox, use_element_size_with_mount},
    compose::{join_styles, merge_global_attributes},
    state::sync_signal,
};

use crate::slider::{runtime::use_slider_context, types::Orientation};

/// Props for [`SliderTrack`].
#[derive(Clone, PartialEq, Props)]
pub struct SliderTrackProps {
    #[props(default)]
    pub id: Option<String>,
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
            let orientation = runtime.def.read().orientation;
            if let Some(size) = *track_size_sig.read() {
                let s = match orientation {
                    Orientation::Vertical => size.height,
                    _ => size.width,
                };
                sync_signal(&mut runtime.track_size, Some(s));
            }
        }
    });

    let combined_style = join_styles([attrs.style(), props.style.as_deref().unwrap_or("")]);
    let merged = merge_global_attributes(
        props.class.as_deref(),
        combined_style.as_deref(),
        props.attributes,
    );

    rsx! {
        div {
            id: "{track_id}",
            class: merged.class.as_deref().unwrap_or_default(),
            style: merged.style.as_deref().unwrap_or_default(),
            "data-orientation": "{attrs.data_orientation()}",
            "data-disabled": attrs.data_disabled(),
            "data-monoxus-slider-track": "",
            onmounted: move |_| mounted_rev += 1,
            ..merged.rest,
            {props.children}
        }
    }
}

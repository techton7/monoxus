use dioxus::prelude::*;

use crate::foundation::{
    browser::{ResizeObserverBox, use_element_size_with_mount},
    compose::{collect_user_attrs, merge_props, push_attr_opt, push_event_handler},
    state::sync_signal,
};

use crate::slider::{runtime::use_slider_context, types::Orientation};

/// Props for [`SliderTrack`].
#[derive(Clone, PartialEq, Props)]
pub struct SliderTrackProps {
    #[props(default, into)]
    pub id: Option<String>,
    #[props(default)]
    pub child: Option<Callback<Vec<Attribute>, Element>>,
    pub children: Element,
    #[props(default, into)]
    pub class: Option<String>,
    #[props(default, into)]
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

    let base_style = attrs.style();
    let mut base = Vec::with_capacity(6);
    base.push(Attribute::new("id", track_id, None, false));
    if !base_style.is_empty() {
        base.push(Attribute::new("style", base_style, None, false));
    }
    base.push(Attribute::new(
        "data-orientation",
        attrs.data_orientation(),
        None,
        false,
    ));
    push_attr_opt(&mut base, "data-disabled", attrs.data_disabled());
    base.push(Attribute::new("data-monoxus-slider-track", "", None, false));
    push_event_handler(
        &mut base,
        "onmounted",
        Some(EventHandler::new(move |_evt: MountedEvent| {
            mounted_rev += 1;
        })),
    );

    let user = collect_user_attrs(props.class, props.style, props.attributes);
    let merged = merge_props(base, user);

    if let Some(child) = props.child {
        return child.call(merged);
    }

    rsx! {
        div {
            ..merged,
            {props.children}
        }
    }
}

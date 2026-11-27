use dioxus::prelude::*;

use crate::foundation::compose::{
    collect_user_attrs, merge_props, push_attr_opt, push_event_handler,
};

pub use crate::compat::aspect_ratio::*;

#[derive(Clone, PartialEq, Props)]
pub struct AspectRatioProps {
    #[props(default = 1.0)]
    pub ratio: f64,
    #[props(default)]
    pub child: Option<Callback<Vec<Attribute>, Element>>,
    #[props(default, into)]
    pub id: Option<String>,
    #[props(default, into)]
    pub class: Option<String>,
    #[props(default, into)]
    pub style: Option<String>,
    #[props(default)]
    pub on_click: Option<EventHandler<MouseEvent>>,
    #[props(default)]
    pub on_mounted: Option<EventHandler<MountedEvent>>,
    #[props(extends = GlobalAttributes)]
    pub attributes: Vec<Attribute>,
    #[props(default)]
    pub children: Element,
}

/// Renders a padding wrapper around an absolutely inset Root.
#[component]
pub fn AspectRatio(props: AspectRatioProps) -> Element {
    let state = AspectRatio::new(props.ratio);
    let attributes = state.attributes();

    let mut base = Vec::with_capacity(3);
    push_attr_opt(&mut base, "id", props.id);
    base.push(Attribute::new("style", attributes.root_style, None, false));
    base.push(Attribute::new(
        "data-aspect-ratio-root",
        attributes.data_aspect_ratio_root,
        None,
        false,
    ));

    let mut user = collect_user_attrs(props.class, props.style, props.attributes);
    push_event_handler(&mut user, "onclick", props.on_click);
    push_event_handler(&mut user, "onmounted", props.on_mounted);

    let merged = merge_props(base, user);
    let root = if let Some(child) = props.child {
        child.call(merged)
    } else {
        rsx! {
            div {
                ..merged,
                {props.children}
            }
        }
    };

    rsx! {
        div {
            style: attributes.wrapper_style,
            "data-radix-aspect-ratio-wrapper": attributes.data_radix_aspect_ratio_wrapper,
            "data-aspect-ratio-wrapper": attributes.data_aspect_ratio_wrapper,
            {root}
        }
    }
}

pub use AspectRatio as AspectRatioRoot;

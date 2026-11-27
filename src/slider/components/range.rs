use dioxus::prelude::*;

use crate::foundation::compose::{collect_user_attrs, merge_props, push_attr_opt};

use crate::slider::runtime::use_slider_context;

/// Props for [`SliderRange`].
#[derive(Clone, PartialEq, Props)]
pub struct SliderRangeProps {
    #[props(default, into)]
    pub id: Option<String>,
    #[props(default)]
    pub child: Option<Callback<Vec<Attribute>, Element>>,
    #[props(default)]
    pub children: Option<Element>,
    #[props(default, into)]
    pub class: Option<String>,
    #[props(default, into)]
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

    let base_style = attrs.style();
    let mut base = Vec::with_capacity(5);
    base.push(Attribute::new("id", range_id, None, false));
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
    base.push(Attribute::new("data-monoxus-slider-range", "", None, false));

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

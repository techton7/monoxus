use dioxus::prelude::*;

use crate::foundation::compose::{collect_user_attrs, merge_props, push_attr_opt};

use crate::slider::runtime::use_slider_context;

/// Props for [`SliderTick`].
#[derive(Clone, PartialEq, Props)]
pub struct SliderTickProps {
    pub value: f64,
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

/// Headless discrete tick indicator positioned along the slider track.
#[component]
pub fn SliderTick(props: SliderTickProps) -> Element {
    let ctx = use_slider_context();
    let attrs = ctx.0.tick_attrs(props.value);
    let tick_id = props.id.unwrap_or_else(|| attrs.id().to_string());

    let base_style = attrs.style();
    let mut base = Vec::with_capacity(8);
    base.push(Attribute::new("id", tick_id, None, false));
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
    base.push(Attribute::new(
        "data-value",
        attrs.data_value().to_string(),
        None,
        false,
    ));
    push_attr_opt(&mut base, "data-bounded", attrs.data_bounded());
    push_attr_opt(&mut base, "data-selected", attrs.data_selected());
    base.push(Attribute::new("data-monoxus-slider-tick", "", None, false));

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

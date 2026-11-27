use dioxus::prelude::*;

use crate::foundation::compose::{collect_user_attrs, merge_props, push_attr_opt};

use crate::slider::{runtime::use_slider_context, types::SliderLabelPosition};

/// Props for [`SliderThumbLabel`].
#[derive(Clone, PartialEq, Props)]
pub struct SliderThumbLabelProps {
    #[props(default = 0)]
    pub index: usize,
    #[props(default)]
    pub position: Option<SliderLabelPosition>,
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

/// Headless dynamic value bubble/tag tracking active thumb position.
#[component]
pub fn SliderThumbLabel(props: SliderThumbLabelProps) -> Element {
    let ctx = use_slider_context();
    let attrs = ctx.0.thumb_label_attrs(props.index, props.position);
    let label_id = props.id.unwrap_or_else(|| attrs.id().to_string());

    let base_style = attrs.style();
    let mut base = Vec::with_capacity(8);
    base.push(Attribute::new("id", label_id, None, false));
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
    push_attr_opt(&mut base, "data-active", attrs.data_active());
    base.push(Attribute::new(
        "data-position",
        attrs.data_position(),
        None,
        false,
    ));
    base.push(Attribute::new(
        "data-monoxus-slider-thumb-label",
        "",
        None,
        false,
    ));

    let user = collect_user_attrs(props.class, props.style, props.attributes);
    let merged = merge_props(base, user);

    if let Some(child) = props.child {
        return child.call(merged);
    }

    rsx! {
        div {
            ..merged,
            if let Some(ref children) = props.children {
                {children}
            } else {
                "{attrs.data_value()}"
            }
        }
    }
}

use dioxus::prelude::*;

use crate::foundation::compose::{
    collect_user_attrs, merge_props, push_attr_opt, push_event_handler,
};

pub use ui_sdk::meter::*;

/// Properties for the headless Meter component.
#[derive(Clone, PartialEq, Props)]
pub struct MeterProps {
    #[props(default, into)]
    pub id: Option<String>,
    #[props(default, into)]
    pub class: Option<String>,
    #[props(default, into)]
    pub style: Option<String>,
    #[props(default = 0.0)]
    pub min: f64,
    #[props(default = 0.0)]
    pub value: f64,
    #[props(default = 100.0)]
    pub max: f64,
    #[props(default)]
    pub child: Option<Callback<Vec<Attribute>, Element>>,
    #[props(default)]
    pub on_mounted: Option<EventHandler<MountedEvent>>,
    #[props(extends = GlobalAttributes)]
    pub attributes: Vec<Attribute>,
    #[props(default)]
    pub children: Element,
}

/// Headless meter publishing Bits-compatible ARIA and data values without validation.
#[component]
pub fn MeterRoot(props: MeterProps) -> Element {
    let meter = MeterState::new()
        .with_min(props.min)
        .with_value(props.value)
        .with_max(props.max);
    let attrs = meter.attributes();

    let mut base = Vec::with_capacity(9);
    push_attr_opt(&mut base, "id", props.id);
    base.push(Attribute::new("role", attrs.role, None, false));
    base.push(Attribute::new(
        "aria-valuemin",
        attrs.aria_valuemin,
        None,
        false,
    ));
    base.push(Attribute::new(
        "aria-valuenow",
        attrs.aria_valuenow,
        None,
        false,
    ));
    base.push(Attribute::new(
        "aria-valuemax",
        attrs.aria_valuemax,
        None,
        false,
    ));
    base.push(Attribute::new("data-min", attrs.data_min, None, false));
    base.push(Attribute::new("data-value", attrs.data_value, None, false));
    base.push(Attribute::new("data-max", attrs.data_max, None, false));
    base.push(Attribute::new(
        "data-meter-root",
        attrs.data_meter_root,
        None,
        false,
    ));

    let mut user = collect_user_attrs(props.class, props.style, props.attributes);
    push_event_handler(&mut user, "onmounted", props.on_mounted);

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

pub use MeterRoot as Meter;

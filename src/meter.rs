use dioxus::prelude::*;

use crate::foundation::compose::merge_global_attributes;

pub use ui_sdk::meter::*;

/// Values supplied to an explicit as-child renderer.
#[derive(Clone, Debug, PartialEq, Props)]
pub struct MeterRenderProps {
    #[props(default)]
    pub id: Option<String>,
    #[props(default)]
    pub class: Option<String>,
    #[props(default)]
    pub style: Option<String>,
    #[props(default)]
    pub aria_label: Option<String>,
    #[props(default)]
    pub aria_labelledby: Option<String>,
    #[props(default)]
    pub aria_valuetext: Option<String>,
    pub meter: MeterAttributes,
    #[props(default)]
    pub on_mounted: Option<EventHandler<MountedEvent>>,
    #[props(extends = GlobalAttributes)]
    pub attributes: Vec<Attribute>,
    pub children: Element,
}

/// Properties for the headless Meter component.
#[derive(Clone, PartialEq, Props)]
pub struct MeterProps {
    #[props(default)]
    pub id: Option<String>,
    #[props(default)]
    pub class: Option<String>,
    #[props(default)]
    pub style: Option<String>,
    #[props(default = 0.0)]
    pub min: f64,
    #[props(default = 0.0)]
    pub value: f64,
    #[props(default = 100.0)]
    pub max: f64,
    #[props(default)]
    pub aria_label: Option<String>,
    #[props(default)]
    pub aria_labelledby: Option<String>,
    #[props(default)]
    pub aria_valuetext: Option<String>,
    #[props(default = false)]
    pub as_child: bool,
    #[props(default)]
    pub render_child: Option<Callback<MeterRenderProps, Element>>,
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
    let children = props.children;

    if props.as_child {
        if let Some(render_child) = props.render_child {
            return render_child(MeterRenderProps {
                id: props.id,
                class: props.class,
                style: props.style,
                aria_label: props.aria_label,
                aria_labelledby: props.aria_labelledby,
                aria_valuetext: props.aria_valuetext,
                meter: attrs,
                on_mounted: props.on_mounted,
                attributes: props.attributes,
                children,
            });
        }
    }

    let merged = merge_global_attributes(
        props.class.as_deref(),
        props.style.as_deref(),
        props.attributes,
    );

    rsx! {
        div {
            id: props.id,
            class: merged.class,
            style: merged.style,
            role: attrs.role,
            aria_valuemin: attrs.aria_valuemin,
            aria_valuenow: attrs.aria_valuenow,
            aria_valuemax: attrs.aria_valuemax,
            aria_label: props.aria_label,
            aria_labelledby: props.aria_labelledby,
            aria_valuetext: props.aria_valuetext,
            "data-min": attrs.data_min,
            "data-value": attrs.data_value,
            "data-max": attrs.data_max,
            "data-meter-root": attrs.data_meter_root,
            onmounted: move |event| {
                if let Some(handler) = props.on_mounted.as_ref() {
                    handler.call(event);
                }
            },
            ..merged.rest,
            {children}
        }
    }
}

pub use MeterRoot as Meter;

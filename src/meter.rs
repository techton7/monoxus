use dioxus::prelude::*;

use crate::foundation::compose::{Slottable, project_as_child};

/// Pure, deliberately non-normalizing state for the Meter primitive.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct MeterState {
    min: f64,
    value: f64,
    max: f64,
}

impl Default for MeterState {
    fn default() -> Self {
        Self::new()
    }
}

impl MeterState {
    pub const fn new() -> Self {
        Self {
            min: 0.0,
            value: 0.0,
            max: 100.0,
        }
    }

    pub const fn with_min(mut self, min: f64) -> Self {
        self.min = min;
        self
    }

    pub const fn with_value(mut self, value: f64) -> Self {
        self.value = value;
        self
    }

    pub const fn with_max(mut self, max: f64) -> Self {
        self.max = max;
        self
    }

    pub const fn min(&self) -> f64 {
        self.min
    }

    pub const fn value(&self) -> f64 {
        self.value
    }

    pub const fn max(&self) -> f64 {
        self.max
    }

    pub const fn attributes(&self) -> MeterAttributes {
        MeterAttributes {
            role: "meter",
            aria_valuemin: self.min,
            aria_valuenow: self.value,
            aria_valuemax: self.max,
            data_min: self.min,
            data_value: self.value,
            data_max: self.max,
            data_meter_root: "",
        }
    }
}

/// WAI-ARIA and data attributes owned by the Meter root.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct MeterAttributes {
    pub role: &'static str,
    pub aria_valuemin: f64,
    pub aria_valuenow: f64,
    pub aria_valuemax: f64,
    pub data_min: f64,
    pub data_value: f64,
    pub data_max: f64,
    pub data_meter_root: &'static str,
}

/// Projects Meter state onto a slottable child without adding a wrapper element.
pub fn project_meter_as_child<T, C>(
    meter: &MeterState,
    target: T,
    content: Slottable<C>,
) -> (T, MeterAttributes, C) {
    let (target, content) = project_as_child(target, content);
    (target, meter.attributes(), content)
}

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

    rsx! {
        div {
            id: props.id,
            class: props.class,
            style: props.style,
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
            ..props.attributes,
            {children}
        }
    }
}

pub type Meter = MeterState;
pub use MeterRoot as Meter;

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn defaults_match_bits_meter_contract() {
        let attrs = Meter::new().attributes();
        assert_eq!(attrs.role, "meter");
        assert_eq!(attrs.aria_valuemin, 0.0);
        assert_eq!(attrs.aria_valuenow, 0.0);
        assert_eq!(attrs.aria_valuemax, 100.0);
        assert_eq!(attrs.data_min, 0.0);
        assert_eq!(attrs.data_value, 0.0);
        assert_eq!(attrs.data_max, 100.0);
        assert_eq!(attrs.data_meter_root, "");
    }

    #[test]
    fn values_are_published_without_clamping_or_validation() {
        let meter = Meter::new().with_min(20.0).with_value(-5.0).with_max(10.0);
        let attrs = meter.attributes();
        assert_eq!(attrs.data_min, 20.0);
        assert_eq!(attrs.data_value, -5.0);
        assert_eq!(attrs.data_max, 10.0);

        let non_finite = Meter::new()
            .with_min(f64::NEG_INFINITY)
            .with_value(f64::NAN)
            .with_max(f64::INFINITY);
        assert!(non_finite.min().is_infinite() && non_finite.min().is_sign_negative());
        assert!(non_finite.value().is_nan());
        assert!(non_finite.max().is_infinite() && non_finite.max().is_sign_positive());
    }

    #[test]
    fn as_child_projection_preserves_target_and_content() {
        let meter = Meter::new().with_value(7.5);
        let (target, attrs, content) =
            project_meter_as_child(&meter, "target", Slottable::new("content"));
        assert_eq!(target, "target");
        assert_eq!(content, "content");
        assert_eq!(attrs.role, "meter");
        assert_eq!(attrs.data_value, 7.5);
    }
}

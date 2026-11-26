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

pub type Meter = MeterState;

#[cfg(test)]
mod tests {
    use crate::foundation::compose::{Slottable, project_attrs_as_child};
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
            project_attrs_as_child(&meter.attributes(), "target", Slottable::new("content"));
        assert_eq!(target, "target");
        assert_eq!(content, "content");
        assert_eq!(attrs.role, "meter");
        assert_eq!(attrs.data_value, 7.5);
    }
}

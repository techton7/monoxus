use dioxus::prelude::*;
use monoxus::foundation::compose::Slottable;
#[allow(unused_imports)]
use monoxus::foundation::compose::project_attrs_as_child;
use monoxus::meter::{Meter, MeterRoot, MeterState};

#[component]
fn MeterDefaultsScenario() -> Element {
    rsx! {
        MeterRoot {
            id: "meter-default",
            aria_label: "Capacity".to_string(),
        }
    }
}

#[component]
fn MeterUnclampedScenario() -> Element {
    rsx! {
        MeterRoot {
            id: "meter-unclamped",
            min: 20.0,
            value: -7.5,
            max: 10.0,
            aria_labelledby: "meter-title".to_string(),
            aria_valuetext: "unchanged".to_string(),
            span { "probe" }
        }
    }
}

#[test]
fn meter_scenarios_render_cleanly() {
    let mut defaults = VirtualDom::new(MeterDefaultsScenario);
    defaults.rebuild_in_place();

    let mut unclamped = VirtualDom::new(MeterUnclampedScenario);
    unclamped.rebuild_in_place();
}

#[test]
fn meter_public_defaults_are_exact() {
    let attrs = Meter::new().attributes();
    assert_eq!(attrs, MeterState::new().attributes());
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
fn meter_public_values_react_without_normalization() {
    let initial = Meter::new().with_min(5.0).with_value(40.0).with_max(50.0);
    let updated = initial.with_min(25.0).with_value(400.0).with_max(10.0);
    assert_eq!(initial.attributes().data_value, 40.0);
    assert_eq!(updated.attributes().data_min, 25.0);
    assert_eq!(updated.attributes().data_value, 400.0);
    assert_eq!(updated.attributes().data_max, 10.0);
}

#[test]
fn meter_public_projection_supports_as_child() {
    let state = Meter::new().with_value(f64::NAN);
    let (target, attrs, content) =
        project_attrs_as_child(&state.attributes(), "custom-meter", Slottable::new("reading"));
    assert_eq!(target, "custom-meter");
    assert_eq!(content, "reading");
    assert!(attrs.aria_valuenow.is_nan());
    assert!(attrs.data_value.is_nan());
}

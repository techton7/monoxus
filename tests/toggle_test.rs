use std::cell::RefCell;
use std::rc::Rc;

use dioxus::prelude::*;
#[allow(unused_imports)]
use monoxus::{
    foundation::{compose::Slottable, state::DataState},
    toggle::{project_toggle_as_child, ButtonType, Toggle, ToggleAttributes, ToggleRoot},
};

#[cfg(feature = "native")]
mod common;

#[test]
fn test_toggle_state_pure_matrix() {
    let mut toggle = Toggle::new();
    assert!(!toggle.is_pressed());
    assert!(!toggle.is_disabled());
    assert_eq!(toggle.button_type(), ButtonType::Button);

    let attrs = toggle.attributes();
    assert_eq!(attrs.r#type, "button");
    assert_eq!(attrs.aria_pressed, "false");
    assert_eq!(attrs.data_state, DataState::Off);
    assert_eq!(attrs.data_state_str(), "off");
    assert_eq!(attrs.data_disabled, None);
    assert_eq!(attrs.data_toggle_root, "");
    assert!(!attrs.disabled);

    // Toggle on
    toggle.toggle();
    assert!(toggle.is_pressed());
    let on_attrs = toggle.attributes();
    assert_eq!(on_attrs.aria_pressed, "true");
    assert_eq!(on_attrs.data_state, DataState::On);
    assert_eq!(on_attrs.data_state_str(), "on");
    assert_eq!(on_attrs.data_disabled, None);

    // Toggle off
    toggle.toggle();
    assert!(!toggle.is_pressed());
    assert_eq!(toggle.attributes().aria_pressed, "false");

    // Disabled toggle
    let mut disabled_toggle = Toggle::new().with_disabled(true);
    assert!(disabled_toggle.is_disabled());
    let dis_attrs = disabled_toggle.attributes();
    assert_eq!(dis_attrs.data_disabled, Some(""));
    assert!(dis_attrs.disabled);

    disabled_toggle.toggle();
    assert!(!disabled_toggle.is_pressed());
}

#[test]
fn test_project_toggle_as_child_projection() {
    let toggle = Toggle::new().with_pressed(true);
    let slottable = Slottable::new("Slotted child content");
    let (target, attrs, content) = project_toggle_as_child(&toggle, "custom-button", slottable);

    assert_eq!(target, "custom-button");
    assert_eq!(content, "Slotted child content");
    assert_eq!(attrs.aria_pressed, "true");
    assert_eq!(attrs.data_state, DataState::On);
    assert_eq!(attrs.data_disabled, None);
}

#[component]
fn UncontrolledToggleApp() -> Element {
    rsx! {
        ToggleRoot {
            id: "uncontrolled-toggle",
            default_pressed: false,
            "Bookmark"
        }
    }
}

#[test]
fn test_uncontrolled_toggle_vdom_mounts() {
    let mut dom = VirtualDom::new(UncontrolledToggleApp);
    dom.rebuild_in_place();
}

#[component]
fn ControlledToggleApp(pressed: Rc<RefCell<bool>>, change_count: Rc<RefCell<u32>>) -> Element {
    let p = *pressed.borrow();
    let change_count = change_count.clone();

    rsx! {
        ToggleRoot {
            id: "controlled-toggle",
            pressed: p,
            on_pressed_change: move |_next: bool| {
                *change_count.borrow_mut() += 1;
            },
            "Controlled"
        }
    }
}

#[test]
fn test_controlled_toggle_vdom() {
    let pressed = Rc::new(RefCell::new(false));
    let change_count = Rc::new(RefCell::new(0));

    let mut dom = VirtualDom::new_with_props(
        ControlledToggleApp,
        ControlledToggleAppProps {
            pressed: pressed.clone(),
            change_count: change_count.clone(),
        },
    );
    dom.rebuild_in_place();
    assert_eq!(*change_count.borrow(), 0);
}

#[component]
fn DisabledToggleApp() -> Element {
    rsx! {
        ToggleRoot {
            id: "disabled-toggle",
            disabled: true,
            "Disabled Item"
        }
    }
}

#[test]
fn test_disabled_toggle_vdom_renders() {
    let mut dom = VirtualDom::new(DisabledToggleApp);
    dom.rebuild_in_place();
}

#[cfg(feature = "native")]
mod native_tests {
    use super::*;
    use common::native_fixture::*;

    #[component]
    fn NativeToggleClickApp() -> Element {
        let mut clicks = use_signal(|| 0u32);
        rsx! {
            div {
                id: "click-probe",
                "data-clicks": "{clicks}",
                ToggleRoot {
                    id: "native-toggle",
                    on_click: move |_| {
                        clicks += 1;
                    },
                    "Toggle Me"
                }
            }
        }
    }

    #[test]
    fn test_native_toggle_click_and_disabled() {
        let mut harness = mount_native_scenario(NativeToggleClickApp);

        let id = harness.query("#native-toggle").expect("Toggle mounted");
        {
            let base = harness.base();
            let node = base.get_node(id).expect("Node exists");
            assert_eq!(node.attr("aria-pressed".into()), Some("false"));
            assert_eq!(node.attr("data-state".into()), Some("off"));
            assert_eq!(node.attr("data-disabled".into()), None);
        }

        click_and_pump(&mut harness, "#native-toggle");
        let probe_id = harness.node("#click-probe");
        assert_eq!(
            harness.base().get_node(probe_id).unwrap().attr("data-clicks".into()),
            Some("1")
        );
        assert_eq!(
            harness.base().get_node(id).unwrap().attr("aria-pressed".into()),
            Some("true")
        );
        assert_eq!(
            harness.base().get_node(id).unwrap().attr("data-state".into()),
            Some("on")
        );
    }
}

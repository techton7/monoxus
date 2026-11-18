use std::cell::RefCell;
use std::rc::Rc;

use dioxus::prelude::*;
use monoxus::checkbox::{Checkbox, CheckboxIndicator, CheckedState};

#[cfg(feature = "native")]
use crate::common;

#[component]
fn UncontrolledCheckboxInteractionApp() -> Element {
    rsx! {
        div {
            Checkbox {
                id: "cb-uncontrolled",
                default_checked: CheckedState::Unchecked,
                CheckboxIndicator { id: "ind-uncontrolled", "✓" }
            }
            Checkbox {
                id: "cb-from-indeterminate",
                default_checked: CheckedState::Indeterminate,
                CheckboxIndicator { id: "ind-from-ind", "-" }
            }
            Checkbox {
                id: "cb-dis-test",
                default_checked: CheckedState::Unchecked,
                disabled: true,
                CheckboxIndicator { id: "ind-dis", "✓" }
            }
            Checkbox {
                id: "cb-ro-test",
                default_checked: CheckedState::Unchecked,
                read_only: true,
                CheckboxIndicator { id: "ind-ro", "✓" }
            }
        }
    }
}

#[test]
fn test_uncontrolled_checkbox_vdom_mounts() {
    let mut dom = VirtualDom::new(UncontrolledCheckboxInteractionApp);
    dom.rebuild_in_place();
}

#[component]
fn ControlledCheckboxApp(
    checked_state: Signal<CheckedState>,
    change_history: Rc<RefCell<Vec<CheckedState>>>,
) -> Element {
    let history = change_history.clone();
    rsx! {
        Checkbox {
            id: "cb-controlled",
            checked: *checked_state.read(),
            on_checked_change: move |next: CheckedState| {
                history.borrow_mut().push(next);
            },
            CheckboxIndicator { id: "ind-controlled", "✓" }
        }
    }
}

#[component]
fn ControlledCheckboxTestWrapper() -> Element {
    let checked_state = use_signal(|| CheckedState::Checked);
    let history = use_hook(|| Rc::new(RefCell::new(Vec::new())));
    rsx! {
        ControlledCheckboxApp {
            checked_state,
            change_history: history,
        }
    }
}

#[component]
fn AnimatedCheckboxPresenceApp() -> Element {
    rsx! {
        style {
            "@keyframes checkbox-exit {{ from {{ opacity: 1; }} to {{ opacity: 0; }} }}"
            "#checkbox-animated-indicator[data-state=\"unchecked\"] {{ animation: checkbox-exit 200ms linear; }}"
        }
        Checkbox {
            id: "checkbox-animated",
            default_checked: CheckedState::Unchecked,
            CheckboxIndicator {
                id: "checkbox-animated-indicator",
                "✓"
            }
        }
    }
}

#[test]
fn test_controlled_checkbox_vdom_mounts() {
    let mut dom = VirtualDom::new(ControlledCheckboxTestWrapper);
    dom.rebuild_in_place();
}

#[cfg(feature = "native")]
mod native_tests {
    use super::*;
    use common::native_fixture::*;
    use keyboard_types::{Code, Key, NamedKey};

    #[test]
    fn test_native_checkbox_real_keyframe_exit_retains_until_animation_end() {
        let _lock = NATIVE_TEST_LOCK.lock().unwrap_or_else(|e| e.into_inner());
        let mut harness = mount_native_scenario(AnimatedCheckboxPresenceApp);
        advance_frame_and_pump(&mut harness, 0.0);

        click_and_pump(&mut harness, "#checkbox-animated");
        click_and_pump(&mut harness, "#checkbox-animated");
        advance_frame_and_pump(&mut harness, 0.0);
        advance_frame_and_pump(&mut harness, 0.05);
        assert!(
            harness.query("#checkbox-animated-indicator").is_some(),
            "indicator must remain mounted at 50ms of a 200ms native keyframe exit"
        );

        advance_frame_and_pump(&mut harness, 0.21);
        assert!(
            harness.query("#checkbox-animated-indicator").is_none(),
            "indicator must unmount after the real 200ms animation completes"
        );
    }

    #[test]
    fn test_native_checkbox_click_toggle_and_from_indeterminate() {
        let _lock = NATIVE_TEST_LOCK.lock().unwrap_or_else(|e| e.into_inner());
        let mut harness = mount_native_scenario(UncontrolledCheckboxInteractionApp);
        advance_frame_and_pump(&mut harness, 0.016);

        // 1. Uncontrolled: Unchecked -> click -> Checked -> click -> Unchecked
        let cb_id = harness.node("#cb-uncontrolled");
        assert_eq!(
            harness.base().get_node(cb_id).unwrap().attr("aria-checked".into()),
            Some("false")
        );
        assert!(harness.query("#ind-uncontrolled").is_none());

        click_and_pump(&mut harness, "#cb-uncontrolled");
        assert_eq!(
            harness.base().get_node(cb_id).unwrap().attr("aria-checked".into()),
            Some("true")
        );
        assert!(harness.query("#ind-uncontrolled").is_some());

        click_and_pump(&mut harness, "#cb-uncontrolled");
        assert_eq!(
            harness.base().get_node(cb_id).unwrap().attr("aria-checked".into()),
            Some("false")
        );
        // With no exit animation, the next resolved frame completes the close cycle immediately.
        advance_frame_and_pump(&mut harness, 0.016);
        assert!(harness.query("#ind-uncontrolled").is_none(), "Indicator unmounted after exit animation");

        // 2. From Indeterminate -> click -> Checked -> click -> Unchecked
        let ind_cb_id = harness.node("#cb-from-indeterminate");
        assert_eq!(
            harness.base().get_node(ind_cb_id).unwrap().attr("aria-checked".into()),
            Some("mixed")
        );
        assert!(harness.query("#ind-from-ind").is_some());

        click_and_pump(&mut harness, "#cb-from-indeterminate");
        assert_eq!(
            harness.base().get_node(ind_cb_id).unwrap().attr("aria-checked".into()),
            Some("true")
        );

        click_and_pump(&mut harness, "#cb-from-indeterminate");
        assert_eq!(
            harness.base().get_node(ind_cb_id).unwrap().attr("aria-checked".into()),
            Some("false")
        );
        advance_frame_and_pump(&mut harness, 0.016);
        assert!(harness.query("#ind-from-ind").is_none());
    }

    #[test]
    fn test_native_checkbox_enter_suppression_vs_space_toggle() {
        let _lock = NATIVE_TEST_LOCK.lock().unwrap_or_else(|e| e.into_inner());
        let mut harness = mount_native_scenario(UncontrolledCheckboxInteractionApp);
        advance_frame_and_pump(&mut harness, 0.016);

        let cb_id = harness.node("#cb-uncontrolled");
        focus_node(&mut harness, "#cb-uncontrolled");

        // 1. Enter key must NOT toggle checkbox (CB-6: Enter key suppression)
        send_key_and_pump(&mut harness, Key::Named(NamedKey::Enter), Code::Enter);
        assert_eq!(
            harness.base().get_node(cb_id).unwrap().attr("aria-checked".into()),
            Some("false"),
            "Enter key must be suppressed on Checkbox"
        );

        // 2. Space key DOES toggle checkbox (CB-7: Space KeyUp synthesizes click)
        send_key_and_pump(&mut harness, Key::Character(" ".into()), Code::Space);
        assert_eq!(
            harness.base().get_node(cb_id).unwrap().attr("aria-checked".into()),
            Some("true"),
            "Space key must toggle Checkbox"
        );

        // Enter again: still suppressed
        send_key_and_pump(&mut harness, Key::Named(NamedKey::Enter), Code::Enter);
        assert_eq!(
            harness.base().get_node(cb_id).unwrap().attr("aria-checked".into()),
            Some("true"),
            "Enter key remains suppressed when checked"
        );

        // Space again: toggles back to false
        send_key_and_pump(&mut harness, Key::Character(" ".into()), Code::Space);
        assert_eq!(
            harness.base().get_node(cb_id).unwrap().attr("aria-checked".into()),
            Some("false")
        );
    }

    #[test]
    fn test_native_checkbox_disabled_and_readonly_guard() {
        let _lock = NATIVE_TEST_LOCK.lock().unwrap_or_else(|e| e.into_inner());
        let mut harness = mount_native_scenario(UncontrolledCheckboxInteractionApp);
        advance_frame_and_pump(&mut harness, 0.016);

        // Disabled
        let dis_id = harness.node("#cb-dis-test");
        assert_eq!(
            harness.base().get_node(dis_id).unwrap().attr("aria-checked".into()),
            Some("false")
        );
        click_and_pump(&mut harness, "#cb-dis-test");
        assert_eq!(
            harness.base().get_node(dis_id).unwrap().attr("aria-checked".into()),
            Some("false")
        );

        // Read-only
        let ro_id = harness.node("#cb-ro-test");
        assert_eq!(
            harness.base().get_node(ro_id).unwrap().attr("aria-checked".into()),
            Some("false")
        );
        click_and_pump(&mut harness, "#cb-ro-test");
        assert_eq!(
            harness.base().get_node(ro_id).unwrap().attr("aria-checked".into()),
            Some("false")
        );
    }

    #[component]
    fn LabelForwardingApp() -> Element {
        rsx! {
            div {
                label {
                    id: "cb-label",
                    r#for: "cb-labeled",
                    "Terms and conditions"
                }
                Checkbox {
                    id: "cb-labeled",
                    default_checked: CheckedState::Unchecked,
                    CheckboxIndicator { id: "ind-labeled", "✓" }
                }
            }
        }
    }

    #[test]
    fn test_native_checkbox_label_focus_forwarding() {
        let _lock = NATIVE_TEST_LOCK.lock().unwrap_or_else(|e| e.into_inner());
        let mut harness = mount_native_scenario(LabelForwardingApp);
        advance_frame_and_pump(&mut harness, 0.016);

        // Click the label
        click_and_pump(&mut harness, "#cb-label");

        // Blitz Native labelable element click forwards focus to the target <button> with :focus-visible false
        assert_native_focus_id(&harness, "cb-labeled");
        assert_native_focus_visible(&harness, "cb-labeled", false);
    }

    #[component]
    fn OnClickCancellationApp() -> Element {
        rsx! {
            Checkbox {
                id: "cb-prevented",
                default_checked: CheckedState::Unchecked,
                on_click: move |evt: MouseEvent| {
                    evt.prevent_default();
                },
                CheckboxIndicator { id: "ind-prevented", "✓" }
            }
        }
    }

    #[test]
    fn test_native_checkbox_consumer_on_click_cancellation() {
        let _lock = NATIVE_TEST_LOCK.lock().unwrap_or_else(|e| e.into_inner());
        let mut harness = mount_native_scenario(OnClickCancellationApp);
        advance_frame_and_pump(&mut harness, 0.016);

        let cb_id = harness.node("#cb-prevented");
        click_and_pump(&mut harness, "#cb-prevented");
        assert_eq!(
            harness.base().get_node(cb_id).unwrap().attr("aria-checked".into()),
            Some("false"),
            "Consumer on_click evt.prevent_default() must suppress state change"
        );
    }
}

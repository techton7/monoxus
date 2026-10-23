//! Native Presence Exit Animation Dogfooding Test
//!
//! Verifies that Monoxus overlay presence unmount is driven directly by native
//! CSS AnimationEvent (onanimationend) in Blitz Native without thread::sleep fallbacks.

mod common;
use common::native_fixture::*;
use dioxus::prelude::*;
use monoxus::foundation::overlay::{Presence, PresenceState};

static TEST_LOCK: std::sync::Mutex<()> = std::sync::Mutex::new(());

#[test]
fn test_native_presence_exit_animation_unmount() {
    let _lock = TEST_LOCK.lock().unwrap_or_else(|e| e.into_inner());

    fn presence_scenario() -> Element {
        let mut presence = use_signal(|| Presence::new(true).with_retained_mount(true));

        rsx! {
            style {
                "@keyframes exit-fade {{ from {{ opacity: 1; }} to {{ opacity: 0; }} }}"
            }
            if presence.read().is_mounted() {
                div {
                    id: "exit-target",
                    style: if presence.read().state() == PresenceState::Suspended {
                        "animation: exit-fade 0.2s linear forwards;"
                    } else {
                        "opacity: 1;"
                    },
                    onanimationend: move |_| {
                        // Direct synchronous unmount triggered by native onanimationend
                        presence.write().complete_unmount();
                    },
                    button {
                        id: "close-trigger",
                        onclick: move |_| {
                            presence.write().sync(false);
                        },
                        "Close"
                    }
                }
            }
        }
    }

    let mut harness = mount_native_scenario(presence_scenario);
    assert!(
        harness.query("#exit-target").is_some(),
        "Initial mount confirmed"
    );

    // Step 1: Click close trigger -> presence transitions to Suspended, node retained for exit animation
    click_and_pump(&mut harness, "#close-trigger");
    assert!(
        harness.query("#exit-target").is_some(),
        "Node retained during exit animation"
    );

    // Step 2: Advance simulated animation clock past 0.2s duration (zero thread::sleep!)
    harness.doc.inner.borrow_mut().resolve(0.25);
    harness.doc.flush_animation_events();
    harness.pump();

    // Step 3: Verify node is completely unmounted via native onanimationend event
    assert!(
        harness.query("#exit-target").is_none(),
        "Node unmounted on native animationend"
    );
}

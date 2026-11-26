//! Native Presence Exit Animation Dogfooding Test
//!
//! Verifies that Monoxus overlay presence unmount is driven directly by native
//! CSS AnimationEvent (onanimationend) in Blitz Native without thread::sleep fallbacks.

use crate::common;
use common::native_fixture::*;
use dioxus::prelude::*;
use monoxus::foundation::overlay::{Presence, PresenceState, use_presence};


#[test]
fn test_native_presence_exit_animation_unmount() {
    let _lock = NATIVE_TEST_LOCK.lock().unwrap_or_else(|e| e.into_inner());

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

#[test]
fn fm_w1_guard_presence_handle_restores_animation_fill_mode_and_syncs_retain_mount() {
    let _lock = NATIVE_TEST_LOCK.lock().unwrap_or_else(|e| e.into_inner());

    fn presence_restore_scenario() -> Element {
        let mut open_a = use_signal(|| true);
        let mut open_b = use_signal(|| true);
        let mut open_c = use_signal(|| true);
        let mut retain_c = use_signal(|| true);

        let pres_a = use_presence(open_a(), "presence-prior-fill", true, None);
        let pres_b = use_presence(open_b(), "presence-no-fill", true, None);
        let pres_c = use_presence(open_c(), "presence-retain-sync", retain_c(), None);

        let state_str = |s: PresenceState| match s {
            PresenceState::Unmounted => "unmounted",
            PresenceState::Mounted => "mounted",
            PresenceState::Suspended => "suspended",
        };

        rsx! {
            style {
                "@keyframes pres-exit-fade {{ from {{ opacity: 1; }} to {{ opacity: 0; }} }}
                 .pres-exiting {{ animation: pres-exit-fade 0.2s linear; }}"
            }
            button {
                id: "close-fill-targets",
                onclick: move |_| {
                    open_a.set(false);
                    open_b.set(false);
                },
                "Close Fill Targets"
            }
            // Retained force_mount-style nodes stay in DOM across exit so we can verify restoration.
            div {
                id: "presence-prior-fill",
                class: if pres_a.state() == PresenceState::Suspended { "pres-exiting" } else { "" },
                style: "opacity: 1; animation-fill-mode: backwards;",
                "data-presence": state_str(pres_a.state()),
                "Prior Fill"
            }
            div {
                id: "presence-no-fill",
                class: if pres_b.state() == PresenceState::Suspended { "pres-exiting" } else { "" },
                style: "opacity: 1;",
                "data-presence": state_str(pres_b.state()),
                "No Fill"
            }
            button {
                id: "close-retain-target",
                onclick: move |_| open_c.set(false),
                "Close Retain"
            }
            button {
                id: "disable-retain-mount",
                onclick: move |_| retain_c.set(false),
                "Disable Retain"
            }
            if pres_c.should_render() {
                div {
                    id: "presence-retain-sync",
                    class: if pres_c.state() == PresenceState::Suspended { "pres-exiting" } else { "" },
                    "data-presence": state_str(pres_c.state()),
                    "Retain Target"
                }
            }
        }
    }

    let mut harness = mount_native_scenario(presence_restore_scenario);

    // 1. Trigger exit on both fill-mode targets -> transition to Suspended.
    click_and_pump(&mut harness, "#close-fill-targets");
    assert_eq!(
        harness.attr("#presence-prior-fill", "data-presence").as_deref(),
        Some("suspended")
    );
    assert_eq!(
        harness.attr("#presence-no-fill", "data-presence").as_deref(),
        Some("suspended")
    );

    // 2. Advance past 0.2s exit animation -> AnimationEnd completes close cycle and applies forwards guard.
    advance_frame_and_pump(&mut harness, 0.25);
    assert_eq!(
        harness.attr("#presence-prior-fill", "data-presence").as_deref(),
        Some("unmounted")
    );
    assert_eq!(
        harness.attr("#presence-no-fill", "data-presence").as_deref(),
        Some("unmounted")
    );

    // 3. Advance 1 frame -> next_frame restore task restores prior inline animation-fill-mode.
    advance_frame_and_pump(&mut harness, 0.02);
    advance_frame_and_pump(&mut harness, 0.02);

    {
        let nid_a = harness.node("#presence-prior-fill");
        let nid_b = harness.node("#presence-no-fill");
        let base = harness.base();

        let decls_a = base
            .get_node(nid_a)
            .and_then(|n| {
                let block = n.element_data()?.style_attribute.as_ref()?;
                Some(format!("{:?}", block.read_with(&n.guard().read())))
            })
            .unwrap_or_default();
        let decls_a_lc = decls_a.to_ascii_lowercase();
        assert!(
            decls_a_lc.contains("backwards") && !decls_a_lc.contains("forwards"),
            "M-3: Prior inline animation-fill-mode: backwards must be restored after exit frame, got: {decls_a}"
        );

        let decls_b = base
            .get_node(nid_b)
            .and_then(|n| {
                let block = n.element_data()?.style_attribute.as_ref()?;
                Some(format!("{:?}", block.read_with(&n.guard().read())))
            })
            .unwrap_or_default();
        let decls_b_lc = decls_b.to_ascii_lowercase();
        assert!(
            !decls_b_lc.contains("forwards"),
            "M-3: Node without prior animation-fill-mode must not remain stuck at forwards after exit frame, got: {decls_b}"
        );
    }

    // 4. Reactive `retain_mount` sync: close -> Suspended -> flip retain_mount=false -> unmounts immediately.
    click_and_pump(&mut harness, "#close-retain-target");
    assert!(
        harness.query("#presence-retain-sync").is_some(),
        "Retained node must stay mounted while Suspended"
    );
    assert_eq!(
        harness
            .attr("#presence-retain-sync", "data-presence")
            .as_deref(),
        Some("suspended")
    );

    click_and_pump(&mut harness, "#disable-retain-mount");
    assert!(
        harness.query("#presence-retain-sync").is_none(),
        "M-3: Flipping retain_mount from true to false post-mount must immediately unmount Suspended presence"
    );
}

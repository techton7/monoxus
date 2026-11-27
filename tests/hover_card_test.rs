use std::{cell::RefCell, rc::Rc};

use dioxus::prelude::*;
use monoxus::foundation::overlay::{OverlayBranchContext, PlacementAlign, PlacementSide};
use monoxus::foundation::state::DataState;
use monoxus::hover_card::{
    HoverCard, HoverCardArrow, HoverCardContent, HoverCardContentStatic, HoverCardOutsideBehavior,
    HoverCardPortal, HoverCardRoot, HoverCardRootContext, HoverCardTrigger,
};

#[derive(Clone, Default, PartialEq)]
struct HoverCardProbeResults {
    root_state: Option<String>,
    trigger_href: Option<String>,
    trigger_state: Option<String>,
    trigger_data_disabled: Option<String>,
    content_state: Option<String>,
    content_side: Option<String>,
    content_align: Option<String>,
    content_tabindex: Option<i32>,
    arrow_state: Option<String>,
}

#[component]
fn HoverCardProbe(results: Rc<RefCell<HoverCardProbeResults>>) -> Element {
    let ctx = use_context::<HoverCardRootContext>();
    let runtime = ctx.runtime.read();
    let root = runtime.root();
    let trigger = runtime.trigger(Some("https://example.com".to_string()), None, None, false);
    let content = runtime.content();
    let arrow = runtime.arrow();

    *results.borrow_mut() = HoverCardProbeResults {
        root_state: Some(root.data_state().as_str().to_string()),
        trigger_href: trigger.href().map(str::to_string),
        trigger_state: Some(trigger.data_state().as_str().to_string()),
        trigger_data_disabled: trigger.data_disabled().map(str::to_string),
        content_state: Some(content.data_state().as_str().to_string()),
        content_side: Some(content.data_side().to_string()),
        content_align: Some(content.data_align().to_string()),
        content_tabindex: Some(content.tabindex()),
        arrow_state: Some(arrow.data_state().as_str().to_string()),
    };

    rsx! {}
}

#[component]
fn HoverCardCompoundSample(
    is_open: bool,
    probe_out: Rc<RefCell<HoverCardProbeResults>>,
) -> Element {
    rsx! {
        HoverCard {
            id: "hc-sample",
            default_open: is_open,
            HoverCardTrigger {
                id: "hc-trigger",
                href: "https://example.com",
                "@monoxus"
            }
            HoverCardPortal {
                HoverCardContent {
                    id: "hc-content",
                    "Preview details"
                    HoverCardArrow {
                        id: "hc-arrow"
                    }
                }
            }
            HoverCardProbe { results: probe_out }
        }
    }
}

#[test]
fn test_hover_card_compound_parts_and_default_contract() {
    let probe_open = Rc::new(RefCell::new(HoverCardProbeResults::default()));
    let mut dom_open = VirtualDom::new_with_props(
        HoverCardCompoundSample,
        HoverCardCompoundSampleProps {
            is_open: true,
            probe_out: probe_open.clone(),
        },
    );
    dom_open.rebuild_in_place();

    let open_res = probe_open.borrow();
    assert_eq!(open_res.root_state.as_deref(), Some("open"));
    assert_eq!(open_res.trigger_state.as_deref(), Some("open"));
    assert_eq!(
        open_res.trigger_href.as_deref(),
        Some("https://example.com")
    );
    assert_eq!(open_res.trigger_data_disabled, None);
    assert_eq!(open_res.content_state.as_deref(), Some("open"));
    assert_eq!(open_res.content_side.as_deref(), Some("bottom"));
    assert_eq!(open_res.content_align.as_deref(), Some("center"));
    assert_eq!(open_res.content_tabindex, Some(-1));
    assert_eq!(open_res.arrow_state.as_deref(), Some("open"));

    let probe_closed = Rc::new(RefCell::new(HoverCardProbeResults::default()));
    let mut dom_closed = VirtualDom::new_with_props(
        HoverCardCompoundSample,
        HoverCardCompoundSampleProps {
            is_open: false,
            probe_out: probe_closed.clone(),
        },
    );
    dom_closed.rebuild_in_place();
    let closed_res = probe_closed.borrow();
    assert_eq!(closed_res.root_state.as_deref(), Some("closed"));
    assert_eq!(closed_res.trigger_state.as_deref(), Some("closed"));
}

#[component]
fn HoverCardForceMountStaticSample() -> Element {
    rsx! {
        HoverCardRoot {
            id: "hc-static-root",
            default_open: false,
            HoverCardTrigger {
                "Static Trigger"
            }
            HoverCardPortal {
                force_mount: true,
                HoverCardContentStatic {
                    id: "hc-static-content",
                    force_mount: true,
                    "Always mounted static content"
                }
            }
        }
    }
}

#[test]
fn test_hover_card_static_and_force_mount_renders() {
    let mut dom = VirtualDom::new(HoverCardForceMountStaticSample);
    dom.rebuild_in_place();
}

#[component]
fn HoverCardDismissProbe(
    escape_res: Rc<RefCell<bool>>,
    outside_res: Rc<RefCell<bool>>,
) -> Element {
    let ctx = use_context::<HoverCardRootContext>();
    let runtime = ctx.runtime.read();
    let rt = runtime.clone();
    use_hook(move || {
        *escape_res.borrow_mut() = rt.dismiss_escape();
        *outside_res.borrow_mut() = rt.dismiss_outside_pointer(None);
    });
    rsx! {}
}

#[component]
fn HoverCardDismissSample(
    escape_behavior: HoverCardOutsideBehavior,
    interact_behavior: HoverCardOutsideBehavior,
    escape_fired: Rc<RefCell<bool>>,
    pointer_outside_fired: Rc<RefCell<bool>>,
    interact_outside_fired: Rc<RefCell<bool>>,
    escape_res: Rc<RefCell<bool>>,
    outside_res: Rc<RefCell<bool>>,
) -> Element {
    let esc_c = escape_fired.clone();
    let ptr_c = pointer_outside_fired.clone();
    let int_c = interact_outside_fired.clone();
    rsx! {
        HoverCardRoot {
            id: "hc-dismiss",
            default_open: true,
            HoverCardTrigger { "Trigger" }
            HoverCardPortal {
                HoverCardContent {
                    id: "hc-dismiss-content",
                    escape_key_down_behavior: escape_behavior.into(),
                    interact_outside_behavior: interact_behavior.into(),
                    on_escape_key_down: move |_| *esc_c.borrow_mut() = true,
                    on_pointer_down_outside: move |_| *ptr_c.borrow_mut() = true,
                    on_interact_outside: move |_| *int_c.borrow_mut() = true,
                    "Content"
                }
            }
            HoverCardDismissProbe {
                escape_res,
                outside_res,
            }
        }
    }
}

#[test]
fn test_hover_card_escape_and_outside_dismiss_behaviors() {
    let esc_fired = Rc::new(RefCell::new(false));
    let ptr_fired = Rc::new(RefCell::new(false));
    let int_fired = Rc::new(RefCell::new(false));
    let esc_res = Rc::new(RefCell::new(false));
    let out_res = Rc::new(RefCell::new(false));

    let mut dom_ignore = VirtualDom::new_with_props(
        HoverCardDismissSample,
        HoverCardDismissSampleProps {
            escape_behavior: HoverCardOutsideBehavior::Ignore,
            interact_behavior: HoverCardOutsideBehavior::Ignore,
            escape_fired: esc_fired.clone(),
            pointer_outside_fired: ptr_fired.clone(),
            interact_outside_fired: int_fired.clone(),
            escape_res: esc_res.clone(),
            outside_res: out_res.clone(),
        },
    );
    dom_ignore.rebuild_in_place();

    assert!(*esc_fired.borrow());
    assert!(*ptr_fired.borrow());
    assert!(*int_fired.borrow());
    assert!(!*esc_res.borrow());
    assert!(!*out_res.borrow());

    let esc_res2 = Rc::new(RefCell::new(false));
    let out_res2 = Rc::new(RefCell::new(false));
    let mut dom_close = VirtualDom::new_with_props(
        HoverCardDismissSample,
        HoverCardDismissSampleProps {
            escape_behavior: HoverCardOutsideBehavior::Close,
            interact_behavior: HoverCardOutsideBehavior::Close,
            escape_fired: Rc::new(RefCell::new(false)),
            pointer_outside_fired: Rc::new(RefCell::new(false)),
            interact_outside_fired: Rc::new(RefCell::new(false)),
            escape_res: esc_res2.clone(),
            outside_res: out_res2.clone(),
        },
    );
    dom_close.rebuild_in_place();

    assert!(*esc_res2.borrow());
    assert!(!*out_res2.borrow());
}

#[component]
fn HoverCardBranchSample(
    registered: Rc<RefCell<Vec<String>>>,
    unregistered: Rc<RefCell<Vec<String>>>,
) -> Element {
    let reg_c = registered.clone();
    let unreg_c = unregistered.clone();
    use_context_provider(|| {
        OverlayBranchContext::new(
            move |branch: String| reg_c.borrow_mut().push(branch),
            move |branch: String| unreg_c.borrow_mut().push(branch),
        )
    });

    rsx! {
        HoverCardRoot {
            default_open: true,
            HoverCardTrigger { "Trigger" }
            HoverCardPortal {
                HoverCardContent {
                    id: "nested-hc-content",
                    "Nested preview"
                }
            }
        }
    }
}

#[test]
fn test_hover_card_registers_in_overlay_branch_context() {
    let registered = Rc::new(RefCell::new(Vec::new()));
    let unregistered = Rc::new(RefCell::new(Vec::new()));

    let mut dom = VirtualDom::new_with_props(
        HoverCardBranchSample,
        HoverCardBranchSampleProps {
            registered: registered.clone(),
            unregistered: unregistered.clone(),
        },
    );
    dom.rebuild_in_place();
    assert!(registered.borrow().contains(&"nested-hc-content".to_string()));
}

#[test]
fn test_hover_card_unmount_during_delay_no_panic() {
    let mounted_cell = Rc::new(RefCell::new(None::<Signal<bool>>));
    let m_cell = mounted_cell.clone();

    let mut dom = VirtualDom::new_with_props(
        move |_: ()| {
            let mounted = use_signal(|| true);
            *m_cell.borrow_mut() = Some(mounted);

            if !mounted() {
                return rsx! { div { id: "unmounted", "Unmounted" } };
            }

            rsx! {
                HoverCardRoot {
                    open_delay: 50,
                    close_delay: 50,
                    default_open: true,
                    HoverCardTrigger { "Trigger" }
                    HoverCardPortal {
                        HoverCardContent { "Content" }
                    }
                }
            }
        },
        (),
    );
    dom.rebuild_in_place();

    let mut mounted_sig = mounted_cell.borrow().unwrap();
    mounted_sig.set(false);
    dom.render_immediate(&mut dioxus::core::NoOpMutations);
    std::thread::sleep(std::time::Duration::from_millis(70));
    dom.render_immediate(&mut dioxus::core::NoOpMutations);
}

#[test]
fn test_hover_card_data_state_enum_values() {
    assert_eq!(DataState::Open.as_str(), "open");
    assert_eq!(DataState::Closed.as_str(), "closed");
}

#[cfg(feature = "native")]
mod native_tests {
    use super::*;
    use crate::common::native_fixture::*;
    use blitz_traits::events::{
        BlitzPointerEvent, BlitzPointerId, MouseEventButton, MouseEventButtons, PointerCoords,
        PointerDetails, UiEvent,
    };
    use keyboard_types::{Code, Key, NamedKey};

    #[component]
    fn NativeHoverCardBasicScenario() -> Element {
        rsx! {
            div {
                style: "padding: 40px;",
                button {
                    id: "hc-lead-btn",
                    r#type: "button",
                    "Lead Button"
                }
                HoverCardRoot {
                    id: "native-hc",
                    open_delay: 0,
                    close_delay: 0,
                    HoverCardTrigger {
                        id: "native-hc-trigger",
                        href: "https://example.com",
                        style: "display: inline-block; padding: 8px 12px;",
                        "@monoxus"
                    }
                    HoverCardPortal {
                        HoverCardContent {
                            id: "native-hc-content",
                            style: "width: 220px; padding: 12px;",
                            p { id: "native-hc-text", "Selectable hover preview text" }
                            button { id: "native-hc-inner-btn", r#type: "button", "Inside Action" }
                            a { id: "native-hc-inner-link", href: "https://example.com/a", "Inside Link" }
                            HoverCardArrow { id: "native-hc-arrow" }
                        }
                    }
                }
                button {
                    id: "hc-after-btn",
                    r#type: "button",
                    "After Button"
                }
            }
        }
    }

    #[test]
    fn native_hover_card_opens_on_hover_places_below_and_suppresses_tabbable_descendants() {
        let _lock = NATIVE_TEST_LOCK.lock().unwrap_or_else(|e| e.into_inner());
        let mut harness = mount_native_scenario(NativeHoverCardBasicScenario);

        assert_rect_non_empty(&harness, "#native-hc-trigger");
        assert!(harness.query("#native-hc-content").is_none());
        assert_eq!(
            harness.attr("#native-hc-trigger", "role"),
            None,
            "D-1: HoverCardTrigger must not emit role='button'"
        );
        assert_eq!(
            harness.attr("#native-hc-trigger", "aria-haspopup"),
            None,
            "D-1: HoverCardTrigger must not emit aria-haspopup='dialog'"
        );

        // Hover trigger -> opens with delay 0, places below trigger (D-3), suppresses inner tabbables (HC-8)
        let (tx, ty) = harness.center_of("#native-hc-trigger");
        harness.move_mouse_to(tx, ty);
        advance_frame_and_pump(&mut harness, 0.02);
        advance_frame_and_pump(&mut harness, 0.02);

        assert!(harness.query("#native-hc-content").is_some());
        assert_rect_non_empty(&harness, "#native-hc-content");
        assert_rect_non_empty(&harness, "#native-hc-arrow");
        assert_eq!(
            harness.attr("#native-hc-content", "data-state").as_deref(),
            Some("open")
        );
        assert_eq!(
            harness.attr("#native-hc-content", "data-side").as_deref(),
            Some("bottom")
        );
        assert_relative_below(&harness, "#native-hc-trigger", "#native-hc-content", 30.0);

        // HC-8: tabbable descendants inside content must have tabindex="-1"
        assert_eq!(
            harness.attr("#native-hc-inner-btn", "tabindex").as_deref(),
            Some("-1")
        );
        assert_eq!(
            harness.attr("#native-hc-inner-link", "tabindex").as_deref(),
            Some("-1")
        );

        // Leave far away -> closes
        harness.move_mouse_to(2.0, 2.0);
        advance_frame_and_pump(&mut harness, 0.05);
        advance_frame_and_pump(&mut harness, 0.25);
        assert!(harness.query("#native-hc-content").is_none());
    }

    #[test]
    fn native_hover_card_focus_visible_opens_and_pointer_leave_does_not_close() {
        let _lock = NATIVE_TEST_LOCK.lock().unwrap_or_else(|e| e.into_inner());
        let mut harness = mount_native_scenario(NativeHoverCardBasicScenario);

        // Tab from lead button onto HoverCardTrigger (:focus-visible = true)
        click_and_pump(&mut harness, "#hc-lead-btn");
        send_key_and_pump(&mut harness, Key::Named(NamedKey::Tab), Code::Tab);
        advance_frame_and_pump(&mut harness, 0.05);

        assert_native_focus_visible(&harness, "native-hc-trigger", true);
        assert!(
            harness.query("#native-hc-content").is_some(),
            "D-2: Keyboard focus-visible must open HoverCard"
        );

        // Moving pointer across trigger and leaving must NOT close an opened_via_focus card
        let (tx, ty) = harness.center_of("#native-hc-trigger");
        harness.move_mouse_to(tx, ty);
        harness.pump();
        harness.move_mouse_to(2.0, 2.0);
        advance_frame_and_pump(&mut harness, 0.05);
        assert!(
            harness.query("#native-hc-content").is_some(),
            "D-2: Pointer leave must not close HoverCard while opened_via_focus"
        );

        // Escape dismisses focus-opened HoverCard
        send_key_and_pump(&mut harness, Key::Named(NamedKey::Escape), Code::Escape);
        advance_frame_and_pump(&mut harness, 0.25);
        assert!(harness.query("#native-hc-content").is_none());
    }

    #[test]
    fn native_hover_card_selection_guard_keeps_card_open_while_text_is_selected() {
        let _lock = NATIVE_TEST_LOCK.lock().unwrap_or_else(|e| e.into_inner());
        let mut harness = mount_native_scenario(NativeHoverCardBasicScenario);

        let (tx, ty) = harness.center_of("#native-hc-trigger");
        harness.move_mouse_to(tx, ty);
        advance_frame_and_pump(&mut harness, 0.02);
        advance_frame_and_pump(&mut harness, 0.02);
        assert!(harness.query("#native-hc-content").is_some());

        let text_rect = harness.layout_rect("#native-hc-text");
        let start_x = text_rect.x + 4.0;
        let end_x = text_rect.x + text_rect.width - 4.0;
        let mid_y = text_rect.y + text_rect.height * 0.5;

        let make_ptr = |x: f32, y: f32, buttons: MouseEventButtons| BlitzPointerEvent {
            id: BlitzPointerId::Mouse,
            is_primary: true,
            coords: PointerCoords {
                page_x: x,
                page_y: y,
                screen_x: x,
                screen_y: y,
                client_x: x,
                client_y: y,
            },
            button: MouseEventButton::Main,
            buttons,
            mods: Default::default(),
            details: PointerDetails::default(),
            element: Default::default(),
            active_pointers: Default::default(),
        };

        // Drag across text inside HoverCardContent and release outside
        harness.dispatch(UiEvent::PointerDown(make_ptr(
            start_x,
            mid_y,
            MouseEventButtons::Primary,
        )));
        harness.pump();
        harness.dispatch(UiEvent::PointerMove(make_ptr(
            end_x,
            mid_y,
            MouseEventButtons::Primary,
        )));
        harness.pump();
        harness.dispatch(UiEvent::PointerUp(make_ptr(
            5.0,
            5.0,
            MouseEventButtons::None,
        )));
        advance_frame_and_pump(&mut harness, 0.05);

        if harness.base().has_text_selection() {
            assert!(
                harness.query("#native-hc-content").is_some(),
                "HC-7: HoverCard must stay open while text selection inside content is active"
            );
        }
    }

    #[component]
    fn NativeHoverCardDelayedScenario() -> Element {
        rsx! {
            div {
                style: "padding: 40px;",
                HoverCardRoot {
                    id: "delayed-hc",
                    open_delay: 200,
                    close_delay: 50,
                    HoverCardTrigger {
                        id: "delayed-hc-trigger",
                        style: "display: inline-block; padding: 8px 12px;",
                        "Delayed Trigger"
                    }
                    HoverCardPortal {
                        HoverCardContent {
                            id: "delayed-hc-content",
                            "Delayed Content"
                        }
                    }
                }
            }
        }
    }

    #[test]
    fn native_hover_card_early_leave_cancels_pending_open_delay() {
        let _lock = NATIVE_TEST_LOCK.lock().unwrap_or_else(|e| e.into_inner());
        let mut harness = mount_native_scenario(NativeHoverCardDelayedScenario);

        let (tx, ty) = harness.center_of("#delayed-hc-trigger");
        harness.move_mouse_to(tx, ty);
        harness.pump();
        // Immediately leave before 200ms open_delay
        harness.move_mouse_to(2.0, 2.0);
        harness.pump();

        std::thread::sleep(std::time::Duration::from_millis(240));
        advance_frame_and_pump(&mut harness, 0.05);
        assert!(
            harness.query("#delayed-hc-content").is_none(),
            "HC-3: Leaving trigger before open_delay must cancel pending open"
        );
    }

    #[component]
    fn NativeHoverCardFmW1Hc1Scenario() -> Element {
        rsx! {
            div {
                style: "padding: 40px;",
                button {
                    id: "hc1-lead-btn",
                    r#type: "button",
                    "Lead"
                }
                HoverCardRoot {
                    id: "hc1-root",
                    open_delay: 140,
                    close_delay: 0,
                    HoverCardTrigger {
                        id: "hc1-trigger",
                        href: "#stay",
                        style: "display: inline-block; padding: 8px 16px;",
                        on_click: move |evt: MouseEvent| evt.prevent_default(),
                        "@monoxus-link"
                    }
                    HoverCardPortal {
                        HoverCardContent {
                            id: "hc1-content",
                            style: "width: 220px; padding: 12px;",
                            "Card Body"
                        }
                    }
                }
                button {
                    id: "hc1-outside",
                    r#type: "button",
                    style: "margin-left: 40px; padding: 8px 12px;",
                    "Outside"
                }
            }
        }
    }

    #[test]
    fn fm_w1_hc1_hover_card_immediate_dismiss_on_trigger_and_outside_click_and_pointerleave_unblocked() {
        let _lock = NATIVE_TEST_LOCK.lock().unwrap_or_else(|e| e.into_inner());
        let mut harness = mount_native_scenario(NativeHoverCardFmW1Hc1Scenario);

        // 1. Open via hover, then 1 click on trigger closes immediately and never re-opens after open_delay.
        let (tx, ty) = harness.center_of("#hc1-trigger");
        harness.move_mouse_to(tx, ty);
        harness.pump();
        std::thread::sleep(std::time::Duration::from_millis(180));
        advance_frame_and_pump(&mut harness, 0.02);
        advance_frame_and_pump(&mut harness, 0.02);
        drain_polls_and_resolve(&mut harness);
        assert!(
            harness.query("#hc1-content").is_some(),
            "HoverCard must open after open_delay"
        );

        click_and_pump(&mut harness, "#hc1-trigger");
        advance_frame_and_pump(&mut harness, 0.02);
        assert_eq!(
            harness.attr("#hc1-trigger", "data-state").as_deref(),
            Some("closed"),
            "DEC-HC-1: 1 trigger click must immediately set HoverCard state to closed"
        );
        assert!(
            harness.query("#hc1-content").is_none(),
            "DEC-HC-1: 1 trigger click must dismiss HoverCard immediately"
        );

        std::thread::sleep(std::time::Duration::from_millis(200));
        advance_frame_and_pump(&mut harness, 0.02);
        assert!(
            harness.query("#hc1-content").is_none(),
            "DEC-HC-1: Clicking non-navigating <a> trigger must not re-open HoverCard after open_delay"
        );

        // 2. Hover while closed and click trigger before open_delay -> pending open cancelled, never opens.
        harness.move_mouse_to(2.0, 2.0);
        harness.pump();
        harness.move_mouse_to(tx, ty);
        harness.pump();
        click_and_pump(&mut harness, "#hc1-trigger");
        std::thread::sleep(std::time::Duration::from_millis(200));
        advance_frame_and_pump(&mut harness, 0.02);
        assert!(
            harness.query("#hc1-content").is_none(),
            "DEC-HC-1: Trigger pointerdown before open_delay must cancel pending open"
        );

        // 3. Re-open via hover (even after trigger was clicked/focused), then 1 outside click dismisses immediately.
        harness.move_mouse_to(2.0, 2.0);
        harness.pump();
        harness.move_mouse_to(tx, ty);
        harness.pump();
        std::thread::sleep(std::time::Duration::from_millis(180));
        advance_frame_and_pump(&mut harness, 0.02);
        advance_frame_and_pump(&mut harness, 0.02);
        drain_polls_and_resolve(&mut harness);
        assert!(harness.query("#hc1-content").is_some());

        click_and_pump(&mut harness, "#hc1-outside");
        advance_frame_and_pump(&mut harness, 0.02);
        assert!(
            harness.query("#hc1-content").is_none(),
            "DEC-HC-1: 1 outside click must dismiss HoverCard immediately"
        );

        // 4. Click trigger (non-keyboard focus), re-open via hover, then pointerleave must close (not stuck on trigger_has_focus).
        click_and_pump(&mut harness, "#hc1-trigger");
        harness.move_mouse_to(2.0, 2.0);
        harness.pump();
        harness.move_mouse_to(tx, ty);
        harness.pump();
        std::thread::sleep(std::time::Duration::from_millis(180));
        advance_frame_and_pump(&mut harness, 0.02);
        advance_frame_and_pump(&mut harness, 0.02);
        drain_polls_and_resolve(&mut harness);
        assert!(harness.query("#hc1-content").is_some());

        harness.move_mouse_to(2.0, 2.0);
        advance_frame_and_pump(&mut harness, 0.05);
        advance_frame_and_pump(&mut harness, 0.05);
        assert!(
            harness.query("#hc1-content").is_none(),
            "DEC-HC-1: pointerleave after click-focus must not get stuck open"
        );

        // 5. Open via keyboard Tab focus (`opened_via_focus = true`), close via Escape (`opened_via_focus` reset),
        // then re-open via hover and verify pointerleave closes cleanly.
        click_and_pump(&mut harness, "#hc1-lead-btn");
        send_key_and_pump(&mut harness, Key::Named(NamedKey::Tab), Code::Tab);
        std::thread::sleep(std::time::Duration::from_millis(180));
        advance_frame_and_pump(&mut harness, 0.02);
        advance_frame_and_pump(&mut harness, 0.02);
        drain_polls_and_resolve(&mut harness);
        assert!(harness.query("#hc1-content").is_some());

        send_key_and_pump(&mut harness, Key::Named(NamedKey::Escape), Code::Escape);
        advance_frame_and_pump(&mut harness, 0.02);
        assert!(harness.query("#hc1-content").is_none());

        harness.move_mouse_to(2.0, 2.0);
        harness.pump();
        harness.move_mouse_to(tx, ty);
        harness.pump();
        std::thread::sleep(std::time::Duration::from_millis(180));
        advance_frame_and_pump(&mut harness, 0.02);
        advance_frame_and_pump(&mut harness, 0.02);
        drain_polls_and_resolve(&mut harness);
        assert!(harness.query("#hc1-content").is_some());

        harness.move_mouse_to(2.0, 2.0);
        advance_frame_and_pump(&mut harness, 0.05);
        advance_frame_and_pump(&mut harness, 0.05);
        assert!(
            harness.query("#hc1-content").is_none(),
            "DEC-HC-1: opened_via_focus must reset on close so subsequent hover-leave closes cleanly"
        );
    }

    #[component]
    fn NativeHoverCardFmW1Hc2Scenario() -> Element {
        rsx! {
            div {
                style: "padding: 40px;",
                button {
                    id: "hc2-lead",
                    r#type: "button",
                    "Lead"
                }
                HoverCardRoot {
                    id: "hc2-root",
                    open_delay: 140,
                    close_delay: 0,
                    HoverCardTrigger {
                        id: "hc2-trigger",
                        href: "https://example.com",
                        style: "display: inline-block; padding: 8px 16px;",
                        "Trigger"
                    }
                    HoverCardPortal {
                        HoverCardContent {
                            id: "hc2-content",
                            side: PlacementSide::Bottom,
                            side_offset: 24.0,
                            style: "width: 220px; height: 80px; padding: 12px;",
                            "Corridor Content"
                        }
                    }
                }
            }
        }
    }

    #[test]
    fn fm_w1_hc2_hover_card_focus_respects_open_delay_and_corridor_closes_on_stall() {
        let _lock = NATIVE_TEST_LOCK.lock().unwrap_or_else(|e| e.into_inner());
        let mut harness = mount_native_scenario(NativeHoverCardFmW1Hc2Scenario);

        // DEC-HC-2: Keyboard focus must respect open_delay (not open immediately before 140ms).
        click_and_pump(&mut harness, "#hc2-lead");
        send_key_and_pump(&mut harness, Key::Named(NamedKey::Tab), Code::Tab);
        advance_frame_and_pump(&mut harness, 0.02);
        assert!(
            harness.query("#hc2-content").is_none(),
            "DEC-HC-2: Keyboard focus must wait for open_delay_ms before opening"
        );

        std::thread::sleep(std::time::Duration::from_millis(170));
        advance_frame_and_pump(&mut harness, 0.02);
        assert!(
            harness.query("#hc2-content").is_some(),
            "DEC-HC-2: Keyboard focus must open once open_delay_ms elapses"
        );

        send_key_and_pump(&mut harness, Key::Named(NamedKey::Escape), Code::Escape);
        advance_frame_and_pump(&mut harness, 0.02);
        assert!(harness.query("#hc2-content").is_none());

        // H-3: Pausing inside the grace corridor must close once stall_timeout_ms (300ms) elapses.
        let (tx, ty) = harness.center_of("#hc2-trigger");
        harness.move_mouse_to(tx, ty);
        harness.pump();
        std::thread::sleep(std::time::Duration::from_millis(170));
        advance_frame_and_pump(&mut harness, 0.02);
        advance_frame_and_pump(&mut harness, 0.02);
        assert!(harness.query("#hc2-content").is_some());

        let trig_rect = harness.layout_rect("#hc2-trigger");
        let content_rect = harness.layout_rect("#hc2-content");
        let mid_x = trig_rect.x + trig_rect.width * 0.5;
        let gap_y = (trig_rect.y + trig_rect.height + content_rect.y) * 0.5;

        harness.move_mouse_to(mid_x, gap_y);
        advance_frame_and_pump(&mut harness, 0.02);
        assert!(
            harness.query("#hc2-content").is_some(),
            "HoverCard must stay open while pointer enters grace corridor"
        );

        // Stall inside corridor without moving for > 300ms -> closes via stall_timeout_ms.
        std::thread::sleep(std::time::Duration::from_millis(360));
        advance_frame_and_pump(&mut harness, 0.02);
        advance_frame_and_pump(&mut harness, 0.02);
        assert!(
            harness.query("#hc2-content").is_none(),
            "H-3: Stalling inside HoverCard grace corridor must close after 300ms timeout"
        );
    }

    #[derive(Clone, Props)]
    struct HoverCardMetaProbeProps {
        arrow_padding_out: Rc<RefCell<f32>>,
        branches_out: Rc<RefCell<Vec<String>>>,
    }

    impl PartialEq for HoverCardMetaProbeProps {
        fn eq(&self, other: &Self) -> bool {
            Rc::ptr_eq(&self.arrow_padding_out, &other.arrow_padding_out)
                && Rc::ptr_eq(&self.branches_out, &other.branches_out)
        }
    }

    #[component]
    fn HoverCardArrowPaddingProbe(out: Rc<RefCell<f32>>) -> Element {
        let ctx = use_context::<HoverCardRootContext>();
        *out.borrow_mut() = ctx.runtime.read().effective_arrow_padding();
        rsx! {}
    }

    #[component]
    fn NativeHoverCardPeAndGuardScenario(props: HoverCardMetaProbeProps) -> Element {
        use monoxus::foundation::shared::Direction;
        use monoxus::popover::{PopoverContent, PopoverPortal, PopoverRoot, PopoverTrigger};

        let branches_reg = props.branches_out.clone();
        let branches_unreg = props.branches_out.clone();
        use_context_provider(|| {
            OverlayBranchContext::new(
                move |branch: String| branches_reg.borrow_mut().push(branch),
                move |branch: String| branches_unreg.borrow_mut().retain(|b| b != &branch),
            )
        });

        rsx! {
            div {
                style: "padding: 40px;",
                // Closed force_mount HoverCard must NOT register parent OverlayBranchContext (M-2).
                HoverCardRoot {
                    id: "hc-fm-closed-root",
                    default_open: false,
                    HoverCardTrigger { id: "hc-fm-closed-trigger", "Closed Trigger" }
                    HoverCardPortal {
                        force_mount: true,
                        HoverCardContent {
                            id: "hc-fm-closed-content",
                            force_mount: true,
                            "Closed Force Mount"
                        }
                    }
                }
                // Modal Popover activates modal pointer lock so nested HoverCard emits pointer-events: auto (H-5).
                // Also verifies floating HoverCardContent keeps dir="rtl" (M-5) and forwards arrow_padding (M-4).
                PopoverRoot {
                    id: "hc-modal-host",
                    default_open: true,
                    modal: Some(true),
                    PopoverTrigger { id: "hc-modal-trigger", "Host" }
                    PopoverPortal {
                        PopoverContent {
                            id: "hc-modal-popover-content",
                            HoverCardRoot {
                                id: "hc-rtl-root",
                                default_open: true,
                                HoverCardTrigger {
                                    id: "hc-rtl-trigger",
                                    style: "display: inline-block; padding: 6px 12px;",
                                    "RTL Trigger"
                                }
                                HoverCardPortal {
                                    HoverCardContent {
                                        id: "hc-rtl-content",
                                        dir: Some(Direction::Rtl),
                                        align: PlacementAlign::Start,
                                        arrow_padding: 12.0,
                                        style: "width: 180px; padding: 8px;",
                                        "RTL Modal Card"
                                        HoverCardArrow { id: "hc-rtl-arrow" }
                                        HoverCardArrowPaddingProbe {
                                            out: props.arrow_padding_out.clone(),
                                        }
                                    }
                                }
                            }
                        }
                    }
                }
            }
        }
    }

    #[test]
    fn fm_w1_pe_and_guard_hover_card_modal_pointer_events_rtl_dir_arrow_padding_and_force_mount_branch() {
        let _lock = NATIVE_TEST_LOCK.lock().unwrap_or_else(|e| e.into_inner());
        let arrow_padding_out = Rc::new(RefCell::new(0.0_f32));
        let branches_out = Rc::new(RefCell::new(Vec::<String>::new()));

        let mut harness = mount_native_scenario_with_props(
            NativeHoverCardPeAndGuardScenario,
            HoverCardMetaProbeProps {
                arrow_padding_out: arrow_padding_out.clone(),
                branches_out: branches_out.clone(),
            },
        );
        advance_frame_and_pump(&mut harness, 0.02);

        // M-2: Closed force_mount HoverCardContent must NOT register into parent OverlayBranchContext.
        assert!(
            !branches_out
                .borrow()
                .iter()
                .any(|b| b == "hc-fm-closed-content"),
            "M-2: Closed force_mount HoverCardContent must not register in parent OverlayBranchContext, got {:?}",
            *branches_out.borrow()
        );

        // H-5: Open HoverCardContent under active modal pointer lock must emit `pointer-events: auto;`.
        let style = harness
            .attr("#hc-rtl-content", "style")
            .unwrap_or_default();
        assert!(
            style.contains("pointer-events: auto;"),
            "H-5: HoverCardContent must emit `pointer-events: auto;` under modal pointer lock, got: {style}"
        );

        // M-5: Floating HoverCardContent must preserve `dir="rtl"` attribute for Floating UI `is_rtl`.
        assert_eq!(
            harness.attr("#hc-rtl-content", "dir").as_deref(),
            Some("rtl"),
            "M-5: Floating HoverCardContent must preserve dir='rtl'"
        );

        // M-4: HoverCardContent's `arrow_padding` override must flow to HoverCardRuntime / HoverCardArrow.
        assert!(
            (*arrow_padding_out.borrow() - 12.0).abs() < 1e-4,
            "M-4: Expected effective arrow_padding 12.0, got {}",
            *arrow_padding_out.borrow()
        );
    }
}

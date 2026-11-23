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
            side: PlacementSide::Top,
            align: PlacementAlign::Start,
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
                    escape_keydown_behavior: escape_behavior,
                    interact_outside_behavior: interact_behavior,
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
}

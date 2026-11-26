use std::{cell::RefCell, rc::Rc};

use dioxus::prelude::*;
use monoxus::context_menu::{
    ContextMenu, ContextMenuContent, ContextMenuContentRenderProps, ContextMenuContentStatic,
    ContextMenuItem, ContextMenuPortal, ContextMenuRoot, ContextMenuSub, ContextMenuSubContent,
    ContextMenuSubTrigger, ContextMenuTrigger, ContextMenuTriggerRenderProps, MenuRootContext,
};

#[derive(Clone, Default, PartialEq)]
struct ContextProbeSnapshot {
    open: bool,
    modal: bool,
    trigger_id: String,
    content_id: String,
}

#[component]
fn ContextProbe(out: Rc<RefCell<ContextProbeSnapshot>>) -> Element {
    let root = use_context::<MenuRootContext>();
    *out.borrow_mut() = ContextProbeSnapshot {
        open: root.is_open(),
        modal: root.is_modal(),
        trigger_id: root.trigger_id.read().clone(),
        content_id: root.root_surface.current_content_id(),
    };
    rsx! {}
}

#[component]
fn ContextVdomSample(out: Rc<RefCell<ContextProbeSnapshot>>) -> Element {
    rsx! {
        ContextMenuRoot {
            default_open: true,
            ContextMenuTrigger {
                id: "cm-vdom-trig",
                "Right click me"
            }
            ContextMenuPortal {
                ContextMenuContent {
                    id: "cm-vdom-content",
                    ContextMenuItem { id: "cm-vdom-item", "Inspect" }
                    ContextMenuSub {
                        ContextMenuSubTrigger { id: "cm-vdom-sub-trig", "Tools" }
                        ContextMenuPortal {
                            ContextMenuSubContent {
                                id: "cm-vdom-sub-content",
                                ContextMenuItem { "Console" }
                            }
                        }
                    }
                }
            }
            ContextProbe { out: out }
        }
    }
}

#[test]
fn test_context_menu_defaults_to_modal_and_mounts_cleanly() {
    let probe = Rc::new(RefCell::new(ContextProbeSnapshot::default()));
    let mut dom = VirtualDom::new_with_props(
        ContextVdomSample,
        ContextVdomSampleProps {
            out: probe.clone(),
        },
    );
    dom.rebuild_in_place();

    let snap = probe.borrow();
    assert!(snap.open);
    assert!(snap.modal, "3B D-1: ContextMenu must default to modal = true");
    assert_eq!(snap.trigger_id, "cm-vdom-trig");
    assert_eq!(snap.content_id, "cm-vdom-content");
}

#[component]
fn ContextAsChildAndStaticSample() -> Element {
    let trig_render = Callback::new(|props: ContextMenuTriggerRenderProps| {
        rsx! {
            section {
                id: "{props.attrs.id}",
                style: "{props.style}",
                "data-state": "{props.attrs.data_state.as_str()}",
                oncontextmenu: move |evt| props.on_context_menu.call(evt),
                onpointerdown: move |evt| props.on_pointer_down.call(evt),
                onpointermove: move |evt| props.on_pointer_move.call(evt),
                onpointerup: move |evt| props.on_pointer_up.call(evt),
                onpointercancel: move |evt| props.on_pointer_cancel.call(evt),
                ..props.attributes,
                {props.children}
            }
        }
    });

    let content_render = Callback::new(|props: ContextMenuContentRenderProps| {
        rsx! {
            div {
                id: "{props.attrs.id}",
                role: "{props.attrs.role}",
                tabindex: "{props.attrs.tabindex}",
                ..props.attributes,
                {props.children}
            }
        }
    });

    rsx! {
        ContextMenu {
            default_open: true,
            ContextMenuTrigger {
                id: "cm-ac-trig",
                as_child: true,
                render_child: Some(trig_render),
                "Trigger Area"
            }
            ContextMenuContentStatic {
                id: "cm-static-content",
                as_child: true,
                render_child: Some(content_render),
                ContextMenuItem { "Copy Link" }
            }
        }
    }
}

#[test]
fn test_context_menu_as_child_and_static_render_cleanly() {
    let mut dom = VirtualDom::new(ContextAsChildAndStaticSample);
    dom.rebuild_in_place();
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

    fn make_pointer_event(
        id: BlitzPointerId,
        x: f32,
        y: f32,
        button: MouseEventButton,
        buttons: MouseEventButtons,
    ) -> BlitzPointerEvent {
        BlitzPointerEvent {
            id,
            is_primary: true,
            coords: PointerCoords {
                page_x: x,
                page_y: y,
                screen_x: x,
                screen_y: y,
                client_x: x,
                client_y: y,
            },
            button,
            buttons,
            mods: Default::default(),
            details: PointerDetails::default(),
            element: Default::default(),
            active_pointers: Default::default(),
        }
    }

    fn right_click_at(
        harness: &mut blitz_test_harness::Harness<dioxus_native_dom::DioxusDocument>,
        x: f32,
        y: f32,
    ) {
        harness.dispatch(UiEvent::PointerDown(make_pointer_event(
            BlitzPointerId::Mouse,
            x,
            y,
            MouseEventButton::Secondary,
            MouseEventButtons::Secondary,
        )));
        drain_polls_and_resolve(harness);
        harness.dispatch(UiEvent::PointerUp(make_pointer_event(
            BlitzPointerId::Mouse,
            x,
            y,
            MouseEventButton::Secondary,
            MouseEventButtons::empty(),
        )));
        drain_polls_and_resolve(harness);
        advance_frame_and_pump(harness, 0.02);
        advance_frame_and_pump(harness, 0.02);
        drain_polls_and_resolve(harness);
    }

    #[component]
    fn NativeContextMenuMultiScenario() -> Element {
        let mut open_a_events = use_signal(Vec::<bool>::new);
        let mut selected = use_signal(String::new);

        rsx! {
            div {
                style: "padding: 20px; display: flex; flex-direction: column; gap: 16px;",
                ContextMenuRoot {
                    on_open_change: move |open| open_a_events.write().push(open),
                    ContextMenuTrigger {
                        id: "cm-trig-a",
                        style: "width: 240px; height: 120px; background: #eee;",
                        "Region A"
                        ContextMenuRoot {
                            ContextMenuTrigger {
                                id: "cm-trig-nested",
                                style: "margin: 20px; width: 100px; height: 50px; background: #ddd;",
                                "Nested"
                            }
                            ContextMenuPortal {
                                ContextMenuContent {
                                    id: "cm-content-nested",
                                    style: "width: 120px; padding: 4px; background: white;",
                                    ContextMenuItem { "Nested Action" }
                                }
                            }
                        }
                    }
                    ContextMenuPortal {
                        ContextMenuContent {
                            id: "cm-content-a",
                            style: "width: 140px; padding: 6px; background: white;",
                            ContextMenuItem {
                                id: "cm-item-a1",
                                on_select: move |_| selected.set("a1".to_string()),
                                "Action A1"
                            }
                        }
                    }
                }
                ContextMenuRoot {
                    ContextMenuTrigger {
                        id: "cm-trig-b",
                        style: "width: 200px; height: 80px; background: #eef;",
                        "Region B"
                    }
                    ContextMenuPortal {
                        ContextMenuContent {
                            id: "cm-content-b",
                            style: "width: 140px; padding: 6px; background: white;",
                            ContextMenuItem {
                                id: "cm-item-b1",
                                "Action B1"
                            }
                        }
                    }
                }
                ContextMenuRoot {
                    ContextMenuTrigger {
                        id: "cm-trig-disabled",
                        disabled: true,
                        style: "width: 200px; height: 60px; background: #fee;",
                        "Disabled Region"
                    }
                    ContextMenuPortal {
                        ContextMenuContent {
                            id: "cm-content-disabled",
                            ContextMenuItem { "Never" }
                        }
                    }
                }
                div {
                    id: "cm-probe",
                    "data-selected": "{selected}",
                    "data-a-count": "{open_a_events.read().len()}",
                }
            }
        }
    }

    #[test]
    fn native_context_menu_virtual_anchor_reanchor_nested_and_multi_trigger() {
        let _lock = NATIVE_TEST_LOCK.lock().unwrap_or_else(|e| e.into_inner());
        let mut harness = mount_native_scenario(NativeContextMenuMultiScenario);

        // Trigger omits aria-haspopup/aria-expanded and includes -webkit-touch-callout: none
        assert_eq!(harness.attr("#cm-trig-a", "aria-haspopup"), None);
        assert_eq!(harness.attr("#cm-trig-a", "aria-expanded"), None);
        assert_eq!(
            harness
                .attr("#cm-trig-a", "data-monoxus-context-menu-trigger")
                .as_deref(),
            Some("")
        );
        let trig_style = harness.attr("#cm-trig-a", "style").unwrap_or_default();
        assert!(
            trig_style.contains("-webkit-touch-callout: none;"),
            "trig_style={trig_style}"
        );

        // Right-click Region A at top-left quadrant (away from nested region)
        let rect_a = harness.layout_rect("#cm-trig-a");
        let p1x = rect_a.x + 8.0;
        let p1y = rect_a.y + 8.0;
        right_click_at(&mut harness, p1x, p1y);
        assert!(harness.query("#cm-content-a").is_some());
        assert_eq!(
            harness.attr("#cm-trig-a", "data-state").as_deref(),
            Some("open")
        );

        // Verify ContextMenu CSS custom properties (`--monoxus-context-menu-*` & `--radix-context-menu-*`)
        let style1 = harness.attr("#cm-content-a", "style").unwrap_or_default();
        assert!(
            style1.contains("--monoxus-context-menu-content-transform-origin:"),
            "style1={style1}"
        );
        assert!(
            style1.contains("--radix-context-menu-trigger-width: 0px;"),
            "style1={style1}"
        );
        let box1 = harness.layout_rect("#cm-content-a");

        // Re-right-click Region A at second location -> re-anchors to new coordinates (`CM-3`)
        let p2x = rect_a.x + 160.0;
        let p2y = rect_a.y + 100.0;
        right_click_at(&mut harness, p2x, p2y);
        assert!(harness.query("#cm-content-a").is_some());
        let box2 = harness.layout_rect("#cm-content-a");
        assert!(
            (box2.x - box1.x).abs() > 20.0 || (box2.y - box1.y).abs() > 20.0,
            "re-right-click must reposition content: box1={box1:?}, box2={box2:?}"
        );

        // Left-clicking Region A while Menu A is open closes Menu A (`CM-6`)
        click_at_and_pump(&mut harness, p1x, p1y);
        advance_frame_and_pump(&mut harness, 0.02);
        assert!(harness.query("#cm-content-a").is_none());

        // Right-clicking Region B while Menu A is open closes A and opens B (`CM-6`)
        right_click_at(&mut harness, p1x, p1y);
        assert!(harness.query("#cm-content-a").is_some());
        let (bx, by) = harness.center_of("#cm-trig-b");
        right_click_at(&mut harness, bx, by);
        assert!(harness.query("#cm-content-a").is_none());
        assert!(harness.query("#cm-content-b").is_some());

        // Escape closes B
        send_key_and_pump(&mut harness, Key::Named(NamedKey::Escape), Code::Escape);
        advance_frame_and_pump(&mut harness, 0.02);
        assert!(harness.query("#cm-content-b").is_none());

        // Disabled trigger never opens (`CM-4`)
        let (dx, dy) = harness.center_of("#cm-trig-disabled");
        right_click_at(&mut harness, dx, dy);
        assert!(harness.query("#cm-content-disabled").is_none());

        // Nested ContextMenuTrigger claims innermost trigger only (`CM-7`)
        let (nx, ny) = harness.center_of("#cm-trig-nested");
        right_click_at(&mut harness, nx, ny);
        assert!(harness.query("#cm-content-nested").is_some());
        assert!(harness.query("#cm-content-a").is_none());
    }

    #[test]
    fn native_context_menu_touch_long_press_opens_after_700ms_and_move_cancels() {
        let _lock = NATIVE_TEST_LOCK.lock().unwrap_or_else(|e| e.into_inner());
        let mut harness = mount_native_scenario(NativeContextMenuMultiScenario);

        let rect_b = harness.layout_rect("#cm-trig-b");
        let tx = rect_b.x + 24.0;
        let ty = rect_b.y + 24.0;

        // Touch down followed by pointer move cancels long press
        harness.dispatch(UiEvent::PointerDown(make_pointer_event(
            BlitzPointerId::Finger(1),
            tx,
            ty,
            MouseEventButton::Main,
            MouseEventButtons::Primary,
        )));
        harness.pump();
        harness.dispatch(UiEvent::PointerMove(make_pointer_event(
            BlitzPointerId::Finger(1),
            tx + 12.0,
            ty + 4.0,
            MouseEventButton::Main,
            MouseEventButtons::Primary,
        )));
        std::thread::sleep(std::time::Duration::from_millis(740));
        advance_frame_and_pump(&mut harness, 0.02);
        assert!(harness.query("#cm-content-b").is_none());

        // Clean touch long press without moving opens after 700ms (`CM-5`)
        harness.dispatch(UiEvent::PointerDown(make_pointer_event(
            BlitzPointerId::Finger(2),
            tx,
            ty,
            MouseEventButton::Main,
            MouseEventButtons::Primary,
        )));
        harness.pump();
        std::thread::sleep(std::time::Duration::from_millis(740));
        advance_frame_and_pump(&mut harness, 0.02);
        assert!(harness.query("#cm-content-b").is_some());
    }

    #[component]
    fn NativeContextMenuScrollAndEscapeScenario() -> Element {
        rsx! {
            div {
                style: "padding: 20px; display: flex; flex-direction: column; gap: 12px;",
                button {
                    id: "cm-esc-outside",
                    r#type: "button",
                    style: "width: 110px; height: 32px;",
                    "Outside"
                }
                ContextMenuRoot {
                    ContextMenuTrigger {
                        id: "cm-scroll-trig",
                        style: "width: 200px; height: 100px; background: #eee;",
                        "Scrollable Canvas"
                    }
                    ContextMenuPortal {
                        ContextMenuContent {
                            id: "cm-scroll-content",
                            style: "box-sizing: border-box; width: 180px; max-height: 100px; overflow-y: auto; padding: 4px; display: flex; flex-direction: column; gap: 2px; background: white;",
                            ContextMenuItem { id: "cm-scroll-alpha", style: "height: 28px; flex-shrink: 0;", "Alpha" }
                            ContextMenuItem { id: "cm-scroll-bravo", style: "height: 28px; flex-shrink: 0;", "Bravo" }
                            ContextMenuItem { id: "cm-scroll-charlie", style: "height: 28px; flex-shrink: 0;", "Charlie" }
                            ContextMenuItem { id: "cm-scroll-delta", style: "height: 28px; flex-shrink: 0;", "Delta" }
                            ContextMenuItem { id: "cm-scroll-echo", style: "height: 28px; flex-shrink: 0;", "Echo" }
                            ContextMenuItem { id: "cm-scroll-foxtrot", style: "height: 28px; flex-shrink: 0;", "Foxtrot" }
                            ContextMenuItem { id: "cm-scroll-zulu", style: "height: 28px; flex-shrink: 0;", "Zulu" }
                        }
                    }
                }
            }
        }
    }

    #[test]
    fn native_context_menu_keyboard_and_typeahead_auto_scrolls_scrollable_content() {
        let _lock = NATIVE_TEST_LOCK.lock().unwrap_or_else(|e| e.into_inner());
        let mut harness = mount_native_scenario(NativeContextMenuScrollAndEscapeScenario);

        let rect = harness.layout_rect("#cm-scroll-trig");
        right_click_at(&mut harness, rect.x + 20.0, rect.y + 20.0);
        assert!(harness.query("#cm-scroll-content").is_some());

        let content_node = harness.node("#cm-scroll-content");
        let content_rect = harness.layout_rect("#cm-scroll-content");
        assert!(content_rect.height <= 102.0);
        let initial_scroll_y = harness
            .base()
            .get_node(content_node)
            .expect("content node")
            .scroll_offset()
            .y;
        assert!(initial_scroll_y <= 1.0);

        for expected in [
            "cm-scroll-alpha",
            "cm-scroll-bravo",
            "cm-scroll-charlie",
            "cm-scroll-delta",
            "cm-scroll-echo",
            "cm-scroll-foxtrot",
        ] {
            send_key_and_pump(&mut harness, Key::Named(NamedKey::ArrowDown), Code::ArrowDown);
            assert_native_focus_id(&harness, expected);
        }
        advance_frame_and_pump(&mut harness, 0.02);

        let after_arrow_scroll_y = harness
            .base()
            .get_node(content_node)
            .expect("content node")
            .scroll_offset()
            .y;
        assert!(
            after_arrow_scroll_y > 0.0,
            "ArrowDown to offscreen ContextMenu item must scroll container"
        );
        let foxtrot_rect = harness.layout_rect("#cm-scroll-foxtrot");
        assert!(
            foxtrot_rect.y >= content_rect.y - 1.0
                && foxtrot_rect.y + foxtrot_rect.height <= content_rect.y + content_rect.height + 1.0,
            "Foxtrot {foxtrot_rect:?} must be visible inside {content_rect:?}"
        );

        send_key_and_pump(&mut harness, Key::Character("z".into()), Code::KeyZ);
        advance_frame_and_pump(&mut harness, 0.02);
        assert_native_focus_id(&harness, "cm-scroll-zulu");
        let after_typeahead_scroll_y = harness
            .base()
            .get_node(content_node)
            .expect("content node")
            .scroll_offset()
            .y;
        assert!(
            after_typeahead_scroll_y > after_arrow_scroll_y,
            "Typeahead to Zulu must scroll further down (prev={after_arrow_scroll_y}, now={after_typeahead_scroll_y})"
        );
        let zulu_rect = harness.layout_rect("#cm-scroll-zulu");
        assert!(
            zulu_rect.y >= content_rect.y - 1.0
                && zulu_rect.y + zulu_rect.height <= content_rect.y + content_rect.height + 1.0,
            "Zulu {zulu_rect:?} must be visible inside {content_rect:?}"
        );

        send_key_and_pump(&mut harness, Key::Named(NamedKey::Home), Code::Home);
        advance_frame_and_pump(&mut harness, 0.02);
        assert_native_focus_id(&harness, "cm-scroll-alpha");
        let after_home_scroll_y = harness
            .base()
            .get_node(content_node)
            .expect("content node")
            .scroll_offset()
            .y;
        assert!(
            after_home_scroll_y <= 5.0,
            "Home must scroll back to top (within 4px container padding), got {after_home_scroll_y}"
        );
        let alpha_rect = harness.layout_rect("#cm-scroll-alpha");
        assert!(
            alpha_rect.y >= content_rect.y - 1.0
                && alpha_rect.y + alpha_rect.height <= content_rect.y + content_rect.height + 1.0,
            "Alpha {alpha_rect:?} must be visible inside {content_rect:?}"
        );
    }

    #[test]
    fn native_context_menu_escape_does_not_focus_non_tabbable_trigger_and_restores_previous() {
        let _lock = NATIVE_TEST_LOCK.lock().unwrap_or_else(|e| e.into_inner());
        let mut harness = mount_native_scenario(NativeContextMenuScrollAndEscapeScenario);

        let trig_node = harness.node("#cm-scroll-trig");
        let rect = harness.layout_rect("#cm-scroll-trig");

        // Case 1: Nothing focused prior to right-click -> Escape leaves non-tabbable trigger unfocused
        right_click_at(&mut harness, rect.x + 20.0, rect.y + 20.0);
        assert!(harness.query("#cm-scroll-content").is_some());
        send_key_and_pump(&mut harness, Key::Named(NamedKey::ArrowDown), Code::ArrowDown);
        assert_native_focus_id(&harness, "cm-scroll-alpha");

        send_key_and_pump(&mut harness, Key::Named(NamedKey::Escape), Code::Escape);
        advance_frame_and_pump(&mut harness, 0.02);
        assert!(harness.query("#cm-scroll-content").is_none());
        assert_ne!(
            harness.base().active_focus_node_id(),
            Some(trig_node),
            "Escape must NOT focus non-tabbable ContextMenuTrigger <div>"
        );
        assert!(
            !harness.base().is_focus_visible(trig_node),
            "Non-tabbable ContextMenuTrigger <div> must not gain :focus-visible on Escape"
        );

        // Case 2: External button focused prior to right-click -> Escape restores previous focus, never trigger
        focus_node(&mut harness, "#cm-esc-outside");
        right_click_at(&mut harness, rect.x + 20.0, rect.y + 20.0);
        assert!(harness.query("#cm-scroll-content").is_some());
        send_key_and_pump(&mut harness, Key::Named(NamedKey::ArrowDown), Code::ArrowDown);
        assert_native_focus_id(&harness, "cm-scroll-alpha");

        send_key_and_pump(&mut harness, Key::Named(NamedKey::Escape), Code::Escape);
        advance_frame_and_pump(&mut harness, 0.02);
        assert!(harness.query("#cm-scroll-content").is_none());
        assert_ne!(harness.base().active_focus_node_id(), Some(trig_node));
        assert!(!harness.base().is_focus_visible(trig_node));
        assert_native_focus_id(&harness, "cm-esc-outside");
    }

    #[component]
    fn NativeContextMenuVirtualHideAndSiblingScenario() -> Element {
        rsx! {
            div { style: "padding: 20px; display: flex; gap: 16px;",
                ContextMenuRoot {
                    modal: true,
                    ContextMenuTrigger {
                        id: "cm-vh-trig-a",
                        style: "width: 160px; height: 80px; background: #eee;",
                        "Region A"
                    }
                    ContextMenuPortal {
                        ContextMenuContent {
                            id: "cm-vh-content-a",
                            hide_when_detached: true,
                            style: "width: 140px; padding: 4px; background: white;",
                            ContextMenuItem { id: "cm-vh-a1", "Alpha" }
                        }
                    }
                }
                ContextMenuRoot {
                    modal: true,
                    ContextMenuTrigger {
                        id: "cm-vh-trig-b",
                        style: "width: 160px; height: 80px; background: #eef;",
                        "Region B"
                    }
                    ContextMenuPortal {
                        ContextMenuContent {
                            id: "cm-vh-content-b",
                            style: "width: 140px; padding: 4px; background: white;",
                            ContextMenuItem { id: "cm-vh-b1", "Beta" }
                        }
                    }
                }
            }
        }
    }

    #[test]
    fn fm_ovl_5_and_7_context_menu_virtual_hide_and_aria_hidden_only_sibling() {
        let _lock = NATIVE_TEST_LOCK.lock().unwrap_or_else(|e| e.into_inner());
        let mut harness = mount_native_scenario(NativeContextMenuVirtualHideAndSiblingScenario);

        // Right-click Region A with `hide_when_detached: true` on VirtualPoint anchor -> no panic (`FM-OVL-5`).
        let (ax, ay) = harness.center_of("#cm-vh-trig-a");
        right_click_at(&mut harness, ax, ay);
        assert!(harness.query("#cm-vh-content-a").is_some());
        assert_eq!(
            harness.attr("#cm-vh-content-a", "data-side").as_deref(),
            Some("right")
        );

        // Modal ContextMenu uses `ModalShieldPolicy::AriaHiddenOnly` so sibling Region B is NOT inert
        // and right-clicking Region B closes A and opens B (`FM-OVL-5`).
        assert_eq!(harness.attr("#cm-vh-trig-b", "inert"), None);
        let (bx, by) = harness.center_of("#cm-vh-trig-b");
        right_click_at(&mut harness, bx, by);
        assert!(harness.query("#cm-vh-content-a").is_none());
        assert!(harness.query("#cm-vh-content-b").is_some());
    }
}

#[test]
fn fm_ovl_7_context_menu_defaults_to_right_and_2px_side_offset() {
    use monoxus::foundation::overlay::PlacementSide;
    use monoxus::menu::MENU_CONTEXT_DEFAULT_SIDE_OFFSET;

    #[derive(Clone, Default, PartialEq)]
    struct ContextDefaultsSnapshot {
        side: Option<PlacementSide>,
        side_offset: f32,
    }

    #[component]
    fn ContextDefaultsProbe(out: Rc<RefCell<ContextDefaultsSnapshot>>) -> Element {
        let root = use_context::<MenuRootContext>();
        *out.borrow_mut() = ContextDefaultsSnapshot {
            side: Some(*root.root_surface.side.read()),
            side_offset: *root.root_surface.side_offset.read(),
        };
        rsx! {}
    }

    #[component]
    fn ContextDefaultsSample(out: Rc<RefCell<ContextDefaultsSnapshot>>) -> Element {
        rsx! {
            ContextMenuRoot {
                default_open: true,
                ContextMenuTrigger { "Canvas" }
                ContextMenuPortal {
                    ContextMenuContent {
                        ContextMenuItem { "Inspect" }
                    }
                }
                ContextDefaultsProbe { out }
            }
        }
    }

    let snap = Rc::new(RefCell::new(ContextDefaultsSnapshot::default()));
    let mut dom = VirtualDom::new_with_props(
        ContextDefaultsSample,
        ContextDefaultsSampleProps { out: snap.clone() },
    );
    dom.rebuild_in_place();

    let res = snap.borrow();
    assert_eq!(res.side, Some(PlacementSide::Right));
    assert_eq!(res.side_offset, MENU_CONTEXT_DEFAULT_SIDE_OFFSET);
}

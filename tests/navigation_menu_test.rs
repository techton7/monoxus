use std::{cell::RefCell, rc::Rc};

use dioxus::prelude::*;
use monoxus::foundation::shared::{Direction, Orientation};
use monoxus::navigation_menu::{
    NavigationMenu, NavigationMenuActivationMode, NavigationMenuContent,
    NavigationMenuContentRenderProps, NavigationMenuIndicator, NavigationMenuItem,
    NavigationMenuLink, NavigationMenuLinkSelectEvent, NavigationMenuList,
    NavigationMenuRoot, NavigationMenuRootRenderProps, NavigationMenuSub,
    NavigationMenuTrigger, NavigationMenuTriggerRenderProps, NavigationMenuViewport,
};

#[derive(Clone, Default, PartialEq)]
struct NavProbeSnapshot {
    value: String,
    previous_value: String,
    has_viewport: bool,
    dir: Direction,
    orientation: Orientation,
    activation_mode: NavigationMenuActivationMode,
    trigger_count: usize,
}

#[component]
fn NavProbe(out: Rc<RefCell<NavProbeSnapshot>>) -> Element {
    let ctx = use_context::<monoxus::navigation_menu::NavigationMenuContext>();
    *out.borrow_mut() = NavProbeSnapshot {
        value: ctx.current_value(),
        previous_value: ctx.previous_value_str(),
        has_viewport: ctx.has_viewport(),
        dir: ctx.current_dir(),
        orientation: ctx.current_orientation(),
        activation_mode: ctx.current_activation_mode(),
        trigger_count: ctx.triggers.read().len(),
    };
    rsx! {}
}

#[component]
fn NavViewportSample(
    default_value: Option<String>,
    probe_out: Rc<RefCell<NavProbeSnapshot>>,
) -> Element {
    rsx! {
        NavigationMenuRoot {
            id: "nm-vp-sample",
            default_value: default_value,
            NavigationMenuList {
                id: "nm-vp-list",
                NavigationMenuItem {
                    value: Some("learn".to_string()),
                    NavigationMenuTrigger {
                        id: "nm-vp-trig-learn",
                        "Learn"
                    }
                    NavigationMenuContent {
                        id: "nm-vp-content-learn",
                        NavigationMenuLink {
                            id: "nm-vp-link-docs",
                            href: "/docs",
                            active: true,
                            "Docs"
                        }
                    }
                }
                NavigationMenuItem {
                    value: Some("build".to_string()),
                    NavigationMenuTrigger {
                        id: "nm-vp-trig-build",
                        disabled: true,
                        "Build"
                    }
                    NavigationMenuContent {
                        id: "nm-vp-content-build",
                        NavigationMenuLink {
                            href: "/build",
                            "Build Link"
                        }
                    }
                }
                NavigationMenuIndicator { id: "nm-vp-indicator" }
            }
            NavigationMenuViewport { id: "nm-vp-viewport" }
            NavProbe { out: probe_out }
        }
    }
}

#[test]
fn test_navigation_menu_viewport_mode_and_alias_mount() {
    let probe = Rc::new(RefCell::new(NavProbeSnapshot::default()));
    let mut dom = VirtualDom::new_with_props(
        NavViewportSample,
        NavViewportSampleProps {
            default_value: Some("learn".to_string()),
            probe_out: probe.clone(),
        },
    );
    dom.rebuild_in_place();

    let snap = probe.borrow();
    assert_eq!(snap.value, "learn");
    assert_eq!(snap.previous_value, "");
    assert_eq!(snap.dir, Direction::Ltr);
    assert_eq!(snap.orientation, Orientation::Horizontal);
    assert_eq!(
        snap.activation_mode,
        NavigationMenuActivationMode::Automatic
    );
    assert_eq!(snap.trigger_count, 2);
}

#[component]
fn NavInlineAndControlledSample(
    controlled_out: Rc<RefCell<Option<Signal<String>>>>,
    probe_out: Rc<RefCell<NavProbeSnapshot>>,
    changes_out: Rc<RefCell<Vec<String>>>,
) -> Element {
    let val = use_signal(|| "one".to_string());
    *controlled_out.borrow_mut() = Some(val);
    let changes = changes_out.clone();

    rsx! {
        NavigationMenu {
            id: "nm-inline-root",
            value: Some(val),
            orientation: Orientation::Vertical,
            dir: Direction::Rtl,
            activation_mode: NavigationMenuActivationMode::Manual,
            on_value_change: move |next: String| changes.borrow_mut().push(next),
            NavigationMenuList {
                NavigationMenuItem {
                    value: Some("one".to_string()),
                    NavigationMenuTrigger {
                        id: "nm-inline-trig-1",
                        "One"
                    }
                    NavigationMenuContent {
                        id: "nm-inline-content-1",
                        force_mount: true,
                        NavigationMenuSub {
                            id: "nm-sub",
                            default_value: Some("sub-a".to_string()),
                            NavigationMenuList {
                                NavigationMenuItem {
                                    value: Some("sub-a".to_string()),
                                    NavigationMenuTrigger { "Sub A" }
                                    NavigationMenuContent {
                                        NavigationMenuLink { href: "#sub-a", "Sub A Link" }
                                    }
                                }
                            }
                        }
                    }
                }
            }
            NavProbe { out: probe_out }
        }
    }
}

#[test]
fn test_navigation_menu_inline_controlled_and_nested_sub() {
    let controlled = Rc::new(RefCell::new(None::<Signal<String>>));
    let probe = Rc::new(RefCell::new(NavProbeSnapshot::default()));
    let changes = Rc::new(RefCell::new(Vec::new()));

    let mut dom = VirtualDom::new_with_props(
        NavInlineAndControlledSample,
        NavInlineAndControlledSampleProps {
            controlled_out: controlled.clone(),
            probe_out: probe.clone(),
            changes_out: changes.clone(),
        },
    );
    dom.rebuild_in_place();

    {
        let snap = probe.borrow();
        assert_eq!(snap.value, "one");
        assert!(!snap.has_viewport);
        assert_eq!(snap.dir, Direction::Rtl);
        assert_eq!(snap.orientation, Orientation::Vertical);
        assert_eq!(snap.activation_mode, NavigationMenuActivationMode::Manual);
    }

    let mut sig = controlled.borrow().unwrap();
    sig.set(String::new());
    dom.render_immediate(&mut dioxus::core::NoOpMutations);

    let snap2 = probe.borrow();
    assert_eq!(snap2.value, "");
    assert_eq!(snap2.previous_value, "one");
}

#[component]
fn NavAsChildSample() -> Element {
    let root_render = Callback::new(|props: NavigationMenuRootRenderProps| {
        rsx! {
            article {
                id: "{props.attrs.id()}",
                aria_label: props.attrs.aria_label(),
                dir: "{props.attrs.dir()}",
                "data-orientation": "{props.attrs.data_orientation()}",
                "data-custom-root": "true",
                {props.children}
            }
        }
    });
    let trig_render = Callback::new(|props: NavigationMenuTriggerRenderProps| {
        rsx! {
            button {
                id: "{props.attrs.id()}",
                r#type: "button",
                aria_expanded: "{props.attrs.aria_expanded()}",
                "data-state": props.attrs.data_state().as_str(),
                "data-custom-trigger": "true",
                {props.children}
            }
        }
    });
    let content_render = Callback::new(|props: NavigationMenuContentRenderProps| {
        rsx! {
            section {
                id: "{props.attrs.id()}",
                "data-state": props.attrs.data_state().as_str(),
                "data-custom-content": "true",
                {props.children}
            }
        }
    });

    rsx! {
        NavigationMenuRoot {
            id: "nm-as-child",
            default_value: Some("a".to_string()),
            as_child: true,
            render_child: Some(root_render),
            NavigationMenuList {
                NavigationMenuItem {
                    value: Some("a".to_string()),
                    NavigationMenuTrigger {
                        as_child: true,
                        render_child: Some(trig_render),
                        "A"
                    }
                    NavigationMenuContent {
                        as_child: true,
                        render_child: Some(content_render),
                        NavigationMenuLink { href: "#a", "Link A" }
                    }
                }
            }
        }
    }
}

#[test]
fn test_navigation_menu_as_child_renders_cleanly() {
    let mut dom = VirtualDom::new(NavAsChildSample);
    dom.rebuild_in_place();
}

#[cfg(feature = "native")]
mod native_tests {
    use super::*;
    use crate::common::native_fixture::*;
    use keyboard_types::{Code, Key, Modifiers, NamedKey};

    #[component]
    fn NativeNavMenuViewportScenario() -> Element {
        let prevent_close_next = use_signal(|| false);
        let mut p_flag = prevent_close_next;
        rsx! {
            div {
                style: "padding: 20px;",
                button {
                    id: "nm-prev-btn",
                    r#type: "button",
                    "Prev"
                }
                NavigationMenuRoot {
                    id: "native-nm",
                    delay_duration: 60,
                    skip_delay_duration: 160,
                    NavigationMenuList {
                        id: "native-nm-list",
                        style: "display: flex; flex-direction: row; gap: 12px; list-style: none; padding: 0; margin: 0;",
                        NavigationMenuItem {
                            value: Some("one".to_string()),
                            NavigationMenuTrigger {
                                id: "trig-one",
                                style: "box-sizing: border-box; width: 100px; height: 32px;",
                                "One"
                            }
                            NavigationMenuContent {
                                id: "content-one",
                                style: "width: 240px; height: 120px; padding: 8px;",
                                NavigationMenuLink {
                                    id: "link-one-a",
                                    href: "#one-a",
                                    style: "display: inline-block; width: 70px; height: 24px;",
                                    active: true,
                                    "One A"
                                }
                                button {
                                    id: "btn-one-b",
                                    r#type: "button",
                                    "One B"
                                }
                                NavigationMenuLink {
                                    id: "link-one-prevent",
                                    href: "#keep-open",
                                    style: "display: inline-block; width: 90px; height: 24px;",
                                    on_select: move |evt: NavigationMenuLinkSelectEvent| {
                                        if *p_flag.peek() {
                                            evt.prevent_default();
                                        }
                                    },
                                    "Prevent Link"
                                }
                            }
                        }
                        NavigationMenuItem {
                            value: Some("two".to_string()),
                            NavigationMenuTrigger {
                                id: "trig-two",
                                style: "box-sizing: border-box; width: 140px; height: 32px;",
                                "Two"
                            }
                            NavigationMenuContent {
                                id: "content-two",
                                style: "width: 320px; height: 180px; padding: 8px;",
                                NavigationMenuLink {
                                    id: "link-two-a",
                                    href: "#two-a",
                                    style: "display: inline-block; width: 70px; height: 24px;",
                                    "Two A"
                                }
                                input { id: "input-two", r#type: "text" }
                                NavigationMenuLink {
                                    id: "link-two-b",
                                    href: "#two-b",
                                    style: "display: inline-block; width: 70px; height: 24px;",
                                    "Two B"
                                }
                            }
                        }
                        NavigationMenuItem {
                            value: Some("disabled-item".to_string()),
                            NavigationMenuTrigger {
                                id: "trig-disabled",
                                disabled: true,
                                style: "width: 80px; height: 32px;",
                                "Disabled"
                            }
                        }
                        NavigationMenuItem {
                            value: Some("direct-link-item".to_string()),
                            NavigationMenuLink {
                                id: "direct-list-link",
                                href: "#docs",
                                style: "display: inline-block; width: 80px; height: 32px;",
                                "Direct"
                            }
                        }
                        NavigationMenuIndicator { id: "native-nm-indicator" }
                    }
                    NavigationMenuViewport {
                        id: "native-nm-viewport",
                        style: "margin-top: 8px;",
                    }
                }
                button {
                    id: "nm-next-btn",
                    r#type: "button",
                    "Next"
                }
                button {
                    id: "nm-toggle-prevent",
                    r#type: "button",
                    onclick: move |_| {
                        let cur = *p_flag.peek();
                        p_flag.set(!cur);
                    },
                    "Toggle Prevent"
                }
            }
        }
    }

    #[test]
    fn native_navigation_menu_aria_controls_disabled_and_viewport_sizing_indicator() {
        let _lock = NATIVE_TEST_LOCK.lock().unwrap_or_else(|e| e.into_inner());
        let mut harness = mount_native_scenario(NativeNavMenuViewportScenario);

        // Closed state: aria-controls omitted, aria-expanded="false", data-disabled omitted on enabled and "" on disabled
        assert_eq!(
            harness.attr("#native-nm", "aria-label").as_deref(),
            Some("Main")
        );
        assert_eq!(
            harness.attr("#trig-one", "aria-expanded").as_deref(),
            Some("false")
        );
        assert_eq!(harness.attr("#trig-one", "aria-controls"), None);
        assert_eq!(harness.attr("#trig-one", "data-disabled"), None);
        assert_eq!(harness.attr("#trig-disabled", "data-disabled").as_deref(), Some(""));
        assert!(harness.query("#native-nm-viewport").is_none());
        assert!(harness.query("#native-nm-indicator").is_none());

        // Click trig-one -> opens viewport, sets aria-controls to mounted content-one, sizes viewport & positions indicator
        click_and_pump(&mut harness, "#trig-one");
        advance_frame_and_pump(&mut harness, 0.02);

        assert_eq!(
            harness.attr("#trig-one", "aria-expanded").as_deref(),
            Some("true")
        );
        let controls_id = harness
            .attr("#trig-one", "aria-controls")
            .expect("open trigger must expose aria-controls");
        assert_eq!(controls_id, "content-one");
        assert!(harness.query("#content-one").is_some());
        assert!(harness.query("#native-nm-viewport").is_some());

        // content-one has width: 240px; height: 120px; padding: 8px; (content-box -> 256px x 136px offset_rect)
        let vp_style = harness.attr("#native-nm-viewport", "style").unwrap_or_default();
        assert!(
            vp_style.contains("--monoxus-navigation-menu-viewport-width: 256px;"),
            "vp_style={vp_style}"
        );
        assert!(
            vp_style.contains("--radix-navigation-menu-viewport-height: 136px;"),
            "vp_style={vp_style}"
        );

        assert!(harness.query("#native-nm-indicator").is_some());
        assert_eq!(
            harness.attr("#native-nm-indicator", "data-state").as_deref(),
            Some("visible")
        );
        let ind_style = harness
            .attr("#native-nm-indicator", "style")
            .unwrap_or_default();
        assert!(ind_style.contains("width: 100px;"), "ind_style={ind_style}");
        assert!(
            ind_style.contains("--radix-navigation-menu-indicator-translate-x:"),
            "ind_style={ind_style}"
        );

        // Initial open must omit data-motion
        assert_eq!(harness.attr("#content-one", "data-motion"), None);
        assert_eq!(
            harness.attr("#link-one-a", "aria-current").as_deref(),
            Some("page")
        );
        assert_eq!(harness.attr("#link-one-a", "data-active").as_deref(), Some(""));

        // Switch to trig-two -> indicator tracks offset 112px (100px + 12px gap) and width 140px
        click_and_pump(&mut harness, "#trig-two");
        advance_frame_and_pump(&mut harness, 0.02);
        let ind_style_two = harness
            .attr("#native-nm-indicator", "style")
            .unwrap_or_default();
        assert!(
            ind_style_two.contains("width: 140px;"),
            "ind_style_two={ind_style_two}"
        );
        assert!(
            ind_style_two.contains("--radix-navigation-menu-indicator-translate-x: 112px;"),
            "ind_style_two={ind_style_two}"
        );
    }

    #[test]
    fn native_navigation_menu_hover_delay_grace_skip_delay_and_motion_direction() {
        let _lock = NATIVE_TEST_LOCK.lock().unwrap_or_else(|e| e.into_inner());
        let mut harness = mount_native_scenario(NativeNavMenuViewportScenario);

        // Hover trig-one -> wait 20ms (not open yet), wait past 60ms -> opens
        let (t1x, t1y) = harness.center_of("#trig-one");
        harness.move_mouse_to(t1x, t1y);
        advance_frame_and_pump(&mut harness, 0.02);
        assert!(harness.query("#content-one").is_none());

        std::thread::sleep(std::time::Duration::from_millis(80));
        advance_frame_and_pump(&mut harness, 0.02);
        assert!(harness.query("#content-one").is_some());

        // Immediate click right after hover-open within 300ms guard keeps content-one open (3C D-5)
        click_and_pump(&mut harness, "#trig-one");
        assert!(
            harness.query("#content-one").is_some(),
            "has_pointer_move_opened guard must keep hover-opened menu open on immediate click"
        );

        // Hover trig-two while open -> immediate switch without delay, and data-motion="from-end"
        let (t2x, t2y) = harness.center_of("#trig-two");
        harness.move_mouse_to(t2x, t2y);
        advance_frame_and_pump(&mut harness, 0.02);
        assert!(harness.query("#content-two").is_some());
        assert_eq!(
            harness.attr("#content-two", "data-motion").as_deref(),
            Some("from-end")
        );

        // Switch back to trig-one -> data-motion="from-start"
        harness.move_mouse_to(t1x, t1y);
        advance_frame_and_pump(&mut harness, 0.02);
        assert_eq!(
            harness.attr("#content-one", "data-motion").as_deref(),
            Some("from-start")
        );

        // Move into viewport -> stays open past 150ms close grace
        let (vx, vy) = harness.center_of("#native-nm-viewport");
        harness.move_mouse_to(vx, vy);
        std::thread::sleep(std::time::Duration::from_millis(180));
        advance_frame_and_pump(&mut harness, 0.02);
        assert!(harness.query("#content-one").is_some());

        // Hover direct list link -> dismisses open hover menu immediately (3C D-5)
        let (dlx, dly) = harness.center_of("#direct-list-link");
        harness.move_mouse_to(dlx, dly);
        advance_frame_and_pump(&mut harness, 0.02);
        assert!(harness.query("#content-one").is_none());

        // Re-enter trig-one within skipDelayDuration (160ms) -> opens immediately without 60ms delay
        harness.move_mouse_to(t1x, t1y);
        advance_frame_and_pump(&mut harness, 0.02);
        assert!(harness.query("#content-one").is_some());

        // Leave outside and wait past 150ms close grace -> closes
        harness.move_mouse_to(2.0, 2.0);
        std::thread::sleep(std::time::Duration::from_millis(180));
        advance_frame_and_pump(&mut harness, 0.02);
        advance_frame_and_pump(&mut harness, 0.02);
        assert!(harness.query("#content-one").is_none());
    }

    #[test]
    fn native_navigation_menu_roving_arrows_entry_key_focus_proxy_and_content_arrows() {
        let _lock = NATIVE_TEST_LOCK.lock().unwrap_or_else(|e| e.into_inner());
        let mut harness = mount_native_scenario(NativeNavMenuViewportScenario);

        // List roving arrows skip disabled trigger and stop at boundary (non-looping)
        focus_node(&mut harness, "#trig-one");
        send_key_and_pump(&mut harness, Key::Named(NamedKey::ArrowRight), Code::ArrowRight);
        assert_native_focus_id(&harness, "trig-two");

        send_key_and_pump(&mut harness, Key::Named(NamedKey::ArrowRight), Code::ArrowRight);
        assert_native_focus_id(&harness, "direct-list-link");

        // Clamped at end (no wrap)
        send_key_and_pump(&mut harness, Key::Named(NamedKey::ArrowRight), Code::ArrowRight);
        assert_native_focus_id(&harness, "direct-list-link");

        send_key_and_pump(&mut harness, Key::Named(NamedKey::Home), Code::Home);
        assert_native_focus_id(&harness, "trig-one");

        // Open trig-one via Enter, then press ArrowDown entry key -> focuses first tabbable inside content-one
        send_key_and_pump(&mut harness, Key::Named(NamedKey::Enter), Code::Enter);
        assert!(harness.query("#content-one").is_some());
        send_key_and_pump(&mut harness, Key::Named(NamedKey::ArrowDown), Code::ArrowDown);
        assert_native_focus_id(&harness, "link-one-a");

        // 4-way arrow navigation inside content-one
        send_key_and_pump(&mut harness, Key::Named(NamedKey::ArrowDown), Code::ArrowDown);
        assert_native_focus_id(&harness, "btn-one-b");
        send_key_and_pump(&mut harness, Key::Named(NamedKey::ArrowUp), Code::ArrowUp);
        assert_native_focus_id(&harness, "link-one-a");

        // Escape inside content closes menu and restores focus to trig-one
        send_key_and_pump(&mut harness, Key::Named(NamedKey::Escape), Code::Escape);
        advance_frame_and_pump(&mut harness, 0.02);
        assert!(harness.query("#content-one").is_none());
        assert_native_focus_id(&harness, "trig-one");

        // Open trig-two via Enter, press Tab -> FocusProxy routes focus into content-two
        send_key_and_pump(&mut harness, Key::Named(NamedKey::ArrowRight), Code::ArrowRight);
        assert_native_focus_id(&harness, "trig-two");
        send_key_and_pump(&mut harness, Key::Named(NamedKey::Enter), Code::Enter);
        assert!(harness.query("#content-two").is_some());
        send_key_and_pump(&mut harness, Key::Named(NamedKey::Tab), Code::Tab);
        assert_native_focus_id(&harness, "link-two-a");

        // Tab through content-two -> input-two -> link-two-b
        send_key_and_pump(&mut harness, Key::Named(NamedKey::Tab), Code::Tab);
        assert_native_focus_id(&harness, "input-two");
        // ArrowRight inside <input> is ignored (enable_ignored_element)
        send_key_and_pump(&mut harness, Key::Named(NamedKey::ArrowRight), Code::ArrowRight);
        assert_native_focus_id(&harness, "input-two");

        send_key_and_pump(&mut harness, Key::Named(NamedKey::Tab), Code::Tab);
        assert_native_focus_id(&harness, "link-two-b");

        // Shift+Tab walks backward inside content-two -> input-two -> link-two-a
        send_key_and_pump_with(
            &mut harness,
            Key::Named(NamedKey::Tab),
            Code::Tab,
            Modifiers::SHIFT,
        );
        assert_native_focus_id(&harness, "input-two");
    }

    #[test]
    fn native_navigation_menu_link_select_and_focus_outside_guard() {
        let _lock = NATIVE_TEST_LOCK.lock().unwrap_or_else(|e| e.into_inner());
        let mut harness = mount_native_scenario(NativeNavMenuViewportScenario);

        // Open trig-one without focusing inside menu
        click_and_pump(&mut harness, "#trig-one");
        assert!(harness.query("#content-one").is_some());

        // Focusing #nm-prev-btn when focus came from trig-one (inside menu) dismisses menu
        focus_node(&mut harness, "#nm-prev-btn");
        advance_frame_and_pump(&mut harness, 0.02);
        assert!(harness.query("#content-one").is_none());

        // Enable prevent_default on link-one-prevent, reopen trig-one, click link -> stays open
        click_and_pump(&mut harness, "#nm-toggle-prevent");
        click_and_pump(&mut harness, "#trig-one");
        assert!(harness.query("#content-one").is_some());
        click_and_pump(&mut harness, "#link-one-prevent");
        assert!(
            harness.query("#content-one").is_some(),
            "prevent_default on NavigationMenuLink.on_select must keep menu open"
        );

        // Click link-one-a without prevent_default -> dismisses menu
        click_and_pump(&mut harness, "#link-one-a");
        advance_frame_and_pump(&mut harness, 0.02);
        assert!(
            harness.query("#content-one").is_none(),
            "default NavigationMenuLink click without prevent_default must dismiss menu"
        );

        // Reopen and click outside #nm-prev-btn -> dismisses
        click_and_pump(&mut harness, "#trig-one");
        assert!(harness.query("#content-one").is_some());
        click_and_pump(&mut harness, "#nm-prev-btn");
        advance_frame_and_pump(&mut harness, 0.02);
        assert!(harness.query("#content-one").is_none());
    }

    #[component]
    fn NativeNavManualSubAndRtlScenario() -> Element {
        rsx! {
            div {
                style: "padding: 20px;",
                button { id: "outside-a", r#type: "button", "Out A" }
                button { id: "outside-b", r#type: "button", "Out B" }
                NavigationMenuRoot {
                    id: "manual-nm",
                    default_value: Some("group".to_string()),
                    activation_mode: NavigationMenuActivationMode::Manual,
                    dir: Direction::Rtl,
                    NavigationMenuList {
                        NavigationMenuItem {
                            value: Some("group".to_string()),
                            NavigationMenuTrigger { id: "m-trig-1", "Group" }
                            NavigationMenuContent {
                                id: "m-content-1",
                                NavigationMenuSub {
                                    id: "m-sub",
                                    default_value: Some("s1".to_string()),
                                    NavigationMenuList {
                                        NavigationMenuItem {
                                            value: Some("s1".to_string()),
                                            NavigationMenuTrigger { id: "sub-t1", "S1" }
                                            NavigationMenuContent {
                                                id: "sub-c1",
                                                NavigationMenuLink { id: "sub-l1", href: "#s1", "L1" }
                                            }
                                        }
                                        NavigationMenuItem {
                                            value: Some("s2".to_string()),
                                            NavigationMenuTrigger { id: "sub-t2", "S2" }
                                            NavigationMenuContent {
                                                id: "sub-c2",
                                                NavigationMenuLink { id: "sub-l2", href: "#s2", "L2" }
                                            }
                                        }
                                    }
                                }
                            }
                        }
                        NavigationMenuItem {
                            value: Some("click-only".to_string()),
                            open_on_hover: Some(false),
                            NavigationMenuTrigger { id: "m-trig-2", "Click Only" }
                            NavigationMenuContent {
                                id: "m-content-2",
                                NavigationMenuLink { id: "m-link-2", href: "#2", "M2" }
                            }
                        }
                    }
                }
            }
        }
    }

    #[test]
    fn native_navigation_menu_manual_sub_disable_toggle_and_focus_outside_3473() {
        let _lock = NATIVE_TEST_LOCK.lock().unwrap_or_else(|e| e.into_inner());
        let mut harness = mount_native_scenario(NativeNavManualSubAndRtlScenario);

        // Inline mode: content renders inside item, no viewport
        assert!(harness.query("#m-content-1").is_some());
        assert!(harness.query("#sub-c1").is_some());

        // #3473 guard: moving focus between outside-a and outside-b never originated in menu -> stays open
        focus_node(&mut harness, "#outside-a");
        focus_node(&mut harness, "#outside-b");
        advance_frame_and_pump(&mut harness, 0.02);
        assert!(
            harness.query("#m-content-1").is_some(),
            "#3473: focus never originated in menu, so external focus move must not dismiss"
        );

        // Manual mode inherited by Sub: hovering sub-t2 must NOT switch sub-c1 -> sub-c2
        let (s2x, s2y) = harness.center_of("#sub-t2");
        harness.move_mouse_to(s2x, s2y);
        std::thread::sleep(std::time::Duration::from_millis(80));
        advance_frame_and_pump(&mut harness, 0.02);
        assert!(harness.query("#sub-c1").is_some());
        assert!(harness.query("#sub-c2").is_none());

        // Clicking sub-t2 switches to sub-c2, and re-clicking sub-t2 keeps it open (disable_toggle=true)
        click_and_pump(&mut harness, "#sub-t2");
        assert!(harness.query("#sub-c2").is_some());
        click_and_pump(&mut harness, "#sub-t2");
        assert!(harness.query("#sub-c2").is_some());

        // RTL ArrowLeft on m-trig-1 moves forward to m-trig-2
        focus_node(&mut harness, "#m-trig-1");
        send_key_and_pump(&mut harness, Key::Named(NamedKey::ArrowLeft), Code::ArrowLeft);
        assert_native_focus_id(&harness, "m-trig-2");

        // Manual mode click toggles m-trig-2 open and closed
        click_and_pump(&mut harness, "#m-trig-2");
        assert!(harness.query("#m-content-2").is_some());
        click_and_pump(&mut harness, "#m-trig-2");
        assert!(harness.query("#m-content-2").is_none());
    }

    #[test]
    fn native_navigation_menu_escape_inside_content_restores_focus_visible() {
        let _lock = NATIVE_TEST_LOCK.lock().unwrap_or_else(|e| e.into_inner());
        let mut harness = mount_native_scenario(NativeNavMenuViewportScenario);

        focus_node(&mut harness, "#trig-one");
        send_key_and_pump(&mut harness, Key::Named(NamedKey::Enter), Code::Enter);
        assert!(harness.query("#content-one").is_some());

        send_key_and_pump(&mut harness, Key::Named(NamedKey::ArrowDown), Code::ArrowDown);
        assert_native_focus_id(&harness, "link-one-a");

        send_key_and_pump(&mut harness, Key::Named(NamedKey::Escape), Code::Escape);
        advance_frame_and_pump(&mut harness, 0.02);
        assert!(harness.query("#content-one").is_none());
        assert_native_focus_visible(&harness, "trig-one", true);
    }

    #[component]
    fn NativeNavMenuWithNestedPopoverScenario() -> Element {
        use monoxus::popover::{PopoverContent, PopoverPortal, PopoverRoot, PopoverTrigger};

        let nav_val = use_signal(|| "products".to_string());
        let pop_open = use_signal(|| false);

        rsx! {
            div {
                id: "nav-nested-wrapper",
                NavigationMenuRoot {
                    id: "nav-nested-root",
                    value: Some(nav_val),
                    NavigationMenuList {
                        NavigationMenuItem {
                            value: Some("products".to_string()),
                            NavigationMenuTrigger { id: "nav-nested-trig", "Products" }
                            NavigationMenuContent {
                                id: "nav-nested-content",
                                PopoverRoot {
                                    id: "nav-inner-popover",
                                    open: pop_open,
                                    PopoverTrigger { id: "nav-inner-pop-trig", "Nested Popover" }
                                    PopoverPortal {
                                        PopoverContent {
                                            id: "nav-inner-pop-content",
                                            button { id: "nav-inner-pop-btn", "Pop Action" }
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
    fn fm_ovl_1_and_fm_ovl_8_navigation_menu_ime_escape_and_nested_popover_escape_coordination() {
        use blitz_traits::events::{BlitzKeyEvent, KeyState, UiEvent};
        use keyboard_types::{Location, Modifiers};

        let _lock = NATIVE_TEST_LOCK.lock().unwrap_or_else(|e| e.into_inner());
        let mut harness = mount_native_scenario(NativeNavMenuWithNestedPopoverScenario);
        advance_frame_and_pump(&mut harness, 0.02);

        assert!(harness.query("#nav-nested-content").is_some());

        // Open nested Popover inside NavigationMenuContent
        click_and_pump(&mut harness, "#nav-inner-pop-trig");
        advance_frame_and_pump(&mut harness, 0.02);
        assert!(harness.query("#nav-inner-pop-content").is_some());
        assert!(harness.query("#nav-nested-content").is_some());

        // FM-OVL-1: IME composing Escape must dismiss neither nested Popover nor NavigationMenu
        let ime_down = BlitzKeyEvent {
            key: Key::Named(NamedKey::Escape),
            code: Code::Escape,
            modifiers: Modifiers::default(),
            location: Location::Standard,
            is_auto_repeating: false,
            is_composing: true,
            state: KeyState::Pressed,
            text: None,
        };
        let ime_up = BlitzKeyEvent {
            key: Key::Named(NamedKey::Escape),
            code: Code::Escape,
            modifiers: Modifiers::default(),
            location: Location::Standard,
            is_auto_repeating: false,
            is_composing: true,
            state: KeyState::Released,
            text: None,
        };
        harness.dispatch(UiEvent::KeyDown(ime_down));
        harness.dispatch(UiEvent::KeyUp(ime_up));
        drain_polls_and_resolve(&mut harness);
        advance_frame_and_pump(&mut harness, 0.02);
        assert!(harness.query("#nav-inner-pop-content").is_some());
        assert!(harness.query("#nav-nested-content").is_some());

        // FM-OVL-8: First normal Escape closes ONLY nested Popover; NavigationMenu stays open
        send_key_and_pump(&mut harness, Key::Named(NamedKey::Escape), Code::Escape);
        advance_frame_and_pump(&mut harness, 0.02);
        assert!(
            harness.query("#nav-inner-pop-content").is_none(),
            "First Escape must close nested Popover"
        );
        assert!(
            harness.query("#nav-nested-content").is_some(),
            "NavigationMenu must remain open when nested Popover consumes Escape"
        );

        // Second normal Escape closes NavigationMenu
        send_key_and_pump(&mut harness, Key::Named(NamedKey::Escape), Code::Escape);
        advance_frame_and_pump(&mut harness, 0.02);
        assert!(
            harness.query("#nav-nested-content").is_none(),
            "Second Escape must close NavigationMenu"
        );
    }
}

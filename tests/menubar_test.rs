use std::{cell::RefCell, rc::Rc};

use dioxus::prelude::*;
use monoxus::foundation::shared::Direction;
use monoxus::menubar::{
    Menubar, MenubarContent, MenubarContentStatic, MenubarItem, MenubarMenu, MenubarPortal,
    MenubarRoot, MenubarRootRenderProps, MenubarRuntimeContext, MenubarSub, MenubarSubContent,
    MenubarSubTrigger, MenubarTrigger, MenubarTriggerRenderProps,
};

#[derive(Clone, Default, PartialEq)]
struct MenubarProbeSnapshot {
    active_value: Option<String>,
    menu_count: usize,
}

#[component]
fn MenubarProbe(out: Rc<RefCell<MenubarProbeSnapshot>>) -> Element {
    let runtime = use_context::<MenubarRuntimeContext>();
    *out.borrow_mut() = MenubarProbeSnapshot {
        active_value: runtime.active_value(),
        menu_count: runtime.state.read().menus().len(),
    };
    rsx! {}
}

#[component]
fn MenubarVdomSample(
    default_value: Option<String>,
    out: Rc<RefCell<MenubarProbeSnapshot>>,
) -> Element {
    rsx! {
        MenubarRoot {
            id: "mb-vdom-root",
            default_value: default_value,
            MenubarMenu {
                value: Some("file".to_string()),
                MenubarTrigger { id: "mb-vdom-file", "File" }
                MenubarPortal {
                    MenubarContent {
                        id: "mb-vdom-file-content",
                        MenubarItem { id: "mb-vdom-new", "New Tab" }
                    }
                }
            }
            MenubarMenu {
                value: Some("edit".to_string()),
                MenubarTrigger { id: "mb-vdom-edit", "Edit" }
                MenubarPortal {
                    MenubarContent {
                        id: "mb-vdom-edit-content",
                        MenubarItem { id: "mb-vdom-undo", "Undo" }
                    }
                }
            }
            MenubarProbe { out: out }
        }
    }
}

#[test]
fn test_menubar_root_registers_menus_and_default_value() {
    let probe = Rc::new(RefCell::new(MenubarProbeSnapshot::default()));
    let mut dom = VirtualDom::new_with_props(
        MenubarVdomSample,
        MenubarVdomSampleProps {
            default_value: Some("file".to_string()),
            out: probe.clone(),
        },
    );
    dom.rebuild_in_place();

    let snap = probe.borrow();
    assert_eq!(snap.active_value.as_deref(), Some("file"));
    assert_eq!(snap.menu_count, 2);
}

#[component]
fn MenubarAsChildAndStaticSample() -> Element {
    let root_render = Callback::new(|props: MenubarRootRenderProps| {
        rsx! {
            nav {
                id: "{props.attrs.id}",
                role: "{props.attrs.role}",
                "aria-orientation": "{props.attrs.aria_orientation}",
                dir: "{props.attrs.dir}",
                ..props.attributes,
                {props.children}
            }
        }
    });

    let trig_render = Callback::new(|props: MenubarTriggerRenderProps| {
        rsx! {
            button {
                r#type: "button",
                id: "{props.attrs.id}",
                role: "{props.attrs.role}",
                tabindex: "{props.attrs.tabindex}",
                "aria-haspopup": "{props.attrs.aria_haspopup}",
                "aria-expanded": "{props.attrs.aria_expanded}",
                onpointerdown: move |evt| props.on_pointer_down.call(evt),
                onclick: move |evt| props.on_click.call(evt),
                onpointerenter: move |evt| props.on_pointer_enter.call(evt),
                onfocus: move |evt| props.on_focus.call(evt),
                onblur: move |evt| props.on_blur.call(evt),
                onkeydown: move |evt| props.on_key_down.call(evt),
                ..props.attributes,
                {props.children}
            }
        }
    });

    rsx! {
        Menubar {
            id: "mb-ac-root",
            default_value: Some("view".to_string()),
            as_child: true,
            render_child: Some(root_render),
            MenubarMenu {
                value: Some("view".to_string()),
                MenubarTrigger {
                    id: "mb-ac-trig",
                    as_child: true,
                    render_child: Some(trig_render),
                    "View"
                }
                MenubarContentStatic {
                    id: "mb-ac-static-content",
                    MenubarItem { "Zoom In" }
                }
            }
        }
    }
}

#[test]
fn test_menubar_as_child_and_static_content_render_cleanly() {
    let mut dom = VirtualDom::new(MenubarAsChildAndStaticSample);
    dom.rebuild_in_place();
}

#[cfg(feature = "native")]
mod native_tests {
    use super::*;
    use crate::common::native_fixture::*;
    use keyboard_types::{Code, Key, NamedKey};

    #[component]
    fn NativeMenubarScenario() -> Element {
        let mut active_val = use_signal(String::new);

        rsx! {
            div {
                style: "padding: 24px;",
                MenubarRoot {
                    id: "mb-root",
                    on_value_change: move |v| active_val.set(v),
                    style: "display: flex; gap: 8px;",
                    MenubarMenu {
                        value: Some("file".to_string()),
                        MenubarTrigger {
                            id: "mb-trig-file",
                            style: "box-sizing: border-box; width: 60px; height: 28px;",
                            "File"
                        }
                        MenubarPortal {
                            MenubarContent {
                                id: "mb-content-file",
                                style: "width: 160px; padding: 4px; background: white;",
                                MenubarItem { id: "mb-file-new", "New Tab" }
                                MenubarSub {
                                    MenubarSubTrigger {
                                        id: "mb-file-share",
                                        "Share"
                                    }
                                    MenubarPortal {
                                        MenubarSubContent {
                                            id: "mb-file-share-sub",
                                            style: "width: 130px; padding: 4px; background: white;",
                                            MenubarItem { id: "mb-share-email", "Email" }
                                        }
                                    }
                                }
                                MenubarItem { id: "mb-file-print", "Print" }
                            }
                        }
                    }
                    MenubarMenu {
                        value: Some("edit".to_string()),
                        MenubarTrigger {
                            id: "mb-trig-edit",
                            disabled: true,
                            style: "width: 60px; height: 28px;",
                            "Edit"
                        }
                        MenubarPortal {
                            MenubarContent {
                                id: "mb-content-edit",
                                MenubarItem { "Undo" }
                            }
                        }
                    }
                    MenubarMenu {
                        value: Some("view".to_string()),
                        MenubarTrigger {
                            id: "mb-trig-view",
                            style: "box-sizing: border-box; width: 60px; height: 28px;",
                            "View"
                        }
                        MenubarPortal {
                            MenubarContent {
                                id: "mb-content-view",
                                style: "width: 160px; padding: 4px; background: white;",
                                MenubarItem { id: "mb-view-zoom", "Zoom In" }
                                MenubarItem { id: "mb-view-reset", "Reset Zoom" }
                            }
                        }
                    }
                }
                button {
                    id: "mb-outside-btn",
                    r#type: "button",
                    style: "margin-top: 120px;",
                    "Outside"
                }
                div {
                    id: "mb-probe",
                    "data-active": "{active_val}",
                }
            }
        }
    }

    #[test]
    fn native_menubar_apg_attrs_roving_arrows_and_hover_swap() {
        let _lock = NATIVE_TEST_LOCK.lock().unwrap_or_else(|e| e.into_inner());
        let mut harness = mount_native_scenario(NativeMenubarScenario);

        // WAI-ARIA APG menubar root & roving trigger tabindex
        assert_eq!(harness.attr("#mb-root", "role").as_deref(), Some("menubar"));
        assert_eq!(
            harness.attr("#mb-root", "aria-orientation").as_deref(),
            Some("horizontal")
        );
        assert_eq!(harness.attr("#mb-trig-file", "role").as_deref(), Some("menuitem"));
        assert_eq!(harness.attr("#mb-trig-file", "tabindex").as_deref(), Some("0"));
        assert_eq!(harness.attr("#mb-trig-view", "tabindex").as_deref(), Some("-1"));
        assert_eq!(harness.attr("#mb-trig-file", "aria-controls"), None);

        // Hovering View while closed does NOT open View (`MB-3`)
        let (vx, vy) = harness.center_of("#mb-trig-view");
        harness.move_mouse_to(vx, vy);
        advance_frame_and_pump(&mut harness, 0.02);
        assert!(harness.query("#mb-content-view").is_none());

        // Closed horizontal ArrowRight skips disabled Edit trigger and moves focus to View WITHOUT opening (`MB-2`)
        focus_node(&mut harness, "#mb-trig-file");
        send_key_and_pump(&mut harness, Key::Named(NamedKey::ArrowRight), Code::ArrowRight);
        assert_native_focus_id(&harness, "mb-trig-view");
        assert!(harness.query("#mb-content-view").is_none());

        // ArrowRight wraps back to File trigger (`loop_focus: true`, `3B D-5`)
        send_key_and_pump(&mut harness, Key::Named(NamedKey::ArrowRight), Code::ArrowRight);
        assert_native_focus_id(&harness, "mb-trig-file");

        // ArrowDown opens File and focuses first item (`MB-2`)
        send_key_and_pump(&mut harness, Key::Named(NamedKey::ArrowDown), Code::ArrowDown);
        advance_frame_and_pump(&mut harness, 0.02);
        assert!(harness.query("#mb-content-file").is_some());
        assert_relative_below(&harness, "#mb-trig-file", "#mb-content-file", 2.0);
        assert_native_focus_id(&harness, "mb-file-new");
        assert_eq!(
            harness.attr("#mb-trig-file", "aria-controls").as_deref(),
            Some("mb-content-file")
        );

        // Hovering View while File is open switches immediately (`MB-3`, `MB-5`)
        let (fx, fy) = harness.center_of("#mb-trig-file");
        harness.move_mouse_to(fx, fy);
        drain_polls_and_resolve(&mut harness);
        advance_frame_and_pump(&mut harness, 0.01);
        harness.move_mouse_to(vx, vy);
        drain_polls_and_resolve(&mut harness);
        advance_frame_and_pump(&mut harness, 0.02);
        drain_polls_and_resolve(&mut harness);
        assert!(harness.query("#mb-content-file").is_none());
        assert!(harness.query("#mb-content-view").is_some());
        assert_eq!(
            harness.attr("#mb-probe", "data-active").as_deref(),
            Some("view")
        );

        // Clicking outside dismisses open menu (`MB-4`)
        click_and_pump(&mut harness, "#mb-outside-btn");
        advance_frame_and_pump(&mut harness, 0.02);
        assert!(harness.query("#mb-content-view").is_none());
        assert_eq!(harness.attr("#mb-probe", "data-active").as_deref(), Some(""));
    }

    #[test]
    fn native_menubar_cross_menu_arrows_and_submenu_precedence() {
        let _lock = NATIVE_TEST_LOCK.lock().unwrap_or_else(|e| e.into_inner());
        let mut harness = mount_native_scenario(NativeMenubarScenario);

        // Open File via ArrowDown -> mb-file-new
        focus_node(&mut harness, "#mb-trig-file");
        send_key_and_pump(&mut harness, Key::Named(NamedKey::ArrowDown), Code::ArrowDown);
        advance_frame_and_pump(&mut harness, 0.02);
        assert_native_focus_id(&harness, "mb-file-new");

        // Navigate down to SubTrigger (mb-file-share), ArrowRight opens submenu (`MB-6`)
        send_key_and_pump(&mut harness, Key::Named(NamedKey::ArrowDown), Code::ArrowDown);
        assert_native_focus_id(&harness, "mb-file-share");
        send_key_and_pump(&mut harness, Key::Named(NamedKey::ArrowRight), Code::ArrowRight);
        advance_frame_and_pump(&mut harness, 0.02);
        assert!(harness.query("#mb-file-share-sub").is_some());
        assert!(harness.query("#mb-content-file").is_some());
        assert_native_focus_id(&harness, "mb-share-email");

        // Inside SubContent, ArrowLeft closes ONLY submenu back to SubTrigger (does NOT switch menubar menu) (`MB-6`)
        send_key_and_pump(&mut harness, Key::Named(NamedKey::ArrowLeft), Code::ArrowLeft);
        advance_frame_and_pump(&mut harness, 0.02);
        assert!(harness.query("#mb-file-share-sub").is_none());
        assert!(harness.query("#mb-content-file").is_some());
        assert_native_focus_id(&harness, "mb-file-share");

        // Re-open submenu and test ArrowRight on leaf item inside SubContent -> switches to next Menubar menu ("view") (`MB-6`)
        send_key_and_pump(&mut harness, Key::Named(NamedKey::ArrowRight), Code::ArrowRight);
        advance_frame_and_pump(&mut harness, 0.02);
        assert_native_focus_id(&harness, "mb-share-email");
        send_key_and_pump(&mut harness, Key::Named(NamedKey::ArrowRight), Code::ArrowRight);
        advance_frame_and_pump(&mut harness, 0.02);
        assert!(harness.query("#mb-content-file").is_none());
        assert!(harness.query("#mb-content-view").is_some());
        assert_native_focus_id(&harness, "mb-view-zoom");

        // Inside top-level View content, ArrowLeft switches back to File (`MB-6`)
        send_key_and_pump(&mut harness, Key::Named(NamedKey::ArrowLeft), Code::ArrowLeft);
        advance_frame_and_pump(&mut harness, 0.02);
        assert!(harness.query("#mb-content-view").is_none());
        assert!(harness.query("#mb-content-file").is_some());
        assert_native_focus_id(&harness, "mb-file-new");
    }

    #[component]
    fn NativeMenubarRtlScrollableScenario() -> Element {
        rsx! {
            div {
                id: "mb-rtl-scroll-parent",
                style: "margin-left: 260px; width: 520px; height: 220px; overflow-y: auto; padding: 24px 24px 24px 120px; box-sizing: border-box;",
                MenubarRoot {
                    id: "mb-rtl-root",
                    dir: Some(Direction::Rtl),
                    style: "display: inline-flex; margin-left: 80px; gap: 8px;",
                    MenubarMenu {
                        value: Some("file".to_string()),
                        MenubarTrigger {
                            id: "mb-rtl-trig-file",
                            style: "box-sizing: border-box; width: 70px; height: 28px;",
                            "File"
                        }
                        MenubarPortal {
                            MenubarContent {
                                id: "mb-rtl-content-file",
                                style: "width: 150px; padding: 4px; background: white;",
                                MenubarItem { id: "mb-rtl-item-1", "New Tab" }
                                MenubarSub {
                                    MenubarSubTrigger {
                                        id: "mb-rtl-sub-trig",
                                        "Share"
                                    }
                                    MenubarPortal {
                                        MenubarSubContent {
                                            id: "mb-rtl-sub-content",
                                            style: "width: 120px; padding: 4px; background: white;",
                                            MenubarItem { id: "mb-rtl-sub-item", "Email" }
                                        }
                                    }
                                }
                                MenubarItem { id: "mb-rtl-item-2", "Print" }
                            }
                        }
                    }
                }
                div { style: "height: 240px;" }
            }
        }
    }

    #[test]
    fn native_menubar_rtl_submenu_opens_left_inside_scrollable_container() {
        let _lock = NATIVE_TEST_LOCK.lock().unwrap_or_else(|e| e.into_inner());
        let mut harness = mount_native_scenario(NativeMenubarRtlScrollableScenario);

        // Open File via ArrowDown -> mb-rtl-item-1, then ArrowDown -> mb-rtl-sub-trig
        focus_node(&mut harness, "#mb-rtl-trig-file");
        send_key_and_pump(&mut harness, Key::Named(NamedKey::ArrowDown), Code::ArrowDown);
        advance_frame_and_pump(&mut harness, 0.02);
        assert!(harness.query("#mb-rtl-content-file").is_some());
        assert_native_focus_id(&harness, "mb-rtl-item-1");

        send_key_and_pump(&mut harness, Key::Named(NamedKey::ArrowDown), Code::ArrowDown);
        assert_native_focus_id(&harness, "mb-rtl-sub-trig");

        // In RTL, ArrowLeft opens the submenu to the LEFT of SubTrigger
        send_key_and_pump(&mut harness, Key::Named(NamedKey::ArrowLeft), Code::ArrowLeft);
        advance_frame_and_pump(&mut harness, 0.02);
        advance_frame_and_pump(&mut harness, 0.02);

        assert!(harness.query("#mb-rtl-sub-content").is_some());
        assert!(harness.query("[data-portal-host='default']").is_some());
        let trigger_rect = harness.layout_rect("#mb-rtl-sub-trig");
        let content_rect = harness.layout_rect("#mb-rtl-content-file");
        let sub_rect = harness.layout_rect("#mb-rtl-sub-content");
        let vp = harness.base().viewport().window_size;
        assert_eq!(
            harness.attr("#mb-rtl-sub-content", "data-side").as_deref(),
            Some("left"),
            "sub_rect={sub_rect:?}, trigger_rect={trigger_rect:?}, content_rect={content_rect:?}, vp={vp:?}"
        );
        assert!(
            sub_rect.x < trigger_rect.x && sub_rect.x + sub_rect.width <= trigger_rect.x + 2.0,
            "RTL SubContent must open to the left of SubTrigger: sub={sub_rect:?}, trigger={trigger_rect:?}"
        );
    }
}

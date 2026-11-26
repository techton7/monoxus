use std::{cell::RefCell, rc::Rc};

use dioxus::prelude::*;
use monoxus::dropdown_menu::{
    CheckedState, DropdownMenu, DropdownMenuArrow, DropdownMenuCheckboxGroup,
    DropdownMenuCheckboxItem, DropdownMenuContent, DropdownMenuContentRenderProps,
    DropdownMenuContentStatic, DropdownMenuGroup, DropdownMenuGroupHeading, DropdownMenuItem,
    DropdownMenuItemIndicator, DropdownMenuLabel, DropdownMenuPortal, DropdownMenuRadioGroup,
    DropdownMenuRadioItem, DropdownMenuRoot, DropdownMenuSeparator, DropdownMenuSub,
    DropdownMenuSubContent, DropdownMenuSubContentStatic, DropdownMenuSubTrigger,
    DropdownMenuTrigger, DropdownMenuTriggerRenderProps, MenuRootContext, MenuSelectEvent,
};
use monoxus::foundation::shared::Direction;

#[derive(Clone, Default, PartialEq)]
struct DropdownProbeSnapshot {
    open: bool,
    modal: bool,
    dir: Direction,
    trigger_id: String,
    content_id: String,
}

#[component]
fn DropdownProbe(out: Rc<RefCell<DropdownProbeSnapshot>>) -> Element {
    let root = use_context::<MenuRootContext>();
    *out.borrow_mut() = DropdownProbeSnapshot {
        open: root.is_open(),
        modal: root.is_modal(),
        dir: root.direction(),
        trigger_id: root.trigger_id.read().clone(),
        content_id: root.root_surface.current_content_id(),
    };
    rsx! {}
}

#[component]
fn DropdownVdomSample(
    default_open: bool,
    probe_out: Rc<RefCell<DropdownProbeSnapshot>>,
) -> Element {
    let checked_state = use_signal(|| CheckedState::Indeterminate);
    let group_vals = use_signal(|| vec!["bold".to_string()]);
    let radio_val = use_signal(|| "pedro".to_string());

    rsx! {
        DropdownMenuRoot {
            default_open: default_open,
            DropdownMenuTrigger {
                id: "dm-vdom-trigger",
                "Open Menu"
            }
            DropdownMenuPortal {
                DropdownMenuContent {
                    id: "dm-vdom-content",
                    DropdownMenuLabel { id: "dm-vdom-label", "My Account" }
                    DropdownMenuGroup {
                        id: "dm-vdom-group",
                        DropdownMenuGroupHeading { id: "dm-vdom-heading", "Formatting" }
                        DropdownMenuCheckboxGroup {
                            value: Some(group_vals()),
                            DropdownMenuCheckboxItem {
                                id: "dm-cb-bold",
                                value: Some("bold".to_string()),
                                "Bold"
                                DropdownMenuItemIndicator { "check" }
                            }
                            DropdownMenuCheckboxItem {
                                id: "dm-cb-italic",
                                value: Some("italic".to_string()),
                                "Italic"
                                DropdownMenuItemIndicator { force_mount: true, "ind" }
                            }
                        }
                    }
                    DropdownMenuSeparator { id: "dm-vdom-sep" }
                    DropdownMenuCheckboxItem {
                        id: "dm-cb-tri",
                        checked: Some(checked_state()),
                        "Mixed Option"
                        DropdownMenuItemIndicator { "mixed" }
                    }
                    DropdownMenuRadioGroup {
                        value: Some(radio_val()),
                        DropdownMenuRadioItem {
                            id: "dm-radio-pedro",
                            value: "pedro",
                            "Pedro"
                            DropdownMenuItemIndicator { "dot" }
                        }
                    }
                    DropdownMenuSub {
                        DropdownMenuSubTrigger {
                            id: "dm-vdom-sub-trig",
                            "More"
                        }
                        DropdownMenuPortal {
                            DropdownMenuSubContent {
                                id: "dm-vdom-sub-content",
                                DropdownMenuItem { id: "dm-vdom-sub-item", "Email" }
                            }
                        }
                    }
                    DropdownMenuArrow { id: "dm-vdom-arrow" }
                }
            }
            DropdownProbe { out: probe_out }
        }
    }
}

#[test]
fn test_dropdown_menu_alias_and_default_modal_contract() {
    let probe = Rc::new(RefCell::new(DropdownProbeSnapshot::default()));
    let mut dom = VirtualDom::new_with_props(
        DropdownVdomSample,
        DropdownVdomSampleProps {
            default_open: true,
            probe_out: probe.clone(),
        },
    );
    dom.rebuild_in_place();

    let snap = probe.borrow();
    assert!(snap.open);
    assert!(snap.modal, "3B D-1: DropdownMenu must default to modal = true");
    assert_eq!(snap.dir, Direction::Ltr);
    assert_eq!(snap.trigger_id, "dm-vdom-trigger");
    assert_eq!(snap.content_id, "dm-vdom-content");
}

#[component]
fn DropdownStaticAndAsChildSample() -> Element {
    let trig_render = Callback::new(|props: DropdownMenuTriggerRenderProps| {
        rsx! {
            button {
                r#type: "button",
                id: "{props.attrs.id}",
                class: props.class.as_deref().unwrap_or_default(),
                "aria-haspopup": "{props.attrs.aria_haspopup}",
                "aria-expanded": "{props.attrs.aria_expanded}",
                "data-state": "{props.attrs.data_state.as_str()}",
                onpointerdown: move |evt| props.on_pointer_down.call(evt),
                onclick: move |evt| props.on_click.call(evt),
                onkeydown: move |evt| props.on_key_down.call(evt),
                ..props.attributes,
                {props.children}
            }
        }
    });

    let content_render = Callback::new(|props: DropdownMenuContentRenderProps| {
        rsx! {
            section {
                id: "{props.attrs.id}",
                role: "{props.attrs.role}",
                tabindex: "{props.attrs.tabindex}",
                "data-state": "{props.attrs.data_state.as_str()}",
                onkeydown: move |evt| props.on_key_down.call(evt),
                ..props.attributes,
                {props.children}
            }
        }
    });

    rsx! {
        DropdownMenu {
            default_open: true,
            DropdownMenuTrigger {
                id: "dm-ac-trig",
                as_child: true,
                render_child: Some(trig_render),
                "Custom Trigger"
            }
            DropdownMenuContentStatic {
                id: "dm-static-content",
                as_child: true,
                render_child: Some(content_render),
                DropdownMenuItem { id: "dm-static-item", "Static Action" }
                DropdownMenuSub {
                    default_open: true,
                    DropdownMenuSubTrigger { id: "dm-static-sub-trig", "More" }
                    DropdownMenuSubContentStatic {
                        id: "dm-static-sub-content",
                        DropdownMenuItem { id: "dm-static-sub-item", "Nested" }
                    }
                }
            }
        }
    }
}

#[test]
fn test_dropdown_menu_static_content_and_as_child_render_cleanly() {
    let mut dom = VirtualDom::new(DropdownStaticAndAsChildSample);
    dom.rebuild_in_place();
}

#[cfg(feature = "native")]
mod native_tests {
    use super::*;
    use crate::common::native_fixture::*;
    use keyboard_types::{Code, Key, Modifiers, NamedKey};

    #[component]
    fn NativeDropdownFullScenario() -> Element {
        let mut selected_log = use_signal(Vec::<String>::new);
        let mut tri_checked = use_signal(|| CheckedState::Indeterminate);
        let mut group_values = use_signal(|| vec!["bold".to_string()]);
        let mut radio_value = use_signal(|| "left".to_string());
        let mut modal_mode = use_signal(|| true);
        let mut outside_clicks = use_signal(|| 0usize);

        rsx! {
            div {
                style: "padding: 24px; display: flex; align-items: center; gap: 8px;",
                button {
                    id: "dm-prev-focus",
                    r#type: "button",
                    "Prev Focus"
                }
                DropdownMenuRoot {
                    modal: modal_mode(),
                    DropdownMenuTrigger {
                        id: "dm-trig",
                        style: "box-sizing: border-box; width: 120px; height: 32px;",
                        "Actions"
                    }
                    DropdownMenuPortal {
                        DropdownMenuContent {
                            id: "dm-content",
                            style: "width: 180px; padding: 6px; background: white;",
                            DropdownMenuGroup {
                                id: "dm-grp",
                                DropdownMenuGroupHeading { id: "dm-grp-heading", "Edit" }
                                DropdownMenuItem {
                                    id: "dm-item-cut",
                                    on_select: move |_| selected_log.write().push("cut".to_string()),
                                    "Cut"
                                }
                                DropdownMenuItem {
                                    id: "dm-item-disabled",
                                    disabled: true,
                                    on_select: move |_| selected_log.write().push("disabled".to_string()),
                                    "Copy"
                                }
                                DropdownMenuItem {
                                    id: "dm-item-paste",
                                    close_on_select: false,
                                    on_select: move |_| selected_log.write().push("paste".to_string()),
                                    "Paste"
                                }
                                DropdownMenuItem {
                                    id: "dm-item-prevent",
                                    on_select: move |evt: MenuSelectEvent| {
                                        evt.prevent_default();
                                        selected_log.write().push("prevent".to_string());
                                    },
                                    "Print"
                                }
                            }
                            DropdownMenuSeparator { id: "dm-sep" }
                            DropdownMenuCheckboxItem {
                                id: "dm-item-tri",
                                checked: Some(tri_checked()),
                                close_on_select: false,
                                on_checked_change: move |next| tri_checked.set(next),
                                "Status"
                                DropdownMenuItemIndicator {
                                    id: "dm-tri-ind",
                                    "*"
                                }
                            }
                            DropdownMenuCheckboxGroup {
                                id: "dm-cb-grp",
                                value: Some(group_values()),
                                on_value_change: move |next| group_values.set(next),
                                DropdownMenuCheckboxItem {
                                    id: "dm-cb-bold",
                                    value: Some("bold".to_string()),
                                    close_on_select: false,
                                    "Bold"
                                    DropdownMenuItemIndicator { id: "dm-bold-ind", "B" }
                                }
                                DropdownMenuCheckboxItem {
                                    id: "dm-cb-italic",
                                    value: Some("italic".to_string()),
                                    close_on_select: false,
                                    "Italic"
                                    DropdownMenuItemIndicator {
                                        id: "dm-italic-ind",
                                        force_mount: true,
                                        "I"
                                    }
                                }
                            }
                            DropdownMenuRadioGroup {
                                id: "dm-rg",
                                value: Some(radio_value()),
                                on_value_change: move |next| radio_value.set(next),
                                DropdownMenuRadioItem {
                                    id: "dm-rad-left",
                                    value: "left",
                                    close_on_select: false,
                                    "Left"
                                    DropdownMenuItemIndicator { id: "dm-rad-left-ind", "L" }
                                }
                                DropdownMenuRadioItem {
                                    id: "dm-rad-right",
                                    value: "right",
                                    close_on_select: false,
                                    "Right"
                                    DropdownMenuItemIndicator { id: "dm-rad-right-ind", "R" }
                                }
                            }
                            DropdownMenuSub {
                                DropdownMenuSubTrigger {
                                    id: "dm-sub-trig",
                                    "Share"
                                }
                                DropdownMenuPortal {
                                    DropdownMenuSubContent {
                                        id: "dm-sub-content",
                                        style: "width: 140px; padding: 6px; background: white;",
                                        DropdownMenuItem {
                                            id: "dm-sub-email",
                                            on_select: move |_| selected_log.write().push("email".to_string()),
                                            "Email"
                                        }
                                        DropdownMenuItem {
                                            id: "dm-sub-slack",
                                            on_select: move |_| selected_log.write().push("slack".to_string()),
                                            "Slack"
                                        }
                                    }
                                }
                            }
                        }
                    }
                }
                button {
                    id: "dm-next-focus",
                    r#type: "button",
                    onclick: move |_| {
                        let next = *outside_clicks.peek() + 1;
                        outside_clicks.set(next);
                    },
                    "Next Focus"
                }
                button {
                    id: "dm-toggle-modal",
                    r#type: "button",
                    onclick: move |_| {
                        let cur = *modal_mode.peek();
                        modal_mode.set(!cur);
                    },
                    "Toggle Modal"
                }
                div {
                    id: "dm-probe-state",
                    "data-selected": "{selected_log.read().join(\",\")}",
                    "data-outside-clicks": "{outside_clicks}",
                    "data-group": "{group_values.read().join(\",\")}",
                    "data-radio": "{radio_value}",
                }
            }
        }
    }

    #[test]
    fn native_dropdown_menu_pointer_open_placement_css_vars_and_aria() {
        let _lock = NATIVE_TEST_LOCK.lock().unwrap_or_else(|e| e.into_inner());
        let mut harness = mount_native_scenario(NativeDropdownFullScenario);

        // Closed ARIA state
        assert_eq!(
            harness.attr("#dm-trig", "aria-haspopup").as_deref(),
            Some("menu")
        );
        assert_eq!(
            harness.attr("#dm-trig", "aria-expanded").as_deref(),
            Some("false")
        );
        assert_eq!(harness.attr("#dm-trig", "aria-controls"), None);
        assert_eq!(
            harness.attr("#dm-trig", "data-monoxus-dropdown-menu-trigger").as_deref(),
            Some("")
        );
        assert!(harness.query("#dm-content").is_none());

        // Click trigger -> opens on pointerdown below trigger with canonical + radix CSS vars
        click_and_pump(&mut harness, "#dm-trig");
        advance_frame_and_pump(&mut harness, 0.02);

        assert_eq!(
            harness.attr("#dm-trig", "aria-expanded").as_deref(),
            Some("true")
        );
        assert_eq!(
            harness.attr("#dm-trig", "aria-controls").as_deref(),
            Some("dm-content")
        );
        assert!(harness.query("#dm-content").is_some());
        assert_relative_below(&harness, "#dm-trig", "#dm-content", 2.0);

        let content_style = harness.attr("#dm-content", "style").unwrap_or_default();
        assert!(
            content_style.contains("--monoxus-dropdown-menu-content-transform-origin:"),
            "content_style={content_style}"
        );
        assert!(
            content_style.contains("--monoxus-dropdown-menu-anchor-width: 120px;"),
            "content_style={content_style}"
        );
        assert!(
            content_style.contains("--radix-dropdown-menu-trigger-width: 120px;"),
            "content_style={content_style}"
        );
        assert!(
            content_style.contains("--radix-dropdown-menu-content-available-height:"),
            "content_style={content_style}"
        );

        // Group heading automatically linked via aria-labelledby (3B D-6)
        assert_eq!(
            harness.attr("#dm-grp", "aria-labelledby").as_deref(),
            Some("dm-grp-heading")
        );
        assert_eq!(harness.attr("#dm-grp", "role").as_deref(), Some("group"));
        assert_eq!(harness.attr("#dm-sep", "role").as_deref(), Some("separator"));

        // Modal outside click closes menu without clicking outside button (`DM-3`)
        click_and_pump(&mut harness, "#dm-next-focus");
        advance_frame_and_pump(&mut harness, 0.02);
        assert!(harness.query("#dm-content").is_none());
        assert_eq!(
            harness.attr("#dm-probe-state", "data-outside-clicks").as_deref(),
            Some("0")
        );
    }

    #[test]
    fn native_dropdown_menu_keyboard_nav_typeahead_selection_and_escape() {
        let _lock = NATIVE_TEST_LOCK.lock().unwrap_or_else(|e| e.into_inner());
        let mut harness = mount_native_scenario(NativeDropdownFullScenario);

        // Enter on trigger opens and focuses first enabled item (`DM-2`)
        focus_node(&mut harness, "#dm-trig");
        send_key_and_pump(&mut harness, Key::Named(NamedKey::Enter), Code::Enter);
        advance_frame_and_pump(&mut harness, 0.02);
        assert!(harness.query("#dm-content").is_some());
        assert_native_focus_id(&harness, "dm-item-cut");
        assert_eq!(
            harness.attr("#dm-item-cut", "data-highlighted").as_deref(),
            Some("")
        );

        // ArrowDown skips disabled item ("dm-item-disabled") and lands on "dm-item-paste" (`M-6`)
        send_key_and_pump(&mut harness, Key::Named(NamedKey::ArrowDown), Code::ArrowDown);
        assert_native_focus_id(&harness, "dm-item-paste");

        // Enter on item with close_on_select = false fires on_select and keeps menu open (`M-10`)
        send_key_and_pump(&mut harness, Key::Named(NamedKey::Enter), Code::Enter);
        assert!(harness.query("#dm-content").is_some());
        assert_eq!(
            harness.attr("#dm-probe-state", "data-selected").as_deref(),
            Some("paste")
        );

        // ArrowDown to "dm-item-prevent" (Print) and Space activates with prevent_default() -> stays open (`M-10`)
        send_key_and_pump(&mut harness, Key::Named(NamedKey::ArrowDown), Code::ArrowDown);
        assert_native_focus_id(&harness, "dm-item-prevent");
        send_key_and_pump(&mut harness, Key::Character(" ".into()), Code::Space);
        assert!(harness.query("#dm-content").is_some());
        assert_eq!(
            harness.attr("#dm-probe-state", "data-selected").as_deref(),
            Some("paste,prevent")
        );

        // Typeahead 'p' cycles from Print -> Paste (`M-13`)
        send_key_and_pump(&mut harness, Key::Character("p".into()), Code::KeyP);
        assert_native_focus_id(&harness, "dm-item-paste");

        // End highlights last enabled item ("dm-sub-trig"), ArrowDown wraps to first ("dm-item-cut") (`3B D-5`)
        send_key_and_pump(&mut harness, Key::Named(NamedKey::End), Code::End);
        assert_native_focus_id(&harness, "dm-sub-trig");
        send_key_and_pump(&mut harness, Key::Named(NamedKey::ArrowDown), Code::ArrowDown);
        assert_native_focus_id(&harness, "dm-item-cut");

        // Escape closes menu and returns focus to trigger (`3B D-2`)
        send_key_and_pump(&mut harness, Key::Named(NamedKey::Escape), Code::Escape);
        advance_frame_and_pump(&mut harness, 0.02);
        assert!(harness.query("#dm-content").is_none());
        assert_native_focus_id(&harness, "dm-trig");
    }

    #[test]
    fn native_dropdown_menu_checkbox_radio_submenu_and_non_modal_tab_out() {
        let _lock = NATIVE_TEST_LOCK.lock().unwrap_or_else(|e| e.into_inner());
        let mut harness = mount_native_scenario(NativeDropdownFullScenario);

        // Switch to non-modal first, then open via Enter
        click_and_pump(&mut harness, "#dm-toggle-modal");
        focus_node(&mut harness, "#dm-trig");
        send_key_and_pump(&mut harness, Key::Named(NamedKey::ArrowDown), Code::ArrowDown);
        advance_frame_and_pump(&mut harness, 0.02);
        assert!(harness.query("#dm-content").is_some());

        // Indeterminate CheckboxItem transitions Indeterminate -> Checked (`M-8`, `3B D-4`)
        assert_eq!(
            harness.attr("#dm-item-tri", "aria-checked").as_deref(),
            Some("mixed")
        );
        assert_eq!(
            harness.attr("#dm-item-tri", "data-state").as_deref(),
            Some("indeterminate")
        );
        assert!(harness.query("#dm-tri-ind").is_some());

        click_and_pump(&mut harness, "#dm-item-tri");
        assert_eq!(
            harness.attr("#dm-item-tri", "aria-checked").as_deref(),
            Some("true")
        );

        // CheckboxGroup toggles bold/italic (`3B D-4`) and force_mount keeps italic indicator mounted (`M-10`)
        assert!(harness.query("#dm-italic-ind").is_some());
        assert_eq!(
            harness.attr("#dm-italic-ind", "data-state").as_deref(),
            Some("unchecked")
        );
        click_and_pump(&mut harness, "#dm-cb-italic");
        assert_eq!(
            harness.attr("#dm-probe-state", "data-group").as_deref(),
            Some("bold,italic")
        );

        // RadioGroup switches exclusive selection (`M-9`)
        click_and_pump(&mut harness, "#dm-rad-right");
        assert_eq!(
            harness.attr("#dm-probe-state", "data-radio").as_deref(),
            Some("right")
        );
        assert_eq!(
            harness.attr("#dm-rad-right", "aria-checked").as_deref(),
            Some("true")
        );
        assert_eq!(
            harness.attr("#dm-rad-left", "aria-checked").as_deref(),
            Some("false")
        );

        // Hover SubTrigger opens after 100ms delay (`3B D-1`)
        let (stx, sty) = harness.center_of("#dm-sub-trig");
        harness.move_mouse_to(stx, sty);
        std::thread::sleep(std::time::Duration::from_millis(130));
        advance_frame_and_pump(&mut harness, 0.02);
        assert!(harness.query("#dm-sub-content").is_some());

        // Focus sub-item then ArrowLeft closes submenu back to SubTrigger
        focus_node(&mut harness, "#dm-sub-email");
        send_key_and_pump(&mut harness, Key::Named(NamedKey::ArrowLeft), Code::ArrowLeft);
        advance_frame_and_pump(&mut harness, 0.02);
        assert!(harness.query("#dm-sub-content").is_none());
        assert_native_focus_id(&harness, "dm-sub-trig");

        // Non-modal Tab moves focus to next tabbable element after trigger ("dm-next-focus") (`3B D-3`)
        send_key_and_pump(&mut harness, Key::Named(NamedKey::Tab), Code::Tab);
        advance_frame_and_pump(&mut harness, 0.02);
        assert!(harness.query("#dm-content").is_none());
        assert_native_focus_id(&harness, "dm-next-focus");

        // Re-open and Shift+Tab moves focus to previous tabbable element before trigger ("dm-prev-focus")
        focus_node(&mut harness, "#dm-trig");
        send_key_and_pump(&mut harness, Key::Named(NamedKey::Enter), Code::Enter);
        advance_frame_and_pump(&mut harness, 0.02);
        send_key_and_pump_with(
            &mut harness,
            Key::Named(NamedKey::Tab),
            Code::Tab,
            Modifiers::SHIFT,
        );
        advance_frame_and_pump(&mut harness, 0.02);
        assert!(harness.query("#dm-content").is_none());
        assert_native_focus_id(&harness, "dm-prev-focus");
    }

    #[component]
    fn NativeDropdownScrollableScenario() -> Element {
        rsx! {
            div {
                style: "padding: 24px;",
                DropdownMenuRoot {
                    DropdownMenuTrigger {
                        id: "dm-scroll-trig",
                        style: "box-sizing: border-box; width: 120px; height: 32px;",
                        "Scrollable"
                    }
                    DropdownMenuPortal {
                        DropdownMenuContent {
                            id: "dm-scroll-content",
                            style: "box-sizing: border-box; width: 180px; max-height: 100px; overflow-y: auto; padding: 4px; display: flex; flex-direction: column; gap: 2px; background: white;",
                            DropdownMenuItem { id: "dm-scroll-alpha", style: "height: 28px; flex-shrink: 0;", "Alpha" }
                            DropdownMenuItem { id: "dm-scroll-bravo", style: "height: 28px; flex-shrink: 0;", "Bravo" }
                            DropdownMenuItem { id: "dm-scroll-charlie", style: "height: 28px; flex-shrink: 0;", "Charlie" }
                            DropdownMenuItem { id: "dm-scroll-delta", style: "height: 28px; flex-shrink: 0;", "Delta" }
                            DropdownMenuItem { id: "dm-scroll-echo", style: "height: 28px; flex-shrink: 0;", "Echo" }
                            DropdownMenuItem { id: "dm-scroll-foxtrot", style: "height: 28px; flex-shrink: 0;", "Foxtrot" }
                            DropdownMenuItem { id: "dm-scroll-golf", style: "height: 28px; flex-shrink: 0;", "Golf" }
                            DropdownMenuItem { id: "dm-scroll-zulu", style: "height: 28px; flex-shrink: 0;", "Zulu" }
                        }
                    }
                }
            }
        }
    }

    #[test]
    fn native_dropdown_menu_keyboard_and_typeahead_auto_scrolls_scrollable_content() {
        let _lock = NATIVE_TEST_LOCK.lock().unwrap_or_else(|e| e.into_inner());
        let mut harness = mount_native_scenario(NativeDropdownScrollableScenario);

        focus_node(&mut harness, "#dm-scroll-trig");
        send_key_and_pump(&mut harness, Key::Named(NamedKey::Enter), Code::Enter);
        advance_frame_and_pump(&mut harness, 0.02);
        advance_frame_and_pump(&mut harness, 0.02);
        assert!(harness.query("#dm-scroll-content").is_some());
        assert_native_focus_id(&harness, "dm-scroll-alpha");

        let content_node = harness.node("#dm-scroll-content");
        let content_rect = harness.layout_rect("#dm-scroll-content");
        assert!(
            content_rect.height <= 102.0,
            "Scrollable menu content must honor max-height, got {}",
            content_rect.height
        );
        let initial_scroll_y = harness
            .base()
            .get_node(content_node)
            .expect("content node")
            .scroll_offset()
            .y;
        assert!(initial_scroll_y <= 1.0, "initial_scroll_y={initial_scroll_y}");

        // ArrowDown down to offscreen Foxtrot must scroll container
        for expected in [
            "dm-scroll-bravo",
            "dm-scroll-charlie",
            "dm-scroll-delta",
            "dm-scroll-echo",
            "dm-scroll-foxtrot",
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
            "ArrowDown to offscreen item must scroll menu container"
        );
        let foxtrot_rect = harness.layout_rect("#dm-scroll-foxtrot");
        assert!(
            foxtrot_rect.y >= content_rect.y - 1.0
                && foxtrot_rect.y + foxtrot_rect.height <= content_rect.y + content_rect.height + 1.0,
            "Foxtrot {foxtrot_rect:?} must be visible inside {content_rect:?}"
        );

        // Pointer hover over visible sibling (Echo) must NOT scroll (`prevent_scroll = true`)
        let (ex, ey) = harness.center_of("#dm-scroll-echo");
        harness.move_mouse_to(ex, ey);
        harness.pump();
        let after_hover_scroll_y = harness
            .base()
            .get_node(content_node)
            .expect("content node")
            .scroll_offset()
            .y;
        assert!(
            (after_hover_scroll_y - after_arrow_scroll_y).abs() <= 1.0,
            "Pointer hover must not change scroll (before={after_arrow_scroll_y}, after={after_hover_scroll_y})"
        );

        // Typeahead 'z' jumps to Zulu at the bottom and scrolls it into view
        send_key_and_pump(&mut harness, Key::Character("z".into()), Code::KeyZ);
        advance_frame_and_pump(&mut harness, 0.02);
        assert_native_focus_id(&harness, "dm-scroll-zulu");
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
        let zulu_rect = harness.layout_rect("#dm-scroll-zulu");
        assert!(
            zulu_rect.y >= content_rect.y - 1.0
                && zulu_rect.y + zulu_rect.height <= content_rect.y + content_rect.height + 1.0,
            "Zulu {zulu_rect:?} must be visible inside {content_rect:?}"
        );

        // Home returns to Alpha at the top and scrolls back to top
        send_key_and_pump(&mut harness, Key::Named(NamedKey::Home), Code::Home);
        advance_frame_and_pump(&mut harness, 0.02);
        assert_native_focus_id(&harness, "dm-scroll-alpha");
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
        let alpha_rect = harness.layout_rect("#dm-scroll-alpha");
        assert!(
            alpha_rect.y >= content_rect.y - 1.0
                && alpha_rect.y + alpha_rect.height <= content_rect.y + content_rect.height + 1.0,
            "Alpha {alpha_rect:?} must be visible inside {content_rect:?}"
        );
    }

    #[component]
    fn NativeDropdownLoopClampScenario() -> Element {
        rsx! {
            div {
                style: "padding: 24px;",
                DropdownMenuRoot {
                    loop_focus: false,
                    DropdownMenuTrigger {
                        id: "dm-clamp-trig",
                        style: "box-sizing: border-box; width: 120px; height: 32px;",
                        "Clamp"
                    }
                    DropdownMenuPortal {
                        DropdownMenuContent {
                            id: "dm-clamp-content",
                            style: "width: 160px; padding: 4px; background: white;",
                            DropdownMenuItem { id: "dm-clamp-1", "First" }
                            DropdownMenuItem { id: "dm-clamp-2", "Second" }
                            DropdownMenuItem { id: "dm-clamp-3", "Third" }
                        }
                    }
                }
            }
        }
    }

    #[test]
    fn native_dropdown_menu_loop_focus_false_inherited_from_root_clamps_navigation() {
        let _lock = NATIVE_TEST_LOCK.lock().unwrap_or_else(|e| e.into_inner());
        let mut harness = mount_native_scenario(NativeDropdownLoopClampScenario);

        focus_node(&mut harness, "#dm-clamp-trig");
        send_key_and_pump(&mut harness, Key::Named(NamedKey::Enter), Code::Enter);
        advance_frame_and_pump(&mut harness, 0.02);
        assert!(harness.query("#dm-clamp-content").is_some());
        assert_native_focus_id(&harness, "dm-clamp-1");

        // ArrowUp at first item must clamp at first item (no wrap to dm-clamp-3)
        send_key_and_pump(&mut harness, Key::Named(NamedKey::ArrowUp), Code::ArrowUp);
        assert_native_focus_id(&harness, "dm-clamp-1");

        // Jump to last item, ArrowDown at last item must clamp at last item (no wrap to dm-clamp-1)
        send_key_and_pump(&mut harness, Key::Named(NamedKey::End), Code::End);
        assert_native_focus_id(&harness, "dm-clamp-3");
        send_key_and_pump(&mut harness, Key::Named(NamedKey::ArrowDown), Code::ArrowDown);
        assert_native_focus_id(&harness, "dm-clamp-3");
    }

    #[test]
    fn fm_ovl_1_dropdown_menu_ime_process_key_does_not_dismiss() {
        let _lock = NATIVE_TEST_LOCK.lock().unwrap_or_else(|e| e.into_inner());
        let mut harness = mount_native_scenario(NativeDropdownLoopClampScenario);

        focus_node(&mut harness, "#dm-clamp-trig");
        send_key_and_pump(&mut harness, Key::Named(NamedKey::Enter), Code::Enter);
        advance_frame_and_pump(&mut harness, 0.02);
        assert!(harness.query("#dm-clamp-content").is_some());

        // IME Process key must not dismiss open menu (`FM-OVL-1`, `radix#4153`).
        send_key_and_pump(&mut harness, Key::Named(NamedKey::Process), Code::Unidentified);
        advance_frame_and_pump(&mut harness, 0.02);
        assert!(harness.query("#dm-clamp-content").is_some());

        // Normal Escape closes menu and restores trigger focus.
        send_key_and_pump(&mut harness, Key::Named(NamedKey::Escape), Code::Escape);
        advance_frame_and_pump(&mut harness, 0.02);
        assert!(harness.query("#dm-clamp-content").is_none());
        assert_native_focus_id(&harness, "dm-clamp-trig");
    }

    #[component]
    fn NativeDropdownCallbackToggleScenario() -> Element {
        let mut stage = use_signal(|| 0_u8);
        let mut cb1_hits = use_signal(|| 0_u32);
        let mut cb2_hits = use_signal(|| 0_u32);

        let on_esc = match stage() {
            0 => None,
            1 => Some(EventHandler::new(move |_evt: monoxus::menu::MenuPreventableEvent| {
                cb1_hits.set(cb1_hits() + 1);
            })),
            _ => Some(EventHandler::new(move |evt: monoxus::menu::MenuPreventableEvent| {
                cb2_hits.set(cb2_hits() + 1);
                evt.prevent_default();
            })),
        };

        rsx! {
            div { style: "padding: 24px;",
                button {
                    id: "dm-cb-next-stage",
                    r#type: "button",
                    style: "width: 80px; height: 24px;",
                    onclick: move |_| stage.set(stage() + 1),
                    "Next Stage"
                }
                span { id: "dm-cb-hits", "{cb1_hits()}:{cb2_hits()}" }
                DropdownMenuRoot {
                    DropdownMenuTrigger {
                        id: "dm-cb-trig",
                        style: "width: 100px; height: 28px;",
                        "Toggle Menu"
                    }
                    DropdownMenuPortal {
                        DropdownMenuContent {
                            id: "dm-cb-content",
                            style: "width: 140px; padding: 4px; background: white;",
                            on_escape_key_down: on_esc,
                            DropdownMenuItem { id: "dm-cb-item-1", "Alpha" }
                        }
                    }
                }
            }
        }
    }

    #[test]
    fn fm_api_4_dropdown_menu_content_callback_toggle_none_some_some() {
        let _lock = NATIVE_TEST_LOCK.lock().unwrap_or_else(|e| e.into_inner());
        let mut harness = mount_native_scenario(NativeDropdownCallbackToggleScenario);

        // Toggle `on_escape_key_down` across `None -> Some(cb1) -> Some(cb2)` without panic (`Dioxus #5894`).
        click_and_pump(&mut harness, "#dm-cb-next-stage"); // None -> Some(cb1)
        click_and_pump(&mut harness, "#dm-cb-next-stage"); // Some(cb1) -> Some(cb2)

        focus_node(&mut harness, "#dm-cb-trig");
        send_key_and_pump(&mut harness, Key::Named(NamedKey::Enter), Code::Enter);
        advance_frame_and_pump(&mut harness, 0.02);
        assert!(harness.query("#dm-cb-content").is_some());

        // Escape must invoke `cb2` (which calls `prevent_default()` -> menu stays open).
        send_key_and_pump(&mut harness, Key::Named(NamedKey::Escape), Code::Escape);
        advance_frame_and_pump(&mut harness, 0.02);
        assert!(harness.query("#dm-cb-content").is_some());
        assert_eq!(harness.text_content("#dm-cb-hits"), "0:1");
    }

    #[component]
    fn NativeMenuPointerEventsScenario() -> Element {
        use monoxus::dialog::{DialogClose, DialogContent, DialogPortal, DialogRoot};

        let float_open = use_signal(|| false);
        let dlg_open = use_signal(|| true);

        rsx! {
            div { style: "padding: 24px;",
                // Part 1: Floating MenuContent force_mount closed -> open pointer-events gating
                DropdownMenuRoot {
                    open: Some(float_open()),
                    on_open_change: move |next| {
                        let mut sig = float_open;
                        sig.set(next);
                    },
                    DropdownMenuTrigger {
                        id: "w1-pe-float-trig",
                        style: "width: 100px; height: 28px;",
                        "Float Trigger"
                    }
                    DropdownMenuContent {
                        id: "w1-pe-float-content",
                        force_mount: true,
                        style: "width: 140px; background: white;",
                        DropdownMenuItem { id: "w1-pe-float-item", "Float Item" }
                    }
                }

                // Part 2: Non-modal Static MenuContentStatic inside modal Dialog (modal pointer lock from Dialog)
                DialogRoot {
                    id: "w1-pe-modal-dlg",
                    open: Some(dlg_open),
                    DialogPortal {
                        DialogContent {
                            id: "w1-pe-modal-dlg-content",
                            style: "width: 320px; height: 200px; padding: 12px; background: white;",
                            DialogClose {
                                id: "w1-pe-close-dlg",
                                style: "width: 100px; height: 28px;",
                                "Close Dialog"
                            }
                            DropdownMenuRoot {
                                modal: false,
                                default_open: true,
                                DropdownMenuTrigger {
                                    id: "w1-pe-static-trig",
                                    style: "width: 100px; height: 28px;",
                                    "Static Trigger"
                                }
                                DropdownMenuContentStatic {
                                    id: "w1-pe-static-content",
                                    DropdownMenuItem { id: "w1-pe-static-item", "Static Item" }
                                }
                            }
                        }
                    }
                }
            }
        }
    }

    #[test]
    fn fm_w1_pe_menu_floating_closed_pointer_events_none_and_static_modal_auto() {
        let _lock = NATIVE_TEST_LOCK.lock().unwrap_or_else(|e| e.into_inner());
        let mut harness = mount_native_scenario(NativeMenuPointerEventsScenario);
        advance_frame_and_pump(&mut harness, 0.02);

        // H-5 (floating closed force_mount): both wrapper & content must emit pointer-events: none; (never auto)
        let closed_wrapper_style = harness
            .attr("[data-monoxus-floating-content-wrapper]", "style")
            .unwrap_or_default();
        assert!(
            closed_wrapper_style.contains("pointer-events: none;")
                && !closed_wrapper_style.contains("pointer-events: auto;"),
            "H-5: closed floating wrapper must emit pointer-events: none; (not auto), got: {closed_wrapper_style}"
        );
        let closed_float_style = harness.attr("#w1-pe-float-content", "style").unwrap_or_default();
        assert_eq!(
            harness.attr("#w1-pe-float-content", "data-state").as_deref(),
            Some("closed")
        );
        assert!(
            closed_float_style.contains("pointer-events: none;"),
            "H-5: closed floating MenuContent must emit pointer-events: none;, got: {closed_float_style}"
        );
        assert!(
            !closed_float_style.contains("pointer-events: auto;"),
            "H-5: closed floating MenuContent must NOT emit pointer-events: auto;, got: {closed_float_style}"
        );

        // H-5 (static under modal pointer lock): must emit pointer-events: auto;
        let static_modal_style = harness.attr("#w1-pe-static-content", "style").unwrap_or_default();
        assert_eq!(
            harness.attr("#w1-pe-static-content", "data-state").as_deref(),
            Some("open")
        );
        assert!(
            static_modal_style.contains("pointer-events: auto;"),
            "H-5: open MenuContentStatic under modal pointer lock must emit pointer-events: auto;, got: {static_modal_style}"
        );

        // Close modal dialog cleanly via DialogClose so floating trigger is unshielded
        click_and_pump(&mut harness, "#w1-pe-close-dlg");
        advance_frame_and_pump(&mut harness, 0.02);
        assert!(
            harness.query("#w1-pe-modal-dlg-content").is_none(),
            "Modal dialog must close cleanly before opening floating dropdown"
        );

        click_and_pump(&mut harness, "#w1-pe-float-trig");
        advance_frame_and_pump(&mut harness, 0.02);
        let open_float_style = harness.attr("#w1-pe-float-content", "style").unwrap_or_default();
        assert_eq!(
            harness.attr("#w1-pe-float-content", "data-state").as_deref(),
            Some("open")
        );
        assert!(
            open_float_style.contains("pointer-events: auto;"),
            "H-5: open floating MenuContent must emit pointer-events: auto;, got: {open_float_style}"
        );
    }

    #[test]
    fn fm_w1_guard_dropdown_trigger_ime_composing_blocked() {
        use blitz_traits::events::{BlitzKeyEvent, KeyState, UiEvent};
        use keyboard_types::{Location, Modifiers};

        let _lock = NATIVE_TEST_LOCK.lock().unwrap_or_else(|e| e.into_inner());
        let mut harness = mount_native_scenario(NativeDropdownCallbackToggleScenario);
        advance_frame_and_pump(&mut harness, 0.02);

        focus_node(&mut harness, "#dm-cb-trig");
        for (key, code) in [
            (Key::Named(NamedKey::Enter), Code::Enter),
            (Key::Character(" ".to_string()), Code::Space),
            (Key::Named(NamedKey::ArrowDown), Code::ArrowDown),
        ] {
            let down = BlitzKeyEvent {
                key: key.clone(),
                code,
                modifiers: Modifiers::default(),
                location: Location::Standard,
                is_auto_repeating: false,
                is_composing: true,
                state: KeyState::Pressed,
                text: None,
            };
            let up = BlitzKeyEvent {
                key,
                code,
                modifiers: Modifiers::default(),
                location: Location::Standard,
                is_auto_repeating: false,
                is_composing: true,
                state: KeyState::Released,
                text: None,
            };
            harness.dispatch(UiEvent::KeyDown(down));
            harness.dispatch(UiEvent::KeyUp(up));
            drain_polls_and_resolve(&mut harness);
            advance_frame_and_pump(&mut harness, 0.02);
            assert!(
                harness.query("#dm-cb-content").is_none(),
                "M-1: IME composing key on DropdownMenuTrigger must NOT open menu"
            );
        }

        // Normal Enter opens menu cleanly
        send_key_and_pump(&mut harness, Key::Named(NamedKey::Enter), Code::Enter);
        advance_frame_and_pump(&mut harness, 0.02);
        assert!(harness.query("#dm-cb-content").is_some());
    }
}

#[test]
fn fm_ovl_5_submenu_corridor_progress_and_coalesced_leave_direction() {
    use monoxus::foundation::overlay::{
        GraceTransit, PlacementSide, Point2D, Rect2D, SUBMENU_INTENT_MIN_PROGRESS_PX,
        point_to_rect_distance,
    };
    use monoxus::menu::MenuSurfaceRuntime;

    // Coalesced pointerleave horizontal direction recomputation (`radix menu.tsx:488`).
    assert!(MenuSurfaceRuntime::is_pointer_leaving_away_from_submenu(
        PlacementSide::Right,
        -(SUBMENU_INTENT_MIN_PROGRESS_PX + 1.0),
        false,
    ));
    assert!(!MenuSurfaceRuntime::is_pointer_leaving_away_from_submenu(
        PlacementSide::Right,
        SUBMENU_INTENT_MIN_PROGRESS_PX + 1.0,
        false,
    ));
    assert!(MenuSurfaceRuntime::is_pointer_leaving_away_from_submenu(
        PlacementSide::Left,
        SUBMENU_INTENT_MIN_PROGRESS_PX + 1.0,
        false,
    ));
    assert!(MenuSurfaceRuntime::is_pointer_leaving_away_from_submenu(
        PlacementSide::Right,
        SUBMENU_INTENT_MIN_PROGRESS_PX + 1.0,
        true,
    ));

    // Monotonic progress threshold (`bits#2192`): >= 1.0px progress vs < 1.0px micro-jitter.
    let from = Rect2D::new(0.0, 0.0, 120.0, 32.0);
    let to = Rect2D::new(140.0, 0.0, 140.0, 160.0);
    let exit = Point2D::new(120.0, 16.0);
    let transit = GraceTransit::new(exit, from, to, Some(5.0), false);
    let d0 = point_to_rect_distance(exit, &transit.to_rect);
    let d_jitter = point_to_rect_distance(Point2D::new(120.5, 16.0), &transit.to_rect);
    let d_progress = point_to_rect_distance(Point2D::new(125.0, 20.0), &transit.to_rect);
    assert!(d0 - d_jitter < SUBMENU_INTENT_MIN_PROGRESS_PX);
    assert!(d0 - d_progress >= SUBMENU_INTENT_MIN_PROGRESS_PX);
}

#[test]
fn fm_ovl_7_dropdown_menu_defaults_to_bottom_and_4px_side_offset() {
    use monoxus::foundation::overlay::PlacementSide;
    use monoxus::menu::MENU_DEFAULT_SIDE_OFFSET;

    #[derive(Clone, Default, PartialEq)]
    struct DefaultsSnapshot {
        side: Option<PlacementSide>,
        side_offset: f32,
    }

    #[component]
    fn DefaultsProbe(out: Rc<RefCell<DefaultsSnapshot>>) -> Element {
        let root = use_context::<MenuRootContext>();
        *out.borrow_mut() = DefaultsSnapshot {
            side: Some(*root.root_surface.side.read()),
            side_offset: *root.root_surface.side_offset.read(),
        };
        rsx! {}
    }

    #[component]
    fn DropdownDefaultsSample(out: Rc<RefCell<DefaultsSnapshot>>) -> Element {
        rsx! {
            DropdownMenuRoot {
                default_open: true,
                DropdownMenuTrigger { "Open" }
                DropdownMenuPortal {
                    DropdownMenuContent {
                        DropdownMenuItem { "One" }
                    }
                }
                DefaultsProbe { out }
            }
        }
    }

    let snap = Rc::new(RefCell::new(DefaultsSnapshot::default()));
    let mut dom = VirtualDom::new_with_props(
        DropdownDefaultsSample,
        DropdownDefaultsSampleProps { out: snap.clone() },
    );
    dom.rebuild_in_place();

    let res = snap.borrow();
    assert_eq!(res.side, Some(PlacementSide::Bottom));
    assert_eq!(res.side_offset, MENU_DEFAULT_SIDE_OFFSET);
}

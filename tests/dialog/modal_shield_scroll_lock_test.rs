#[cfg(feature = "native")]
use dioxus::prelude::*;
#[cfg(feature = "native")]
use monoxus::dialog::{
    DialogClose, DialogContent, DialogOverlay, DialogPortal, DialogRoot, DialogTitle,
    DialogTrigger,
};

#[cfg(feature = "native")]
use crate::common;

#[cfg(feature = "native")]
#[component]
fn ControlledLockDialog(open: Signal<bool>, prevent_body_pointer_events: bool) -> Element {
    rsx! {
        DialogRoot {
            id: "lock-test-dialog",
            open: open,
            prevent_body_pointer_events: prevent_body_pointer_events,
            DialogTrigger { id: "lock-test-trigger", "Open" }
            DialogPortal {
                DialogOverlay { id: "lock-test-overlay" }
                DialogContent {
                    id: "lock-test-content",
                    DialogTitle { "Title" }
                    DialogClose { id: "lock-test-close", "Close" }
                }
            }
        }
    }
}

#[cfg(feature = "native")]
#[test]
fn test_dialog_scroll_lock_prevent_body_pointer_events_and_synchronous_unlock() {
    let _guard = common::native_fixture::NATIVE_TEST_LOCK
        .lock()
        .unwrap_or_else(|e| e.into_inner());

    // 1. Default prevent_body_pointer_events = true
    #[component]
    fn HostBlocking() -> Element {
        let open = use_signal(|| true);
        rsx! {
            ControlledLockDialog {
                open: open,
                prevent_body_pointer_events: true,
            }
        }
    }

    {
        let mut dom = VirtualDom::new(HostBlocking);
        dom.rebuild_in_place();
        dom.render_immediate(&mut dioxus_core::NoOpMutations);
    }

    // After drop, lock is released cleanly
    assert!(!oxidase::dom::is_body_pointer_events_locked());

    // 2. prevent_body_pointer_events = false leaves pointer-events unlocked while scroll is locked
    oxidase::dom::acquire_scroll_lock_with_options(
        "manual-non-blocking",
        oxidase::dom::ScrollLockOptions {
            prevent_body_pointer_events: false,
        },
    );
    assert!(oxidase::dom::is_scroll_locked());
    assert!(!oxidase::dom::is_body_pointer_events_locked());
    oxidase::dom::release_scroll_lock("manual-non-blocking", None);
    assert!(!oxidase::dom::is_scroll_locked());
}

#[cfg(feature = "native")]
mod native_tests {
    use super::*;
    use common::native_fixture::*;
    use monoxus::foundation::shared::ScopeHandle;
    use monoxus::popover::{
        PopoverClose, PopoverContent, PopoverPortal, PopoverRoot, PopoverTrigger,
    };
    use monoxus::select::{
        Select, SelectContent, SelectItem, SelectRoot, SelectTrigger, SelectValue, SelectViewport,
        use_select_runtime,
    };

    #[component]
    fn ModalShieldScenario() -> Element {
        let open = use_signal(|| false);
        rsx! {
            div {
                id: "shield-scenario-wrapper",
                style: "display: contents;",
                div {
                    id: "external-sibling",
                    button { id: "external-btn", "Background Action" }
                }
                div {
                    id: "pre-existing-hidden",
                    "aria-hidden": "true",
                    "Decorative icon"
                }
                div {
                    id: "live-toast-region",
                    "aria-live": "polite",
                    "Live Toast Notification"
                }
                DialogRoot {
                    id: "shield-dialog",
                    open: open,
                    DialogTrigger { id: "shield-trigger", "Open Modal" }
                    DialogPortal {
                        DialogOverlay { id: "shield-overlay" }
                        DialogContent {
                            id: "shield-content",
                            DialogTitle { "Modal Title" }
                            DialogClose { id: "shield-close", "Close" }
                        }
                    }
                }
            }
        }
    }

    #[test]
    fn test_native_dialog_modal_shield_aria_live_exemption_and_uncontrolled_restore() {
        let _lock = NATIVE_TEST_LOCK.lock().unwrap_or_else(|e| e.into_inner());
        let mut harness = mount_native_scenario(ModalShieldScenario);

        let ext_id = harness.node("#external-sibling");
        let pre_id = harness.node("#pre-existing-hidden");
        let live_id = harness.node("#live-toast-region");

        {
            let base = harness.base();
            assert_eq!(base.get_node(ext_id).unwrap().attr("aria-hidden".into()), None);
            assert_eq!(base.get_node(ext_id).unwrap().attr("inert".into()), None);
            assert_eq!(
                base.get_node(pre_id).unwrap().attr("aria-hidden".into()),
                Some("true")
            );
            assert_eq!(base.get_node(live_id).unwrap().attr("aria-hidden".into()), None);
        }

        // Open modal Dialog
        click_and_pump(&mut harness, "#shield-trigger");
        advance_frame_and_pump(&mut harness, 0.02);

        {
            let content_node_id = harness.node("#shield-content");
            let overlay_node_id = harness.node("#shield-overlay");
            let base = harness.base();

            // External sibling is shielded with both aria-hidden="true" and inert=""
            let ext_node = base.get_node(ext_id).unwrap();
            assert_eq!(ext_node.attr("aria-hidden".into()), Some("true"));
            assert!(ext_node.attr("inert".into()).is_some());

            // [aria-live] region is exempt from shielding
            let live_node = base.get_node(live_id).unwrap();
            assert_eq!(live_node.attr("aria-hidden".into()), None);
            assert_eq!(live_node.attr("inert".into()), None);

            // Modal content & overlay carry first-frame pointer-events: auto and are not inert
            let content_node = base.get_node(content_node_id).unwrap();
            assert_eq!(content_node.attr("inert".into()), None);
            let content_style = content_node.attr("style".into()).unwrap_or_default();
            assert!(
                content_style.contains("pointer-events: auto"),
                "Expected pointer-events: auto on open DialogContent, got: {content_style}"
            );

            let overlay_node = base.get_node(overlay_node_id).unwrap();
            assert_eq!(overlay_node.attr("inert".into()), None);
            let overlay_style = overlay_node.attr("style".into()).unwrap_or_default();
            assert!(
                overlay_style.contains("pointer-events: auto"),
                "Expected pointer-events: auto on open DialogOverlay, got: {overlay_style}"
            );
        }

        // Close modal Dialog
        click_and_pump(&mut harness, "#shield-close");
        advance_frame_and_pump(&mut harness, 0.25);

        {
            let base = harness.base();
            let ext_node = base.get_node(ext_id).unwrap();
            assert_eq!(ext_node.attr("aria-hidden".into()), None);
            assert_eq!(ext_node.attr("inert".into()), None);

            // Pre-existing consumer aria-hidden="true" is preserved intact
            let pre_node = base.get_node(pre_id).unwrap();
            assert_eq!(pre_node.attr("aria-hidden".into()), Some("true"));
        }
    }

    #[test]
    fn test_native_body_cssom_scroll_lock_non_destructive_restore() {
        let _lock = NATIVE_TEST_LOCK.lock().unwrap_or_else(|e| e.into_inner());
        let mut harness = mount_native_scenario(ModalShieldScenario);

        // Pre-populate <body> inline style with priority declarations
        let doc = oxidase::dom::Document::from_base(harness.doc.inner.clone());
        oxidase::dom::Document::set_current(Some(doc));
        let body_id = harness.node("body");
        {
            let mut base = harness.base_mut();
            let mut mutator = base.mutate();
            mutator.set_attribute(
                body_id,
                blitz_dom::qual_name!("style"),
                "overflow-y: scroll; margin-right: 12px !important;",
            );
        }
        harness.pump();

        // Open modal Dialog -> body acquires scroll lock + pointer-events: none
        click_and_pump(&mut harness, "#shield-trigger");
        advance_frame_and_pump(&mut harness, 0.02);

        assert!(oxidase::dom::is_scroll_locked());
        assert!(oxidase::dom::is_body_pointer_events_locked());
        {
            let base = harness.base();
            let body_style = base
                .get_node(body_id)
                .unwrap()
                .attr("style".into())
                .unwrap_or_default();
            assert!(
                body_style.contains("pointer-events: none"),
                "Expected pointer-events: none on locked body, got: {body_style}"
            );
            assert!(
                body_style.contains("overflow: hidden"),
                "Expected overflow: hidden on locked body, got: {body_style}"
            );
        }

        // Click close -> pointer-events restores synchronously before 200ms restore_delay!
        harness.click("#shield-close");
        harness.pump();
        assert!(
            !oxidase::dom::is_body_pointer_events_locked(),
            "pointer-events lock must release synchronously before restore_delay"
        );
        {
            let base = harness.base();
            let body_style = base
                .get_node(body_id)
                .unwrap()
                .attr("style".into())
                .unwrap_or_default();
            assert!(
                !body_style.contains("pointer-events: none"),
                "pointer-events: none must be removed immediately on close, got: {body_style}"
            );
        }

        // Advance past 200ms restore_delay -> pre-existing CSSOM properties & priority restored
        advance_frame_and_pump(&mut harness, 0.3);
        {
            let base = harness.base();
            let body_style = base
                .get_node(body_id)
                .unwrap()
                .attr("style".into())
                .unwrap_or_default();
            assert!(
                body_style.contains("overflow-y: scroll"),
                "Expected overflow-y: scroll restored, got: {body_style}"
            );
            assert!(
                body_style.contains("margin-right: 12px !important"),
                "Expected margin-right: 12px !important restored, got: {body_style}"
            );
            assert!(!body_style.contains("overflow: hidden"));
        }
    }

    #[component]
    fn ModalPopoverAndScrollLockedSelectScenario() -> Element {
        let select_open = use_signal(|| false);
        let scope = ScopeHandle::root("shield-test").child("select");
        let select_def = Select::new(scope)
            .with_trigger_id("shield-test-select-trigger")
            .with_content_id("shield-test-select-content")
            .with_open(select_open());
        let select_rt = use_select_runtime(
            select_def,
            None::<fn(Option<String>)>,
            Some(move |next: bool| {
                let mut sig = select_open;
                sig.set(next);
            }),
        );

        rsx! {
            div {
                style: "display: contents;",
                div { id: "popover-external-sibling", "Outside sibling" }
                div { id: "popover-live-region", "aria-live": "assertive", "Alert region" }
                PopoverRoot {
                    id: "modal-popover",
                    modal: true,
                    PopoverTrigger { id: "modal-popover-trigger", "Open Modal Popover" }
                    PopoverPortal {
                        PopoverContent {
                            id: "modal-popover-content",
                            PopoverClose { id: "modal-popover-close", "Close" }
                        }
                    }
                }
                SelectRoot {
                    runtime: select_rt,
                    SelectTrigger {
                        SelectValue { placeholder: "Pick...".to_string() }
                    }
                    SelectContent {
                        prevent_scroll: true,
                        SelectViewport {
                            SelectItem { value: "a".to_string(), text: "Alpha".to_string(), "Alpha" }
                        }
                    }
                }
            }
        }
    }

    #[test]
    fn test_native_modal_popover_shield_and_select_prevent_scroll_pointer_events() {
        let _lock = NATIVE_TEST_LOCK.lock().unwrap_or_else(|e| e.into_inner());
        let mut harness = mount_native_scenario(ModalPopoverAndScrollLockedSelectScenario);

        let ext_id = harness.node("#popover-external-sibling");
        let live_id = harness.node("#popover-live-region");

        click_and_pump(&mut harness, "#modal-popover-trigger");
        advance_frame_and_pump(&mut harness, 0.02);

        {
            let pop_cid = harness.node("#modal-popover-content");
            let base = harness.base();
            assert_eq!(
                base.get_node(ext_id).unwrap().attr("aria-hidden".into()),
                Some("true")
            );
            assert!(base.get_node(ext_id).unwrap().attr("inert".into()).is_some());
            assert_eq!(base.get_node(live_id).unwrap().attr("aria-hidden".into()), None);

            let pop_style = base
                .get_node(pop_cid)
                .unwrap()
                .attr("style".into())
                .unwrap_or_default();
            assert!(
                pop_style.contains("pointer-events: auto"),
                "Expected pointer-events: auto on modal PopoverContent, got: {pop_style}"
            );
        }

        click_and_pump(&mut harness, "#modal-popover-close");
        advance_frame_and_pump(&mut harness, 0.35);

        {
            let base = harness.base();
            assert_eq!(base.get_node(ext_id).unwrap().attr("aria-hidden".into()), None);
            assert_eq!(base.get_node(ext_id).unwrap().attr("inert".into()), None);
        }

        // Open Select with prevent_scroll: true -> SelectContent gets pointer-events: auto
        click_and_pump(&mut harness, "#shield-test-select-trigger");
        advance_frame_and_pump(&mut harness, 0.02);

        {
            let sel_cid = harness.node("#shield-test-select-content");
            let base = harness.base();
            let sel_style = base
                .get_node(sel_cid)
                .unwrap()
                .attr("style".into())
                .unwrap_or_default();
            assert!(
                sel_style.contains("pointer-events: auto"),
                "Expected pointer-events: auto on prevent_scroll SelectContent, got: {sel_style}"
            );
        }
    }

    fn dispatch_ime_key_and_pump(
        harness: &mut blitz_test_harness::Harness<dioxus_native_dom::DioxusDocument>,
        key: keyboard_types::Key,
        code: keyboard_types::Code,
        is_composing: bool,
    ) {
        use blitz_traits::events::{BlitzKeyEvent, KeyState, UiEvent};
        use keyboard_types::{Location, Modifiers};

        let down = BlitzKeyEvent {
            key: key.clone(),
            code,
            modifiers: Modifiers::default(),
            location: Location::Standard,
            is_auto_repeating: false,
            is_composing,
            state: KeyState::Pressed,
            text: None,
        };
        let up = BlitzKeyEvent {
            key,
            code,
            modifiers: Modifiers::default(),
            location: Location::Standard,
            is_auto_repeating: false,
            is_composing,
            state: KeyState::Released,
            text: None,
        };
        harness.dispatch(UiEvent::KeyDown(down));
        harness.dispatch(UiEvent::KeyUp(up));
        drain_polls_and_resolve(harness);
        oxidase::frame::tick(std::time::Duration::from_millis(16));
        drain_polls_and_resolve(harness);
        harness.tick(0.01);
    }

    #[component]
    fn NestedDialogPopoverLifoScenario() -> Element {
        let dialog_open = use_signal(|| false);
        let popover_open = use_signal(|| false);
        let mut parent_tick = use_signal(|| 0_u32);
        let tick_val = parent_tick();

        rsx! {
            div {
                id: "lifo-host",
                "data-tick": "{tick_val}",
                button {
                    id: "bump-parent-tick",
                    onclick: move |_| parent_tick += 1,
                    "Bump {tick_val}"
                }
                DialogRoot {
                    id: "lifo-dialog",
                    open: dialog_open,
                    DialogTrigger { id: "lifo-dialog-trigger", "Open Dialog" }
                    DialogPortal {
                        DialogOverlay { id: "lifo-dialog-overlay" }
                        DialogContent {
                            id: "lifo-dialog-content",
                            class: Some(format!("tick-{tick_val}")),
                            DialogTitle { "Nested LIFO Dialog" }
                            button {
                                id: "bump-inside-dialog",
                                onclick: move |_| parent_tick += 1,
                                "Re-render Parent ({tick_val})"
                            }
                            PopoverRoot {
                                id: "lifo-popover",
                                open: popover_open,
                                PopoverTrigger { id: "lifo-popover-trigger", "Open Popover" }
                                PopoverPortal {
                                    PopoverContent {
                                        id: "lifo-popover-content",
                                        button {
                                            id: "lifo-popover-btn",
                                            onclick: move |_| parent_tick += 1,
                                            "Popover Action ({tick_val})"
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
    fn fm_ovl_1_and_fm_ovl_2_dialog_popover_ime_escape_lifo_stability_and_frame0_pointer_events() {
        use keyboard_types::{Code, Key, NamedKey};

        let _lock = NATIVE_TEST_LOCK.lock().unwrap_or_else(|e| e.into_inner());
        let mut harness = mount_native_scenario(NestedDialogPopoverLifoScenario);

        // 1. Open Dialog -> frame-0 render immediately carries pointer-events: auto (`radix#4105`)
        click_and_pump(&mut harness, "#lifo-dialog-trigger");
        {
            let content_style = harness
                .base()
                .get_node(harness.node("#lifo-dialog-content"))
                .unwrap()
                .attr("style".into())
                .unwrap_or_default()
                .to_string();
            let overlay_style = harness
                .base()
                .get_node(harness.node("#lifo-dialog-overlay"))
                .unwrap()
                .attr("style".into())
                .unwrap_or_default()
                .to_string();
            assert!(
                content_style.contains("pointer-events: auto"),
                "DialogContent must emit pointer-events: auto on frame 0, got: {content_style}"
            );
            assert!(
                overlay_style.contains("pointer-events: auto"),
                "DialogOverlay must emit pointer-events: auto on frame 0, got: {overlay_style}"
            );
        }

        // 2. Open nested Popover inside Dialog
        click_and_pump(&mut harness, "#lifo-popover-trigger");
        advance_frame_and_pump(&mut harness, 0.02);
        assert!(harness.query("#lifo-popover-content").is_some());

        // 3. Re-render parent Dialog with non-open prop change (triggered from inside Popover so
        //    outside-pointer dismiss is not fired) -> must NOT reorder LIFO stack
        click_and_pump(&mut harness, "#lifo-popover-btn");
        advance_frame_and_pump(&mut harness, 0.02);
        assert!(harness.query("#lifo-dialog-content").is_some());
        assert!(harness.query("#lifo-popover-content").is_some());

        // 4. FM-OVL-1: IME Escape (is_composing=true) & Process key must NOT dismiss Popover or Dialog
        dispatch_ime_key_and_pump(
            &mut harness,
            Key::Named(NamedKey::Escape),
            Code::Escape,
            true,
        );
        dispatch_ime_key_and_pump(
            &mut harness,
            Key::Named(NamedKey::Process),
            Code::Escape,
            false,
        );
        assert!(
            harness.query("#lifo-popover-content").is_some(),
            "IME Escape/Process must not dismiss nested Popover"
        );
        assert!(
            harness.query("#lifo-dialog-content").is_some(),
            "IME Escape/Process must not dismiss parent Dialog"
        );

        // 5. Normal Escape closes ONLY inner Popover; parent Dialog stays open
        send_key_and_pump(&mut harness, Key::Named(NamedKey::Escape), Code::Escape);
        advance_frame_and_pump(&mut harness, 0.02);
        assert!(
            harness.query("#lifo-popover-content").is_none(),
            "First Escape must close nested Popover"
        );
        assert!(
            harness.query("#lifo-dialog-content").is_some(),
            "Parent Dialog must remain open after nested Popover closes"
        );

        // 6. IME Escape on Dialog itself still must not dismiss
        dispatch_ime_key_and_pump(
            &mut harness,
            Key::Named(NamedKey::Escape),
            Code::Escape,
            true,
        );
        assert!(
            harness.query("#lifo-dialog-content").is_some(),
            "IME Escape must not dismiss Dialog"
        );

        // 7. Second normal Escape closes parent Dialog
        send_key_and_pump(&mut harness, Key::Named(NamedKey::Escape), Code::Escape);
        advance_frame_and_pump(&mut harness, 0.05);
        assert!(
            harness.query("#lifo-dialog-content").is_none(),
            "Second Escape must close parent Dialog"
        );
    }
}

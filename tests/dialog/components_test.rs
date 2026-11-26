use dioxus::prelude::*;
use monoxus::alert_dialog::{AlertDialogAction, AlertDialogCancel, AlertDialogRoot};
use monoxus::dialog::{
    DialogClose, DialogContent, DialogDescription, DialogOverlay, DialogPortal, DialogRoot,
    DialogTitle, DialogTrigger,
};

#[component]
fn DialogCompoundSample() -> Element {
    rsx! {
        DialogRoot {
            default_open: true,
            DialogTrigger {
                "Open Dialog"
            }
            DialogPortal {
                DialogOverlay {}
                DialogContent {
                    DialogTitle { "Sample Title" }
                    DialogDescription { "Sample Description" }
                    DialogClose { "Close" }
                }
            }
        }
    }
}

#[component]
fn AlertDialogCompoundSample() -> Element {
    rsx! {
        AlertDialogRoot {
            default_open: true,
            DialogTrigger {
                "Open Alert"
            }
            DialogPortal {
                DialogOverlay {}
                DialogContent {
                    DialogTitle { "Confirm Action" }
                    DialogDescription { "Are you sure?" }
                    AlertDialogCancel { "Cancel" }
                    AlertDialogAction { "Confirm" }
                }
            }
        }
    }
}

#[test]
fn test_dialog_compound_components_render() {
    let mut dom = VirtualDom::new(DialogCompoundSample);
    dom.rebuild_in_place();
}

#[test]
fn test_alert_dialog_compound_components_render() {
    let mut dom = VirtualDom::new(AlertDialogCompoundSample);
    dom.rebuild_in_place();
}

#[component]
fn DeclarativeDialogClosedSample() -> Element {
    let open = use_signal(|| false);
    rsx! {
        DialogRoot {
            open: open,
            DialogTrigger {
                id: "test-trigger",
                "Trigger"
            }
            DialogPortal {
                DialogOverlay {
                    id: "test-overlay"
                }
                DialogContent {
                    id: "test-content",
                    DialogTitle { "Title" }
                    DialogClose {
                        id: "test-close",
                        "Close"
                    }
                }
            }
        }
    }
}

#[component]
fn DeclarativeDialogOpenSample() -> Element {
    let open = use_signal(|| true);
    rsx! {
        DialogRoot {
            open: open,
            DialogTrigger {
                id: "test-trigger",
                "Trigger"
            }
            DialogPortal {
                DialogOverlay {
                    id: "test-overlay"
                }
                DialogContent {
                    id: "test-content",
                    DialogTitle { "Title" }
                    DialogClose {
                        id: "test-close",
                        "Close"
                    }
                }
            }
        }
    }
}

#[test]
fn test_declarative_dialog_render_toggle() {
    let mut dom = VirtualDom::new(DeclarativeDialogClosedSample);
    dom.rebuild_in_place();

    let mut dom_open = VirtualDom::new(DeclarativeDialogOpenSample);
    dom_open.rebuild_in_place();
}

#[component]
fn DialogTitleDescriptionOmissionSample(
    title_registered: std::rc::Rc<std::cell::RefCell<Option<String>>>,
    desc_registered: std::rc::Rc<std::cell::RefCell<Option<String>>>,
) -> Element {
    let title_c = title_registered.clone();
    let desc_c = desc_registered.clone();
    rsx! {
        DialogRoot {
            default_open: true,
            DialogTrigger { "Open" }
            DialogPortal {
                DialogContent {
                    DialogTitleDescriptionProbe {
                        title_target: title_c,
                        desc_target: desc_c,
                    }
                }
            }
        }
    }
}

#[component]
fn DialogTitleDescriptionProbe(
    title_target: std::rc::Rc<std::cell::RefCell<Option<String>>>,
    desc_target: std::rc::Rc<std::cell::RefCell<Option<String>>>,
) -> Element {
    let ctx = use_context::<monoxus::dialog::DialogContext>();
    *title_target.borrow_mut() = ctx.title_id.read().clone();
    *desc_target.borrow_mut() = ctx.description_id.read().clone();
    rsx! {}
}

#[test]
fn test_dialog_title_description_omission_leaves_context_ids_none() {
    let title_res = std::rc::Rc::new(std::cell::RefCell::new(Some("initial".to_string())));
    let desc_res = std::rc::Rc::new(std::cell::RefCell::new(Some("initial".to_string())));

    let mut dom = VirtualDom::new_with_props(
        DialogTitleDescriptionOmissionSample,
        DialogTitleDescriptionOmissionSampleProps {
            title_registered: title_res.clone(),
            desc_registered: desc_res.clone(),
        },
    );
    dom.rebuild_in_place();

    assert_eq!(
        *title_res.borrow(),
        None,
        "title_id must be None when DialogTitle is omitted"
    );
    assert_eq!(
        *desc_res.borrow(),
        None,
        "description_id must be None when DialogDescription is omitted"
    );
}

#[component]
fn DialogTitleDescriptionMountSample(
    title_registered: std::rc::Rc<std::cell::RefCell<Option<String>>>,
    desc_registered: std::rc::Rc<std::cell::RefCell<Option<String>>>,
) -> Element {
    let title_c = title_registered.clone();
    let desc_c = desc_registered.clone();
    rsx! {
        DialogRoot {
            default_open: true,
            DialogTrigger { "Open" }
            DialogPortal {
                DialogContent {
                    DialogTitle { id: "custom-title", "Custom Title" }
                    DialogDescription { id: "custom-desc", "Custom Description" }
                    DialogTitleDescriptionProbe {
                        title_target: title_c,
                        desc_target: desc_c,
                    }
                }
            }
        }
    }
}

#[test]
fn test_dialog_title_description_mounting_registers_ids() {
    let title_res = std::rc::Rc::new(std::cell::RefCell::new(None));
    let desc_res = std::rc::Rc::new(std::cell::RefCell::new(None));

    let mut dom = VirtualDom::new_with_props(
        DialogTitleDescriptionMountSample,
        DialogTitleDescriptionMountSampleProps {
            title_registered: title_res.clone(),
            desc_registered: desc_res.clone(),
        },
    );
    dom.rebuild_in_place();

    assert_eq!(*title_res.borrow(), Some("custom-title".to_string()));
    assert_eq!(*desc_res.borrow(), Some("custom-desc".to_string()));
}

#[cfg(feature = "native")]
#[test]
fn fm_w1_ctx_dialog_post_mount_on_open_change_policy_sync_and_close_focus() {
    use keyboard_types::{Code, Key, NamedKey};

    use crate::common::native_fixture::*;

    let _guard = NATIVE_TEST_LOCK.lock().unwrap_or_else(|e| e.into_inner());

    #[component]
    fn DialogReactivePropsScenario() -> Element {
        use monoxus::dialog::DialogMode;

        let open = use_signal(|| false);
        let mut swapped = use_signal(|| false);
        let mut cb1_log = use_signal(Vec::<bool>::new);
        let mut cb2_log = use_signal(Vec::<bool>::new);

        let mode = if swapped() {
            DialogMode::NonModal
        } else {
            DialogMode::Modal
        };
        let on_open_change = if swapped() {
            EventHandler::new(move |next: bool| cb2_log.write().push(next))
        } else {
            EventHandler::new(move |next: bool| cb1_log.write().push(next))
        };

        rsx! {
            div { style: "padding: 24px;",
                button {
                    id: "w1-dlg-swap",
                    r#type: "button",
                    style: "width: 90px; height: 24px;",
                    onclick: move |_| swapped.set(true),
                    "Swap Policy"
                }
                span { id: "w1-dlg-cb1", "{cb1_log.read().len()}" }
                span { id: "w1-dlg-cb2", "{cb2_log.read().len()}" }

                DialogRoot {
                    id: "w1-ctx-dlg",
                    open: Some(open),
                    mode: Some(mode),
                    on_open_change: Some(on_open_change),
                    DialogTrigger {
                        id: "w1-ctx-dlg-trig",
                        style: "width: 100px; height: 28px;",
                        "Open Dialog"
                    }
                    DialogPortal {
                        DialogOverlay { id: "w1-ctx-dlg-overlay" }
                        DialogContent {
                            id: "w1-ctx-dlg-content",
                            style: "width: 280px; height: 160px; padding: 12px; background: white;",
                            button {
                                id: "w1-ctx-dlg-swap-inside",
                                r#type: "button",
                                style: "width: 90px; height: 24px;",
                                onclick: move |_| swapped.set(true),
                                "Swap Inside"
                            }
                            DialogClose { id: "w1-ctx-dlg-close", "Close" }
                        }
                    }
                }
            }
        }
    }

    let mut harness = mount_native_scenario(DialogReactivePropsScenario);
    advance_frame_and_pump(&mut harness, 0.02);

    // Open Dialog in Modal mode with cb1
    click_and_pump(&mut harness, "#w1-ctx-dlg-trig");
    advance_frame_and_pump(&mut harness, 0.02);
    assert!(harness.query("#w1-ctx-dlg-content").is_some());
    assert_eq!(
        harness
            .attr("#w1-ctx-dlg-content", "aria-modal")
            .as_deref(),
        Some("true")
    );
    assert_eq!(harness.text_content("#w1-dlg-cb1"), "1");
    assert_eq!(harness.text_content("#w1-dlg-cb2"), "0");

    // H-6: Post-mount change mode Modal -> NonModal (with open unchanged!) AND swap on_open_change to cb2
    click_and_pump(&mut harness, "#w1-ctx-dlg-swap-inside");
    advance_frame_and_pump(&mut harness, 0.02);
    assert_eq!(
        harness
            .attr("#w1-ctx-dlg-content", "aria-modal")
            .as_deref(),
        Some("false"),
        "H-6: DialogRoot must sync runtime_sig when mode changes even if open is unchanged"
    );

    // Dismiss via Escape (invokes DialogContent's dismiss_handler) -> must fire cb2 (not stale cb1)
    // and L-1 restores focus cleanly to trigger after unshield
    send_key_and_pump(&mut harness, Key::Named(NamedKey::Escape), Code::Escape);
    advance_frame_and_pump(&mut harness, 0.02);
    assert!(harness.query("#w1-ctx-dlg-content").is_none());
    assert_eq!(
        harness.text_content("#w1-dlg-cb1"),
        "1",
        "H-6: stale cb1 must NOT fire after on_open_change was swapped"
    );
    assert_eq!(
        harness.text_content("#w1-dlg-cb2"),
        "1",
        "H-6: updated cb2 must fire from DialogContent dismiss_handler"
    );
    assert_native_focus_id(&harness, "w1-ctx-dlg-trig");
}


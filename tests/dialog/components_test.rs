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


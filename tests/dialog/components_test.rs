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


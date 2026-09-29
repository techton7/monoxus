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

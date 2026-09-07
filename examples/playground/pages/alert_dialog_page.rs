use dioxus::prelude::*;
use crate::alert_dialog::AlertDialogPlayground;

#[component]
pub fn AlertDialogPage() -> Element {
    rsx! {
        AlertDialogPlayground {}
    }
}

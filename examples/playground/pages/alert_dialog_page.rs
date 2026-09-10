use crate::alert_dialog::AlertDialogPlayground;
use dioxus::prelude::*;

#[component]
pub fn AlertDialogPage() -> Element {
    rsx! {
        AlertDialogPlayground {}
    }
}

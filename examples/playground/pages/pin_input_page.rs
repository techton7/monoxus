use crate::pin_input::PinInputPlayground;
use dioxus::prelude::*;

#[component]
pub fn PinInputPage() -> Element {
    rsx! {
        PinInputPlayground {}
    }
}

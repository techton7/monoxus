use dioxus::prelude::*;
use crate::popover::PopoverPlayground;

#[component]
pub fn PopoverPage() -> Element {
    rsx! {
        PopoverPlayground {}
    }
}

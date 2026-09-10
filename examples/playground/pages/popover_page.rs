use crate::popover::PopoverPlayground;
use dioxus::prelude::*;

#[component]
pub fn PopoverPage() -> Element {
    rsx! {
        PopoverPlayground {}
    }
}

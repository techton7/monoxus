use crate::accordion::AccordionPlayground;
use dioxus::prelude::*;

#[component]
pub fn AccordionPage() -> Element {
    rsx! {
        AccordionPlayground {}
    }
}

use dioxus::prelude::*;
use crate::accordion::AccordionPlayground;

#[component]
pub fn AccordionPage() -> Element {
    rsx! {
        AccordionPlayground {}
    }
}

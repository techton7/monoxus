use dioxus::prelude::*;
use crate::tooltip::TooltipPlayground;

#[component]
pub fn TooltipPage() -> Element {
    rsx! {
        TooltipPlayground {}
    }
}

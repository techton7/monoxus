use crate::tooltip::TooltipPlayground;
use dioxus::prelude::*;

#[component]
pub fn TooltipPage() -> Element {
    rsx! {
        TooltipPlayground {}
    }
}

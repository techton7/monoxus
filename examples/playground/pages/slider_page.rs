use crate::slider::SliderPlayground;
use dioxus::prelude::*;

#[component]
pub fn SliderPage() -> Element {
    rsx! {
        SliderPlayground {}
    }
}

use crate::meter::MeterPlayground;
use dioxus::prelude::*;

#[component]
pub fn MeterPage() -> Element {
    rsx! {
        MeterPlayground {}
    }
}

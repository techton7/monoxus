use crate::date_picker::DatePickerShowcase;
use dioxus::prelude::*;

#[component]
pub fn DatePickerPage() -> Element {
    rsx! {
        DatePickerShowcase {}
    }
}

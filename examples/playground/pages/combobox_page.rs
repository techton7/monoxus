use crate::combobox::ComboboxShowcase;
use dioxus::prelude::*;

#[component]
pub fn ComboboxPage() -> Element {
    rsx! {
        ComboboxShowcase {}
    }
}

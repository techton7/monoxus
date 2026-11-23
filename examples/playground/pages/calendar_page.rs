use crate::calendar::CalendarPlayground;
use dioxus::prelude::*;

#[component]
pub fn CalendarPage() -> Element {
    rsx! {
        CalendarPlayground {}
    }
}

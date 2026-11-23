use crate::range_calendar::RangeCalendarPlayground;
use dioxus::prelude::*;

#[component]
pub fn RangeCalendarPage() -> Element {
    rsx! {
        RangeCalendarPlayground {}
    }
}

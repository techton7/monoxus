use crate::avatar::AvatarPlayground;
use dioxus::prelude::*;

#[component]
pub fn AvatarPage() -> Element {
    rsx! {
        AvatarPlayground {}
    }
}

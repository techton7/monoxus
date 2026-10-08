use dioxus::prelude::*;

use crate::popover::components::PopoverContext;

#[derive(Props, Clone, PartialEq)]
pub struct PopoverAnchorProps {
    #[props(default)]
    pub id: Option<String>,
    #[props(default)]
    pub class: Option<String>,
    #[props(default)]
    pub style: Option<String>,
    pub children: Element,
}

#[component]
pub fn PopoverAnchor(props: PopoverAnchorProps) -> Element {
    let ctx = use_context::<PopoverContext>();
    let runtime = ctx.runtime.read();
    let anchor = runtime.anchor();
    let anchor_id = props.id.unwrap_or_else(|| anchor.id().to_string());

    rsx! {
        div {
            id: "{anchor_id}",
            class: props.class.as_deref().unwrap_or_default(),
            style: props.style.as_deref().unwrap_or_default(),
            onmounted: runtime.mount_anchor(),
            {props.children}
        }
    }
}

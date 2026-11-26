use dioxus::prelude::*;

use crate::foundation::compose::merge_global_attributes;
use crate::popover::components::PopoverContext;

#[derive(Props, Clone, PartialEq)]
pub struct PopoverAnchorProps {
    #[props(default)]
    pub id: Option<String>,
    #[props(default)]
    pub class: Option<String>,
    #[props(default)]
    pub style: Option<String>,
    #[props(extends = GlobalAttributes)]
    pub attributes: Vec<Attribute>,
    pub children: Element,
}

#[component]
pub fn PopoverAnchor(props: PopoverAnchorProps) -> Element {
    let ctx = use_context::<PopoverContext>();
    let runtime = ctx.runtime.read();
    let anchor = runtime.anchor();
    let anchor_id = props.id.unwrap_or_else(|| anchor.id().to_string());
    let merged = merge_global_attributes(
        props.class.as_deref(),
        props.style.as_deref(),
        props.attributes,
    );

    rsx! {
        div {
            id: "{anchor_id}",
            class: merged.class.as_deref(),
            style: merged.style.as_deref(),
            onmounted: runtime.mount_anchor(),
            ..merged.rest,
            {props.children}
        }
    }
}

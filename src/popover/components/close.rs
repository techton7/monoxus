use dioxus::prelude::*;

use crate::foundation::{
    compose::merge_global_attributes,
    shared::{data_disabled_attr, use_stable_id},
};
use crate::popover::components::PopoverContext;

#[derive(Props, Clone, PartialEq)]
pub struct PopoverCloseProps {
    #[props(default)]
    pub id: Option<String>,
    #[props(default)]
    pub class: Option<String>,
    #[props(default)]
    pub style: Option<String>,
    #[props(default = false)]
    pub disabled: bool,
    #[props(extends = GlobalAttributes)]
    pub attributes: Vec<Attribute>,
    pub children: Element,
}

#[component]
pub fn PopoverClose(props: PopoverCloseProps) -> Element {
    let ctx = use_context::<PopoverContext>();
    let runtime = ctx.runtime.read();
    let close = runtime.close();
    let stable_fallback = use_stable_id(None);
    let close_id = props
        .id
        .unwrap_or_else(|| format!("{stable_fallback}-close"));
    let merged = merge_global_attributes(
        props.class.as_deref(),
        props.style.as_deref(),
        props.attributes,
    );

    rsx! {
        button {
            id: "{close_id}",
            r#type: "button",
            class: merged.class.as_deref(),
            style: merged.style.as_deref(),
            disabled: props.disabled,
            "data-state": close.data_state().as_str(),
            "data-disabled": data_disabled_attr(props.disabled),
            onmounted: runtime.mount_close(),
            onpointerdown: runtime.close_pointer_down(),
            onclick: runtime.close_click(),
            ..merged.rest,
            {props.children}
        }
    }
}

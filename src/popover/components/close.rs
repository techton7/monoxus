use dioxus::prelude::*;

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
    pub children: Element,
}

#[component]
pub fn PopoverClose(props: PopoverCloseProps) -> Element {
    let ctx = use_context::<PopoverContext>();
    let runtime = ctx.runtime.read();
    let close = runtime.close();
    let close_id = props.id.unwrap_or_else(|| close.id().to_string());

    rsx! {
        button {
            id: "{close_id}",
            r#type: "button",
            class: props.class.as_deref().unwrap_or_default(),
            style: props.style.as_deref().unwrap_or_default(),
            disabled: props.disabled,
            "data-state": close.data_state().as_str(),
            "data-disabled": if props.disabled { "true" } else { "false" },
            onmounted: runtime.mount_close(),
            onclick: runtime.close_click(),
            {props.children}
        }
    }
}

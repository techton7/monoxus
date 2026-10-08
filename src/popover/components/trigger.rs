use dioxus::prelude::*;

use crate::popover::components::PopoverContext;

#[derive(Props, Clone, PartialEq)]
pub struct PopoverTriggerProps {
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
pub fn PopoverTrigger(props: PopoverTriggerProps) -> Element {
    let ctx = use_context::<PopoverContext>();
    let runtime = ctx.runtime.read();
    let trigger = runtime.trigger();
    let trigger_id = props.id.unwrap_or_else(|| trigger.id().to_string());

    rsx! {
        button {
            id: "{trigger_id}",
            r#type: "button",
            class: props.class.as_deref().unwrap_or_default(),
            style: props.style.as_deref().unwrap_or_default(),
            disabled: props.disabled,
            aria_haspopup: "dialog",
            aria_expanded: "{trigger.aria_expanded()}",
            aria_controls: "{trigger.aria_controls()}",
            "data-state": trigger.data_state().as_str(),
            "data-disabled": if props.disabled { "true" } else { "false" },
            onmounted: runtime.mount_trigger(),
            onclick: runtime.trigger_click(),
            {props.children}
        }
    }
}

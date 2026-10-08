use dioxus::prelude::*;

use crate::tooltip::components::TooltipRootContext;

#[derive(Props, Clone, PartialEq)]
pub struct TooltipTriggerProps {
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
pub fn TooltipTrigger(props: TooltipTriggerProps) -> Element {
    let ctx = use_context::<TooltipRootContext>();
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
            aria_describedby: trigger.aria_describedby(),
            "data-state": trigger.data_state().as_str(),
            "data-disabled": if props.disabled { "true" } else { "false" },
            onmounted: runtime.mount_trigger(),
            onpointerdown: runtime.trigger_pointer_down(),
            onmouseenter: runtime.trigger_pointer_enter(),
            onmouseleave: runtime.trigger_pointer_leave(),
            onfocus: runtime.trigger_focus(),
            onblur: runtime.trigger_blur(),
            onclick: runtime.trigger_click(),
            onkeydown: runtime.escape_keydown(),
            {props.children}
        }
    }
}

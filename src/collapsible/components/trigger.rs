use dioxus::prelude::*;

use crate::collapsible::components::CollapsibleContext;

#[derive(Props, Clone, PartialEq)]
pub struct CollapsibleTriggerProps {
    #[props(default)]
    pub id: Option<String>,
    #[props(default)]
    pub class: Option<String>,
    #[props(default)]
    pub style: Option<String>,
    #[props(default = false)]
    pub disabled: bool,
    #[props(default)]
    pub aria_controls: Option<String>,
    #[props(default)]
    pub onclick: Option<EventHandler<MouseEvent>>,
    pub children: Element,
}

#[component]
pub fn CollapsibleTrigger(props: CollapsibleTriggerProps) -> Element {
    let ctx = use_context::<CollapsibleContext>();
    let runtime = ctx.runtime.read();
    let trigger = runtime.trigger();
    let trigger_id = props.id.unwrap_or_else(|| trigger.id().to_string());

    let is_disabled = props.disabled || runtime.is_disabled();
    let aria_expanded = trigger.aria_expanded();
    let aria_controls = props
        .aria_controls
        .or_else(|| ctx.content_id.read().clone())
        .unwrap_or_else(|| trigger.aria_controls().to_string());

    let click_runtime = runtime.clone();
    let onclick_handler = props.onclick;

    let on_click = move |evt: MouseEvent| {
        if let Some(h) = onclick_handler {
            h.call(evt);
        }
        if !is_disabled {
            click_runtime.toggle();
        }
    };

    rsx! {
        button {
            id: "{trigger_id}",
            r#type: "button",
            class: props.class.as_deref().unwrap_or_default(),
            style: props.style.as_deref().unwrap_or_default(),
            disabled: is_disabled,
            aria_expanded: "{aria_expanded}",
            aria_controls: "{aria_controls}",
            "data-state": trigger.data_state_str(),
            "data-disabled": if is_disabled { "true" } else { "false" },
            onclick: on_click,
            {props.children}
        }
    }
}

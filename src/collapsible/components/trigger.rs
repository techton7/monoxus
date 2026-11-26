use dioxus::prelude::*;

use crate::collapsible::components::CollapsibleContext;
use crate::foundation::{compose::merge_global_attributes, shared::data_disabled_attr};

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
    #[props(extends = GlobalAttributes)]
    pub attributes: Vec<Attribute>,
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
    let is_mounted = (ctx.content_mounted)();
    let should_emit_controls = runtime.is_open() || is_mounted;
    let target_id = props
        .aria_controls
        .or_else(|| ctx.content_id.read().clone())
        .unwrap_or_else(|| trigger.target_content_id().to_string());
    let aria_controls = should_emit_controls.then_some(target_id);

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

    let merged = merge_global_attributes(
        props.class.as_deref(),
        props.style.as_deref(),
        props.attributes,
    );

    rsx! {
        button {
            id: "{trigger_id}",
            r#type: "button",
            class: merged.class.as_deref().unwrap_or_default(),
            style: merged.style.as_deref().unwrap_or_default(),
            disabled: is_disabled,
            aria_expanded: "{aria_expanded}",
            aria_controls: aria_controls.as_deref(),
            "data-state": trigger.data_state_str(),
            "data-disabled": data_disabled_attr(is_disabled),
            onclick: on_click,
            ..merged.rest,
            {props.children}
        }
    }
}

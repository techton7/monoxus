use dioxus::core::{AttributeValue, Event, ListenerCallback};
use dioxus::html::PlatformEventData;
use dioxus::prelude::*;
use ui_sdk::kernel::{ChangeContext, ChangeReason, ChangeSource};

use crate::collapsible::components::CollapsibleContext;
use crate::foundation::compose::{
    collect_user_attrs, merge_props, push_attr_opt, push_bool_data_attr, push_event_handler,
};

#[derive(Props, Clone, PartialEq)]
pub struct CollapsibleTriggerProps {
    #[props(default, into)]
    pub id: Option<String>,
    #[props(default, into)]
    pub class: Option<String>,
    #[props(default, into)]
    pub style: Option<String>,
    #[props(default = false)]
    pub disabled: bool,
    #[props(default, into)]
    pub aria_controls: Option<String>,
    #[props(default)]
    pub on_click: Option<EventHandler<MouseEvent>>,
    #[props(default)]
    pub child: Option<Callback<Vec<Attribute>, Element>>,
    #[props(extends = GlobalAttributes)]
    pub attributes: Vec<Attribute>,
    #[props(default)]
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

    let mut base = Vec::with_capacity(9);
    base.push(Attribute::new("id", trigger_id, None, false));
    base.push(Attribute::new("type", "button", None, false));
    if is_disabled {
        base.push(Attribute::new("disabled", true, None, false));
    }
    base.push(Attribute::new("aria-expanded", aria_expanded, None, false));
    push_attr_opt(&mut base, "aria-controls", aria_controls);
    base.push(Attribute::new(
        "data-state",
        trigger.data_state_str(),
        None,
        false,
    ));
    push_bool_data_attr(&mut base, "data-disabled", is_disabled);
    base.push(Attribute::new(
        "onclick",
        AttributeValue::Listener(
            ListenerCallback::<PlatformEventData>::new(move |evt: Event<PlatformEventData>| {
                if is_disabled {
                    evt.prevent_default();
                    return;
                }
                if evt.default_action_enabled() {
                    let _ = click_runtime.toggle_with_context(ChangeContext::new(
                        ChangeReason::Toggle,
                        ChangeSource::Mouse,
                    ));
                }
            })
            .erase(),
        ),
        None,
        false,
    ));

    let mut user = collect_user_attrs(props.class, props.style, props.attributes);
    if !is_disabled {
        push_event_handler(&mut user, "onclick", props.on_click);
    }

    let merged = merge_props(base, user);

    if let Some(child) = props.child {
        return child.call(merged);
    }

    rsx! {
        button {
            ..merged,
            {props.children}
        }
    }
}

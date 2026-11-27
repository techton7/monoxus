use dioxus::core::{AttributeValue, Event, ListenerCallback};
use dioxus::html::PlatformEventData;
use dioxus::prelude::*;

use crate::foundation::{
    compose::{
        collect_user_attrs, merge_props, push_attr_opt, push_event_handler,
    },
    roving_focus::{RovingItemMeta, use_roving_focus_item},
    shared::use_stable_id,
};
use crate::toolbar::components::ToolbarRootContext;

#[derive(Clone, PartialEq, Props)]
pub struct ToolbarButtonProps {
    #[props(default, into)]
    pub id: Option<String>,
    #[props(default, into)]
    pub class: Option<String>,
    #[props(default, into)]
    pub style: Option<String>,
    #[props(default = false)]
    pub disabled: bool,
    #[props(default, into)]
    pub value: Option<String>,
    #[props(default)]
    pub on_click: Option<EventHandler<MouseEvent>>,
    #[props(default)]
    pub on_key_down: Option<EventHandler<KeyboardEvent>>,
    #[props(default)]
    pub on_mounted: Option<EventHandler<MountedEvent>>,
    #[props(default)]
    pub child: Option<Callback<Vec<Attribute>, Element>>,
    #[props(extends = GlobalAttributes)]
    pub attributes: Vec<Attribute>,
    #[props(default)]
    pub children: Element,
}

#[component]
pub fn ToolbarButton(props: ToolbarButtonProps) -> Element {
    let ctx = use_context::<ToolbarRootContext>();
    let runtime = ctx.runtime;
    let toolbar_state = runtime.state();

    let stable_id = use_stable_id(None);
    let button_id = props
        .id
        .clone()
        .unwrap_or_else(|| toolbar_state.relationships().button_id(&stable_id));
    let roving_key = props
        .value
        .clone()
        .unwrap_or_else(|| format!("btn:{button_id}"));

    use_roving_focus_item(
        runtime.roving,
        &roving_key,
        RovingItemMeta::new(button_id.clone(), !props.disabled, false),
    );

    let tabindex = runtime.roving.tab_index_for(&roving_key, !props.disabled);
    let attrs = toolbar_state.button_attributes(button_id, props.disabled, tabindex);

    let disabled = props.disabled;
    let roving_key_down = roving_key.clone();
    let roving_for_key = runtime.roving;
    let roving_for_focus = runtime.roving;

    let mut base = Vec::with_capacity(12);
    base.push(Attribute::new("id", attrs.id().to_string(), None, false));
    base.push(Attribute::new("type", attrs.button_type(), None, false));
    base.push(Attribute::new(
        "tabindex",
        attrs.tabindex().to_string(),
        None,
        false,
    ));
    if attrs.is_disabled() {
        base.push(Attribute::new("disabled", true, None, false));
    }
    push_attr_opt(&mut base, "data-disabled", attrs.data_disabled());
    base.push(Attribute::new(
        "data-orientation",
        attrs.data_orientation(),
        None,
        false,
    ));
    base.push(Attribute::new(
        "data-toolbar-button",
        attrs.data_toolbar_button(),
        None,
        false,
    ));
    base.push(Attribute::new(
        "data-toolbar-item",
        attrs.data_toolbar_item(),
        None,
        false,
    ));
    base.push(Attribute::new(
        "onclick",
        AttributeValue::Listener(
            ListenerCallback::<PlatformEventData>::new(move |evt: Event<PlatformEventData>| {
                if disabled {
                    evt.prevent_default();
                }
            })
            .erase(),
        ),
        None,
        false,
    ));
    base.push(Attribute::new(
        "onkeydown",
        AttributeValue::Listener(
            ListenerCallback::<PlatformEventData>::new(move |evt: Event<PlatformEventData>| {
                let kb_evt: KeyboardEvent = evt.map(|d| KeyboardData::from(d));
                if !kb_evt.default_action_enabled() || disabled {
                    return;
                }
                roving_for_key.handle_keydown(
                    Some(&roving_key_down),
                    &kb_evt,
                    None::<fn(&str)>,
                );
            })
            .erase(),
        ),
        None,
        false,
    ));
    base.push(Attribute::new(
        "onfocus",
        AttributeValue::Listener(
            ListenerCallback::<PlatformEventData>::new(move |_: Event<PlatformEventData>| {
                roving_for_focus.handle_item_focus(&roving_key, !disabled);
            })
            .erase(),
        ),
        None,
        false,
    ));

    let mut user = collect_user_attrs(props.class, props.style, props.attributes);
    if !disabled {
        push_event_handler(&mut user, "onclick", props.on_click);
    }
    push_event_handler(&mut user, "onkeydown", props.on_key_down);
    push_event_handler(&mut user, "onmounted", props.on_mounted);

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

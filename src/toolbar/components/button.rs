use dioxus::prelude::*;

use crate::foundation::{
    compose::merge_global_attributes,
    roving_focus::{RovingItemMeta, use_roving_focus_item},
    shared::use_stable_id,
};
use crate::toolbar::components::ToolbarRootContext;

#[derive(Clone, PartialEq, Props)]
pub struct ToolbarButtonProps {
    #[props(default)]
    pub id: Option<String>,
    #[props(default)]
    pub class: Option<String>,
    #[props(default)]
    pub style: Option<String>,
    #[props(default = false)]
    pub disabled: bool,
    #[props(default)]
    pub value: Option<String>,
    #[props(default)]
    pub on_click: Option<EventHandler<MouseEvent>>,
    #[props(default)]
    pub on_key_down: Option<EventHandler<KeyboardEvent>>,
    #[props(default)]
    pub on_mounted: Option<EventHandler<MountedEvent>>,
    #[props(extends = GlobalAttributes)]
    pub attributes: Vec<Attribute>,
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

    let on_click = {
        let consumer_on_click = props.on_click;
        let disabled = props.disabled;
        move |evt: MouseEvent| {
            if disabled {
                evt.prevent_default();
                return;
            }
            if let Some(ref handler) = consumer_on_click {
                handler.call(evt);
            }
        }
    };

    let on_key_down = {
        let roving = runtime.roving;
        let key_for_roving = roving_key.clone();
        let disabled = props.disabled;
        let consumer_on_keydown = props.on_key_down;
        move |evt: KeyboardEvent| {
            if let Some(ref handler) = consumer_on_keydown {
                handler.call(evt.clone());
            }
            if !evt.default_action_enabled() || disabled {
                return;
            }
            roving.handle_keydown(Some(&key_for_roving), &evt, None::<fn(&str)>);
        }
    };

    let on_focus = {
        let roving = runtime.roving;
        let key_for_focus = roving_key;
        let disabled = props.disabled;
        move |_| {
            roving.handle_item_focus(&key_for_focus, !disabled);
        }
    };

    let on_mounted = {
        let consumer_on_mounted = props.on_mounted;
        move |evt: MountedEvent| {
            if let Some(ref handler) = consumer_on_mounted {
                handler.call(evt);
            }
        }
    };

    let merged = merge_global_attributes(
        props.class.as_deref(),
        props.style.as_deref(),
        props.attributes,
    );

    rsx! {
        button {
            id: "{attrs.id()}",
            class: merged.class.as_deref().unwrap_or_default(),
            style: merged.style.as_deref().unwrap_or_default(),
            r#type: attrs.button_type(),
            tabindex: attrs.tabindex(),
            disabled: attrs.is_disabled(),
            "data-disabled": attrs.data_disabled(),
            "data-orientation": attrs.data_orientation(),
            "data-toolbar-button": attrs.data_toolbar_button(),
            "data-toolbar-item": attrs.data_toolbar_item(),
            onclick: on_click,
            onkeydown: on_key_down,
            onfocus: on_focus,
            onmounted: on_mounted,
            ..merged.rest,
            {props.children}
        }
    }
}

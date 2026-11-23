use dioxus::prelude::keyboard_types::{Key, NamedKey};
use dioxus::prelude::*;

use crate::foundation::{
    roving_focus::{RovingItemMeta, use_roving_focus_item},
    shared::use_stable_id,
};
use crate::toolbar::components::{ToolbarRootContext, ToolbarToggleGroupContext};

#[derive(Clone, PartialEq, Props)]
pub struct ToolbarToggleItemProps {
    #[props(default)]
    pub id: Option<String>,
    #[props(default)]
    pub class: Option<String>,
    #[props(default)]
    pub style: Option<String>,
    pub value: String,
    #[props(default = false)]
    pub disabled: bool,
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
pub fn ToolbarToggleItem(props: ToolbarToggleItemProps) -> Element {
    let toolbar_ctx = use_context::<ToolbarRootContext>();
    let group_ctx = use_context::<ToolbarToggleGroupContext>();
    let toolbar_runtime = toolbar_ctx.runtime;
    let group_runtime = group_ctx.runtime;
    let toolbar_state = toolbar_runtime.state();

    let stable_id = use_stable_id(None);
    let group_dom_id = group_runtime.group_dom_id();
    let item_id = props
        .id
        .clone()
        .unwrap_or_else(|| format!("{}-{}", toolbar_state.relationships().toggle_item_id(&group_dom_id, &props.value), stable_id));
    let roving_key = format!("tg:{group_dom_id}:{}", props.value);

    let effective_disabled = group_runtime.is_effective_disabled(props.disabled);
    let is_pressed = group_runtime.is_item_pressed(&props.value);

    use_roving_focus_item(
        toolbar_runtime.roving,
        &roving_key,
        RovingItemMeta::new(item_id.clone(), !effective_disabled, is_pressed),
    );

    let tabindex = toolbar_runtime
        .roving
        .tab_index_for(&roving_key, !effective_disabled);
    let attrs = group_runtime.item_attributes(
        &toolbar_state,
        &props.value,
        Some(item_id),
        props.disabled,
        tabindex,
    );

    let on_click = {
        let group_runtime = group_runtime.clone();
        let roving = toolbar_runtime.roving;
        let val = props.value.clone();
        let key_for_roving = roving_key.clone();
        let consumer_on_click = props.on_click;
        move |evt: MouseEvent| {
            if let Some(ref handler) = consumer_on_click {
                handler.call(evt.clone());
            }
            if evt.default_action_enabled() && !effective_disabled {
                group_runtime.toggle_item(&val);
                roving.set_current_tab_stop(Some(key_for_roving.clone()));
            }
        }
    };

    let on_key_down = {
        let group_runtime = group_runtime;
        let roving = toolbar_runtime.roving;
        let val = props.value.clone();
        let key_for_roving = roving_key.clone();
        let consumer_on_keydown = props.on_key_down;
        move |evt: KeyboardEvent| {
            if let Some(ref handler) = consumer_on_keydown {
                handler.call(evt.clone());
            }
            if !evt.default_action_enabled() || effective_disabled {
                return;
            }
            let key = evt.key();
            if key == Key::Named(NamedKey::Enter) || key == Key::Character(" ".to_string()) {
                evt.prevent_default();
                group_runtime.toggle_item(&val);
                roving.set_current_tab_stop(Some(key_for_roving.clone()));
                return;
            }
            roving.handle_keydown(Some(&key_for_roving), &evt, None::<fn(&str)>);
        }
    };

    let on_focus = {
        let roving = toolbar_runtime.roving;
        let key_for_focus = roving_key;
        move |_| {
            roving.handle_item_focus(&key_for_focus, !effective_disabled);
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

    rsx! {
        button {
            id: "{attrs.id()}",
            class: props.class.as_deref().unwrap_or_default(),
            style: props.style.as_deref().unwrap_or_default(),
            r#type: attrs.button_type(),
            role: attrs.role(),
            "aria-checked": attrs.aria_checked(),
            "aria-pressed": attrs.aria_pressed(),
            tabindex: attrs.tabindex(),
            disabled: attrs.is_disabled(),
            "data-disabled": attrs.data_disabled(),
            "data-state": attrs.data_state().as_str(),
            "data-value": attrs.data_value(),
            "data-orientation": attrs.data_orientation(),
            "data-toolbar-group-item": attrs.data_toolbar_group_item(),
            "data-toolbar-item": attrs.data_toolbar_item(),
            onclick: on_click,
            onkeydown: on_key_down,
            onfocus: on_focus,
            onmounted: on_mounted,
            ..props.attributes,
            {props.children}
        }
    }
}

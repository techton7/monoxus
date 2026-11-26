use std::borrow::Cow;

use dioxus::prelude::*;

use crate::foundation::{
    compose::merge_global_attributes,
    keyboard::is_activation_key,
    roving_focus::{RovingItemMeta, use_roving_focus_item},
    shared::{Direction, ScopeHandle, use_direction_context, use_stable_id},
};

use super::{
    runtime::{ToggleGroupRuntime, use_toggle_group_runtime},
    types::{ToggleGroupMode, ToggleGroupOrientation},
};

#[derive(Clone)]
pub struct ToggleGroupContext {
    pub runtime: ToggleGroupRuntime,
}

#[derive(Props, Clone, PartialEq)]
pub struct ToggleGroupRootProps {
    #[props(default)]
    pub id: Option<String>,
    #[props(default)]
    pub class: Option<String>,
    #[props(default)]
    pub style: Option<String>,
    #[props(default)]
    pub r#type: ToggleGroupMode,
    #[props(default)]
    pub value: Option<String>,
    #[props(default)]
    pub default_value: Option<String>,
    #[props(default)]
    pub on_value_change: Option<EventHandler<String>>,
    #[props(default)]
    pub values: Option<Vec<String>>,
    #[props(default)]
    pub default_values: Option<Vec<String>>,
    #[props(default)]
    pub on_values_change: Option<EventHandler<Vec<String>>>,
    #[props(default = false)]
    pub disabled: bool,
    #[props(default = true)]
    pub roving_focus: bool,
    #[props(default = true)]
    pub loop_focus: bool,
    #[props(default)]
    pub orientation: ToggleGroupOrientation,
    #[props(default)]
    pub dir: Option<Direction>,
    #[props(default)]
    pub role: Option<String>,
    #[props(extends = GlobalAttributes)]
    pub attributes: Vec<Attribute>,
    pub children: Element,
}

#[component]
pub fn ToggleGroupRoot(props: ToggleGroupRootProps) -> Element {
    let resolved_dir = use_direction_context(props.dir);
    let scope_id = use_stable_id(props.id.clone().map(Cow::Owned)).into_owned();
    let initial_root_dom_id = props.id.clone().unwrap_or_else(|| {
        ScopeHandle::root("toggle-group")
            .child(scope_id.clone())
            .qualify("root")
    });

    let runtime = use_toggle_group_runtime(
        props.r#type,
        props.value.clone(),
        props.default_value.clone(),
        props.on_value_change,
        props.values.clone(),
        props.default_values.clone(),
        props.on_values_change,
        props.orientation,
        resolved_dir,
        props.roving_focus,
        props.loop_focus,
        props.disabled,
        scope_id,
        initial_root_dom_id,
    );

    use_context_provider(|| ToggleGroupContext {
        runtime: runtime.clone(),
    });

    let root_attrs = runtime.root_attributes(None);
    let resolved_id = props.id.as_deref().unwrap_or(root_attrs.id());
    runtime.roving.set_root_dom_id(resolved_id);
    let resolved_role = props.role.as_deref().unwrap_or(root_attrs.role());
    let merged = merge_global_attributes(
        props.class.as_deref(),
        props.style.as_deref(),
        props.attributes,
    );

    rsx! {
        div {
            id: "{resolved_id}",
            class: merged.class.as_deref().unwrap_or_default(),
            style: merged.style.as_deref().unwrap_or_default(),
            role: "{resolved_role}",
            dir: "{root_attrs.dir}",
            "data-orientation": "{root_attrs.data_orientation}",
            "data-disabled": root_attrs.data_disabled,
            "data-toggle-group-root": "{root_attrs.data_toggle_group_root}",
            ..merged.rest,
            {props.children}
        }
    }
}

#[derive(Props, Clone, PartialEq)]
pub struct ToggleGroupItemProps {
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
pub fn ToggleGroupItem(props: ToggleGroupItemProps) -> Element {
    let ctx = use_context::<ToggleGroupContext>();
    let runtime = ctx.runtime;

    let item_id = props
        .id
        .clone()
        .unwrap_or_else(|| runtime.state().relationships().item_id(&props.value));
    let effective_disabled = runtime.is_effective_disabled(props.disabled);
    let is_pressed = runtime.is_item_pressed(&props.value);

    use_roving_focus_item(
        runtime.roving,
        &props.value,
        RovingItemMeta::new(item_id.clone(), !effective_disabled, is_pressed),
    );

    let item_attrs = runtime.item_attributes(&props.value, Some(item_id), props.disabled);

    let on_click = {
        let runtime = runtime.clone();
        let val = props.value.clone();
        let consumer_on_click = props.on_click;
        move |evt: MouseEvent| {
            if let Some(ref handler) = consumer_on_click {
                handler.call(evt.clone());
            }
            if evt.default_action_enabled() && !effective_disabled {
                runtime.toggle_item(&val);
            }
        }
    };

    let on_key_down = {
        let runtime = runtime.clone();
        let val = props.value.clone();
        let consumer_on_keydown = props.on_key_down;
        move |evt: KeyboardEvent| {
            if let Some(ref handler) = consumer_on_keydown {
                handler.call(evt.clone());
            }

            if !evt.default_action_enabled() || effective_disabled {
                return;
            }

            if is_activation_key(&evt.key()) {
                evt.prevent_default();
                runtime.toggle_item(&val);
                return;
            }

            runtime
                .roving
                .handle_keydown(Some(&val), &evt, None::<fn(&str)>);
        }
    };

    let on_focus = {
        let runtime = runtime.clone();
        let val = props.value.clone();
        move |_| {
            runtime.roving.handle_item_focus(&val, !effective_disabled);
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
            id: "{item_attrs.id}",
            class: merged.class.as_deref().unwrap_or_default(),
            style: merged.style.as_deref().unwrap_or_default(),
            r#type: "button",
            role: item_attrs.role,
            "aria-checked": item_attrs.aria_checked,
            "aria-pressed": item_attrs.aria_pressed,
            tabindex: item_attrs.tabindex,
            disabled: item_attrs.disabled,
            "data-disabled": item_attrs.data_disabled,
            "data-state": item_attrs.data_state.as_str(),
            "data-value": "{item_attrs.data_value}",
            "data-orientation": "{item_attrs.data_orientation}",
            "data-toggle-group-item": "{item_attrs.data_toggle_group_item}",
            onclick: on_click,
            onkeydown: on_key_down,
            onfocus: on_focus,
            onmounted: on_mounted,
            ..merged.rest,
            {props.children}
        }
    }
}

use std::rc::Rc;

use dioxus::prelude::keyboard_types::{Key, NamedKey};
use dioxus::prelude::*;

use crate::foundation::{
    roving_focus::{RovingFocusConfig, RovingItemMeta, use_roving_focus_item},
    shared::{Direction, ScopeHandle, use_direction_context},
    state::{ControllableStateProps, use_controllable_state},
};

use super::{
    runtime::ToggleGroupRuntime,
    types::{
        ToggleGroupMode, ToggleGroupMultipleChangeHandler, ToggleGroupOrientation,
        ToggleGroupSingleChangeHandler,
    },
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
    let scope_id = props
        .id
        .clone()
        .unwrap_or_else(|| ScopeHandle::root("toggle-group").token());

    let on_single_change_cb = props.on_value_change.as_ref().map(|handler| {
        let handler = *handler;
        Rc::new(move |next: String| {
            handler.call(next);
        }) as ToggleGroupSingleChangeHandler
    });

    let on_multiple_change_cb = props.on_values_change.as_ref().map(|handler| {
        let handler = *handler;
        Rc::new(move |next: Vec<String>| {
            handler.call(next);
        }) as ToggleGroupMultipleChangeHandler
    });

    let single_handle = use_controllable_state(ControllableStateProps {
        value: props.value.clone(),
        default_value: props.default_value.clone().unwrap_or_default(),
        on_change: on_single_change_cb.as_ref().map(|cb| {
            let cb = cb.clone();
            move |v: String| cb(v)
        }),
    });

    let multiple_handle = use_controllable_state(ControllableStateProps {
        value: props.values.clone(),
        default_value: props.default_values.clone().unwrap_or_default(),
        on_change: on_multiple_change_cb.as_ref().map(|cb| {
            let cb = cb.clone();
            move |v: Vec<String>| cb(v)
        }),
    });

    let single_set: ToggleGroupSingleChangeHandler = {
        let handle = single_handle.clone();
        Rc::new(move |val: String| {
            handle.set(val);
        })
    };

    let multiple_set: ToggleGroupMultipleChangeHandler = {
        let handle = multiple_handle.clone();
        Rc::new(move |val: Vec<String>| {
            handle.set(val);
        })
    };

    let initial_single = single_handle.value().clone();
    let initial_multiple = multiple_handle.value().clone();
    let single_is_controlled = single_handle.is_controlled();
    let multiple_is_controlled = multiple_handle.is_controlled();
    let initial_root_dom_id = props.id.clone().unwrap_or_else(|| {
        ScopeHandle::root("toggle-group")
            .child(scope_id.clone())
            .qualify("root")
    });

    let mut runtime = ToggleGroupRuntime::new(
        props.r#type,
        initial_single,
        initial_multiple,
        single_is_controlled,
        multiple_is_controlled,
        props.orientation,
        resolved_dir,
        props.roving_focus,
        props.loop_focus,
        props.disabled,
        scope_id.clone(),
        initial_root_dom_id,
        Some(single_set),
        Some(multiple_set),
    );

    // Keep reactive props synchronized with runtime signals
    {
        if *runtime.mode.peek() != props.r#type {
            runtime.mode.set(props.r#type);
        }

        if *runtime.single_controlled.peek() != single_is_controlled {
            runtime.single_controlled.set(single_is_controlled);
        }

        if *runtime.multiple_controlled.peek() != multiple_is_controlled {
            runtime.multiple_controlled.set(multiple_is_controlled);
        }

        let resolved_single = single_handle.value().clone();
        if *runtime.single_value.peek() != resolved_single {
            runtime.single_value.set(resolved_single);
        }

        let resolved_multiple = multiple_handle.value().clone();
        if *runtime.multiple_values.peek() != resolved_multiple {
            runtime.multiple_values.set(resolved_multiple);
        }

        if *runtime.orientation.peek() != props.orientation {
            runtime.orientation.set(props.orientation);
        }

        if *runtime.dir.peek() != resolved_dir {
            runtime.dir.set(resolved_dir);
        }

        if *runtime.roving_focus.peek() != props.roving_focus {
            runtime.roving_focus.set(props.roving_focus);
        }

        if *runtime.loop_focus.peek() != props.loop_focus {
            runtime.loop_focus.set(props.loop_focus);
        }

        if *runtime.disabled.peek() != props.disabled {
            runtime.disabled.set(props.disabled);
        }

        runtime.roving.set_config(
            RovingFocusConfig::new()
                .with_orientation(props.orientation.into())
                .with_direction(resolved_dir)
                .with_loop_focus(props.loop_focus)
                .with_enabled(props.roving_focus),
        );
    }

    use_context_provider(|| ToggleGroupContext {
        runtime: runtime.clone(),
    });

    let root_attrs = runtime.root_attributes(None);
    let resolved_id = props.id.as_deref().unwrap_or(root_attrs.id());
    runtime.roving.set_root_dom_id(resolved_id);
    let resolved_role = props.role.as_deref().unwrap_or(root_attrs.role());

    rsx! {
        div {
            id: "{resolved_id}",
            class: props.class.as_deref().unwrap_or_default(),
            style: props.style.as_deref().unwrap_or_default(),
            role: "{resolved_role}",
            dir: "{root_attrs.dir}",
            "data-orientation": "{root_attrs.data_orientation}",
            "data-disabled": root_attrs.data_disabled,
            "data-toggle-group-root": "{root_attrs.data_toggle_group_root}",
            ..props.attributes,
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

    let item_attrs = runtime.item_attributes(&props.value, Some(item_id.clone()), props.disabled);

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

            let key = evt.key();
            if key == Key::Named(NamedKey::Enter) || key == Key::Character(" ".to_string()) {
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

    rsx! {
        button {
            id: "{item_attrs.id}",
            class: props.class.as_deref().unwrap_or_default(),
            style: props.style.as_deref().unwrap_or_default(),
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
            ..props.attributes,
            {props.children}
        }
    }
}

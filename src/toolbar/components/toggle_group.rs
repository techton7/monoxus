use std::rc::Rc;

use dioxus::prelude::*;

use crate::foundation::{
    shared::use_stable_id,
    state::{ControllableStateProps, use_controllable_state},
};
use crate::toggle_group::{ToggleGroupMultipleChangeHandler, ToggleGroupSingleChangeHandler};
use crate::toolbar::{
    components::{ToolbarRootContext, ToolbarToggleGroupContext},
    runtime::ToolbarToggleGroupRuntime,
    types::ToolbarToggleGroupMode,
};

#[derive(Clone, PartialEq, Props)]
pub struct ToolbarToggleGroupProps {
    #[props(default)]
    pub id: Option<String>,
    #[props(default)]
    pub class: Option<String>,
    #[props(default)]
    pub style: Option<String>,
    #[props(default)]
    pub r#type: ToolbarToggleGroupMode,
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
    #[props(default)]
    pub role: Option<String>,
    #[props(extends = GlobalAttributes)]
    pub attributes: Vec<Attribute>,
    pub children: Element,
}

#[component]
pub fn ToolbarToggleGroup(props: ToolbarToggleGroupProps) -> Element {
    let toolbar_ctx = use_context::<ToolbarRootContext>();
    let toolbar_runtime = toolbar_ctx.runtime;
    let toolbar_state = toolbar_runtime.state();

    let stable_id = use_stable_id(None);
    let group_scope_id = props.id.clone().unwrap_or_else(|| stable_id.into_owned());
    let group_dom_id = props
        .id
        .clone()
        .unwrap_or_else(|| toolbar_state.relationships().toggle_group_id(&group_scope_id));

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

    let mut group_runtime = ToolbarToggleGroupRuntime::new(
        props.r#type,
        initial_single,
        initial_multiple,
        single_is_controlled,
        multiple_is_controlled,
        props.disabled,
        group_scope_id,
        group_dom_id.clone(),
        Some(single_set),
        Some(multiple_set),
    );

    {
        if *group_runtime.mode.peek() != props.r#type {
            group_runtime.mode.set(props.r#type);
        }
        if *group_runtime.single_controlled.peek() != single_is_controlled {
            group_runtime.single_controlled.set(single_is_controlled);
        }
        if *group_runtime.multiple_controlled.peek() != multiple_is_controlled {
            group_runtime.multiple_controlled.set(multiple_is_controlled);
        }
        let resolved_single = single_handle.value().clone();
        if *group_runtime.single_value.peek() != resolved_single {
            group_runtime.single_value.set(resolved_single);
        }
        let resolved_multiple = multiple_handle.value().clone();
        if *group_runtime.multiple_values.peek() != resolved_multiple {
            group_runtime.multiple_values.set(resolved_multiple);
        }
        if *group_runtime.disabled.peek() != props.disabled {
            group_runtime.disabled.set(props.disabled);
        }
        if *group_runtime.group_dom_id.peek() != group_dom_id {
            group_runtime.group_dom_id.set(group_dom_id.clone());
        }
    }

    use_context_provider(|| ToolbarToggleGroupContext {
        runtime: group_runtime.clone(),
    });

    let root_attrs = group_runtime.root_attributes(&toolbar_state, None);
    let resolved_role = props.role.as_deref().unwrap_or(root_attrs.role());

    rsx! {
        div {
            id: "{group_dom_id}",
            class: props.class.as_deref().unwrap_or_default(),
            style: props.style.as_deref().unwrap_or_default(),
            role: "{resolved_role}",
            dir: root_attrs.dir(),
            "data-orientation": root_attrs.data_orientation(),
            "data-disabled": root_attrs.data_disabled(),
            "data-toolbar-group": root_attrs.data_toolbar_group(),
            ..props.attributes,
            {props.children}
        }
    }
}

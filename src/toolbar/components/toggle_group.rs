use dioxus::prelude::*;

use crate::foundation::{
    compose::merge_global_attributes,
    shared::use_stable_id,
};
use crate::toolbar::{
    components::{ToolbarRootContext, ToolbarToggleGroupContext},
    runtime::use_toolbar_toggle_group_runtime,
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
    let group_dom_id = props.id.clone().unwrap_or_else(|| {
        toolbar_state
            .relationships()
            .toggle_group_id(&group_scope_id)
    });

    let group_runtime = use_toolbar_toggle_group_runtime(
        props.r#type,
        props.value.clone(),
        props.default_value.clone(),
        props.on_value_change,
        props.values.clone(),
        props.default_values.clone(),
        props.on_values_change,
        toolbar_state.toggle_group_orientation(),
        toolbar_state.dir(),
        props.disabled,
        group_scope_id,
        group_dom_id.clone(),
    );

    use_context_provider(|| ToolbarToggleGroupContext {
        runtime: group_runtime.clone(),
    });

    let root_attrs = group_runtime.root_attributes(&toolbar_state, None);
    let resolved_role = props.role.as_deref().unwrap_or(root_attrs.role());
    let merged = merge_global_attributes(
        props.class.as_deref(),
        props.style.as_deref(),
        props.attributes,
    );

    rsx! {
        div {
            id: "{group_dom_id}",
            class: merged.class.as_deref().unwrap_or_default(),
            style: merged.style.as_deref().unwrap_or_default(),
            role: "{resolved_role}",
            dir: root_attrs.dir(),
            "data-orientation": root_attrs.data_orientation(),
            "data-disabled": root_attrs.data_disabled(),
            "data-toolbar-group": root_attrs.data_toolbar_group(),
            ..merged.rest,
            {props.children}
        }
    }
}

use dioxus::prelude::*;

use crate::foundation::{
    compose::{collect_user_attrs, merge_props, push_attr_opt},
    shared::use_stable_id,
};
use crate::toolbar::{
    components::{ToolbarRootContext, ToolbarToggleGroupContext},
    runtime::use_toolbar_toggle_group_runtime,
    types::ToolbarToggleGroupMode,
};

#[derive(Clone, PartialEq, Props)]
pub struct ToolbarToggleGroupProps {
    #[props(default, into)]
    pub id: Option<String>,
    #[props(default, into)]
    pub class: Option<String>,
    #[props(default, into)]
    pub style: Option<String>,
    #[props(default)]
    pub r#type: ToolbarToggleGroupMode,
    #[props(default, into)]
    pub value: Option<String>,
    #[props(default, into)]
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
    #[props(default, into)]
    pub role: Option<String>,
    #[props(default)]
    pub child: Option<Callback<Vec<Attribute>, Element>>,
    #[props(extends = GlobalAttributes)]
    pub attributes: Vec<Attribute>,
    #[props(default)]
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
    let resolved_role = props
        .role
        .unwrap_or_else(|| root_attrs.role().to_string());

    let mut base = Vec::with_capacity(6);
    base.push(Attribute::new("id", group_dom_id, None, false));
    base.push(Attribute::new("role", resolved_role, None, false));
    base.push(Attribute::new("dir", root_attrs.dir(), None, false));
    base.push(Attribute::new(
        "data-orientation",
        root_attrs.data_orientation(),
        None,
        false,
    ));
    push_attr_opt(&mut base, "data-disabled", root_attrs.data_disabled());
    base.push(Attribute::new(
        "data-toolbar-group",
        root_attrs.data_toolbar_group(),
        None,
        false,
    ));

    let user = collect_user_attrs(props.class, props.style, props.attributes);
    let merged = merge_props(base, user);

    if let Some(child) = props.child {
        return child.call(merged);
    }

    rsx! {
        div {
            ..merged,
            {props.children}
        }
    }
}

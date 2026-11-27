use std::borrow::Cow;

use dioxus::prelude::*;

use crate::collapsible::{
    components::CollapsibleContext,
    relationships::CollapsibleRelationships,
    runtime::use_collapsible_runtime_controllable,
    state::Collapsible,
};
use crate::foundation::{
    compose::{collect_user_attrs, merge_props, push_bool_data_attr},
    shared::{ScopeHandle, use_stable_id},
    state::{sync_signal, use_controllable_signal},
};

#[derive(Props, Clone, PartialEq)]
pub struct CollapsibleRootProps {
    #[props(default, into)]
    pub id: Option<String>,
    #[props(default, into)]
    pub class: Option<String>,
    #[props(default, into)]
    pub style: Option<String>,
    #[props(default)]
    pub open: Option<bool>,
    #[props(default = false)]
    pub default_open: bool,
    #[props(default = false)]
    pub disabled: bool,
    #[props(default)]
    pub on_open_change: Option<EventHandler<bool>>,
    #[props(default)]
    pub child: Option<Callback<Vec<Attribute>, Element>>,
    #[props(extends = GlobalAttributes)]
    pub attributes: Vec<Attribute>,
    #[props(default)]
    pub children: Element,
}

#[component]
pub fn CollapsibleRoot(props: CollapsibleRootProps) -> Element {
    let open_ctl = use_controllable_signal(props.open, props.default_open, props.on_open_change);
    let open_signal = open_ctl.signal();

    let mut disabled_signal = use_signal(|| props.disabled);
    sync_signal(&mut disabled_signal, props.disabled);

    let scope_id = use_stable_id(props.id.clone().map(Cow::Owned));
    let scope = ScopeHandle::root("collapsible").child(scope_id);
    let relationships = CollapsibleRelationships::new(scope.clone());

    let is_open = open_signal();
    let collapsible = Collapsible::new(scope, is_open).with_disabled(props.disabled);

    let runtime = use_collapsible_runtime_controllable(collapsible, open_ctl);
    let mut runtime_sig = use_signal(|| runtime.clone());
    sync_signal(&mut runtime_sig, runtime.clone());

    let content_id = use_signal(|| None);
    let content_mounted = use_signal(|| false);

    use_context_provider(|| CollapsibleContext {
        runtime: runtime_sig,
        open: open_signal,
        disabled: disabled_signal,
        content_id,
        content_mounted,
        on_open_change: props.on_open_change,
        relationships: relationships.clone(),
    });

    let attrs = runtime.root();
    let mut base = Vec::with_capacity(4);
    base.push(Attribute::new("id", attrs.id().to_string(), None, false));
    base.push(Attribute::new("data-state", attrs.data_state_str(), None, false));
    push_bool_data_attr(&mut base, "data-disabled", attrs.is_disabled());

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

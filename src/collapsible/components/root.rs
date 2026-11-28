use std::borrow::Cow;
use std::rc::Rc;

use dioxus::prelude::*;
use ui_sdk::kernel::ChangeEvent;

use crate::agent::{
    ActionDescriptor, AgentHints, AgentSurface, ComponentSnapshot, DispatchExecResult,
    SnapshotValue, SurfaceMetaDefaults, use_agent_surface,
};
use crate::collapsible::{
    components::CollapsibleContext,
    relationships::CollapsibleRelationships,
    runtime::use_collapsible_runtime_controllable,
    state::Collapsible,
};
use crate::foundation::{
    compose::{collect_user_attrs, merge_props, push_bool_data_attr},
    shared::{ScopeHandle, use_stable_id},
    state::{ControllableConfig, sync_signal, use_controllable_signal_config},
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
    pub on_open_change_details: Option<EventHandler<ChangeEvent<bool>>>,
    #[props(default)]
    pub agent: Option<AgentHints>,
    #[props(default)]
    pub child: Option<Callback<Vec<Attribute>, Element>>,
    #[props(extends = GlobalAttributes)]
    pub attributes: Vec<Attribute>,
    #[props(default)]
    pub children: Element,
}

#[component]
pub fn CollapsibleRoot(props: CollapsibleRootProps) -> Element {
    let open_ctl = use_controllable_signal_config(
        ControllableConfig::new(props.open, props.default_open)
            .with_explicit_default(props.default_open)
            .with_on_change(props.on_open_change)
            .with_on_change_details(props.on_open_change_details)
            .with_component("CollapsibleRoot")
            .with_prop("open"),
    );
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
    let agent_label = attrs.id().to_string();
    let reveals = if !is_open {
        vec![String::from("content")]
    } else {
        Vec::new()
    };
    let agent_rt_dispatch = runtime.clone();
    use_agent_surface(
        props.agent.as_ref(),
        SurfaceMetaDefaults::new().with_reveals(reveals),
        || {
            AgentSurface::new("collapsible", agent_label)
                .with_disabled(props.disabled)
                .with_snapshot(
                    ComponentSnapshot::new().with_field("open", SnapshotValue::Bool(is_open)),
                )
                .with_actions(vec![
                    ActionDescriptor::new("toggle", "Toggle collapsible open state", false),
                    ActionDescriptor::new("open", "Open collapsible content", false),
                    ActionDescriptor::new("close", "Close collapsible content", false),
                    ActionDescriptor::new("set", "Set collapsible open state", true),
                ])
        },
        || {
            Rc::new(move |dispatch| match dispatch.action.as_str() {
                "toggle" => {
                    let next = !open_ctl.peek();
                    let outcome = agent_rt_dispatch.set_open_with_context(next, dispatch.ctx);
                    let mut res = DispatchExecResult::ok().with_canceled(outcome.is_canceled());
                    if !outcome.is_canceled() {
                        res = res.with_expected("open", SnapshotValue::Bool(next));
                    }
                    res
                }
                "open" => {
                    let outcome = agent_rt_dispatch.set_open_with_context(true, dispatch.ctx);
                    let mut res = DispatchExecResult::ok().with_canceled(outcome.is_canceled());
                    if !outcome.is_canceled() {
                        res = res.with_expected("open", SnapshotValue::Bool(true));
                    }
                    res
                }
                "close" => {
                    let outcome = agent_rt_dispatch.set_open_with_context(false, dispatch.ctx);
                    let mut res = DispatchExecResult::ok().with_canceled(outcome.is_canceled());
                    if !outcome.is_canceled() {
                        res = res.with_expected("open", SnapshotValue::Bool(false));
                    }
                    res
                }
                "set" => {
                    let Some(SnapshotValue::Bool(next)) = dispatch
                        .args
                        .get("open")
                        .or_else(|| dispatch.args.get("value"))
                    else {
                        return DispatchExecResult::ok()
                            .with_host_error("missing or invalid boolean argument 'open'");
                    };
                    let next = *next;
                    let outcome = agent_rt_dispatch.set_open_with_context(next, dispatch.ctx);
                    let mut res = DispatchExecResult::ok().with_canceled(outcome.is_canceled());
                    if !outcome.is_canceled() {
                        res = res.with_expected("open", SnapshotValue::Bool(next));
                    }
                    res
                }
                other => DispatchExecResult::ok()
                    .with_host_error(format!("unknown collapsible action '{other}'")),
            })
        },
        || {
            Rc::new(move || {
                ComponentSnapshot::new().with_field(
                    "open",
                    SnapshotValue::Bool(open_ctl.try_peek_cloned().unwrap_or(false)),
                )
            })
        },
    );
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

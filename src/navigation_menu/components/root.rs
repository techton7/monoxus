use std::borrow::Cow;
use std::rc::Rc;

use dioxus::prelude::*;
use ui_sdk::agent::{
    ActionDescriptor, ActionDispatch, AgentHints, AgentSurface, ComponentSnapshot, SnapshotValue,
};
use ui_sdk::kernel::ChangeEvent;

use crate::agent::{
    DispatchExecResult, SurfaceMetaDefaults, cancel_current_agent_action, use_agent_surface,
};
use crate::foundation::{
    compose::{collect_user_attrs, merge_props, push_attr_opt, push_event_handler},
    shared::{Direction, Orientation, ScopeHandle, use_direction_context, use_stable_id},
    state::{ControllableConfig, use_controllable_signal_config},
};

use super::super::{
    runtime::{NavigationMenuInContent, use_navigation_menu_root_runtime},
    types::{
        NAVIGATION_MENU_DEFAULT_DELAY_MS, NAVIGATION_MENU_DEFAULT_SKIP_DELAY_MS,
        NavigationMenuActivationMode,
    },
};

#[derive(Clone, PartialEq, Props)]
pub struct NavigationMenuRootProps {
    #[props(default, into)]
    pub id: Option<String>,
    #[props(default, into)]
    pub class: Option<String>,
    #[props(default, into)]
    pub style: Option<String>,
    #[props(default, into)]
    pub value: Option<String>,
    #[props(default, into)]
    pub default_value: Option<String>,
    #[props(default)]
    pub on_value_change: Option<EventHandler<String>>,
    #[props(default)]
    pub on_value_change_details: Option<EventHandler<ChangeEvent<String>>>,
    #[props(default = NAVIGATION_MENU_DEFAULT_DELAY_MS)]
    pub delay_duration: u64,
    #[props(default = NAVIGATION_MENU_DEFAULT_SKIP_DELAY_MS)]
    pub skip_delay_duration: u64,
    #[props(default)]
    pub dir: Option<Direction>,
    #[props(default = Orientation::Horizontal)]
    pub orientation: Orientation,
    #[props(default)]
    pub activation_mode: NavigationMenuActivationMode,
    #[props(default = Some("Main".to_string()), into)]
    pub aria_label: Option<String>,
    #[props(default)]
    pub child: Option<Callback<Vec<Attribute>, Element>>,
    #[props(default)]
    pub on_mounted: Option<EventHandler<MountedEvent>>,
    #[props(default)]
    pub agent: Option<AgentHints>,
    #[props(extends = GlobalAttributes)]
    pub attributes: Vec<Attribute>,
    pub children: Element,
}

#[component]
pub fn NavigationMenuRoot(props: NavigationMenuRootProps) -> Element {
    let resolved_dir = use_direction_context(props.dir);
    let stable_id = use_stable_id(props.id.clone().map(Cow::Owned));
    let scope_id = stable_id.into_owned();
    let scope = ScopeHandle::root("navigation-menu").child(scope_id);

    let has_explicit_default = props.default_value.is_some();
    let value_state = use_controllable_signal_config(
        ControllableConfig::new(props.value, props.default_value.unwrap_or_default())
            .with_explicit_default(has_explicit_default)
            .with_on_change(props.on_value_change)
            .with_on_change_details(props.on_value_change_details)
            .with_component("NavigationMenuRoot")
            .with_prop("value"),
    );

    let initial_rels = super::super::relationships::NavigationMenuRelationships::new(scope.clone());
    let dom_id = props
        .id
        .clone()
        .unwrap_or_else(|| initial_rels.root_id().to_owned());

    let ctx = use_navigation_menu_root_runtime(
        scope,
        dom_id.clone(),
        value_state,
        props.delay_duration,
        props.skip_delay_duration,
        resolved_dir,
        props.orientation,
        props.activation_mode,
        props.aria_label.clone(),
    );

    ctx.sync_root_props(
        &dom_id,
        props.delay_duration,
        props.skip_delay_duration,
        resolved_dir,
        props.orientation,
        props.activation_mode,
        props.aria_label.clone(),
    );

    use_context_provider(|| ctx.clone());
    use_context_provider(|| NavigationMenuInContent(false));

    let reveals: Vec<String> = ctx
        .triggers
        .try_peek()
        .map(|list| list.iter().map(|t| t.value.clone()).collect())
        .unwrap_or_default();
    let build_nav_snapshot = {
        let nav_ctx = ctx.clone();
        move || {
            let cur = nav_ctx
                .value
                .try_peek()
                .map(|v| v.clone())
                .unwrap_or_default();
            let items: Vec<String> = nav_ctx
                .triggers
                .try_peek()
                .map(|list| list.iter().map(|t| t.value.clone()).collect())
                .unwrap_or_default();
            ComponentSnapshot::new()
                .with_field("open", SnapshotValue::Bool(!cur.is_empty()))
                .with_field("value", SnapshotValue::Text(cur))
                .with_items(items)
        }
    };
    let agent_label = props
        .aria_label
        .clone()
        .unwrap_or_else(|| String::from("Navigation Menu"));
    let agent_ctx = ctx.clone();
    let reader_snap = build_nav_snapshot.clone();
    use_agent_surface(
        props.agent.as_ref(),
        SurfaceMetaDefaults::new().with_reveals(reveals),
        move || {
            AgentSurface::new("navigation-menu", agent_label)
                .with_snapshot(build_nav_snapshot())
                .with_actions(vec![
                    ActionDescriptor::new("select", "Open a navigation menu section", true),
                    ActionDescriptor::new("close", "Close active navigation menu section", false),
                ])
        },
        move || {
            Rc::new(move |dispatch: &ActionDispatch| match dispatch.action.as_str() {
                "select" => {
                    let Some(SnapshotValue::Text(v)) = dispatch.args.get("value") else {
                        return DispatchExecResult::ok()
                            .with_host_error("missing or invalid `value` argument");
                    };
                    let outcome = agent_ctx.set_value_with_context(v.clone(), dispatch.ctx);
                    if outcome.is_canceled() {
                        cancel_current_agent_action();
                        DispatchExecResult::ok().with_canceled(true)
                    } else {
                        DispatchExecResult::ok()
                    }
                }
                "close" => {
                    let outcome = agent_ctx.set_value_with_context(String::new(), dispatch.ctx);
                    if outcome.is_canceled() {
                        cancel_current_agent_action();
                        DispatchExecResult::ok().with_canceled(true)
                    } else {
                        DispatchExecResult::ok()
                    }
                }
                other => DispatchExecResult::ok()
                    .with_host_error(format!("unsupported navigation-menu action: {other}")),
            })
        },
        move || Rc::new(reader_snap),
    );

    let root_attrs = ctx.root_attributes(Some(dom_id));

    let mut base = Vec::with_capacity(6);
    base.push(Attribute::new("id", root_attrs.id().to_owned(), None, false));
    push_attr_opt(
        &mut base,
        "aria-label",
        root_attrs.aria_label().map(str::to_owned),
    );
    base.push(Attribute::new("dir", root_attrs.dir(), None, false));
    base.push(Attribute::new(
        "data-orientation",
        root_attrs.data_orientation(),
        None,
        false,
    ));
    base.push(Attribute::new(
        "data-monoxus-navigation-menu-root",
        root_attrs.data_monoxus_navigation_menu_root(),
        None,
        false,
    ));

    let mut user = collect_user_attrs(props.class, props.style, props.attributes);
    push_event_handler::<MountedData>(&mut user, "onmounted", props.on_mounted);
    let merged = merge_props(base, user);

    if let Some(child) = props.child {
        return child.call(merged);
    }

    rsx! {
        nav {
            ..merged,
            {props.children}
        }
    }
}

pub use NavigationMenuRoot as NavigationMenu;

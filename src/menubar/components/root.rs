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
    compose::{collect_user_attrs, merge_props},
    shared::{Direction, ScopeHandle, use_direction_context, use_stable_id},
    state::{ControllableConfig, use_controllable_signal_config},
};
use crate::menu::{MenuOpenFocusTarget, MenuPart, MenuVariant};
use crate::menubar::{MenubarRelationships, MenubarRootAttributes, MenubarRuntimeContext};

#[derive(Props, Clone, PartialEq)]
pub struct MenubarRootProps {
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
    #[props(default)]
    pub dir: Option<Direction>,
    #[props(default = true)]
    pub loop_focus: bool,
    #[props(default)]
    pub submenu_open_delay_ms: Option<u64>,
    #[props(default)]
    pub child: Option<Callback<Vec<Attribute>, Element>>,
    #[props(default)]
    pub agent: Option<AgentHints>,
    #[props(extends = GlobalAttributes)]
    pub attributes: Vec<Attribute>,
    pub children: Element,
}

/// Horizontal `role="menubar"` root container coordinating roving focus, hover swaps, and cross-menu keyboard navigation (`MB-1`..`MB-7`).
#[component]
pub fn MenubarRoot(props: MenubarRootProps) -> Element {
    let effective_dir = use_direction_context(props.dir);
    let token = use_stable_id(None);
    let relationships =
        use_hook(|| MenubarRelationships::new(ScopeHandle::root("menubar").child(token)));
    let root_id = props.id.unwrap_or_else(|| relationships.root_id().to_owned());

    let has_explicit_default = props.default_value.is_some();
    let value_state = use_controllable_signal_config(
        ControllableConfig::new(props.value.clone(), props.default_value.unwrap_or_default())
            .with_explicit_default(has_explicit_default)
            .with_on_change(props.on_value_change)
            .with_on_change_details(props.on_value_change_details)
            .with_component("MenubarRoot")
            .with_prop("value"),
    );

    let runtime = MenubarRuntimeContext::new(
        relationships,
        value_state,
        effective_dir,
        props.loop_focus,
        props.submenu_open_delay_ms,
    );
    runtime.set_root_id(root_id.clone());

    if let Some(controlled) = props.value.as_deref() {
        runtime.sync_controlled_value(Some(controlled));
    }

    {
        let mut st = runtime.state;
        let changed = st.try_peek().is_ok_and(|s| {
            s.direction() != effective_dir || s.loop_focus() != props.loop_focus
        });
        if changed {
            let _ = st.try_write().map(|mut w| {
                let next = w
                    .clone()
                    .with_direction(effective_dir)
                    .with_loop_focus(props.loop_focus);
                *w = next;
            });
        }
    }

    use_context_provider(|| runtime.clone());

    let build_menubar_snapshot = {
        let rt = runtime.clone();
        move || {
            let active = rt
                .state
                .try_peek()
                .ok()
                .and_then(|s| s.active_value().map(str::to_owned));
            let items: Vec<String> = rt
                .state
                .try_peek()
                .map(|s| s.menus().iter().map(|m| m.value.clone()).collect())
                .unwrap_or_default();
            ComponentSnapshot::new()
                .with_field("open", SnapshotValue::Bool(active.is_some()))
                .with_field(
                    "value",
                    active.map(SnapshotValue::Text).unwrap_or(SnapshotValue::Null),
                )
                .with_items(items)
        }
    };
    let agent_rt = runtime;
    let reader_snap = build_menubar_snapshot.clone();
    use_agent_surface(
        props.agent.as_ref(),
        SurfaceMetaDefaults::new(),
        move || {
            AgentSurface::new("menubar", "Menubar")
                .with_snapshot(build_menubar_snapshot())
                .with_actions(vec![
                    ActionDescriptor::new("select", "Open a menubar menu by value", true),
                    ActionDescriptor::new("close", "Close active menubar menu", false),
                ])
        },
        move || {
            Rc::new(move |dispatch: &ActionDispatch| match dispatch.action.as_str() {
                "select" => {
                    let Some(SnapshotValue::Text(v)) = dispatch.args.get("value") else {
                        return DispatchExecResult::ok()
                            .with_host_error("missing or invalid `value` argument");
                    };
                    let outcome = agent_rt.open_menu_with_context(
                        v,
                        MenuOpenFocusTarget::ContentContainer,
                        dispatch.ctx,
                    );
                    if outcome.is_canceled() {
                        cancel_current_agent_action();
                        DispatchExecResult::ok().with_canceled(true)
                    } else {
                        DispatchExecResult::ok()
                    }
                }
                "close" => {
                    let outcome = agent_rt.close_all_with_context(dispatch.ctx);
                    if outcome.is_canceled() {
                        cancel_current_agent_action();
                        DispatchExecResult::ok().with_canceled(true)
                    } else {
                        DispatchExecResult::ok()
                    }
                }
                other => DispatchExecResult::ok()
                    .with_host_error(format!("unsupported menubar action: {other}")),
            })
        },
        move || Rc::new(reader_snap),
    );

    let attrs = MenubarRootAttributes::new(root_id, effective_dir);
    let part_attr = MenuPart::Root.static_data_attr_name(MenuVariant::Menubar);

    let base = vec![
        Attribute::new("id", attrs.id, None, false),
        Attribute::new("role", attrs.role, None, false),
        Attribute::new("aria-orientation", attrs.aria_orientation, None, false),
        Attribute::new("dir", attrs.dir, None, false),
        Attribute::new(part_attr, "", None, false),
    ];
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

pub use MenubarRoot as Menubar;

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
    shared::{Direction, ScopeHandle, use_direction_context, use_stable_id},
    state::{ControllableConfig, sync_signal, use_controllable_signal_config},
};
use crate::menu::{
    MenuActivePortalSurface, MenuRelationships, MenuRootContext, MenuVariant,
    use_menu_surface_lifecycle,
};

#[derive(Props, Clone, PartialEq)]
pub struct ContextMenuRootProps {
    #[props(default)]
    pub open: Option<bool>,
    #[props(default = false)]
    pub default_open: bool,
    #[props(default)]
    pub on_open_change: Option<EventHandler<bool>>,
    #[props(default)]
    pub on_open_change_details: Option<EventHandler<ChangeEvent<bool>>>,
    #[props(default)]
    pub on_open_change_complete: Option<EventHandler<bool>>,
    #[props(default = true)]
    pub modal: bool,
    #[props(default)]
    pub dir: Option<Direction>,
    #[props(default = true)]
    pub loop_focus: bool,
    #[props(default)]
    pub submenu_open_delay_ms: Option<u64>,
    #[props(default)]
    pub agent: Option<AgentHints>,
    pub children: Element,
}

/// Root provider coordinating virtual-point context menu open state, modal policy, direction, and lifecycle.
#[component]
pub fn ContextMenuRoot(props: ContextMenuRootProps) -> Element {
    let effective_dir = use_direction_context(props.dir);
    let token = use_stable_id(None);
    let relationships =
        use_hook(|| MenuRelationships::new(ScopeHandle::root("context-menu").child(token)));
    let open_state = use_controllable_signal_config(
        ControllableConfig::new(props.open, props.default_open)
            .with_explicit_default(props.default_open)
            .with_on_change(props.on_open_change)
            .with_on_change_details(props.on_open_change_details)
            .with_component("ContextMenuRoot")
            .with_prop("open"),
    );

    let root = MenuRootContext::new(
        MenuVariant::ContextMenu,
        relationships,
        open_state,
        props.on_open_change_complete,
        props.modal,
        effective_dir,
        props.loop_focus,
        props.submenu_open_delay_ms,
        None,
    );

    {
        let mut m = root.modal;
        sync_signal(&mut m, props.modal);
        let mut d = root.dir;
        sync_signal(&mut d, effective_dir);
        let mut lf = root.loop_focus;
        sync_signal(&mut lf, props.loop_focus);
    }

    use_menu_surface_lifecycle(root.root_surface.clone(), root.clone());

    let build_snapshot = {
        let open_sig = root.open;
        move || {
            let is_open = open_sig.try_peek().map(|g| *g).unwrap_or(false);
            ComponentSnapshot::new().with_field("open", SnapshotValue::Bool(is_open))
        }
    };
    let agent_root = root.clone();
    let reader_snap = build_snapshot;
    use_agent_surface(
        props.agent.as_ref(),
        SurfaceMetaDefaults::new(),
        move || {
            AgentSurface::new("context-menu", "Context Menu")
                .with_snapshot(build_snapshot())
                .with_actions(vec![
                    ActionDescriptor::new("open", "Open context menu", false),
                    ActionDescriptor::new("close", "Close context menu", false),
                ])
        },
        move || {
            Rc::new(move |dispatch: &ActionDispatch| match dispatch.action.as_str() {
                "open" => {
                    let outcome = agent_root.request_open_change_with_context(true, dispatch.ctx);
                    if outcome.is_canceled() {
                        cancel_current_agent_action();
                        DispatchExecResult::ok().with_canceled(true)
                    } else {
                        DispatchExecResult::ok()
                    }
                }
                "close" => {
                    let event = ChangeEvent::new(false, dispatch.ctx);
                    let outcome = agent_root.close_menu_tree_with_event(true, &event);
                    if outcome.is_canceled() {
                        cancel_current_agent_action();
                        DispatchExecResult::ok().with_canceled(true)
                    } else {
                        DispatchExecResult::ok()
                    }
                }
                other => DispatchExecResult::ok()
                    .with_host_error(format!("unsupported context-menu action: {other}")),
            })
        },
        move || Rc::new(reader_snap),
    );

    use_context_provider(|| MenuActivePortalSurface(root.root_surface.clone()));
    use_context_provider(|| root);

    rsx! { {props.children} }
}

pub use ContextMenuRoot as ContextMenu;

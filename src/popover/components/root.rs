use std::{borrow::Cow, cell::RefCell, rc::Rc};

use dioxus::prelude::*;
use ui_sdk::kernel::{ChangeContext, ChangeEvent, ChangeReason, ChangeSource};

use crate::agent::{
    ActionDescriptor, AgentHints, AgentSurface, ComponentSnapshot, DispatchExecResult,
    SnapshotValue, SurfaceMetaDefaults, cancel_current_agent_action, use_agent_surface,
};
use crate::foundation::{
    overlay::{
        AnyPopoverHandle, FloatingLayer, PlacementSide, PopoverHandleProp,
        use_overlay_handle_version,
    },
    shared::{ScopeHandle, use_stable_id},
    state::{ControllableConfig, use_controllable_signal_config},
};
use crate::popover::{
    components::PopoverContext,
    relationships::PopoverRelationships,
    runtime::use_popover_runtime_with_ctl,
    state::Popover,
    types::POPOVER_GEOMETRY_NAMESPACE,
};

#[derive(Props, Clone, PartialEq)]
pub struct PopoverRootProps {
    #[props(default, into)]
    pub id: Option<String>,
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
    #[props(default)]
    pub modal: Option<bool>,
    #[props(default, into)]
    pub handle: PopoverHandleProp,
    #[props(default)]
    pub agent: Option<AgentHints>,
    pub children: Element,
}

#[component]
pub fn PopoverRoot(props: PopoverRootProps) -> Element {
    let PopoverRootProps {
        id,
        open,
        default_open,
        on_open_change,
        on_open_change_details,
        on_open_change_complete,
        modal,
        handle,
        agent,
        children,
    } = props;

    let open_ctl = use_controllable_signal_config(
        ControllableConfig::new(open, default_open)
            .with_explicit_default(default_open)
            .with_on_change(on_open_change)
            .with_on_change_details(on_open_change_details)
            .with_component("PopoverRoot")
            .with_prop("open"),
    );
    let open_signal = open_ctl.signal();

    let scope_id = use_stable_id(id.clone().map(Cow::Owned)).into_owned();
    let scope = ScopeHandle::root("popover").child(scope_id);
    let relationships = PopoverRelationships::new(scope.clone());
    let root_id = relationships.root_id().to_owned();

    let _handle_version = use_overlay_handle_version(handle.as_any());

    let attached_handle: Rc<RefCell<Option<(AnyPopoverHandle, String)>>> =
        use_hook(|| Rc::new(RefCell::new(None)));
    {
        let mut slot = attached_handle.borrow_mut();
        let needs_rebind = match (&*slot, handle.as_any()) {
            (Some((prev, prev_rid)), Some(next)) => prev != next || prev_rid != &root_id,
            (Some(_), None) | (None, Some(_)) => true,
            (None, None) => false,
        };
        if needs_rebind {
            if let Some((prev, prev_rid)) = slot.take() {
                prev.detach_root(&prev_rid);
            }
            if let Some(next) = handle.as_any() {
                let reader_ctl = open_ctl;
                let setter_ctl = open_ctl;
                next.attach_root(
                    root_id.clone(),
                    Rc::new(move || reader_ctl.try_peek_cloned().unwrap_or(false)),
                    Rc::new(move |next_open| {
                        setter_ctl.set_with_context(
                            next_open,
                            ChangeContext::new(ChangeReason::Programmatic, ChangeSource::Programmatic),
                        );
                    }),
                );
                *slot = Some((next.clone(), root_id.clone()));
            }
        }
    }
    let attached_for_drop = Rc::clone(&attached_handle);
    use_drop(move || {
        if let Some((prev, prev_rid)) = attached_for_drop.borrow_mut().take() {
            prev.detach_root(&prev_rid);
        }
    });

    let is_open = open_signal();
    if let Some(any_handle) = handle.as_any() {
        any_handle.sync_open_state_for_root(&root_id, is_open);
    }

    let handle_active_trigger_id = handle.as_any().and_then(|h| h.active_trigger_id());
    let handle_active_trigger_mounted = handle.as_any().and_then(|h| h.active_trigger_mounted());

    let is_modal = modal.unwrap_or(false);

    let mut popover = if is_modal {
        Popover::new_modal(scope, is_open)
    } else {
        Popover::new(scope, is_open)
    };

    if let Some(m) = modal {
        popover = popover.with_modal(m);
    }

    let floating = FloatingLayer::new(PlacementSide::Bottom)
        .with_side_offset(4.0)
        .with_avoid_collisions(true)
        .with_collision_padding(8.0)
        .with_namespace(POPOVER_GEOMETRY_NAMESPACE);
    popover = popover.with_floating(floating);
    if let Some(any_handle) = handle.as_any() {
        for tid in any_handle.registered_trigger_ids() {
            popover
                .lifecycle_mut()
                .dismiss_layer_mut()
                .register_branch(tid);
        }
    }

    let runtime = use_popover_runtime_with_ctl(
        popover,
        open_ctl,
        handle_active_trigger_id,
        handle_active_trigger_mounted,
    );
    runtime.set_on_open_change_complete(on_open_change_complete);
    let mut runtime_sig = use_signal(|| runtime.clone());
    if runtime_sig.peek().popover() != runtime.popover() {
        runtime_sig.set(runtime.clone());
    }

    let agent_label = id.unwrap_or_else(|| "Popover".to_owned());
    use_agent_surface(
        agent.as_ref(),
        SurfaceMetaDefaults::default(),
        move || {
            AgentSurface::new("popover", agent_label)
                .with_snapshot(
                    ComponentSnapshot::new().with_field("open", SnapshotValue::Bool(is_open)),
                )
                .with_action(ActionDescriptor::new("open", "Open popover", false))
                .with_action(ActionDescriptor::new("close", "Close popover", false))
                .with_action(ActionDescriptor::new("toggle", "Toggle popover", false))
        },
        move || {
            Rc::new(move |dispatch| {
                let cur = open_ctl.try_peek_cloned().unwrap_or(false);
                let next = match dispatch.action.as_str() {
                    "open" => true,
                    "close" => false,
                    "toggle" => !cur,
                    other => {
                        return DispatchExecResult::ok()
                            .with_host_error(format!("unknown popover action: {other}"));
                    }
                };
                let outcome = open_ctl.set_with_context(next, dispatch.ctx);
                if outcome.is_canceled() {
                    cancel_current_agent_action();
                    return DispatchExecResult::ok().with_canceled(true);
                }
                DispatchExecResult::ok().with_expected("open", SnapshotValue::Bool(next))
            })
        },
        move || {
            Rc::new(move || {
                let open_now = open_ctl.try_peek_cloned().unwrap_or(false);
                ComponentSnapshot::new().with_field("open", SnapshotValue::Bool(open_now))
            })
        },
    );

    let title_id = use_signal(|| None);
    let description_id = use_signal(|| None);
    let content_id = use_signal(|| None);

    use_context_provider(|| PopoverContext {
        runtime: runtime_sig,
        open: open_signal,
        open_ctl,
        on_open_change,
        relationships: relationships.clone(),
        title_id,
        description_id,
        content_id,
    });

    rsx! {
        {children}
    }
}

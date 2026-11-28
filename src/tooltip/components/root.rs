use std::{
    borrow::Cow,
    cell::{Cell, RefCell},
    rc::Rc,
};

use dioxus::prelude::*;
use ui_sdk::kernel::{ChangeContext, ChangeEvent, ChangeReason, ChangeSource};

use crate::agent::{
    ActionDescriptor, AgentHints, AgentSurface, ComponentSnapshot, DispatchExecResult,
    SnapshotValue, SurfaceMetaDefaults, cancel_current_agent_action, use_agent_surface,
};
use crate::foundation::{
    overlay::{AnyTooltipHandle, TooltipHandleProp, use_overlay_handle_version},
    shared::{ScopeHandle, use_stable_id},
    state::{ControllableConfig, use_controllable_signal_config},
};
use crate::tooltip::{
    components::{TooltipProviderContext, TooltipRootContext},
    relationships::TooltipRelationships,
    runtime::{use_tooltip_provider_runtime, use_tooltip_runtime_with_ctl},
    state::{Tooltip, TooltipProvider as TooltipProviderDef},
};

#[derive(Props, Clone, PartialEq)]
pub struct TooltipRootProps {
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
    pub delay_duration: Option<u64>,
    #[props(default)]
    pub skip_delay_duration: Option<u64>,
    #[props(default)]
    pub disable_hoverable_content: Option<bool>,
    #[props(default)]
    pub close_on_trigger_click: Option<bool>,
    #[props(default)]
    pub ignore_non_keyboard_focus: Option<bool>,
    #[props(default, into)]
    pub handle: TooltipHandleProp,
    #[props(default)]
    pub agent: Option<AgentHints>,
    pub children: Element,
}

#[component]
pub fn TooltipRoot(props: TooltipRootProps) -> Element {
    let open_ctl = use_controllable_signal_config(
        ControllableConfig::new(props.open, props.default_open)
            .with_explicit_default(props.default_open)
            .with_on_change(props.on_open_change)
            .with_on_change_details(props.on_open_change_details)
            .with_component("TooltipRoot")
            .with_prop("open"),
    );
    let open_signal = open_ctl.signal();

    let scope_id = use_stable_id(props.id.clone().map(Cow::Owned));
    let scope = ScopeHandle::root("tooltip").child(scope_id);
    let relationships = TooltipRelationships::new(scope.clone());
    let root_id = relationships.root_id().to_owned();

    let _handle_version = use_overlay_handle_version(props.handle.as_any());

    let parent_provider_ctx = try_consume_context::<TooltipProviderContext>();

    let fallback_scope = ScopeHandle::root("tooltip")
        .child(format!("{}-fallback-provider", relationships.root_id()));
    let mut fallback_provider_def = TooltipProviderDef::new(fallback_scope);
    if let Some(delay) = props.delay_duration {
        fallback_provider_def = fallback_provider_def.with_delay_duration_ms(delay);
    }
    if let Some(skip) = props.skip_delay_duration {
        fallback_provider_def = fallback_provider_def.with_skip_delay_duration_ms(skip);
    }
    if let Some(dhc) = props.disable_hoverable_content {
        fallback_provider_def = fallback_provider_def.with_disable_hoverable_content(dhc);
    }
    if let Some(cotc) = props.close_on_trigger_click {
        fallback_provider_def = fallback_provider_def.with_close_on_trigger_click(cotc);
    }
    if let Some(inkf) = props.ignore_non_keyboard_focus {
        fallback_provider_def = fallback_provider_def.with_ignore_non_keyboard_focus(inkf);
    }
    let fallback_provider_runtime = use_tooltip_provider_runtime(fallback_provider_def);

    // Preserve parent TooltipProvider shared skip-delay & active-tooltip state even when
    // per-root override props are supplied (`LD-09`).
    let effective_provider_runtime = match parent_provider_ctx {
        Some(ctx) => ctx.runtime.read().with_overrides(
            props.delay_duration,
            props.skip_delay_duration,
            props.disable_hoverable_content,
            props.close_on_trigger_click,
            props.ignore_non_keyboard_focus,
        ),
        None => fallback_provider_runtime,
    };

    let attached_handle: Rc<RefCell<Option<(AnyTooltipHandle, String)>>> =
        use_hook(|| Rc::new(RefCell::new(None)));
    {
        let mut slot = attached_handle.borrow_mut();
        let needs_rebind = match (&*slot, props.handle.as_any()) {
            (Some((prev, prev_rid)), Some(next)) => prev != next || prev_rid != &root_id,
            (Some(_), None) | (None, Some(_)) => true,
            (None, None) => false,
        };
        if needs_rebind {
            if let Some((prev, prev_rid)) = slot.take() {
                prev.detach_root(&prev_rid);
            }
            if let Some(next) = props.handle.as_any() {
                let reader_ctl = open_ctl;
                let setter_ctl = open_ctl;
                next.attach_root(
                    root_id.clone(),
                    Rc::new(move || reader_ctl.try_peek_cloned().unwrap_or(false)),
                    Rc::new(move |next_open| {
                        setter_ctl.set_with_context(
                            next_open,
                            ChangeContext::new(
                                ChangeReason::Programmatic,
                                ChangeSource::Programmatic,
                            ),
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
    if let Some(any_handle) = props.handle.as_any() {
        any_handle.sync_open_state_for_root(&root_id, is_open);
    }

    let handle_active_trigger_id = props.handle.as_any().and_then(|h| h.active_trigger_id());
    let handle_active_trigger_mounted = props
        .handle
        .as_any()
        .and_then(|h| h.active_trigger_mounted());

    let prev_open = use_hook(|| Rc::new(Cell::new(false)));
    let was_open = prev_open.get();
    if is_open && !was_open {
        if effective_provider_runtime
            .active_tooltip_id()
            .as_deref()
            != Some(relationships.root_id())
        {
            effective_provider_runtime.request_open(relationships.root_id(), true);
        }
    } else if !is_open
        && was_open
        && effective_provider_runtime
            .active_tooltip_id()
            .as_deref()
            == Some(relationships.root_id())
    {
        effective_provider_runtime.request_close(relationships.root_id());
    }
    prev_open.set(is_open);

    let tooltip = Tooltip::new(scope, is_open)
        .with_provider(effective_provider_runtime.provider().clone());

    let prov_for_agent = effective_provider_runtime.clone();
    let handle_trigger_ids = props
        .handle
        .as_any()
        .map(|h| h.registered_trigger_ids())
        .unwrap_or_default();
    let runtime = use_tooltip_runtime_with_ctl(
        tooltip,
        Some(effective_provider_runtime),
        open_ctl,
        props.on_open_change_complete,
        handle_active_trigger_id,
        handle_active_trigger_mounted,
        handle_trigger_ids,
    );
    let mut runtime_sig = use_signal(|| runtime.clone());
    runtime_sig.set(runtime.clone());

    let agent_label = props.id.clone().unwrap_or_else(|| "Tooltip".to_owned());
    let rid_for_agent = root_id;
    use_agent_surface(
        props.agent.as_ref(),
        SurfaceMetaDefaults::default(),
        move || {
            AgentSurface::new("tooltip", agent_label)
                .with_snapshot(
                    ComponentSnapshot::new().with_field("open", SnapshotValue::Bool(is_open)),
                )
                .with_action(ActionDescriptor::new("open", "Open tooltip", false))
                .with_action(ActionDescriptor::new("close", "Close tooltip", false))
        },
        move || {
            Rc::new(move |dispatch| {
                let next = match dispatch.action.as_str() {
                    "open" => true,
                    "close" => false,
                    other => {
                        return DispatchExecResult::ok()
                            .with_host_error(format!("unknown tooltip action: {other}"));
                    }
                };
                let outcome = open_ctl.set_with_context(next, dispatch.ctx);
                if outcome.is_canceled() {
                    cancel_current_agent_action();
                    return DispatchExecResult::ok().with_canceled(true);
                }
                if outcome.is_accepted() {
                    if next {
                        prov_for_agent.request_open(rid_for_agent.clone(), true);
                    } else {
                        prov_for_agent.request_close(&rid_for_agent);
                    }
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

    use_context_provider(|| TooltipRootContext {
        runtime: runtime_sig,
        open: open_signal,
        on_open_change: props.on_open_change,
        relationships: relationships.clone(),
    });
    use_context_provider(|| runtime.floating_tooltip_context());

    rsx! {
        {props.children}
    }
}

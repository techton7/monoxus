use std::{cell::RefCell, rc::Rc};

use dioxus::prelude::*;
use ui_sdk::kernel::{ChangeContext, ChangeEvent, ChangeReason, ChangeSource};

use crate::agent::{
    ActionDescriptor, AgentHints, AgentSurface, ComponentSnapshot, DispatchExecResult,
    SnapshotValue, SurfaceMetaDefaults, cancel_current_agent_action, use_agent_surface,
};
use crate::foundation::{
    overlay::{AnyHoverCardHandle, HoverCardHandleProp, use_overlay_handle_version},
    shared::{ScopeHandle, use_stable_id},
    state::{ControllableConfig, use_controllable_signal_config},
};
use crate::hover_card::{
    components::HoverCardRootContext,
    relationships::HoverCardRelationships,
    runtime::use_hover_card_runtime_with_ctl,
    state::HoverCard as HoverCardState,
    types::{HOVER_CARD_DEFAULT_CLOSE_DELAY_MS, HOVER_CARD_DEFAULT_OPEN_DELAY_MS},
};

#[derive(Clone, PartialEq, Props)]
pub struct HoverCardRootProps {
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
    #[props(default = HOVER_CARD_DEFAULT_OPEN_DELAY_MS)]
    pub open_delay: u64,
    #[props(default = HOVER_CARD_DEFAULT_CLOSE_DELAY_MS)]
    pub close_delay: u64,
    #[props(default = false)]
    pub disabled: bool,
    #[props(default = true)]
    pub ignore_non_keyboard_focus: bool,
    #[props(default, into)]
    pub handle: HoverCardHandleProp,
    #[props(default)]
    pub agent: Option<AgentHints>,
    pub children: Element,
}

#[component]
pub fn HoverCardRoot(props: HoverCardRootProps) -> Element {
    let open_ctl = use_controllable_signal_config(
        ControllableConfig::new(props.open, props.default_open)
            .with_explicit_default(props.default_open)
            .with_on_change(props.on_open_change)
            .with_on_change_details(props.on_open_change_details)
            .with_component("HoverCardRoot")
            .with_prop("open"),
    );
    let open_signal = open_ctl.signal();

    let stable_id = use_stable_id(None);
    let scope_id = props.id.clone().unwrap_or_else(|| stable_id.into_owned());
    let scope = ScopeHandle::root("hover-card").child(scope_id);
    let relationships = HoverCardRelationships::new(scope.clone());
    let root_id = relationships.root_id().to_owned();

    let _handle_version = use_overlay_handle_version(props.handle.as_any());

    let attached_handle: Rc<RefCell<Option<(AnyHoverCardHandle, String)>>> =
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

    let hover_card = HoverCardState::new(scope, is_open)
        .with_open_delay_ms(props.open_delay)
        .with_close_delay_ms(props.close_delay)
        .with_disabled(props.disabled)
        .with_ignore_non_keyboard_focus(props.ignore_non_keyboard_focus);

    let on_open_change_complete = props.on_open_change_complete.map(|handler| {
        Rc::new(move |open_val: bool| {
            handler.call(open_val);
        }) as Rc<dyn Fn(bool)>
    });

    let runtime = use_hover_card_runtime_with_ctl(
        hover_card,
        open_ctl,
        on_open_change_complete,
        handle_active_trigger_id,
        handle_active_trigger_mounted,
    );
    let mut runtime_sig = use_signal(|| runtime.clone());
    runtime_sig.set(runtime.clone());

    let agent_label = props.id.clone().unwrap_or_else(|| "HoverCard".to_owned());
    let disabled = props.disabled;
    use_agent_surface(
        props.agent.as_ref(),
        SurfaceMetaDefaults::default(),
        move || {
            let mut surface = AgentSurface::new("hover-card", agent_label)
                .with_disabled(disabled)
                .with_snapshot(
                    ComponentSnapshot::new().with_field("open", SnapshotValue::Bool(is_open)),
                );
            if !disabled {
                surface = surface
                    .with_action(ActionDescriptor::new("open", "Open hover card", false))
                    .with_action(ActionDescriptor::new("close", "Close hover card", false));
            }
            surface
        },
        move || {
            Rc::new(move |dispatch| {
                if disabled {
                    return DispatchExecResult::ok().with_host_error("hover-card is disabled");
                }
                let next = match dispatch.action.as_str() {
                    "open" => true,
                    "close" => false,
                    other => {
                        return DispatchExecResult::ok()
                            .with_host_error(format!("unknown hover-card action: {other}"));
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

    use_context_provider(|| HoverCardRootContext {
        runtime: runtime_sig,
        open: open_signal,
        on_open_change: props.on_open_change,
        relationships: relationships.clone(),
    });

    rsx! {
        {props.children}
    }
}

pub use HoverCardRoot as HoverCard;

use crate::agent::{DispatchExecResult, SurfaceMetaDefaults, use_agent_surface};
use crate::toast::runtime::{TOAST_STORE, toast};
use crate::toast::types::ToastConfig;
use dioxus::prelude::*;
use std::cell::RefCell;
use std::rc::Rc;
use ui_sdk::agent::{
    ActionDescriptor, ActionDispatch, AgentHints, AgentSurface, ComponentSnapshot, SnapshotValue,
};

#[derive(Props, Clone, PartialEq)]
pub struct ToastProviderProps {
    #[props(default)]
    pub config: Option<ToastConfig>,
    #[props(default)]
    pub close_button: Option<bool>,
    #[props(default)]
    pub expand: Option<bool>,
    #[props(default)]
    pub agent: Option<AgentHints>,
    pub children: Element,
}

fn sync_provider_config(
    config: Option<ToastConfig>,
    close_button: Option<bool>,
    expand: Option<bool>,
) {
    if config.is_none() && close_button.is_none() && expand.is_none() {
        return;
    }
    let mut next = config.unwrap_or_else(|| TOAST_STORE.peek().config.clone());
    if let Some(cb) = close_button {
        next.close_button = cb;
    }
    if let Some(exp) = expand {
        next.expand = exp;
    }
    if TOAST_STORE.peek().config != next {
        let mut store = TOAST_STORE.write();
        store.config = next;
        store.sync_interaction_state();
    }
}

/// Optional configuration context provider establishing defaults for the toast viewport.
#[component]
pub fn ToastProvider(props: ToastProviderProps) -> Element {
    let last_seen = use_hook(|| {
        sync_provider_config(props.config.clone(), props.close_button, props.expand);
        Rc::new(RefCell::new((
            props.config.clone(),
            props.close_button,
            props.expand,
        )))
    });

    let mut tracked_props = use_signal(|| {
        (
            props.config.clone(),
            props.close_button,
            props.expand,
        )
    });

    {
        let current = (props.config.clone(), props.close_button, props.expand);
        let mut prev = last_seen.borrow_mut();
        if *prev != current {
            *prev = current.clone();
            tracked_props.set(current);
        }
    }

    use_effect(move || {
        let (cfg, cb, exp) = tracked_props.read().clone();
        sync_provider_config(cfg, cb, exp);
    });

    let has_active_toasts = TOAST_STORE.read().has_active_toasts();
    let mut effective_hints = props.agent.clone().unwrap_or_default();
    if !has_active_toasts {
        effective_hints.hidden = true;
    }
    let build_provider_snapshot = move || {
        let store = TOAST_STORE.peek();
        let active_count = store.toasts.len() as i64;
        let items: Vec<String> = store.toasts.iter().map(|t| t.title.clone()).collect();
        ComponentSnapshot::new()
            .with_field("count", SnapshotValue::Int(active_count))
            .with_field("paused", SnapshotValue::Bool(store.paused))
            .with_field("expanded", SnapshotValue::Bool(store.expanded))
            .with_items(items)
    };
    let reader_snap = build_provider_snapshot;
    use_agent_surface(
        Some(&effective_hints),
        SurfaceMetaDefaults::new(),
        move || {
            AgentSurface::new("toast", "Notifications")
                .with_snapshot(build_provider_snapshot())
                .with_actions(vec![
                    ActionDescriptor::new("dismiss", "Dismiss all active toasts", false),
                    ActionDescriptor::new("pause", "Pause active toast timers", false),
                ])
        },
        move || {
            Rc::new(move |dispatch: &ActionDispatch| match dispatch.action.as_str() {
                "dismiss" => {
                    toast::dismiss(None);
                    DispatchExecResult::ok()
                }
                "pause" => {
                    TOAST_STORE.write().pause_all();
                    DispatchExecResult::ok()
                }
                other => DispatchExecResult::ok()
                    .with_host_error(format!("unsupported toast action: {other}")),
            })
        },
        move || Rc::new(reader_snap),
    );

    rsx! {
        {props.children}
    }
}

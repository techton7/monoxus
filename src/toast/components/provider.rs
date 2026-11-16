use crate::toast::runtime::TOAST_STORE;
use crate::toast::types::ToastConfig;
use dioxus::prelude::*;
use std::cell::RefCell;
use std::rc::Rc;

#[derive(Props, Clone, PartialEq)]
pub struct ToastProviderProps {
    #[props(default)]
    pub config: Option<ToastConfig>,
    #[props(default)]
    pub close_button: Option<bool>,
    #[props(default)]
    pub expand: Option<bool>,
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

    rsx! {
        {props.children}
    }
}

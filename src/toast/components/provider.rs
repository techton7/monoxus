use crate::toast::runtime::TOAST_STORE;
use crate::toast::types::ToastConfig;
use dioxus::prelude::*;

#[derive(Props, Clone, PartialEq)]
pub struct ToastProviderProps {
    #[props(default)]
    pub config: Option<ToastConfig>,
    pub children: Element,
}

/// Optional configuration context provider establishing defaults for the toast viewport.
#[component]
pub fn ToastProvider(props: ToastProviderProps) -> Element {
    use_hook(|| {
        if let Some(cfg) = props.config.clone() {
            TOAST_STORE.write().config = cfg;
        }
    });

    rsx! {
        {props.children}
    }
}

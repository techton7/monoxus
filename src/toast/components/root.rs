use dioxus::prelude::*;
use crate::toast::attrs::ToastRootAttrs;
use crate::toast::types::{ToastId, ToastPhase, ToastType};

#[derive(Props, Clone, PartialEq)]
pub struct ToastRootProps {
    pub id: ToastId,
    #[props(default)]
    pub toast_type: ToastType,
    #[props(default)]
    pub phase: ToastPhase,
    #[props(default)]
    pub index: usize,
    #[props(default = 3)]
    pub visible_limit: usize,
    #[props(default)]
    pub expanded: bool,
    #[props(default)]
    pub test_id: Option<String>,
    #[props(default)]
    pub class: Option<String>,
    #[props(default)]
    pub children: Element,
}

/// Headless toast item container (<li role="status" / role="alert">).
#[component]
pub fn ToastRoot(props: ToastRootProps) -> Element {
    let attrs = ToastRootAttrs::with_options(
        props.toast_type,
        props.phase,
        props.index,
        props.visible_limit,
        props.test_id,
        props.expanded,
    );

    rsx! {
        li {
            role: "{attrs.role}",
            aria_live: "{attrs.aria_live}",
            aria_atomic: "{attrs.aria_atomic}",
            "data-sonner-toast": "",
            "data-id": "{props.id.0}",
            "data-state": "{attrs.data_state}",
            "data-type": "{attrs.data_type}",
            "data-visible": "{attrs.data_visible}",
            "data-front": "{attrs.data_front}",
            "data-expanded": "{attrs.data_expanded}",
            "data-index": "{attrs.data_index}",
            "data-testid": attrs.data_testid.as_deref().unwrap_or_default(),
            class: props.class.as_deref().unwrap_or_default(),
            {props.children}
        }
    }
}

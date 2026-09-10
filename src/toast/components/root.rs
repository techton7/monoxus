use crate::toast::attrs::ToastRootAttrs;
use crate::toast::types::{ToastId, ToastPhase, ToastPosition, ToastType};
use dioxus::prelude::*;

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
    pub position: Option<ToastPosition>,
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
    let position = props.position.unwrap_or(ToastPosition::BottomRight);
    let attrs = ToastRootAttrs::with_options_and_position(
        props.toast_type,
        props.phase,
        props.index,
        props.visible_limit,
        props.test_id,
        props.expanded,
        position,
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
            "data-position": "{attrs.data_position}",
            "data-x-position": "{attrs.data_x_position}",
            "data-y-position": "{attrs.data_y_position}",
            "data-swipe-out": "{attrs.data_swipe_out}",
            "data-index": "{attrs.data_index}",
            "data-testid": attrs.data_testid.as_deref().unwrap_or_default(),
            class: props.class.as_deref().unwrap_or_default(),
            {props.children}
        }
    }
}

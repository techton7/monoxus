use dioxus::prelude::*;

use super::super::attrs::PaginationNextButtonAttributes;
use super::prev_button::{
    PaginationStepButtonSpec, PaginationStepKind, render_pagination_step_button,
};

#[derive(Clone)]
pub struct PaginationNextButtonRenderChildProps {
    pub attrs: PaginationNextButtonAttributes,
    pub children: Element,
}

#[derive(Clone, PartialEq, Props)]
pub struct PaginationNextButtonProps {
    #[props(default)]
    pub id: Option<String>,
    #[props(default = false)]
    pub disabled: bool,
    #[props(default)]
    pub aria_label: Option<String>,
    #[props(default)]
    pub as_child: bool,
    #[props(default)]
    pub render_child: Option<Callback<PaginationNextButtonRenderChildProps, Element>>,
    #[props(default)]
    pub on_click: Option<EventHandler<MouseEvent>>,
    #[props(default)]
    pub on_key_down: Option<EventHandler<KeyboardEvent>>,
    #[props(default)]
    pub on_mounted: Option<EventHandler<MountedEvent>>,
    #[props(default)]
    pub class: Option<String>,
    #[props(default)]
    pub style: Option<String>,
    #[props(extends = GlobalAttributes)]
    pub attributes: Vec<Attribute>,
    #[props(default)]
    pub children: Element,
}

#[component]
pub fn PaginationNextButton(props: PaginationNextButtonProps) -> Element {
    render_pagination_step_button(PaginationStepButtonSpec {
        id: props.id,
        disabled: props.disabled,
        aria_label: props.aria_label,
        as_child: props.as_child,
        on_click: props.on_click,
        on_key_down: props.on_key_down,
        on_mounted: props.on_mounted,
        class: props.class,
        style: props.style,
        attributes: props.attributes,
        children: props.children,
        kind: PaginationStepKind::Next {
            render_child: props.render_child,
        },
    })
}

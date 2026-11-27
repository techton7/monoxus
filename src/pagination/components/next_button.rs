use dioxus::prelude::*;

use super::prev_button::{
    PaginationStepButtonSpec, PaginationStepKind, render_pagination_step_button,
};

#[derive(Clone, PartialEq, Props)]
pub struct PaginationNextButtonProps {
    #[props(default, into)]
    pub id: Option<String>,
    #[props(default = false)]
    pub disabled: bool,
    #[props(default)]
    pub on_click: Option<EventHandler<MouseEvent>>,
    #[props(default)]
    pub on_key_down: Option<EventHandler<KeyboardEvent>>,
    #[props(default)]
    pub on_mounted: Option<EventHandler<MountedEvent>>,
    #[props(default, into)]
    pub class: Option<String>,
    #[props(default, into)]
    pub style: Option<String>,
    #[props(default)]
    pub child: Option<Callback<Vec<Attribute>, Element>>,
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
        on_click: props.on_click,
        on_key_down: props.on_key_down,
        on_mounted: props.on_mounted,
        class: props.class,
        style: props.style,
        child: props.child,
        attributes: props.attributes,
        children: props.children,
        kind: PaginationStepKind::Next,
    })
}

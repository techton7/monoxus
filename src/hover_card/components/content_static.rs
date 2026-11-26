use dioxus::prelude::*;

use crate::foundation::shared::{
    Direction, EscapeKeyDownEvent, InteractOutsideEvent, PointerDownOutsideEvent,
};
use crate::hover_card::{
    components::content::{HoverCardSurfaceSpec, render_hover_card_surface},
    runtime::HoverCardOutsideConfig,
    types::HoverCardOutsideBehavior,
};

#[derive(Clone, PartialEq, Props)]
pub struct HoverCardContentStaticProps {
    #[props(default)]
    pub id: Option<String>,
    #[props(default)]
    pub class: Option<String>,
    #[props(default)]
    pub style: Option<String>,
    #[props(default = false)]
    pub force_mount: bool,
    #[props(default)]
    pub dir: Option<Direction>,
    #[props(default)]
    pub on_escape_key_down: Option<EventHandler<EscapeKeyDownEvent>>,
    #[props(default)]
    pub on_pointer_down_outside: Option<EventHandler<PointerDownOutsideEvent>>,
    #[props(default)]
    pub on_interact_outside: Option<EventHandler<InteractOutsideEvent>>,
    #[props(default)]
    pub escape_keydown_behavior: HoverCardOutsideBehavior,
    #[props(default)]
    pub interact_outside_behavior: HoverCardOutsideBehavior,
    #[props(extends = GlobalAttributes)]
    pub attributes: Vec<Attribute>,
    pub children: Element,
}

#[component]
pub fn HoverCardContentStatic(props: HoverCardContentStaticProps) -> Element {
    render_hover_card_surface(HoverCardSurfaceSpec {
        id: props.id,
        class: props.class,
        style: props.style,
        force_mount: props.force_mount,
        floating: false,
        floating_overrides: None,
        dir: props.dir,
        outside_config: HoverCardOutsideConfig {
            escape_keydown_behavior: props.escape_keydown_behavior,
            interact_outside_behavior: props.interact_outside_behavior,
            on_escape_key_down: props.on_escape_key_down,
            on_pointer_down_outside: props.on_pointer_down_outside,
            on_interact_outside: props.on_interact_outside,
        },
        attributes: props.attributes,
        children: props.children,
    })
}

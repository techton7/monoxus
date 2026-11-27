use dioxus::prelude::*;

use crate::foundation::{
    overlay::DismissBehavior,
    shared::{Direction, EscapeKeyDownEvent, InteractOutsideEvent, PointerDownOutsideEvent},
};
use crate::hover_card::{
    components::content::{HoverCardSurfaceChild, HoverCardSurfaceSpec, render_hover_card_surface},
    runtime::HoverCardOutsideConfig,
};

#[derive(Clone, PartialEq, Props)]
pub struct HoverCardContentStaticProps {
    #[props(default, into)]
    pub id: Option<String>,
    #[props(default, into)]
    pub class: Option<String>,
    #[props(default, into)]
    pub style: Option<String>,
    #[props(default = false)]
    pub force_mount: bool,
    #[props(default = false)]
    pub keep_mounted: bool,
    #[props(default)]
    pub dir: Option<Direction>,
    #[props(default)]
    pub on_escape_key_down: Option<EventHandler<EscapeKeyDownEvent>>,
    #[props(default)]
    pub on_pointer_down_outside: Option<EventHandler<PointerDownOutsideEvent>>,
    #[props(default)]
    pub on_interact_outside: Option<EventHandler<InteractOutsideEvent>>,
    #[props(default)]
    pub escape_key_down_behavior: DismissBehavior,
    #[props(default)]
    pub interact_outside_behavior: DismissBehavior,
    #[props(default)]
    pub child: Option<Callback<Vec<Attribute>, Element>>,
    #[props(extends = GlobalAttributes)]
    pub attributes: Vec<Attribute>,
    #[props(default)]
    pub children: Element,
}

#[component]
pub fn HoverCardContentStatic(props: HoverCardContentStaticProps) -> Element {
    render_hover_card_surface(HoverCardSurfaceSpec {
        id: props.id,
        class: props.class,
        style: props.style,
        force_mount: props.force_mount,
        keep_mounted: props.keep_mounted,
        floating: false,
        floating_overrides: None,
        dir: props.dir,
        outside_config: HoverCardOutsideConfig {
            escape_key_down_behavior: props.escape_key_down_behavior,
            interact_outside_behavior: props.interact_outside_behavior,
            on_escape_key_down: props.on_escape_key_down,
            on_pointer_down_outside: props.on_pointer_down_outside,
            on_interact_outside: props.on_interact_outside,
        },
        child: props.child.map(HoverCardSurfaceChild::Static),
        attributes: props.attributes,
        children: props.children,
    })
}

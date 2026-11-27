use dioxus::prelude::*;

use super::item::{CommandItemKind, CommandItemSurfaceSpec, render_command_item_surface};

#[derive(Clone, PartialEq, Props)]
pub struct CommandLinkItemProps {
    #[props(into)]
    pub value: String,
    #[props(default, into)]
    pub href: Option<String>,
    #[props(default, into)]
    pub target: Option<String>,
    #[props(default, into)]
    pub rel: Option<String>,
    #[props(default, into)]
    pub id: Option<String>,
    #[props(default = false)]
    pub disabled: bool,
    #[props(default = false)]
    pub force_mount: bool,
    #[props(default)]
    pub keywords: Vec<String>,
    #[props(default)]
    pub on_select: Option<EventHandler<()>>,
    #[props(default, into)]
    pub class: Option<String>,
    #[props(default, into)]
    pub style: Option<String>,
    #[props(default)]
    pub child: Option<Callback<Vec<Attribute>, Element>>,
    #[props(default)]
    pub on_click: Option<EventHandler<MouseEvent>>,
    #[props(default)]
    pub on_pointer_move: Option<EventHandler<PointerEvent>>,
    #[props(default)]
    pub on_mounted: Option<EventHandler<MountedEvent>>,
    #[props(extends = GlobalAttributes)]
    pub attributes: Vec<Attribute>,
    #[props(default)]
    pub children: Element,
}

#[component]
pub fn CommandLinkItem(props: CommandLinkItemProps) -> Element {
    render_command_item_surface(CommandItemSurfaceSpec {
        value: props.value,
        id: props.id,
        disabled: props.disabled,
        force_mount: props.force_mount,
        keywords: props.keywords,
        on_select: props.on_select,
        class: props.class,
        style: props.style,
        child: props.child,
        on_click: props.on_click,
        on_pointer_move: props.on_pointer_move,
        on_mounted: props.on_mounted,
        attributes: props.attributes,
        children: props.children,
        kind: CommandItemKind::Link {
            href: props.href,
            target: props.target,
            rel: props.rel,
        },
    })
}

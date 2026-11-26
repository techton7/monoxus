use dioxus::prelude::*;

use super::item::{CommandItemKind, CommandItemSurfaceSpec, render_command_item_surface};

#[derive(Clone, PartialEq, Props)]
pub struct CommandLinkItemRenderProps {
    pub id: String,
    pub role: &'static str,
    pub aria_disabled: &'static str,
    pub aria_selected: &'static str,
    pub data_command_item: &'static str,
    pub data_value: String,
    #[props(default)]
    pub data_group: Option<String>,
    #[props(default)]
    pub data_disabled: Option<&'static str>,
    #[props(default)]
    pub data_selected: Option<&'static str>,
    #[props(default)]
    pub href: Option<String>,
    #[props(default)]
    pub target: Option<String>,
    #[props(default)]
    pub rel: Option<String>,
    #[props(default)]
    pub class: Option<String>,
    #[props(default)]
    pub style: Option<String>,
    pub on_mousedown: EventHandler<MouseEvent>,
    pub on_pointer_move: EventHandler<PointerEvent>,
    pub on_click: EventHandler<MouseEvent>,
    #[props(default)]
    pub on_mounted: Option<EventHandler<MountedEvent>>,
    #[props(extends = GlobalAttributes)]
    pub attributes: Vec<Attribute>,
    pub children: Element,
}

#[derive(Clone, PartialEq, Props)]
pub struct CommandLinkItemProps {
    pub value: String,
    #[props(default)]
    pub href: Option<String>,
    #[props(default)]
    pub target: Option<String>,
    #[props(default)]
    pub rel: Option<String>,
    #[props(default)]
    pub id: Option<String>,
    #[props(default = false)]
    pub disabled: bool,
    #[props(default = false)]
    pub force_mount: bool,
    #[props(default)]
    pub keywords: Vec<String>,
    #[props(default)]
    pub on_select: Option<EventHandler<()>>,
    #[props(default)]
    pub class: Option<String>,
    #[props(default)]
    pub style: Option<String>,
    #[props(default = false)]
    pub as_child: bool,
    #[props(default)]
    pub render_child: Option<Callback<CommandLinkItemRenderProps, Element>>,
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
        as_child: props.as_child,
        on_click: props.on_click,
        on_pointer_move: props.on_pointer_move,
        on_mounted: props.on_mounted,
        attributes: props.attributes,
        children: props.children,
        kind: CommandItemKind::Link {
            href: props.href,
            target: props.target,
            rel: props.rel,
            render_child: props.render_child,
        },
    })
}

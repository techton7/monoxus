use dioxus::prelude::*;

use crate::foundation::shared::Direction;
use crate::menu::{MenuContentRenderProps, MenuContentStatic, MenuPreventableEvent};

#[derive(Props, Clone, PartialEq)]
pub struct MenubarContentStaticProps {
    #[props(default)]
    pub id: Option<String>,
    #[props(default)]
    pub class: Option<String>,
    #[props(default)]
    pub style: Option<String>,
    #[props(default = false)]
    pub force_mount: bool,
    #[props(default)]
    pub loop_focus: Option<bool>,
    #[props(default)]
    pub dir: Option<Direction>,
    #[props(default)]
    pub on_escape_keydown: Option<EventHandler<MenuPreventableEvent>>,
    #[props(default)]
    pub on_open_auto_focus: Option<EventHandler<MenuPreventableEvent>>,
    #[props(default)]
    pub on_close_auto_focus: Option<EventHandler<MenuPreventableEvent>>,
    #[props(default)]
    pub on_key_down: Option<EventHandler<KeyboardEvent>>,
    #[props(default = false)]
    pub as_child: bool,
    #[props(default)]
    pub render_child: Option<Callback<MenuContentRenderProps, Element>>,
    #[props(extends = GlobalAttributes)]
    pub attributes: Vec<Attribute>,
    pub children: Element,
}

/// Inline non-floating [`MenubarMenu`](super::menu::MenubarMenu) surface (`3B D-6`).
#[component]
pub fn MenubarContentStatic(props: MenubarContentStaticProps) -> Element {
    rsx! {
        MenuContentStatic {
            id: props.id,
            class: props.class,
            style: props.style,
            force_mount: props.force_mount,
            loop_focus: props.loop_focus,
            dir: props.dir,
            on_escape_keydown: props.on_escape_keydown,
            on_open_auto_focus: props.on_open_auto_focus,
            on_close_auto_focus: props.on_close_auto_focus,
            on_key_down: props.on_key_down,
            as_child: props.as_child,
            render_child: props.render_child,
            attributes: props.attributes,
            {props.children}
        }
    }
}

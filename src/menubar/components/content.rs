use dioxus::prelude::*;

use crate::foundation::{
    overlay::{PlacementAlign, PlacementSide, StickyStrategy},
    shared::Direction,
};
use crate::menu::{
    MenuContent, MenuContentRenderProps, MenuOutsideEvent, MenuPreventableEvent,
};

#[derive(Props, Clone, PartialEq)]
pub struct MenubarContentProps {
    #[props(default)]
    pub id: Option<String>,
    #[props(default)]
    pub class: Option<String>,
    #[props(default)]
    pub style: Option<String>,
    #[props(default = PlacementSide::Bottom)]
    pub side: PlacementSide,
    #[props(default = PlacementAlign::Start)]
    pub align: PlacementAlign,
    #[props(default = 4.0)]
    pub side_offset: f32,
    #[props(default = 0.0)]
    pub align_offset: f32,
    #[props(default = true)]
    pub avoid_collisions: bool,
    #[props(default = 0.0)]
    pub collision_padding: f32,
    #[props(default = 0.0)]
    pub arrow_padding: f32,
    #[props(default = StickyStrategy::Partial)]
    pub sticky: StickyStrategy,
    #[props(default = false)]
    pub hide_when_detached: bool,
    #[props(default = false)]
    pub force_mount: bool,
    #[props(default)]
    pub loop_focus: Option<bool>,
    #[props(default = false)]
    pub prevent_scroll: bool,
    #[props(default)]
    pub dir: Option<Direction>,
    #[props(default)]
    pub on_escape_keydown: Option<EventHandler<MenuPreventableEvent>>,
    #[props(default)]
    pub on_pointer_down_outside: Option<EventHandler<MenuOutsideEvent>>,
    #[props(default)]
    pub on_focus_outside: Option<EventHandler<MenuOutsideEvent>>,
    #[props(default)]
    pub on_interact_outside: Option<EventHandler<MenuOutsideEvent>>,
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

/// Floating menu panel owned by [`MenubarMenu`](super::menu::MenubarMenu).
#[component]
pub fn MenubarContent(props: MenubarContentProps) -> Element {
    rsx! {
        MenuContent {
            id: props.id,
            class: props.class,
            style: props.style,
            side: props.side,
            align: props.align,
            side_offset: props.side_offset,
            align_offset: props.align_offset,
            avoid_collisions: props.avoid_collisions,
            collision_padding: props.collision_padding,
            arrow_padding: props.arrow_padding,
            sticky: props.sticky,
            hide_when_detached: props.hide_when_detached,
            force_mount: props.force_mount,
            loop_focus: props.loop_focus,
            prevent_scroll: props.prevent_scroll,
            dir: props.dir,
            on_escape_keydown: props.on_escape_keydown,
            on_pointer_down_outside: props.on_pointer_down_outside,
            on_focus_outside: props.on_focus_outside,
            on_interact_outside: props.on_interact_outside,
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

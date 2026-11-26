use dioxus::prelude::*;

use crate::foundation::shared::EscapeKeyDownEvent;

use super::{
    content::{SelectSurfaceSpec, render_select_surface},
    types::SelectContext,
};

#[derive(Props, Clone, PartialEq)]
pub struct SelectContentStaticProps {
    #[props(default)]
    pub class: Option<String>,
    #[props(default)]
    pub style: Option<String>,
    #[props(default = false)]
    pub force_mount: bool,
    #[props(default)]
    pub on_escape_key_down: Option<EventHandler<EscapeKeyDownEvent>>,
    #[props(extends = GlobalAttributes)]
    pub attributes: Vec<Attribute>,
    pub children: Element,
}

#[component]
pub fn SelectContentStatic(props: SelectContentStaticProps) -> Element {
    let ctx = use_context::<SelectContext>();
    ctx.runtime.set_force_mount(props.force_mount);
    ctx.runtime.set_on_escape_key_down(props.on_escape_key_down);

    render_select_surface(SelectSurfaceSpec {
        ctx,
        is_floating: false,
        class: props.class,
        style: props.style,
        same_width: false,
        arrow_padding: 0.0,
        sticky: None,
        prevent_scroll: false,
        prevent_overflow_text_selection: false,
        force_mount: props.force_mount,
        custom_anchor: None,
        attributes: props.attributes,
        children: props.children,
    })
}

use dioxus::prelude::*;

use crate::foundation::{overlay::DismissBehavior, shared::EscapeKeyDownEvent};

use super::{
    content::{SelectSurfaceChild, SelectSurfaceSpec, render_select_surface},
    types::SelectContext,
};

#[derive(Props, Clone, PartialEq)]
pub struct SelectContentStaticProps {
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
    pub escape_key_down_behavior: Option<DismissBehavior>,
    #[props(default)]
    pub interact_outside_behavior: Option<DismissBehavior>,
    #[props(default)]
    pub on_escape_key_down: Option<EventHandler<EscapeKeyDownEvent>>,
    #[props(default)]
    pub child: Option<Callback<Vec<Attribute>, Element>>,
    #[props(extends = GlobalAttributes)]
    pub attributes: Vec<Attribute>,
    pub children: Element,
}

#[component]
pub fn SelectContentStatic(props: SelectContentStaticProps) -> Element {
    let SelectContentStaticProps {
        id,
        class,
        style,
        force_mount,
        keep_mounted,
        escape_key_down_behavior,
        interact_outside_behavior,
        on_escape_key_down,
        child,
        attributes,
        children,
    } = props;
    let ctx = use_context::<SelectContext>();
    ctx.runtime.set_force_mount(force_mount);
    ctx.runtime.set_keep_mounted(keep_mounted);
    ctx.runtime
        .set_escape_key_down_behavior(escape_key_down_behavior);
    ctx.runtime
        .set_interact_outside_behavior(interact_outside_behavior);
    ctx.runtime.set_on_escape_key_down(on_escape_key_down);

    render_select_surface(SelectSurfaceSpec {
        ctx,
        id,
        is_floating: false,
        class,
        style,
        same_width: false,
        arrow_padding: 0.0,
        sticky: None,
        prevent_scroll: false,
        prevent_overflow_text_selection: false,
        force_mount,
        custom_anchor: None,
        child: SelectSurfaceChild::Static(child),
        attributes,
        children,
    })
}

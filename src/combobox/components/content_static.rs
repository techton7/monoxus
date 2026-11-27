use dioxus::prelude::*;

use crate::foundation::{
    overlay::DismissBehavior,
    shared::EscapeKeyDownEvent,
    state::sync_signal,
};

use crate::combobox::{
    monitors::ComboboxInteractOutsideEvent,
    runtime::use_combobox_context,
};

use super::content::{ComboboxChildRenderer, ComboboxSurfaceSpec, render_combobox_surface};

#[derive(Props, Clone, PartialEq)]
pub struct ComboboxContentStaticProps {
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
    pub on_interact_outside: Option<EventHandler<ComboboxInteractOutsideEvent>>,
    #[props(default)]
    pub on_escape_key_down: Option<EventHandler<EscapeKeyDownEvent>>,
    #[props(default)]
    pub child: Option<Callback<Vec<Attribute>, Element>>,
    #[props(extends = GlobalAttributes)]
    pub attributes: Vec<Attribute>,
    pub children: Element,
}

#[component]
pub fn ComboboxContentStatic(props: ComboboxContentStaticProps) -> Element {
    let ctx = use_combobox_context();
    let content_id = props
        .id
        .clone()
        .unwrap_or_else(|| ctx.runtime.relationships().content_id().to_owned());

    {
        let mut fm = ctx.runtime.state.force_mount;
        sync_signal(&mut fm, props.force_mount);
        let mut ekb = ctx.runtime.state.escape_key_down_behavior;
        sync_signal(&mut ekb, props.escape_key_down_behavior);
        let mut iob = ctx.runtime.state.interact_outside_behavior;
        sync_signal(&mut iob, props.interact_outside_behavior);
        let mut oio = ctx.runtime.state.on_interact_outside;
        oio.set(props.on_interact_outside);
        let mut oek = ctx.runtime.state.on_escape_key_down;
        oek.set(props.on_escape_key_down);
    }

    render_combobox_surface(ComboboxSurfaceSpec {
        ctx,
        content_id,
        is_floating: false,
        same_width: false,
        force_mount: props.force_mount,
        keep_mounted: props.keep_mounted,
        class: props.class,
        style: props.style,
        child: props.child.map(ComboboxChildRenderer::Static),
        attributes: props.attributes,
        children: props.children,
    })
}

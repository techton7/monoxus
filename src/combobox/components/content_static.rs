use dioxus::prelude::*;

use crate::foundation::{shared::EscapeKeyDownEvent, state::sync_signal};

use crate::combobox::{
    monitors::ComboboxInteractOutsideEvent,
    runtime::use_combobox_context,
};

pub use super::content::ComboboxContentStaticRenderProps;
use super::content::{ComboboxAsChildRenderer, ComboboxSurfaceSpec, render_combobox_surface};

#[derive(Props, Clone, PartialEq)]
pub struct ComboboxContentStaticProps {
    #[props(default)]
    pub id: Option<String>,
    #[props(default)]
    pub class: Option<String>,
    #[props(default)]
    pub style: Option<String>,
    #[props(default = false)]
    pub force_mount: bool,
    #[props(default)]
    pub on_interact_outside: Option<EventHandler<ComboboxInteractOutsideEvent>>,
    #[props(default)]
    pub on_escape_key_down: Option<EventHandler<EscapeKeyDownEvent>>,
    #[props(default = false)]
    pub as_child: bool,
    #[props(default)]
    pub render_child: Option<Callback<ComboboxContentStaticRenderProps, Element>>,
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
        let mut oio = ctx.runtime.state.on_interact_outside;
        oio.set(props.on_interact_outside);
        let mut oek = ctx.runtime.state.on_escape_key_down;
        oek.set(props.on_escape_key_down);
    }

    let as_child = if props.as_child {
        props.render_child.map(ComboboxAsChildRenderer::Static)
    } else {
        None
    };

    render_combobox_surface(ComboboxSurfaceSpec {
        ctx,
        content_id,
        is_floating: false,
        same_width: false,
        force_mount: props.force_mount,
        class: props.class,
        style: props.style,
        as_child,
        attributes: props.attributes,
        children: props.children,
    })
}

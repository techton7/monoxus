use dioxus::prelude::*;

use crate::foundation::{
    browser::{ResizeObserverBox, use_element_size_with_mount},
    compose::{collect_user_attrs, merge_props, push_attr_opt, push_event_handler},
    state::sync_signal,
};

use super::super::{
    attrs::{CommandViewportAttrs, merge_flex_column_style},
    runtime::CommandContext,
};

#[derive(Clone, PartialEq, Props)]
pub struct CommandViewportProps {
    #[props(default, into)]
    pub id: Option<String>,
    #[props(default, into)]
    pub class: Option<String>,
    #[props(default, into)]
    pub style: Option<String>,
    #[props(default)]
    pub child: Option<Callback<Vec<Attribute>, Element>>,
    #[props(default)]
    pub on_mounted: Option<EventHandler<MountedEvent>>,
    #[props(extends = GlobalAttributes)]
    pub attributes: Vec<Attribute>,
    #[props(default)]
    pub children: Element,
}

#[component]
pub fn CommandViewport(props: CommandViewportProps) -> Element {
    let mut ctx = use_context::<CommandContext>();
    let resolved_id = props
        .id
        .clone()
        .unwrap_or_else(|| ctx.relationships.peek().viewport_id());
    sync_signal(&mut ctx.viewport_id, resolved_id.clone());

    let mut mount_rev = use_signal(|| 0u64);
    let size_sig = use_element_size_with_mount(&resolved_id, ResizeObserverBox::BorderBox, mount_rev);

    use_effect(move || {
        let next_height = size_sig().map(|s| s.height).filter(|&h| h > 0.0);
        sync_signal(&mut ctx.list_height, next_height);
    });

    let attrs = CommandViewportAttrs::new(resolved_id);
    let is_grid = ctx.columns.read().is_some();
    let base_style = merge_flex_column_style(None, is_grid);
    let base_mounted = EventHandler::new(move |_evt: MountedEvent| {
        let next = mount_rev.peek().wrapping_add(1);
        mount_rev.set(next);
    });

    let mut base = Vec::with_capacity(4);
    base.push(Attribute::new("id", attrs.id, None, false));
    base.push(Attribute::new(
        "data-command-viewport",
        attrs.data_command_viewport,
        None,
        false,
    ));
    push_attr_opt(&mut base, "style", base_style);
    push_event_handler(&mut base, "onmounted", Some(base_mounted));

    let mut user = collect_user_attrs(props.class, props.style, props.attributes);
    push_event_handler(&mut user, "onmounted", props.on_mounted);
    let merged = merge_props(base, user);

    if let Some(child) = props.child {
        return child.call(merged);
    }

    rsx! {
        div {
            ..merged,
            {props.children}
        }
    }
}

use dioxus::prelude::*;

use crate::foundation::browser::{ResizeObserverBox, use_element_size_with_mount};

use super::super::{
    attrs::{CommandViewportAttrs, merge_flex_column_style},
    runtime::CommandContext,
};

#[derive(Clone, PartialEq, Props)]
pub struct CommandViewportRenderProps {
    pub id: String,
    pub data_command_viewport: &'static str,
    #[props(default)]
    pub class: Option<String>,
    #[props(default)]
    pub style: Option<String>,
    pub on_mounted: EventHandler<MountedEvent>,
    #[props(extends = GlobalAttributes)]
    pub attributes: Vec<Attribute>,
    pub children: Element,
}

#[derive(Clone, PartialEq, Props)]
pub struct CommandViewportProps {
    #[props(default)]
    pub id: Option<String>,
    #[props(default)]
    pub class: Option<String>,
    #[props(default)]
    pub style: Option<String>,
    #[props(default = false)]
    pub as_child: bool,
    #[props(default)]
    pub render_child: Option<Callback<CommandViewportRenderProps, Element>>,
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
    if *ctx.viewport_id.peek() != resolved_id {
        ctx.viewport_id.set(resolved_id.clone());
    }

    let mut mount_rev = use_signal(|| 0u64);
    let size_sig = use_element_size_with_mount(&resolved_id, ResizeObserverBox::BorderBox, mount_rev);

    use_effect(move || {
        let next_height = size_sig().map(|s| s.height).filter(|&h| h > 0.0);
        if *ctx.list_height.peek() != next_height {
            ctx.list_height.set(next_height);
        }
    });

    let attrs = CommandViewportAttrs::new(resolved_id);
    let is_grid = ctx.columns.read().is_some();
    let merged_style = merge_flex_column_style(props.style.as_deref(), is_grid);
    let consumer_mounted = props.on_mounted;
    let handle_mounted = EventHandler::new(move |evt: MountedEvent| {
        let next = mount_rev.peek().wrapping_add(1);
        mount_rev.set(next);
        if let Some(handler) = consumer_mounted {
            handler.call(evt);
        }
    });

    if props.as_child {
        if let Some(render_child) = props.render_child.as_ref() {
            return render_child.call(CommandViewportRenderProps {
                id: attrs.id,
                data_command_viewport: attrs.data_command_viewport,
                class: props.class,
                style: merged_style,
                on_mounted: handle_mounted,
                attributes: props.attributes,
                children: props.children,
            });
        }
    }

    rsx! {
        div {
            id: "{attrs.id}",
            "data-command-viewport": attrs.data_command_viewport,
            class: props.class,
            style: merged_style,
            onmounted: move |evt| handle_mounted.call(evt),
            ..props.attributes,
            {props.children}
        }
    }
}

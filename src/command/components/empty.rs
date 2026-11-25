use dioxus::prelude::*;

use super::super::{
    attrs::CommandEmptyAttrs,
    runtime::CommandContext,
    state::should_render_empty,
};

#[derive(Clone, PartialEq, Props)]
pub struct CommandEmptyRenderProps {
    pub id: String,
    pub role: &'static str,
    pub data_command_empty: &'static str,
    #[props(default)]
    pub class: Option<String>,
    #[props(default)]
    pub style: Option<String>,
    #[props(default)]
    pub on_mounted: Option<EventHandler<MountedEvent>>,
    #[props(extends = GlobalAttributes)]
    pub attributes: Vec<Attribute>,
    pub children: Element,
}

#[derive(Clone, PartialEq, Props)]
pub struct CommandEmptyProps {
    #[props(default)]
    pub id: Option<String>,
    #[props(default = false)]
    pub force_mount: bool,
    #[props(default)]
    pub class: Option<String>,
    #[props(default)]
    pub style: Option<String>,
    #[props(default = false)]
    pub as_child: bool,
    #[props(default)]
    pub render_child: Option<Callback<CommandEmptyRenderProps, Element>>,
    #[props(default)]
    pub on_mounted: Option<EventHandler<MountedEvent>>,
    #[props(extends = GlobalAttributes)]
    pub attributes: Vec<Attribute>,
    #[props(default)]
    pub children: Element,
}

#[component]
pub fn CommandEmpty(props: CommandEmptyProps) -> Element {
    let ctx = use_context::<CommandContext>();
    let count = ctx.registry.read().filtered().count;
    let settled = *ctx.settled.read();

    if !should_render_empty(count, settled, props.force_mount) {
        return rsx! {};
    }

    let resolved_id = props
        .id
        .clone()
        .unwrap_or_else(|| ctx.relationships.peek().empty_id());
    let attrs = CommandEmptyAttrs::new(resolved_id);

    if props.as_child {
        if let Some(render_child) = props.render_child.as_ref() {
            return render_child.call(CommandEmptyRenderProps {
                id: attrs.id,
                role: attrs.role,
                data_command_empty: attrs.data_command_empty,
                class: props.class,
                style: props.style,
                on_mounted: props.on_mounted,
                attributes: props.attributes,
                children: props.children,
            });
        }
    }

    rsx! {
        div {
            id: "{attrs.id}",
            role: "{attrs.role}",
            "data-command-empty": attrs.data_command_empty,
            class: props.class,
            style: props.style,
            onmounted: move |evt| {
                if let Some(handler) = props.on_mounted.as_ref() {
                    handler.call(evt);
                }
            },
            ..props.attributes,
            {props.children}
        }
    }
}

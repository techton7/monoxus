use dioxus::prelude::*;

use super::super::{
    attrs::{CommandListAttrs, merge_flex_column_style},
    runtime::CommandContext,
};

#[derive(Clone, PartialEq, Props)]
pub struct CommandListRenderProps {
    pub id: String,
    pub role: &'static str,
    pub aria_label: String,
    pub data_command_list: &'static str,
    #[props(default)]
    pub class: Option<String>,
    pub style: String,
    #[props(default)]
    pub on_mounted: Option<EventHandler<MountedEvent>>,
    #[props(extends = GlobalAttributes)]
    pub attributes: Vec<Attribute>,
    pub children: Element,
}

#[derive(Clone, PartialEq, Props)]
pub struct CommandListProps {
    #[props(default)]
    pub id: Option<String>,
    #[props(default)]
    pub aria_label: Option<String>,
    #[props(default)]
    pub class: Option<String>,
    #[props(default)]
    pub style: Option<String>,
    #[props(default = false)]
    pub as_child: bool,
    #[props(default)]
    pub render_child: Option<Callback<CommandListRenderProps, Element>>,
    #[props(default)]
    pub on_mounted: Option<EventHandler<MountedEvent>>,
    #[props(extends = GlobalAttributes)]
    pub attributes: Vec<Attribute>,
    #[props(default)]
    pub children: Element,
}

#[component]
pub fn CommandList(props: CommandListProps) -> Element {
    let mut ctx = use_context::<CommandContext>();
    let resolved_id = props
        .id
        .clone()
        .unwrap_or_else(|| ctx.relationships.peek().list_id());
    if *ctx.list_id.peek() != resolved_id {
        ctx.list_id.set(resolved_id.clone());
    }

    let attrs = CommandListAttrs::new(resolved_id, props.aria_label.as_deref());
    let height_px = *ctx.list_height.read();
    let is_grid = ctx.columns.read().is_some();
    let base_style = CommandListAttrs::format_height_style(height_px, props.style.as_deref());
    let merged_style = merge_flex_column_style(
        if base_style.is_empty() {
            None
        } else {
            Some(base_style.as_str())
        },
        is_grid,
    )
    .unwrap_or_default();

    if props.as_child {
        if let Some(render_child) = props.render_child.as_ref() {
            return render_child.call(CommandListRenderProps {
                id: attrs.id,
                role: attrs.role,
                aria_label: attrs.aria_label,
                data_command_list: attrs.data_command_list,
                class: props.class,
                style: merged_style,
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
            aria_label: "{attrs.aria_label}",
            "data-command-list": attrs.data_command_list,
            class: props.class,
            style: if merged_style.is_empty() { None } else { Some(merged_style) },
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

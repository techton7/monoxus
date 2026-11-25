use dioxus::prelude::*;

use crate::foundation::shared::use_stable_id;

use super::super::{
    attrs::CommandSeparatorAttrs,
    runtime::CommandContext,
    state::should_render_separator,
};

#[derive(Clone, PartialEq, Props)]
pub struct CommandSeparatorRenderProps {
    pub id: String,
    pub aria_hidden: &'static str,
    pub data_command_separator: &'static str,
    #[props(default)]
    pub class: Option<String>,
    #[props(default)]
    pub style: Option<String>,
    #[props(default)]
    pub on_mounted: Option<EventHandler<MountedEvent>>,
    #[props(extends = GlobalAttributes)]
    pub attributes: Vec<Attribute>,
}

#[derive(Clone, PartialEq, Props)]
pub struct CommandSeparatorProps {
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
    pub render_child: Option<Callback<CommandSeparatorRenderProps, Element>>,
    #[props(default)]
    pub on_mounted: Option<EventHandler<MountedEvent>>,
    #[props(extends = GlobalAttributes)]
    pub attributes: Vec<Attribute>,
}

#[component]
pub fn CommandSeparator(props: CommandSeparatorProps) -> Element {
    let ctx = use_context::<CommandContext>();
    let search = ctx.search.read();
    if !should_render_separator(&search, props.force_mount) {
        return rsx! {};
    }

    let fallback_key = use_stable_id(None).into_owned();
    let sep_id = props
        .id
        .clone()
        .unwrap_or_else(|| ctx.relationships.peek().separator_id(&fallback_key));
    let attrs = CommandSeparatorAttrs::new(sep_id);

    if props.as_child {
        if let Some(render_child) = props.render_child.as_ref() {
            return render_child.call(CommandSeparatorRenderProps {
                id: attrs.id,
                aria_hidden: attrs.aria_hidden,
                data_command_separator: attrs.data_command_separator,
                class: props.class,
                style: props.style,
                on_mounted: props.on_mounted,
                attributes: props.attributes,
            });
        }
    }

    rsx! {
        div {
            id: "{attrs.id}",
            aria_hidden: "{attrs.aria_hidden}",
            "data-command-separator": attrs.data_command_separator,
            class: props.class,
            style: props.style,
            onmounted: move |evt| {
                if let Some(handler) = props.on_mounted.as_ref() {
                    handler.call(evt);
                }
            },
            ..props.attributes,
        }
    }
}

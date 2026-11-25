use std::borrow::Cow;

use dioxus::prelude::*;

use crate::foundation::shared::use_stable_id;

use super::super::{attrs::CommandLoadingAttrs, runtime::CommandContext};

#[derive(Clone, PartialEq, Props)]
pub struct CommandLoadingRenderProps {
    pub id: String,
    pub role: &'static str,
    pub aria_valuenow: f64,
    pub aria_valuemin: &'static str,
    pub aria_valuemax: &'static str,
    pub aria_label: String,
    pub data_command_loading: &'static str,
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
pub struct CommandLoadingProps {
    #[props(default)]
    pub id: Option<String>,
    #[props(default = 0.0)]
    pub progress: f64,
    #[props(default)]
    pub aria_label: Option<String>,
    #[props(default)]
    pub class: Option<String>,
    #[props(default)]
    pub style: Option<String>,
    #[props(default = false)]
    pub as_child: bool,
    #[props(default)]
    pub render_child: Option<Callback<CommandLoadingRenderProps, Element>>,
    #[props(default)]
    pub on_mounted: Option<EventHandler<MountedEvent>>,
    #[props(extends = GlobalAttributes)]
    pub attributes: Vec<Attribute>,
    #[props(default)]
    pub children: Element,
}

#[component]
pub fn CommandLoading(props: CommandLoadingProps) -> Element {
    let fallback_id = use_stable_id(props.id.clone().map(Cow::Owned)).into_owned();
    let resolved_id = props.id.clone().unwrap_or_else(|| {
        try_consume_context::<CommandContext>()
            .map(|ctx| ctx.relationships.peek().loading_id())
            .unwrap_or(fallback_id)
    });
    let attrs = CommandLoadingAttrs::new(resolved_id, props.progress, props.aria_label.as_deref());

    if props.as_child {
        if let Some(render_child) = props.render_child.as_ref() {
            return render_child.call(CommandLoadingRenderProps {
                id: attrs.id,
                role: attrs.role,
                aria_valuenow: attrs.aria_valuenow,
                aria_valuemin: attrs.aria_valuemin,
                aria_valuemax: attrs.aria_valuemax,
                aria_label: attrs.aria_label,
                data_command_loading: attrs.data_command_loading,
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
            aria_valuenow: "{attrs.aria_valuenow}",
            aria_valuemin: "{attrs.aria_valuemin}",
            aria_valuemax: "{attrs.aria_valuemax}",
            aria_label: "{attrs.aria_label}",
            "data-command-loading": attrs.data_command_loading,
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

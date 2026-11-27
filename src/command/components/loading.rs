use std::borrow::Cow;

use dioxus::prelude::*;

use crate::foundation::{
    compose::{collect_user_attrs, merge_props, push_event_handler},
    shared::use_stable_id,
};

use super::super::{attrs::CommandLoadingAttrs, runtime::CommandContext};

#[derive(Clone, PartialEq, Props)]
pub struct CommandLoadingProps {
    #[props(default, into)]
    pub id: Option<String>,
    #[props(default = 0.0)]
    pub progress: f64,
    #[props(default, into)]
    pub aria_label: Option<String>,
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
pub fn CommandLoading(props: CommandLoadingProps) -> Element {
    let fallback_id = use_stable_id(props.id.clone().map(Cow::Owned)).into_owned();
    let resolved_id = props.id.clone().unwrap_or_else(|| {
        try_consume_context::<CommandContext>()
            .map(|ctx| ctx.relationships.peek().loading_id())
            .unwrap_or(fallback_id)
    });
    let attrs = CommandLoadingAttrs::new(resolved_id, props.progress, props.aria_label.as_deref());

    let mut base = Vec::with_capacity(7);
    base.push(Attribute::new("id", attrs.id, None, false));
    base.push(Attribute::new("role", attrs.role, None, false));
    base.push(Attribute::new(
        "aria-valuenow",
        attrs.aria_valuenow.to_string(),
        None,
        false,
    ));
    base.push(Attribute::new(
        "aria-valuemin",
        attrs.aria_valuemin,
        None,
        false,
    ));
    base.push(Attribute::new(
        "aria-valuemax",
        attrs.aria_valuemax,
        None,
        false,
    ));
    base.push(Attribute::new("aria-label", attrs.aria_label, None, false));
    base.push(Attribute::new(
        "data-command-loading",
        attrs.data_command_loading,
        None,
        false,
    ));

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
